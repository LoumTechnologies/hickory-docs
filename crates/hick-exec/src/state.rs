//! Multi-document state for tracking element outputs, file outputs, and
//! cross-document references. Adapted from HickoryDocs `xml.rs`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

use crate::node::{
    Context, FileContent, InsertionPoint, Node, ProvenanceMap, StringNode, converge,
    converge_with_provenance,
};
use hick_condition::VariableResolver;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Content Block (for class-based selectors)
// ---------------------------------------------------------------------------

/// Content block with metadata for class-based selection.
///
/// Supports both ID-based (`#id`) and class-based (`.class`) selectors.
/// Class-based selectors concatenate content from all matching elements
/// in document order.
#[derive(Debug, Clone)]
pub struct ContentBlock {
    /// Optional unique identifier for `#id` selector.
    pub id: Option<String>,
    /// Class names for `.class` selector (can match multiple blocks).
    pub classes: Vec<String>,
    /// The content of this block.
    pub content: String,
    /// Document order for deterministic concatenation.
    pub order: usize,
    /// Whether this is a cut block (conceptually consumed but still available).
    pub is_cut: bool,
}

// ---------------------------------------------------------------------------
// Node Content Block (for node-based copy/paste storage)
// ---------------------------------------------------------------------------

/// Node-based content block for reactive paste resolution.
///
/// Stores `Arc<dyn Node>` instead of String, allowing paste to return
/// the stored node directly for streaming.
pub struct NodeContentBlock {
    /// Optional unique identifier for `#id` selector.
    pub id: Option<String>,
    /// Class names for `.class` selector.
    pub classes: Vec<String>,
    /// The node holding this block's content.
    pub node: Arc<dyn Node>,
    /// Document order for deterministic concatenation.
    pub order: usize,
    /// Whether this is a cut block.
    pub is_cut: bool,
}

// ---------------------------------------------------------------------------
// Substitution Definition
// ---------------------------------------------------------------------------

/// A substitution definition for text replacement.
#[derive(Debug, Clone)]
pub struct SubstitutionDef {
    /// The pattern to search for in content.
    pub pattern: String,
    /// The value to replace the pattern with.
    pub value: String,
    /// Whether to auto-generate case variants (PascalCase, snake_case, etc.)
    pub variants: bool,
}

// ---------------------------------------------------------------------------
// Feature Definition
// ---------------------------------------------------------------------------

/// A feature definition parsed from `<hick:feature>` tags.
#[derive(Debug, Clone)]
pub struct FeatureInfo {
    /// Feature name (identifier).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Names of features this feature requires (space-separated in XML).
    pub requires: Vec<String>,
}

// ---------------------------------------------------------------------------
// MultiDocumentState
// ---------------------------------------------------------------------------

/// Global state shared across a hick execution run.
pub struct MultiDocumentState {
    /// Element-keyed insertion points.
    element_outputs: Mutex<HashMap<usize, Arc<InsertionPoint>>>,
    /// File-path-keyed insertion points (for `<hick:file>`).
    file_outputs: Mutex<HashMap<String, Arc<InsertionPoint>>>,
    /// Copy blocks: id → content (legacy format for backwards compatibility).
    copy_blocks: Mutex<HashMap<String, String>>,
    /// Cut blocks: id → content (removed from output).
    cut_blocks: Mutex<HashMap<String, String>>,
    /// Content blocks: unified storage for copy/cut with class support.
    content_blocks: Mutex<Vec<ContentBlock>>,
    /// Node-based content blocks: stores Arc<dyn Node> for reactive paste.
    node_content_blocks: Mutex<Vec<NodeContentBlock>>,
    /// Counter for document order of content blocks.
    content_order_counter: AtomicUsize,
    /// Variables: name → value.
    variables: Mutex<HashMap<String, String>>,
    /// Substitution definitions: name → definition.
    substitutions: Mutex<HashMap<String, SubstitutionDef>>,
    /// Feature definitions: name → info.
    features: Mutex<HashMap<String, FeatureInfo>>,
    /// Enabled features (from CLI --features flag).
    enabled_features: Mutex<std::collections::HashSet<String>>,
    /// File exclusion patterns (glob patterns).
    exclusion_patterns: Mutex<Vec<String>>,
    /// Counter for generating unique container IDs.
    container_id_counter: std::sync::atomic::AtomicU64,
}

