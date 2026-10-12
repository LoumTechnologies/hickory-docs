//! Document state management for hick files open in the editor.

use std::collections::{BTreeMap, HashMap};

use hick_lang::{HickDocument, HickNode};

use crate::virtual_file::{self, VirtualFile};

/// State for a single open .md document.
#[derive(Debug, Clone)]
pub struct HickDocumentState {
    /// The parsed document.
    pub doc: HickDocument,
    /// Copy registry: id -> resolved text content.
    pub copy_registry: HashMap<String, String>,
    /// Generated virtual files.
    pub virtual_files: Vec<VirtualFile>,
    pub evaluated_project: Option<hick_lang::EvaluatedProject>,
    pub project_sources: BTreeMap<String, String>,
    pub project_entry: Option<String>,
    pub project_maps: BTreeMap<String, crate::position_map::PositionMap>,
    pub selected_project: Option<hick_lang::ProjectTree>,
}

impl HickDocumentState {
    pub fn from_project(
        entry: &str,
        sources: &BTreeMap<String, String>,
    ) -> Result<Self, hick_lang::ParseError> {
        Self::from_project_at(
            entry,
            sources,
            &hick_lang::ProjectTree::default(),
            &crate::project::ProjectEndpoint::New,
        )
    }
    pub fn from_project_at(
        entry: &str,
        sources: &BTreeMap<String, String>,
        input: &hick_lang::ProjectTree,
        endpoint: &crate::project::ProjectEndpoint,
    ) -> Result<Self, hick_lang::ParseError> {
        let error = |message| hick_lang::ParseError::Syntax { line: 1, message };
        let source = sources
            .get(entry)
            .ok_or_else(|| error(format!("Missing source {entry}")))?;
        let doc = hick_lang::parse(source)?;
        let project = hick_lang::evaluate_project(entry, sources, input).map_err(error)?;
        let tree = crate::project::endpoint(&project, endpoint)
            .map_err(error)?
            .clone();
        let virtual_files = crate::project::virtual_files(&tree, sources, entry).map_err(error)?;
        let source_map = std::sync::Arc::new(sources.clone());
        let mut project_maps = BTreeMap::new();
        for file in &virtual_files {
            project_maps.insert(
                file.path.clone(),
                crate::position_map::PositionMap::for_project(
                    tree.get(&file.path).unwrap(),
                    source_map.clone(),
                    entry,
                )
                .map_err(error)?,
            );
        }
        Ok(Self {
            doc,
            copy_registry: HashMap::new(),
            virtual_files,
            evaluated_project: Some(project),
            project_sources: sources.clone(),
            project_entry: Some(entry.into()),
            project_maps,
            selected_project: Some(tree),
        })
    }
    /// Ordinary documents retain their established literal virtual-file path.
    pub fn from_source(source: &str) -> Result<Self, hick_lang::ParseError> {
        let doc = hick_lang::parse(source)?;
        if !doc.find_tags("project").is_empty() {
            return Self::from_project(
                "project.md",
                &BTreeMap::from([("project.md".into(), source.into())]),
            );
        }
        if !doc.find_tags("patch").is_empty() {
            return Err(hick_lang::ParseError::Syntax {
                line: 1,
                message: "Patch requires a declared project closure".into(),
            });
        }
        let copy_registry = build_copy_registry(&doc);
        let virtual_files = virtual_file::build_virtual_files(&doc, &copy_registry);
        Ok(Self {
            doc,
            copy_registry,
            virtual_files,
            evaluated_project: None,
            project_sources: BTreeMap::new(),
            project_entry: None,
            project_maps: BTreeMap::new(),
            selected_project: None,
        })
    }
}

/// Recursively walk the document and collect copy definitions (tags named "copy"
/// with an `id` attribute) into a map of id -> dedented text content.
fn build_copy_registry(doc: &HickDocument) -> HashMap<String, String> {
    let mut registry = HashMap::new();
    walk_for_copies(&doc.nodes, &mut registry);
    registry
}

