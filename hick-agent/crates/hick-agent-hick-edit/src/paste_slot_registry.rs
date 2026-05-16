//! Registry of named paste slots discovered in the current pipeline.

use std::collections::HashMap;

/// Indexes `<hick:paste>` slots so the agent can discover composition
/// points without scanning every source file on each request.
///
/// Tracks two distinct kinds of paste element:
/// - **Select slots** (`select="#id"`) — resolved copy/paste content blocks.
/// - **Named slots** (`name="after-fn-foo"`) — structural composition points
///   inserted by [`insert_paste_slots`][crate::paste_slot_inserter::insert_paste_slots].
#[derive(Debug, Default, Clone)]
pub struct PasteSlotRegistry {
    /// Select-based slots: selector (e.g. `"#utils"`) → hick source file path.
    slots: HashMap<String, String>,
    /// Name-based paste slots: slot name (e.g. `"after-fn-main"`) → hick source file path.
    named_slots: HashMap<String, String>,
    /// Reverse index: hick source file path → list of slot names defined in that file.
    by_file: HashMap<String, Vec<String>>,
}

impl PasteSlotRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Select-based (copy/paste) slots ──────────────────────────────────────

    /// Register a select-based paste slot by selector and source file.
    pub fn register(&mut self, selector: impl Into<String>, source_file: impl Into<String>) {
        self.slots.insert(selector.into(), source_file.into());
    }

    /// Look up the source file for a given paste slot selector.
    pub fn lookup(&self, selector: &str) -> Option<&str> {
        self.slots.get(selector).map(String::as_str)
    }

    /// All registered select-based slot selectors.
    pub fn selectors(&self) -> impl Iterator<Item = &str> {
        self.slots.keys().map(String::as_str)
    }

    // ── Named (structural composition) slots ─────────────────────────────────

    /// Register a named paste slot (from `name=` attribute) and its hick source file.
    pub fn register_named(&mut self, name: impl Into<String>, source_file: impl Into<String>) {
        let name = name.into();
        let source_file = source_file.into();
        self.by_file
            .entry(source_file.clone())
            .or_default()
            .push(name.clone());
        self.named_slots.insert(name, source_file);
    }

    /// Look up the hick source file that defines a named paste slot.
    pub fn lookup_named(&self, name: &str) -> Option<&str> {
        self.named_slots.get(name).map(String::as_str)
    }

    /// All named slot names defined in a given hick source file.
    pub fn slots_for_file(&self, hick_file: &str) -> &[String] {
        self.by_file
            .get(hick_file)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Named slots associated with a symbol name.
    ///
    /// Matches slots whose names follow the naming conventions:
    /// - `after-fn-{symbol}` — slot after a function/class/impl named `symbol`
    /// - `impl-{symbol}-body` — slot inside the body of `symbol`
    pub fn slots_for_symbol(&self, symbol: &str) -> Vec<&str> {
        let after_prefix = format!("after-fn-{symbol}");
        let body_prefix = format!("impl-{symbol}-body");
        self.named_slots
            .keys()
            .filter(|name| *name == &after_prefix || *name == &body_prefix)
            .map(String::as_str)
            .collect()
    }

    /// All named slot names across all files.
    pub fn named_slot_names(&self) -> impl Iterator<Item = &str> {
        self.named_slots.keys().map(String::as_str)
    }

    // ── Aggregate helpers ─────────────────────────────────────────────────────

    /// Build a registry by scanning hick source strings for paste slot declarations.
    ///
    /// Recognises both `select="…"` (copy/paste) and `name="…"` (structural) attributes.
    pub fn from_sources(sources: &HashMap<String, String>) -> Self {
        let mut registry = Self::new();
        for (path, content) in sources {
            let mut remaining = content.as_str();

            while let Some(idx) = remaining.find("hick:paste") {
                remaining = &remaining[idx + "hick:paste".len()..];

                // select="…" — copy/paste content block
                if let Some(sel_idx) = remaining.find("select=\"") {
                    let after = &remaining[sel_idx + "select=\"".len()..];
                    if let Some(end) = after.find('"') {
                        registry.register(after[..end].to_string(), path.clone());
                    }
                }

                // name="…" — structural composition slot
                if let Some(name_idx) = remaining.find("name=\"") {
                    let after = &remaining[name_idx + "name=\"".len()..];
                    if let Some(end) = after.find('"') {
                        registry.register_named(after[..end].to_string(), path.clone());
                    }
                }
            }
        }
        registry
    }

    /// Total number of select-based registered slots.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Whether the select-based slot registry is empty.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Total number of named slots.
    pub fn named_len(&self) -> usize {
        self.named_slots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut reg = PasteSlotRegistry::new();
        reg.register("#utils", "lib.hick");
        assert_eq!(reg.lookup("#utils"), Some("lib.hick"));
        assert_eq!(reg.lookup("#missing"), None);
    }

    #[test]
    fn from_sources_finds_slots() {
        let mut sources = HashMap::new();
        sources.insert(
            "main.hick".to_string(),
            r##"<hick:paste select="#utils"></hick:paste>"##.to_string(),
        );
        let reg = PasteSlotRegistry::from_sources(&sources);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.lookup("#utils"), Some("main.hick"));
    }

    #[test]
    fn from_sources_empty_when_no_slots() {
        let mut sources = HashMap::new();
        sources.insert("main.hick".to_string(), "<hick:doc></hick:doc>".to_string());
        let reg = PasteSlotRegistry::from_sources(&sources);
        assert!(reg.is_empty());
    }

    #[test]
    fn register_named_and_lookup() {
        let mut reg = PasteSlotRegistry::new();
        reg.register_named("after-fn-main", "src.hick");
        assert_eq!(reg.lookup_named("after-fn-main"), Some("src.hick"));
        assert_eq!(reg.lookup_named("missing"), None);
    }

    #[test]
    fn slots_for_file_returns_names() {
        let mut reg = PasteSlotRegistry::new();
        reg.register_named("after-fn-main", "src.hick");
        reg.register_named("module-footer", "src.hick");
        reg.register_named("after-fn-greet", "other.hick");

        let mut names = reg.slots_for_file("src.hick").to_vec();
        names.sort();
        assert_eq!(names, vec!["after-fn-main", "module-footer"]);

        assert_eq!(reg.slots_for_file("missing.hick"), &[] as &[String]);
    }

    #[test]
    fn slots_for_symbol() {
        let mut reg = PasteSlotRegistry::new();
        reg.register_named("after-fn-Foo", "a.hick");
        reg.register_named("impl-Foo-body", "a.hick");
        reg.register_named("after-fn-Bar", "a.hick");

        let mut slots = reg.slots_for_symbol("Foo");
        slots.sort();
        assert_eq!(slots, vec!["after-fn-Foo", "impl-Foo-body"]);

        assert!(reg.slots_for_symbol("missing").is_empty());
    }

    #[test]
    fn from_sources_finds_named_slots() {
        let mut sources = HashMap::new();
        sources.insert(
            "src.hick".to_string(),
            r#"<hick:paste name="after-fn-main"/><hick:paste name="module-footer"/>"#.to_string(),
        );
        let reg = PasteSlotRegistry::from_sources(&sources);
        assert_eq!(reg.named_len(), 2);
        assert_eq!(reg.lookup_named("after-fn-main"), Some("src.hick"));
        assert_eq!(reg.lookup_named("module-footer"), Some("src.hick"));
    }

    #[test]
    fn from_sources_finds_both_select_and_named() {
        let mut sources = HashMap::new();
        sources.insert(
            "mixed.hick".to_string(),
            r##"<hick:paste select="#imports"/><hick:paste name="after-fn-foo"/>"##.to_string(),
        );
        let reg = PasteSlotRegistry::from_sources(&sources);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.named_len(), 1);
        assert_eq!(reg.lookup("#imports"), Some("mixed.hick"));
        assert_eq!(reg.lookup_named("after-fn-foo"), Some("mixed.hick"));
    }
}