impl Default for MultiDocumentState {
    fn default() -> Self {
        Self {
            element_outputs: Mutex::new(HashMap::new()),
            file_outputs: Mutex::new(HashMap::new()),
            copy_blocks: Mutex::new(HashMap::new()),
            cut_blocks: Mutex::new(HashMap::new()),
            content_blocks: Mutex::new(Vec::new()),
            node_content_blocks: Mutex::new(Vec::new()),
            content_order_counter: AtomicUsize::new(0),
            variables: Mutex::new(HashMap::new()),
            substitutions: Mutex::new(HashMap::new()),
            features: Mutex::new(HashMap::new()),
            enabled_features: Mutex::new(std::collections::HashSet::new()),
            exclusion_patterns: Mutex::new(Vec::new()),
            container_id_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl MultiDocumentState {
    pub fn add_file_output(&self, path: String, value: Arc<InsertionPoint>) {
        self.file_outputs.lock().unwrap().insert(path, value);
    }

    pub fn add_element_output(&self, key: usize, value: Arc<InsertionPoint>) {
        self.element_outputs.lock().unwrap().insert(key, value);
    }

    pub fn try_get_element_output(&self, key: usize) -> Option<Arc<InsertionPoint>> {
        self.element_outputs.lock().unwrap().get(&key).cloned()
    }

    /// Register a `<hick:copy>` block with optional class attribute.
    ///
    /// If `class` is provided (space-separated class names), the content can
    /// be selected via `.classname` selector which concatenates all matching
    /// blocks in document order.
    pub fn register_copy(&self, id: String, content: String) {
        self.register_copy_with_class(id, None, content);
    }

    /// Register a `<hick:copy>` block with class support.
    pub fn register_copy_with_class(&self, id: String, class: Option<&str>, content: String) {
        // Legacy storage for backwards compatibility
        if !id.is_empty() {
            self.copy_blocks
                .lock()
                .unwrap()
                .insert(id.clone(), content.clone());
        }

        // New unified storage with class support
        let order = self
            .content_order_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let classes: Vec<String> = class
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let block = ContentBlock {
            id: if id.is_empty() { None } else { Some(id) },
            classes,
            content,
            order,
            is_cut: false,
        };

        self.content_blocks.lock().unwrap().push(block);
    }

    /// Register a `<hick:cut>` block.
    pub fn register_cut(&self, id: String, content: String) {
        self.register_cut_with_class(id, None, content);
    }

    /// Register a `<hick:cut>` block with class support.
    pub fn register_cut_with_class(&self, id: String, class: Option<&str>, content: String) {
        // Legacy storage for backwards compatibility
        if !id.is_empty() {
            self.cut_blocks
                .lock()
                .unwrap()
                .insert(id.clone(), content.clone());
        }

        // New unified storage with class support
        let order = self
            .content_order_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let classes: Vec<String> = class
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let block = ContentBlock {
            id: if id.is_empty() { None } else { Some(id) },
            classes,
            content,
            order,
            is_cut: true,
        };

        self.content_blocks.lock().unwrap().push(block);
    }

    /// Resolve a `<hick:paste select="...">` selector.
    ///
    /// Supports two selector types:
    /// - `#id` - CSS ID selector, returns single matching block
    /// - `.class` - CSS class selector, concatenates all matching blocks in document order
    ///
    /// An optional `separator` is inserted between blocks for class selectors.
    pub fn resolve_paste(&self, selector: &str, separator: Option<&str>) -> Option<String> {
        let selector = selector.trim();

        // Handle class selector (.classname)
        if let Some(class_name) = selector.strip_prefix('.') {
            return self.resolve_class_selector(class_name, separator);
        }

        // Handle ID selector (#id) - legacy path
        let id = selector.trim_start_matches('#');

        // Check cut blocks first (they're consumed but available via paste)
        if let Some(content) = self.cut_blocks.lock().unwrap().get(id) {
            return Some(content.clone());
        }

        // Then check copy blocks
        if let Some(content) = self.copy_blocks.lock().unwrap().get(id) {
            return Some(content.clone());
        }

        None
    }

    /// Resolve a class selector, concatenating all matching blocks in document order.
    ///
    /// An optional `separator` is inserted between blocks.
    fn resolve_class_selector(&self, class_name: &str, separator: Option<&str>) -> Option<String> {
        let blocks = self.content_blocks.lock().unwrap();

        // Find all blocks that have this class
        let mut matching: Vec<&ContentBlock> = blocks
            .iter()
            .filter(|b| b.classes.iter().any(|c| c == class_name))
            .collect();

        if matching.is_empty() {
            return None;
        }

        // Sort by document order
        matching.sort_by_key(|b| b.order);

        // Concatenate content with optional separator
        let result = if let Some(sep) = separator {
            matching
                .iter()
                .map(|b| b.content.as_str())
                .collect::<Vec<_>>()
                .join(sep)
        } else {
            matching.iter().map(|b| b.content.as_str()).collect()
        };
        Some(result)
    }

    // -----------------------------------------------------------------------
    // Node-based copy/paste storage
    // -----------------------------------------------------------------------

    /// Register a copy block as a node (for reactive paste).
    ///
    /// The `string_fallback` provides the converged string content for backward
    /// compat with string-based `resolve_paste()`.
    pub fn register_copy_node(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
    ) {
        self.register_node_block(id, class, node, string_fallback, false);
    }

    /// Register a cut block as a node (for reactive paste).
    pub fn register_cut_node(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
    ) {
        self.register_node_block(id, class, node, string_fallback, true);
    }

    /// Internal helper for registering both node-based and string-based storage.
    fn register_node_block(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
        is_cut: bool,
    ) {
        let order = self
            .content_order_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let classes: Vec<String> = class
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        // Store in node-based storage
        let node_block = NodeContentBlock {
            id: if id.is_empty() {
                None
            } else {
                Some(id.clone())
            },
            classes: classes.clone(),
            node,
            order,
            is_cut,
        };
        self.node_content_blocks.lock().unwrap().push(node_block);

        // Also populate string-based storage for backward compat
        if !id.is_empty() {
            if is_cut {
                self.cut_blocks
                    .lock()
                    .unwrap()
                    .insert(id.clone(), string_fallback.clone());
            } else {
                self.copy_blocks
                    .lock()
                    .unwrap()
                    .insert(id.clone(), string_fallback.clone());
            }
        }
        let block = ContentBlock {
            id: if id.is_empty() { None } else { Some(id) },
            classes,
            content: string_fallback,
            order,
            is_cut,
        };
        self.content_blocks.lock().unwrap().push(block);
    }

    /// Resolve a paste selector to a node.
    ///
    /// For `#id` selectors, returns the single matching node.
    /// For `.class` selectors, wraps matching nodes in an `InsertionPoint`.
    ///
    /// An optional `separator` is inserted between blocks for class selectors.
    pub fn resolve_paste_node(
        &self,
        selector: &str,
        separator: Option<&str>,
    ) -> Option<Arc<dyn Node>> {
        // A comma-separated list is one paste of several fragments, in the
        // order written. This used to be treated as a single id — so
        // `select="#a,#b"` looked for a fragment literally named `a,#b`,
        // found nothing, and wove an EMPTY section while `hick test` passed.
        // Silent, and in a chain of documents the silence lands where the
        // upstream requirements were supposed to appear.
        //
        // `hick_lang::fragment_matches` (the agent-side selector) has always
        // split on commas, so this also brings the two halves of the product
        // back into agreement about what a selector means.
        let parts: Vec<&str> = selector
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() > 1 {
            let ip = Arc::new(InsertionPoint::new());
            let mut any = false;
            for part in parts {
                let Some(node) = self.resolve_single_selector(part, separator) else {
                    continue;
                };
                if any && let Some(sep) = separator {
                    ip.add(Arc::new(StringNode::new(sep)) as Arc<dyn Node>);
                }
                ip.add(node);
                any = true;
            }
            // None, not an empty node: the caller reports "selector not
            // found", which is the whole point of noticing.
            return any.then_some(ip as Arc<dyn Node>);
        }

        self.resolve_single_selector(selector.trim(), separator)
    }

    /// One `#id` or `.class` selector.
    fn resolve_single_selector(
        &self,
        selector: &str,
        separator: Option<&str>,
    ) -> Option<Arc<dyn Node>> {
        let selector = selector.trim();

        if let Some(class_name) = selector.strip_prefix('.') {
            return self.resolve_class_node_selector(class_name, separator);
        }

        let id = selector.trim_start_matches('#');
        let blocks = self.node_content_blocks.lock().unwrap();
        blocks
            .iter()
            .find(|b| b.id.as_deref() == Some(id))
            .map(|b| b.node.clone())
    }

    /// Resolve a class selector to a node, wrapping multiple matches in an InsertionPoint.
    ///
    /// An optional `separator` is inserted between blocks.
    fn resolve_class_node_selector(
        &self,
        class_name: &str,
        separator: Option<&str>,
    ) -> Option<Arc<dyn Node>> {
        let blocks = self.node_content_blocks.lock().unwrap();

        let mut matching: Vec<&NodeContentBlock> = blocks
            .iter()
            .filter(|b| b.classes.iter().any(|c| c == class_name))
            .collect();

        if matching.is_empty() {
            return None;
        }

        matching.sort_by_key(|b| b.order);

        if matching.len() == 1 {
            return Some(matching[0].node.clone());
        }

        let ip = Arc::new(InsertionPoint::new());
        for (i, block) in matching.iter().enumerate() {
            if i > 0
                && let Some(sep) = separator
            {
                ip.add(Arc::new(StringNode::new(sep)) as Arc<dyn Node>);
            }
            ip.add(block.node.clone());
        }
        ip.close();
        Some(ip as Arc<dyn Node>)
    }

    /// Count the number of blocks matching a paste selector.
    ///
    /// For `#id` selectors, returns 0 or 1.
    /// For `.class` selectors, returns the count of blocks with that class.
    pub fn count_paste_matches(&self, selector: &str) -> usize {
        // Same list semantics as `resolve_paste_node`, so `min`/`max` count
        // what the paste will actually emit rather than what one selector
        // would have.
        let parts: Vec<&str> = selector
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() > 1 {
            return parts.iter().map(|p| self.count_paste_matches(p)).sum();
        }

        let selector = selector.trim();

        if let Some(class_name) = selector.strip_prefix('.') {
            let blocks = self.content_blocks.lock().unwrap();
            blocks
                .iter()
                .filter(|b| b.classes.iter().any(|c| c == class_name))
                .count()
        } else {
            let id = selector.trim_start_matches('#');
            let has_cut = self.cut_blocks.lock().unwrap().contains_key(id);
            let has_copy = self.copy_blocks.lock().unwrap().contains_key(id);
            if has_cut || has_copy { 1 } else { 0 }
        }
    }

    /// Get all file outputs, converging each to text or binary content.
    pub async fn get_files(&self) -> HashMap<String, FileContent> {
        let outputs = self.file_outputs.lock().unwrap().clone();
        let mut result = HashMap::new();

        for (path, insertion_point) in outputs {
            let contents = converge(insertion_point as Arc<dyn Node>, Context::default()).await;
            result.insert(
                path,
                contents.unwrap_or_else(|| FileContent::Text(String::new())),
            );
        }

        result
    }

    /// Get all file outputs with provenance maps, converging each to text content.
    ///
    /// For each `<hick:file>` output, returns the converged text and a
    /// [`ProvenanceMap`] mapping output byte ranges back to source origins.
    pub async fn get_files_with_provenance(&self) -> HashMap<String, (FileContent, ProvenanceMap)> {
        let outputs = self.file_outputs.lock().unwrap().clone();
        let mut result = HashMap::new();

        for (path, insertion_point) in outputs {
            let ip_node: std::sync::Arc<dyn Node> = insertion_point;
            match converge_with_provenance(ip_node.clone(), Context::default()).await {
                Some((text, prov_map)) => {
                    result.insert(path, (FileContent::Text(text), prov_map));
                }
                None => {
                    result.insert(
                        path,
                        (FileContent::Text(String::new()), ProvenanceMap::new()),
                    );
                }
            }
        }

        result
    }

    /// Register a `<hick:var>` declaration. Does not overwrite existing
    /// values (CLI params loaded first take precedence).
    pub fn register_var(&self, name: String, value: String) {
        let mut vars = self.variables.lock().unwrap();
        vars.entry(name).or_insert(value);
    }

    /// Set a CLI `--param` value. Always overwrites.
    pub fn set_param(&self, name: String, value: String) {
        self.variables.lock().unwrap().insert(name, value);
    }

    /// Resolve a variable by name.
    pub fn resolve_var(&self, name: &str) -> Option<String> {
        self.variables.lock().unwrap().get(name).cloned()
    }

    /// Register a `<hick:substitute>` definition.
    pub fn register_substitution(&self, name: String, def: SubstitutionDef) {
        self.substitutions.lock().unwrap().insert(name, def);
    }

    /// Get all substitution definitions.
    pub fn get_substitutions(&self) -> Vec<SubstitutionDef> {
        self.substitutions
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Register a `<hick:feature>` definition.
    pub fn register_feature(&self, info: FeatureInfo) {
        self.features
            .lock()
            .unwrap()
            .insert(info.name.clone(), info);
    }

    /// Get all feature definitions.
    pub fn get_features(&self) -> Vec<FeatureInfo> {
        self.features.lock().unwrap().values().cloned().collect()
    }

    /// Enable a feature (from CLI --features flag).
    pub fn enable_feature(&self, name: String) {
        self.enabled_features.lock().unwrap().insert(name);
    }

    /// Enable multiple features at once.
    pub fn enable_features(&self, names: impl IntoIterator<Item = String>) {
        let mut features = self.enabled_features.lock().unwrap();
        features.extend(names);
    }

    /// Check if a feature is enabled.
    pub fn is_feature_enabled(&self, name: &str) -> bool {
        self.enabled_features.lock().unwrap().contains(name)
    }

    /// Get all enabled feature names.
    pub fn get_enabled_features(&self) -> Vec<String> {
        self.enabled_features
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    /// Register an exclusion pattern (glob pattern for files to exclude from output).
    pub fn register_exclusion(&self, pattern: String) {
        self.exclusion_patterns.lock().unwrap().push(pattern);
    }

    /// Get all exclusion patterns.
    pub fn get_exclusion_patterns(&self) -> Vec<String> {
        self.exclusion_patterns.lock().unwrap().clone()
    }

    /// Generate a unique container ID for this session.
    pub fn next_container_id(&self) -> String {
        let id = self
            .container_id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("container-{id}")
    }
}

impl VariableResolver for MultiDocumentState {
    fn resolve(&self, name: &str) -> Option<String> {
        self.resolve_var(name)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::StringNode;

    #[test]
    fn copy_paste_roundtrip() {
        let state = MultiDocumentState::default();
        state.register_copy("version".to_string(), "3.12.0".to_string());

        let resolved = state.resolve_paste("#version", None);
        assert_eq!(resolved, Some("3.12.0".to_string()));
    }

    #[test]
    fn cut_available_via_paste() {
        let state = MultiDocumentState::default();
        state.register_cut("secret".to_string(), "hidden data".to_string());

        let resolved = state.resolve_paste("#secret", None);
        assert_eq!(resolved, Some("hidden data".to_string()));
    }

    #[test]
    fn paste_returns_none_for_missing() {
        let state = MultiDocumentState::default();
        assert_eq!(state.resolve_paste("#nonexistent", None), None);
    }

    #[tokio::test]
    async fn file_output_converges() {
        let state = MultiDocumentState::default();
        let ip = Arc::new(InsertionPoint::new());
        ip.add(Arc::new(StringNode::new("hello world")));
        ip.close();
        state.add_file_output("test.txt".to_string(), ip);

        let files = state.get_files().await;
        assert_eq!(*files.get("test.txt").unwrap(), *"hello world");
    }

    #[test]
    fn container_id_increments() {
        let state = MultiDocumentState::default();
        assert_eq!(state.next_container_id(), "container-0");
        assert_eq!(state.next_container_id(), "container-1");
        assert_eq!(state.next_container_id(), "container-2");
    }

    #[test]
    fn substitution_roundtrip() {
        let state = MultiDocumentState::default();
        state.register_substitution(
            "project_name".to_string(),
            SubstitutionDef {
                pattern: "FavoriteApp".to_string(),
                value: "MyProject".to_string(),
                variants: true,
            },
        );

        let subs = state.get_substitutions();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].pattern, "FavoriteApp");
        assert_eq!(subs[0].value, "MyProject");
        assert!(subs[0].variants);
    }

    // ---------------------------------------------------------------------------
    // Class-based selector tests
    // ---------------------------------------------------------------------------

    #[test]
    fn class_selector_single_block() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("imports"), "import foo;".to_string());

        let resolved = state.resolve_paste(".imports", None);
        assert_eq!(resolved, Some("import foo;".to_string()));
    }

    #[test]
    fn class_selector_multiple_blocks_concatenated() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("imports"), "import foo;".to_string());
        state.register_copy_with_class("".to_string(), Some("imports"), "import bar;".to_string());
        state.register_copy_with_class("".to_string(), Some("imports"), "import baz;".to_string());

        let resolved = state.resolve_paste(".imports", None);
        assert_eq!(
            resolved,
            Some("import foo;import bar;import baz;".to_string())
        );
    }

