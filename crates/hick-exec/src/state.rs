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
    /// Where in the source this block was declared, as `file:offset`.
    ///
    /// The identity that lets the early registration and the render pass
    /// register the SAME block without it counting twice. Two genuinely
    /// separate copies with identical text keep separate keys and stay two —
    /// collapsing those is what `distinct` is for, and it is the collector's
    /// call, not this one's.
    pub origin_key: Option<String>,
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
    /// Where in the source this block was declared — see [`ContentBlock`].
    pub origin_key: Option<String>,
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
// Class selectors
// ---------------------------------------------------------------------------

/// The class names a class selector requires — ALL of them.
///
/// CSS has always read `.a.b` as "carries both", and hick's selectors are
/// CSS-shaped, so this is the compound form rather than a new grammar. It is
/// what replaces CodegenBot's name/value caret tags: matching on two
/// properties at once is the load-bearing half, and bare names carry it.
///
/// Returns `None` for anything that is not a class selector, and for a bare
/// `.` — which names no class and must match nothing rather than everything.
pub(crate) fn required_classes(selector: &str) -> Option<Vec<&str>> {
    let rest = selector.trim().strip_prefix('.')?;
    let names: Vec<&str> = rest.split('.').filter(|s| !s.is_empty()).collect();
    if names.is_empty() { None } else { Some(names) }
}

/// Whether a block carries every class a selector requires.
pub(crate) fn has_all_classes(classes: &[String], required: &[&str]) -> bool {
    required
        .iter()
        .all(|need| classes.iter().any(|have| have == need))
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
    /// Output-volume files, as this run produced them, keyed by the path they
    /// will be written to.
    ///
    /// Registered before the weave so a `<hick:sample>` can show bytes from
    /// the run that is happening rather than the one before it. Reading them
    /// from disk instead would make the first run of a document show
    /// "not generated yet" for a file it had just generated.
    produced_files: Mutex<HashMap<String, String>>,
    /// Cardinality violations a `<hick:paste min=/max=>` found.
    ///
    /// Collected rather than returned because the caller that renders a
    /// `<hick:file>` body logs a handler error and carries on — so the
    /// document that asked to be told wove an empty file and exited 0. The
    /// pipeline reads these at the end and refuses.
    paste_failures: Mutex<Vec<String>>,
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
            produced_files: Mutex::new(HashMap::new()),
            paste_failures: Mutex::new(Vec::new()),
        }
    }
}

impl MultiDocumentState {
    /// Record a `min=`/`max=` violation for the pipeline to refuse on.
    pub fn record_paste_failure(&self, message: String) {
        let mut failures = self.paste_failures.lock().unwrap();
        // The same paste is processed on more than one pass, and one mistake
        // should be reported once.
        if !failures.contains(&message) {
            failures.push(message);
        }
    }

    /// Every cardinality violation this run found.
    pub fn paste_failures(&self) -> Vec<String> {
        self.paste_failures.lock().unwrap().clone()
    }

    /// Forget the violations found so far.
    ///
    /// Called before each pass over the documents, because a later round may
    /// satisfy a gate an earlier one could not: a cell that writes fragments
    /// has not run yet when the first pass reads them. Only the last pass's
    /// failures are real.
    pub fn clear_paste_failures(&self) {
        self.paste_failures.lock().unwrap().clear();
    }

    /// Files this run produced whose name ends in `.hick`.
    ///
    /// The rounds loop reads these so a cell can write fragments into an
    /// output VOLUME, which is where a generator's files actually land —
    /// `get_files` sees only `<hick:file>` outputs, and volume bytes are
    /// merged in after the weave has already happened.
    pub fn produced_hick_files(&self) -> Vec<(String, String)> {
        self.produced_files
            .lock()
            .unwrap()
            .iter()
            .filter(|(path, _)| path.ends_with(".hick"))
            .map(|(path, text)| (path.clone(), text.clone()))
            .collect()
    }

