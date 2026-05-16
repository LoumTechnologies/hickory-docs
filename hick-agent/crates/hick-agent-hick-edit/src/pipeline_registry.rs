//! Registry mapping pipeline output files to their owning `.hick` sources.
//!
//! Scans `.hick` sources for `<hick:file path="...">` declarations and builds
//! a lookup table so the agent can decide whether a target file is
//! pipeline-owned (→ produce `hick:copy`/`hick:file` elements) or external
//! (→ fall back to a direct write with a warning).

use std::collections::HashMap;
use std::path::Path;

use crate::paste_slot_registry::PasteSlotRegistry;

/// Ownership record for one pipeline-managed output file.
#[derive(Debug, Clone)]
pub struct FileOwnership {
    /// Relative path of the `.hick` source that declares this output file.
    pub hick_source: String,
    /// Named `<hick:paste name="…"/>` slots available in that source file.
    /// Populated from [`PasteSlotRegistry::slots_for_file`].
    pub paste_slots: Vec<String>,
}

/// Maps output file paths → [`FileOwnership`] records.
///
/// Built by scanning all `.hick` sources for `<hick:file path="…">` elements.
/// The slots are taken from the same source file, so the agent can choose
/// an appropriate composition point without scanning the source itself.
#[derive(Debug, Default, Clone)]
pub struct PipelineRegistry {
    /// output_path → ownership info
    files: HashMap<String, FileOwnership>,
}

impl PipelineRegistry {
    /// Build a registry from in-memory `.hick` source content.
    pub fn from_sources(sources: &HashMap<String, String>) -> Self {
        let slot_registry = PasteSlotRegistry::from_sources(sources);

        let mut files = HashMap::new();
        for (hick_path, content) in sources {
            if let Ok(doc) = hick_lang::parse(content) {
                for file_tag in doc.find_tags("file") {
                    if let Some(output_path) = file_tag.get_attribute("path") {
                        let paste_slots = slot_registry
                            .slots_for_file(hick_path)
                            .to_vec();
                        files.insert(
                            output_path.to_string(),
                            FileOwnership {
                                hick_source: hick_path.clone(),
                                paste_slots,
                            },
                        );
                    }
                }
            }
        }

        Self { files }
    }

    /// Look up ownership info for an output file path.
    pub fn find_owner(&self, output_path: &str) -> Option<&FileOwnership> {
        self.files.get(output_path)
    }

    /// Whether the given output path is owned by the pipeline.
    pub fn is_owned(&self, output_path: &str) -> bool {
        self.files.contains_key(output_path)
    }

    /// Iterator over all pipeline-owned output paths.
    pub fn owned_paths(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }

    /// All ownership records.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &FileOwnership)> {
        self.files
            .iter()
            .map(|(k, v)| (k.as_str(), v))
    }

    /// Append a hick XML element (a `hick:copy` or `hick:file` fragment)
    /// to the owning `.hick` source file on disk, inserting it just before
    /// the closing `</…:doc>` tag.
    ///
    /// Returns `Err` if the file cannot be read/written or has no doc root.
    pub fn append_to_hick_source(
        project_dir: &Path,
        hick_source: &str,
        element: &str,
    ) -> anyhow::Result<()> {
        let path = project_dir.join(hick_source);
        let content = std::fs::read_to_string(&path)?;

        // Find the closing doc tag (e.g. `</hick:doc>` or `</h:doc>`).
        // The prefix is whatever comes before `:doc>`.
        let close_tag = if let Some(pos) = content.rfind(":doc>") {
            // Walk backwards to find `</`.
            let before = &content[..pos];
            if let Some(slash_pos) = before.rfind("</") {
                &content[slash_pos..pos + ":doc>".len()]
            } else {
                anyhow::bail!("no closing doc tag found in {hick_source}");
            }
        } else {
            anyhow::bail!("no closing doc tag found in {hick_source}");
        };

        let insert_pos = content.rfind(close_tag).unwrap();
        let mut new_content = content[..insert_pos].to_string();
        new_content.push('\n');
        new_content.push_str(element);
        new_content.push('\n');
        new_content.push_str(&content[insert_pos..]);

        std::fs::write(&path, new_content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sources(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn empty_sources_yields_empty_registry() {
        let reg = PipelineRegistry::from_sources(&HashMap::new());
        assert!(!reg.is_owned("src/main.rs"));
    }

    #[test]
    fn finds_file_path_from_hick_source() {
        let sources = make_sources(&[(
            "main.hick",
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="src/lib.rs">fn lib() {}</hick:file>
</hick:doc>"#,
        )]);
        let reg = PipelineRegistry::from_sources(&sources);
        assert!(reg.is_owned("src/lib.rs"));
        let owner = reg.find_owner("src/lib.rs").unwrap();
        assert_eq!(owner.hick_source, "main.hick");
    }

    #[test]
    fn finds_multiple_files_in_same_source() {
        let sources = make_sources(&[(
            "multi.hick",
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="a.py">pass</hick:file>
<hick:file path="b.py">pass</hick:file>
</hick:doc>"#,
        )]);
        let reg = PipelineRegistry::from_sources(&sources);
        assert!(reg.is_owned("a.py"));
        assert!(reg.is_owned("b.py"));
        assert_eq!(reg.find_owner("a.py").unwrap().hick_source, "multi.hick");
    }

    #[test]
    fn untracked_file_is_not_owned() {
        let sources = make_sources(&[(
            "main.hick",
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="src/lib.rs">fn lib() {}</hick:file>
</hick:doc>"#,
        )]);
        let reg = PipelineRegistry::from_sources(&sources);
        assert!(!reg.is_owned("src/main.rs"));
    }

    #[test]
    fn paste_slots_populated_from_same_source() {
        let sources = make_sources(&[(
            "src.hick",
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.py">
def foo(): pass
<hick:paste name="after-fn-foo"/>
</hick:file>
</hick:doc>"#,
        )]);
        let reg = PipelineRegistry::from_sources(&sources);
        let owner = reg.find_owner("out.py").unwrap();
        assert!(owner.paste_slots.contains(&"after-fn-foo".to_string()));
    }

    #[test]
    fn append_inserts_before_closing_doc_tag() {
        let dir = tempfile::tempdir().unwrap();
        let hick_path = dir.path().join("test.hick");
        std::fs::write(
            &hick_path,
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="a.py">pass</hick:file>
</hick:doc>"#,
        )
        .unwrap();

        PipelineRegistry::append_to_hick_source(
            dir.path(),
            "test.hick",
            r##"<hick:copy select="#footer">extra</hick:copy>"##,
        )
        .unwrap();

        let result = std::fs::read_to_string(&hick_path).unwrap();
        assert!(result.contains(r##"<hick:copy select="#footer">"##));
        assert!(result.ends_with("</hick:doc>"));
    }
}