    #[test]
    fn class_selector_preserves_document_order() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("items"), "first".to_string());
        state.register_copy_with_class("".to_string(), Some("other"), "other".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "second".to_string());

        let resolved = state.resolve_paste(".items", None);
        assert_eq!(resolved, Some("firstsecond".to_string()));
    }

    #[test]
    fn class_selector_multiple_classes_on_block() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class(
            "".to_string(),
            Some("imports deps"),
            "import shared;".to_string(),
        );
        state.register_copy_with_class(
            "".to_string(),
            Some("imports"),
            "import local;".to_string(),
        );

        // Both selectors should find the shared import
        let imports = state.resolve_paste(".imports", None);
        assert_eq!(imports, Some("import shared;import local;".to_string()));

        let deps = state.resolve_paste(".deps", None);
        assert_eq!(deps, Some("import shared;".to_string()));
    }

    #[test]
    fn class_selector_with_id() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class(
            "header".to_string(),
            Some("sections"),
            "# Header".to_string(),
        );

        // Should be accessible by both ID and class
        let by_id = state.resolve_paste("#header", None);
        assert_eq!(by_id, Some("# Header".to_string()));

        let by_class = state.resolve_paste(".sections", None);
        assert_eq!(by_class, Some("# Header".to_string()));
    }

    #[test]
    fn class_selector_cut_blocks() {
        let state = MultiDocumentState::default();
        state.register_cut_with_class("".to_string(), Some("secrets"), "secret1;".to_string());
        state.register_cut_with_class("".to_string(), Some("secrets"), "secret2;".to_string());

        let resolved = state.resolve_paste(".secrets", None);
        assert_eq!(resolved, Some("secret1;secret2;".to_string()));
    }

    #[test]
    fn class_selector_missing_returns_none() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("imports"), "import foo;".to_string());

        let resolved = state.resolve_paste(".nonexistent", None);
        assert_eq!(resolved, None);
    }

    #[test]
    fn id_selector_still_works_without_class() {
        let state = MultiDocumentState::default();
        state.register_copy("version".to_string(), "1.0.0".to_string());

        let resolved = state.resolve_paste("#version", None);
        assert_eq!(resolved, Some("1.0.0".to_string()));
    }

    // -----------------------------------------------------------------------
    // Separator tests
    // -----------------------------------------------------------------------

    #[test]
    fn class_selector_with_separator() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("items"), "a".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "b".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "c".to_string());

        let resolved = state.resolve_paste(".items", Some(", "));
        assert_eq!(resolved, Some("a, b, c".to_string()));
    }

    #[test]
    fn class_selector_with_newline_separator() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("deps"), "serde".to_string());
        state.register_copy_with_class("".to_string(), Some("deps"), "tokio".to_string());

        let resolved = state.resolve_paste(".deps", Some("\n"));
        assert_eq!(resolved, Some("serde\ntokio".to_string()));
    }

    #[test]
    fn class_selector_separator_single_block_no_separator() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("solo"), "only".to_string());

        let resolved = state.resolve_paste(".solo", Some(", "));
        assert_eq!(resolved, Some("only".to_string()));
    }

    #[test]
    fn id_selector_ignores_separator() {
        let state = MultiDocumentState::default();
        state.register_copy("ver".to_string(), "1.0".to_string());

        // Separator has no effect on ID selectors
        let resolved = state.resolve_paste("#ver", Some(", "));
        assert_eq!(resolved, Some("1.0".to_string()));
    }

    // -----------------------------------------------------------------------
    // Count tests
    // -----------------------------------------------------------------------

    #[test]
    fn count_paste_matches_class() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("items"), "a".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "b".to_string());
        state.register_copy_with_class("".to_string(), Some("other"), "c".to_string());

        assert_eq!(state.count_paste_matches(".items"), 2);
        assert_eq!(state.count_paste_matches(".other"), 1);
        assert_eq!(state.count_paste_matches(".missing"), 0);
    }

    #[test]
    fn count_paste_matches_id() {
        let state = MultiDocumentState::default();
        state.register_copy("ver".to_string(), "1.0".to_string());

        assert_eq!(state.count_paste_matches("#ver"), 1);
        assert_eq!(state.count_paste_matches("#missing"), 0);
    }

    #[test]
    fn a_selector_list_counts_every_part() {
        // `select="#a,#b"` used to count as one missing fragment, so a
        // `min=2` gate on it could never be satisfied — and without a gate it
        // wove nothing at all.
        let state = MultiDocumentState::default();
        state.register_copy("a".to_string(), "ALPHA".to_string());
        state.register_copy("b".to_string(), "BETA".to_string());
        state.register_copy_with_class("".to_string(), Some("extra"), "GAMMA".to_string());

        assert_eq!(state.count_paste_matches("#a,#b"), 2);
        assert_eq!(state.count_paste_matches("#a, #b"), 2, "spaces are allowed");
        assert_eq!(state.count_paste_matches("#a,#missing"), 1);
        assert_eq!(state.count_paste_matches("#a,.extra"), 2, "mixed kinds");
        assert_eq!(
            state.count_paste_matches("#a"),
            1,
            "a single selector is unchanged"
        );
    }

    #[test]
    fn count_paste_matches_cut() {
        let state = MultiDocumentState::default();
        state.register_cut("secret".to_string(), "hidden".to_string());

        assert_eq!(state.count_paste_matches("#secret"), 1);
    }
}
