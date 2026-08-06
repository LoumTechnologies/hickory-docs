//! Handler for `<hick:exec>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{BoxStream, Context, Node, NodeTrace, NodeValue, SourceOrigin, StringNode};
use hick_lang::HickTag;

use crate::{
    ExecShow, ProcessingContext, ProcessingPhase, TagHandler, TagResult, parse_exec_show,
    render_transcript, tag_attr,
};

// ---------------------------------------------------------------------------
// ExecNode — provenance wrapper
// ---------------------------------------------------------------------------

/// Provenance wrapper around `StringNode` for exec transcript output.
///
/// Carries the container name, enabling trace-based identification of
/// which container produced the output.
pub(crate) struct ExecNode {
    inner: StringNode,
    container: String,
    origin: SourceOrigin,
}

impl ExecNode {
    pub fn new(content: impl Into<String>, container: impl Into<String>, tag_line: usize) -> Self {
        let c: String = container.into();
        Self {
            inner: StringNode::new(content),
            origin: SourceOrigin::Exec {
                container: Arc::from(c.as_str()),
                tag_line,
            },
            container: c,
        }
    }
}

impl std::fmt::Debug for ExecNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExecNode(container={:?})", self.container)
    }
}

impl Node for ExecNode {
    fn as_string_value(&self) -> Option<&str> {
        self.inner.as_string_value()
    }

    fn node_value(&self) -> NodeValue {
        self.inner.node_value()
    }

    fn source_origin(&self) -> Option<&SourceOrigin> {
        Some(&self.origin)
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let trace = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { trace }))
    }
}

// ---------------------------------------------------------------------------
// ExecHandler
// ---------------------------------------------------------------------------

/// Handler for `<hick:exec>` tags.
///
/// Renders execution transcript output for a container into file content.
///
/// # Attributes
///
/// - `container` - Required container name to pull transcript from
/// - `show` - Optional: "all" (default), "command", "output", or "none"
pub struct ExecHandler;

impl TagHandler for ExecHandler {
    fn tag_name(&self) -> &str {
        "exec"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let container_name = tag_attr(tag, "container").unwrap_or_default();
        let show = parse_exec_show(tag);

        if show == ExecShow::None {
            return Ok(TagResult::Declaration);
        }

        if let Some(entries) = ctx.transcripts.get(&container_name) {
            let rendered = render_transcript(entries, show);
            Ok(TagResult::Node(Arc::new(ExecNode::new(
                rendered,
                &container_name,
                tag.source_line,
            ))))
        } else {
            Ok(TagResult::Node(Arc::new(ExecNode::new(
                format!("[no transcript for container '{container_name}']"),
                &container_name,
                tag.source_line,
            ))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TranscriptEntry;
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;

    fn make_tag(name: &str, attrs: Vec<(String, String)>) -> HickTag {
        HickTag {
            name: name.to_string(),
            attributes: attrs,
            children: vec![],
            self_closing: true,
            source_line: 1,
            source_column: 0,
            source_span: None,
        }
    }

    #[test]
    fn exec_handler_renders_transcript() {
        let state = Arc::new(MultiDocumentState::default());
        let mut transcripts = HashMap::new();
        transcripts.insert(
            "builder".to_string(),
            vec![TranscriptEntry {
                commands: vec!["echo hello".to_string()],
                output: "hello".to_string(),
            }],
        );

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
        };

        let tag = make_tag(
            "exec",
            vec![("container".to_string(), "builder".to_string())],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("$ echo hello\nhello\n"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn exec_handler_show_none_returns_declaration() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
        };

        let tag = make_tag(
            "exec",
            vec![
                ("container".to_string(), "builder".to_string()),
                ("show".to_string(), "none".to_string()),
            ],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Declaration));
    }

    #[test]
    fn exec_handler_missing_container() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
        };

        let tag = make_tag(
            "exec",
            vec![("container".to_string(), "missing".to_string())],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(
                    node.as_string_value(),
                    Some("[no transcript for container 'missing']")
                );
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn exec_handler_show_command_only() {
        let state = Arc::new(MultiDocumentState::default());
        let mut transcripts = HashMap::new();
        transcripts.insert(
            "builder".to_string(),
            vec![TranscriptEntry {
                commands: vec!["ls".to_string()],
                output: "file.txt".to_string(),
            }],
        );

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
        };

        let tag = make_tag(
            "exec",
            vec![
                ("container".to_string(), "builder".to_string()),
                ("show".to_string(), "command".to_string()),
            ],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("$ ls\n"));
            }
            _ => panic!("Expected Node result"),
        }
    }
}