fn walk_for_copies(nodes: &[HickNode], registry: &mut HashMap<String, String>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "copy"
                && let Some(id) = tag.get_attribute("id")
            {
                let text = tag.text_content();
                let dedented = hick_lang::dedent(&text, tag.source_column);
                registry.insert(id.to_string(), dedented);
            }
            walk_for_copies(&tag.children, registry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::{HickNode, HickTag};

    fn make_doc(nodes: Vec<HickNode>) -> HickDocument {
        HickDocument {
            nodes,
            source: String::new(),
            prefix: "hick".to_string(),
            weave_path: None,
            frontmatter: None,
            volatile: false,
            span_files: Vec::new(),
            root_tag: None,
        }
    }

    #[test]
    fn build_copy_registry_simple() {
        let copy_tag = HickTag {
            name: "copy".to_string(),
            attributes: vec![("id".to_string(), "greeting".to_string())],
            children: vec![HickNode::Text("hello world".to_string(), None)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(copy_tag)]);
        let reg = build_copy_registry(&doc);
        assert_eq!(reg.get("greeting").unwrap(), "hello world");
    }

    #[test]
    fn build_copy_registry_nested() {
        let copy_tag = HickTag {
            name: "copy".to_string(),
            attributes: vec![("id".to_string(), "inner".to_string())],
            children: vec![HickNode::Text("inner text".to_string(), None)],
            self_closing: false,
            source_line: 2,
            source_column: 0,
            source_span: None,
            close_span: None,
        };

        let wrapper = HickTag {
            name: "section".to_string(),
            attributes: vec![],
            children: vec![HickNode::Tag(copy_tag)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(wrapper)]);
        let reg = build_copy_registry(&doc);
        assert_eq!(reg.get("inner").unwrap(), "inner text");
    }

    #[test]
    fn build_copy_registry_ignores_copy_without_id() {
        let copy_tag = HickTag {
            name: "copy".to_string(),
            attributes: vec![("select".to_string(), "#other".to_string())],
            children: vec![],
            self_closing: true,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(copy_tag)]);
        let reg = build_copy_registry(&doc);
        assert!(reg.is_empty());
    }

    #[test]
    fn build_copy_registry_multiple() {
        let copy1 = HickTag {
            name: "copy".to_string(),
            attributes: vec![("id".to_string(), "a".to_string())],
            children: vec![HickNode::Text("alpha".to_string(), None)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        };
        let copy2 = HickTag {
            name: "copy".to_string(),
            attributes: vec![("id".to_string(), "b".to_string())],
            children: vec![HickNode::Text("beta".to_string(), None)],
            self_closing: false,
            source_line: 2,
            source_column: 0,
            source_span: None,
            close_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(copy1), HickNode::Tag(copy2)]);
        let reg = build_copy_registry(&doc);
        assert_eq!(reg.len(), 2);
        assert_eq!(reg["a"], "alpha");
        assert_eq!(reg["b"], "beta");
    }

    #[test]
    fn from_source_integration() {
        // A minimal valid hick document with a file block
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="hello.rs">
fn main() {}
</hick:file>
</hick:doc>"#;

        let state = HickDocumentState::from_source(source).unwrap();
        assert_eq!(state.virtual_files.len(), 1);
        assert_eq!(state.virtual_files[0].path, "hello.rs");
        assert!(state.virtual_files[0].content().contains("fn main()"));
    }

    #[test]
    fn from_source_with_copy_and_paste() {
        let source = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="imports">
use std::io;
</hick:copy>
<hick:file path="main.rs">
<hick:paste select="#imports" />
fn main() {}
</hick:file>
</hick:doc>"##;

        let state = HickDocumentState::from_source(source).unwrap();
        assert!(state.copy_registry.contains_key("imports"));
        assert_eq!(state.virtual_files.len(), 1);
        let content = state.virtual_files[0].content();
        assert!(content.contains("use std::io;"), "content was: {content}");
        assert!(content.contains("fn main()"), "content was: {content}");
    }

    #[test]
    fn a_file_block_is_a_virtual_file_however_deeply_it_is_nested() {
        // `hick ingest` writes `exec > ingested > file`, and the engine's
        // weave has always written those files (`all_tags`). This asked only
        // the top level, so an ingested scaffold had no language server
        // inside it at all and the debugger reported that the document
        // generated nothing. One rule now: the file blocks the weave writes.
        let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
         <hick:exec container=\"sdk\">\n\
         <hick:ingested from=\"#c\" sha256=\"dead\" at=\"2026-09-03\" files=\"1\" skipped=\"0\">\n\
         <hick:file path=\"app/Program.cs\">\n\
         Console.WriteLine(\"Hello\");\n\
         </hick:file>\n\
         </hick:ingested>\n\
         </hick:exec>\n\
         </hick:doc>\n";
        let state = HickDocumentState::from_source(source).expect("parses");
        assert_eq!(state.virtual_files.len(), 1);
        assert_eq!(state.virtual_files[0].path, "app/Program.cs");
        assert_eq!(state.virtual_files[0].language_id, Some("csharp"));
        assert!(
            state.virtual_files[0]
                .content()
                .contains("Console.WriteLine")
        );
    }
}
