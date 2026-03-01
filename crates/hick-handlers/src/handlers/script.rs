//! Handler for `<hick:script>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{BoxStream, Context, Node, NodeTrace, NodeValue, SourceOrigin, StringNode};
use hick_lang::HickTag;

use crate::{
    ExecShow, ProcessingContext, ProcessingPhase, TagHandler, TagResult, parse_exec_show,
    render_transcript, tag_attr,
};

// ---------------------------------------------------------------------------
// ScriptNode — provenance wrapper
// ---------------------------------------------------------------------------

/// Provenance wrapper for script output, parallel to `ExecNode`.
pub(crate) struct ScriptNode {
    inner: StringNode,
    origin: SourceOrigin,
}

impl ScriptNode {
    pub fn new(content: impl Into<String>, tag_line: usize) -> Self {
        Self {
            inner: StringNode::new(content),
            origin: SourceOrigin::Script { tag_line },
        }
    }
}

impl std::fmt::Debug for ScriptNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptNode")
    }
}

impl Node for ScriptNode {
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
// ScriptHandler
// ---------------------------------------------------------------------------

/// Handler for `<hick:script>` tags.
///
/// Renders shell script execution transcript into file content.
/// Uses the same `show` attribute as `<hick:exec>` and looks up
/// transcripts by the synthetic container name (`_script_N`).
pub struct ScriptHandler;

impl TagHandler for ScriptHandler {
    fn tag_name(&self) -> &str {
        "script"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let show = parse_exec_show(tag);

        if show == ExecShow::None {
            return Ok(TagResult::Declaration);
        }

        // Script blocks use synthetic container names like "_script_N".
        // We need to find which synthetic name corresponds to this tag.
        // Use a tag attribute or fall back to searching transcripts.
        let container_name = tag_attr(tag, "container")
            .unwrap_or_else(|| {
                // Search for a matching transcript by convention:
                // the DAG assigns _script_N based on document order.
                // When no explicit container attr, try all _script_ prefixed transcripts.
                ctx.transcripts
                    .keys()
                    .find(|k| k.starts_with("_script_"))
                    .cloned()
                    .unwrap_or_default()
            });

        if let Some(entries) = ctx.transcripts.get(&container_name) {
            let rendered = render_transcript(entries, show);
            Ok(TagResult::Node(Arc::new(ScriptNode::new(
                rendered,
                tag.source_line,
            ))))
        } else {
            Ok(TagResult::Node(Arc::new(ScriptNode::new(
                format!("[no transcript for script '{container_name}']"),
                tag.source_line,
            ))))
        }
    }
}
