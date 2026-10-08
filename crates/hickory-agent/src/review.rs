//! Host review policy, independent of the model and optional for headless callers.
use crate::ToolInvocation;

#[async_trait::async_trait]
pub trait ToolReview: Send + Sync {
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
