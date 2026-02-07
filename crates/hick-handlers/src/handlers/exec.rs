//! Handler for `<hick:exec>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::StringNode;
use hick_lang::HickTag;

use crate::{
    ExecShow, ProcessingContext, ProcessingPhase, TagHandler, TagResult,
    parse_exec_show, render_transcript, tag_attr,
};

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
            Ok(TagResult::Node(Arc::new(StringNode::new(rendered))))
        } else {
            Ok(TagResult::Node(Arc::new(StringNode::new(format!(
                "[no transcript for container '{container_name}']"
            )))))
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
            apply_substitutions: &|s: &str| s.to_string(),
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
            apply_substitutions: &|s: &str| s.to_string(),
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
            apply_substitutions: &|s: &str| s.to_string(),
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
            apply_substitutions: &|s: &str| s.to_string(),
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
