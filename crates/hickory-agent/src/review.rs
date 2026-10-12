//! Host review policy, independent of the model and optional for headless callers.
use crate::ToolInvocation;

#[async_trait::async_trait]
pub trait ToolReview: Send + Sync {
    /// User-authored revision of the exact preview, after approval.
    fn replacement(&self) -> Option<String> {
        None
    }
    fn mode(&self) -> &'static str {
        "review"
    }
    fn completed(&self, _ok: bool) {}
    async fn approve(
        &self,
        invocation: &ToolInvocation,
        preview: &(String, String, String),
    ) -> anyhow::Result<()>;
}

pub(crate) fn whole_edit(
    index: &crate::tools::hashline::LineIndex,
    inv: &crate::ToolInvocation,
) -> Option<Result<(usize, String), String>> {
    if inv.arg("whole") != Some("true") {
        return None;
    }
    Some(if inv.arg("snapshot") != Some(index.snapshot.as_str()) {
        Err("A complete reviewed replacement requires the exact surface snapshot.".into())
    } else {
        Ok((
            index.line_end(index.lines.len() - 1),
            inv.input.clone().unwrap_or_default(),
        ))
    })
}
