//! Registry of named paste slots discovered in the current pipeline.

use std::collections::HashMap;

/// Indexes named `<hick:paste>` slots so the agent can discover composition
/// points without scanning every source file on each request.
#[derive(Debug, Default, Clone)]
pub struct PasteSlotRegistry {
    /// Maps slot selector (e.g. `"#utils"`) to the source file that defines it.
    slots: HashMap<String, String>,
}

impl PasteSlotRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a paste slot by selector and source file.
    pub fn register(&mut self, selector: impl Into<String>, source_file: impl Into<String>) {
        self.slots.insert(selector.into(), source_file.into());
    }

    /// Look up the source file for a given paste slot selector.
    pub fn lookup(&self, selector: &str) -> Option<&str> {
        self.slots.get(selector).map(String::as_str)
    }

    /// All registered slot selectors.
    pub fn selectors(&self) -> impl Iterator<Item = &str> {
        self.slots.keys().map(String::as_str)
    }

    /// Build a registry by scanning hick sources for `<hick:paste select="…">` patterns.
    pub fn from_sources(sources: &HashMap<String, String>) -> Self {
        let mut registry = Self::new();
        for (path, content) in sources {
            let mut remaining = content.as_str();
            while let Some(idx) = remaining.find("hick:paste") {
                remaining = &remaining[idx + "hick:paste".len()..];
                if let Some(sel_idx) = remaining.find("select=\"") {
                    let after = &remaining[sel_idx + "select=\"".len()..];
                    if let Some(end) = after.find('"') {
                        registry.register(after[..end].to_string(), path.clone());
                    }
                }
            }
        }
        registry
    }

    /// Number of registered slots.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
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
}