    /// Record a file this run produced, so the weave can show part of it.
    pub fn register_produced_file(&self, path: &str, text: &str) {
        self.produced_files
            .lock()
            .unwrap()
            .insert(path.to_string(), text.to_string());
    }

    /// A file this run produced, if it produced one by that name.
    pub fn produced_file(&self, path: &str) -> Option<String> {
        self.produced_files.lock().unwrap().get(path).cloned()
    }

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
        self.register_copy_keyed(id, class, content, None);
    }

    /// Register a `<hick:copy>` block, identified by where it was declared.
    ///
    /// `origin_key` is what makes registering the same block twice idempotent
    /// — see [`ContentBlock::origin_key`]. `None` always pushes, which is what
    /// a caller with no source position (a test, a synthesised fragment) wants.
    pub fn register_copy_keyed(
        &self,
        id: String,
        class: Option<&str>,
        content: String,
        origin_key: Option<String>,
    ) {
        // Legacy storage for backwards compatibility
        if !id.is_empty() {
            self.copy_blocks
                .lock()
                .unwrap()
                .insert(id.clone(), content.clone());
        }

        let classes: Vec<String> = class
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let mut blocks = self.content_blocks.lock().unwrap();

        // Same declaration, registered again: replace in place and keep the
        // original order, so early registration does not push a block to the
        // front and the render pass a duplicate to the back.
        if let Some(key) = &origin_key
            && let Some(existing) = blocks
                .iter_mut()
                .find(|b| b.origin_key.as_deref() == Some(key.as_str()))
        {
            existing.id = if id.is_empty() { None } else { Some(id) };
            existing.classes = classes;
            existing.content = content;
            existing.is_cut = false;
            return;
        }

        let order = self
            .content_order_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        blocks.push(ContentBlock {
            id: if id.is_empty() { None } else { Some(id) },
            classes,
            content,
            order,
            is_cut: false,
            origin_key,
        });
    }

    /// Make a literal copy block resolvable **before** the render pass runs.
    ///
    /// Copy blocks are normally registered by the copy handler while the
    /// document is being rendered — which is after every cell has executed.
    /// A `<hick:paste>` used as a cell's stdin is therefore evaluated against
    /// an empty registry, and resolved to nothing: the cell ran with empty
    /// input and no error, which is the worst way for anything to fail.
    ///
    /// This used to fill only the id map, because `content_blocks` is a list
    /// and pushing twice made one copy count as two for `.class` and for
    /// `min=`/`max=`. The cost was an asymmetry nobody could have guessed:
    /// a copy nested inside another tag — the copy handler only reaches
    /// top-level ones — resolved by `#id` and was invisible to `.class`.
    /// `origin_key` removes the reason for the asymmetry, so classes are
    /// registered here too.
    pub fn pre_register_copy_text(
        &self,
        id: &str,
        class: Option<&str>,
        content: &str,
        origin_key: Option<String>,
    ) {
        if id.is_empty() && class.is_none() {
            return;
        }
        if !id.is_empty() {
            self.copy_blocks
                .lock()
                .unwrap()
                .entry(id.to_string())
                .or_insert_with(|| content.to_string());
        }
        self.register_copy_keyed(id.to_string(), class, content.to_string(), origin_key);
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
            origin_key: None,
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
    /// `distinct` collapses matches whose text is identical, keeping the first.
    pub fn resolve_paste(
        &self,
        selector: &str,
        separator: Option<&str>,
        distinct: bool,
    ) -> Option<String> {
        let selector = selector.trim();

        // Handle class selector (.classname, or .a.b for all of them)
        if selector.starts_with('.') {
            return self.resolve_class_selector(selector, separator, distinct);
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
    fn resolve_class_selector(
        &self,
        selector: &str,
        separator: Option<&str>,
        distinct: bool,
    ) -> Option<String> {
        let required = required_classes(selector)?;
        let blocks = self.content_blocks.lock().unwrap();

        // Find all blocks that have EVERY class the selector requires
        let mut matching: Vec<&ContentBlock> = blocks
            .iter()
            .filter(|b| has_all_classes(&b.classes, &required))
            .collect();

        if matching.is_empty() {
            return None;
        }

        // Sort by document order
        matching.sort_by_key(|b| b.order);

        if distinct {
            let mut seen = std::collections::HashSet::new();
            matching.retain(|b| seen.insert(b.content.clone()));
        }

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
        self.register_node_block(id, class, node, string_fallback, false, None);
    }

    /// Register a copy block as a node, identified by where it was declared.
    ///
    /// See [`ContentBlock::origin_key`]: this is what lets the early pass and
    /// the render pass register one declaration without it counting twice.
    pub fn register_copy_node_keyed(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
        origin_key: Option<String>,
    ) {
        self.register_node_block(id, class, node, string_fallback, false, origin_key);
    }

    /// Register a cut block as a node (for reactive paste).
    pub fn register_cut_node(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
    ) {
        self.register_node_block(id, class, node, string_fallback, true, None);
    }

    /// Register a cut block as a node, identified by where it was declared.
    pub fn register_cut_node_keyed(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
        origin_key: Option<String>,
    ) {
        self.register_node_block(id, class, node, string_fallback, true, origin_key);
    }

    /// Internal helper for registering both node-based and string-based storage.
    fn register_node_block(
        &self,
        id: String,
        class: Option<&str>,
        node: Arc<dyn Node>,
        string_fallback: String,
        is_cut: bool,
        origin_key: Option<String>,
    ) {
        let classes: Vec<String> = class
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        // The order this block already holds if it has been registered
        // before, so re-registering a declaration keeps its place in the
        // document rather than jumping to the end.
        let existing_order = origin_key.as_deref().and_then(|key| {
            self.node_content_blocks
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.origin_key.as_deref() == Some(key))
                .map(|b| b.order)
        });
        let order = existing_order.unwrap_or_else(|| {
            self.content_order_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        });

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
            origin_key: origin_key.clone(),
        };
        {
            let mut blocks = self.node_content_blocks.lock().unwrap();
            match origin_key
                .as_deref()
                .and_then(|key| blocks.iter().position(|b| b.origin_key.as_deref() == Some(key)))
            {
                Some(at) => blocks[at] = node_block,
                None => blocks.push(node_block),
            }
        }

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
            origin_key: origin_key.clone(),
        };
        {
            let mut blocks = self.content_blocks.lock().unwrap();
            match origin_key
                .as_deref()
                .and_then(|key| blocks.iter().position(|b| b.origin_key.as_deref() == Some(key)))
            {
                Some(at) => blocks[at] = block,
                None => blocks.push(block),
            }
        }
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
        distinct: bool,
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
            // `distinct` spans the whole paste, not each comma part: two
            // selectors that both match the same fragment are exactly the
            // overlap somebody wrote `distinct` to collapse.
            let mut seen = std::collections::HashSet::new();
            for part in parts {
                let Some(node) = self.resolve_single_selector(part, separator, distinct) else {
                    continue;
                };
                if distinct
                    && let Some(text) = node.as_string_value()
                    && !seen.insert(text.to_string())
                {
                    continue;
                }
                if any && let Some(sep) = separator {
                    ip.add(Arc::new(StringNode::new(sep)) as Arc<dyn Node>);
                }
                ip.add(node);
                any = true;
            }
            ip.close();
            // None, not an empty node: the caller reports "selector not
            // found", which is the whole point of noticing.
            return any.then_some(ip as Arc<dyn Node>);
        }

        self.resolve_single_selector(selector.trim(), separator, distinct)
    }

    /// One `#id` or `.class` (or `.a.b`) selector.
    fn resolve_single_selector(
        &self,
        selector: &str,
        separator: Option<&str>,
        distinct: bool,
    ) -> Option<Arc<dyn Node>> {
        let selector = selector.trim();

        if selector.starts_with('.') {
            return self.resolve_class_node_selector(selector, separator, distinct);
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
        selector: &str,
        separator: Option<&str>,
        distinct: bool,
    ) -> Option<Arc<dyn Node>> {
        let required = required_classes(selector)?;
        let blocks = self.node_content_blocks.lock().unwrap();

        let mut matching: Vec<&NodeContentBlock> = blocks
            .iter()
            .filter(|b| has_all_classes(&b.classes, &required))
            .collect();

        if matching.is_empty() {
            return None;
        }

        matching.sort_by_key(|b| b.order);

        if distinct {
            // First wins, so the surviving bytes belong to the earliest
            // contributor and the ribbon points somewhere stable. A node with
            // no settled text yet cannot be compared, so it is kept: dropping
            // it would be guessing.
            let mut seen = std::collections::HashSet::new();
            matching.retain(|b| match b.node.as_string_value() {
                Some(text) => seen.insert(text.to_string()),
                None => true,
            });
        }

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
    /// For `.class` selectors, returns the count of blocks carrying every
    /// class the selector names. `distinct` counts what survives the collapse,
    /// because `min`/`max` gate what the paste EMITS — a `min="2"` satisfied
    /// by the same line contributed twice was never satisfied.
    pub fn count_paste_matches(&self, selector: &str, distinct: bool) -> usize {
        // Same list semantics as `resolve_paste_node`, so `min`/`max` count
        // what the paste will actually emit rather than what one selector
        // would have.
        let parts: Vec<&str> = selector
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() > 1 {
            if !distinct {
                return parts
                    .iter()
                    .map(|p| self.count_paste_matches(p, false))
                    .sum();
            }
            // Distinct spans the whole paste, so the parts have to be counted
            // together rather than summed.
            let mut seen = std::collections::HashSet::new();
            for part in &parts {
                self.collect_match_texts(part, &mut seen);
            }
            return seen.len();
        }

        let selector = selector.trim();

        if selector.starts_with('.') {
            let Some(required) = required_classes(selector) else {
                return 0;
            };
            let blocks = self.content_blocks.lock().unwrap();
            let matching = blocks
                .iter()
                .filter(|b| has_all_classes(&b.classes, &required));
            if distinct {
                let mut seen = std::collections::HashSet::new();
                return matching.filter(|b| seen.insert(b.content.clone())).count();
            }
            matching.count()
        } else {
            let id = selector.trim_start_matches('#');
            let has_cut = self.cut_blocks.lock().unwrap().contains_key(id);
            let has_copy = self.copy_blocks.lock().unwrap().contains_key(id);
            if has_cut || has_copy { 1 } else { 0 }
        }
    }

    /// The distinct texts one selector part contributes, added to `seen`.
    fn collect_match_texts(&self, selector: &str, seen: &mut std::collections::HashSet<String>) {
        let selector = selector.trim();
        if selector.starts_with('.') {
            let Some(required) = required_classes(selector) else {
                return;
            };
            let blocks = self.content_blocks.lock().unwrap();
            for block in blocks
                .iter()
                .filter(|b| has_all_classes(&b.classes, &required))
            {
                seen.insert(block.content.clone());
            }
            return;
        }
        let id = selector.trim_start_matches('#');
        if let Some(content) = self.cut_blocks.lock().unwrap().get(id) {
            seen.insert(content.clone());
            return;
        }
        if let Some(content) = self.copy_blocks.lock().unwrap().get(id) {
            seen.insert(content.clone());
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

        let resolved = state.resolve_paste("#version", None, false);
        assert_eq!(resolved, Some("3.12.0".to_string()));
    }

    #[test]
    fn cut_available_via_paste() {
        let state = MultiDocumentState::default();
        state.register_cut("secret".to_string(), "hidden data".to_string());

        let resolved = state.resolve_paste("#secret", None, false);
        assert_eq!(resolved, Some("hidden data".to_string()));
    }

    #[test]
    fn paste_returns_none_for_missing() {
        let state = MultiDocumentState::default();
        assert_eq!(state.resolve_paste("#nonexistent", None, false), None);
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

        let resolved = state.resolve_paste(".imports", None, false);
        assert_eq!(resolved, Some("import foo;".to_string()));
    }

    #[test]
    fn class_selector_multiple_blocks_concatenated() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("imports"), "import foo;".to_string());
        state.register_copy_with_class("".to_string(), Some("imports"), "import bar;".to_string());
        state.register_copy_with_class("".to_string(), Some("imports"), "import baz;".to_string());

        let resolved = state.resolve_paste(".imports", None, false);
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

        let resolved = state.resolve_paste(".items", None, false);
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
        let imports = state.resolve_paste(".imports", None, false);
        assert_eq!(imports, Some("import shared;import local;".to_string()));

        let deps = state.resolve_paste(".deps", None, false);
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
        let by_id = state.resolve_paste("#header", None, false);
        assert_eq!(by_id, Some("# Header".to_string()));

        let by_class = state.resolve_paste(".sections", None, false);
        assert_eq!(by_class, Some("# Header".to_string()));
    }

    #[test]
    fn class_selector_cut_blocks() {
        let state = MultiDocumentState::default();
        state.register_cut_with_class("".to_string(), Some("secrets"), "secret1;".to_string());
        state.register_cut_with_class("".to_string(), Some("secrets"), "secret2;".to_string());

        let resolved = state.resolve_paste(".secrets", None, false);
        assert_eq!(resolved, Some("secret1;secret2;".to_string()));
    }

    #[test]
    fn class_selector_missing_returns_none() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("imports"), "import foo;".to_string());

        let resolved = state.resolve_paste(".nonexistent", None, false);
        assert_eq!(resolved, None);
    }

    #[test]
    fn id_selector_still_works_without_class() {
        let state = MultiDocumentState::default();
        state.register_copy("version".to_string(), "1.0.0".to_string());

        let resolved = state.resolve_paste("#version", None, false);
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

        let resolved = state.resolve_paste(".items", Some(", "), false);
        assert_eq!(resolved, Some("a, b, c".to_string()));
    }

    #[test]
    fn class_selector_with_newline_separator() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("deps"), "serde".to_string());
        state.register_copy_with_class("".to_string(), Some("deps"), "tokio".to_string());

        let resolved = state.resolve_paste(".deps", Some("\n"), false);
        assert_eq!(resolved, Some("serde\ntokio".to_string()));
    }

    #[test]
    fn class_selector_separator_single_block_no_separator() {
        let state = MultiDocumentState::default();
        state.register_copy_with_class("".to_string(), Some("solo"), "only".to_string());

        let resolved = state.resolve_paste(".solo", Some(", "), false);
        assert_eq!(resolved, Some("only".to_string()));
    }

    #[test]
    fn id_selector_ignores_separator() {
        let state = MultiDocumentState::default();
        state.register_copy("ver".to_string(), "1.0".to_string());

        // Separator has no effect on ID selectors
        let resolved = state.resolve_paste("#ver", Some(", "), false);
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

        assert_eq!(state.count_paste_matches(".items", false), 2);
        assert_eq!(state.count_paste_matches(".other", false), 1);
        assert_eq!(state.count_paste_matches(".missing", false), 0);
    }

    #[test]
    fn count_paste_matches_id() {
        let state = MultiDocumentState::default();
        state.register_copy("ver".to_string(), "1.0".to_string());

        assert_eq!(state.count_paste_matches("#ver", false), 1);
        assert_eq!(state.count_paste_matches("#missing", false), 0);
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

        assert_eq!(state.count_paste_matches("#a,#b", false), 2);
        assert_eq!(state.count_paste_matches("#a, #b", false), 2, "spaces are allowed");
        assert_eq!(state.count_paste_matches("#a,#missing", false), 1);
        assert_eq!(state.count_paste_matches("#a,.extra", false), 2, "mixed kinds");
        assert_eq!(
            state.count_paste_matches("#a", false),
            1,
            "a single selector is unchanged"
        );
    }

    #[test]
    fn count_paste_matches_cut() {
        let state = MultiDocumentState::default();
        state.register_cut("secret".to_string(), "hidden".to_string());

        assert_eq!(state.count_paste_matches("#secret", false), 1);
    }
}
