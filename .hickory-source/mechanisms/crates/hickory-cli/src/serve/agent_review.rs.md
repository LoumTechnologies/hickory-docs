<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/agent_review.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ee38c84c9207"></a>
## `crates/hickory-cli/src/serve/agent_review.rs`

Related contracts: [Conversation edits use the client's review policy](../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md).

<a id="f-ee38c84c9207-ReviewQueue"></a>
### `ReviewQueue`

Built-in agents use the same review queue and UI as ACP document edits.

<storycode:copy id="f-ee38c84c9207-ReviewQueue">
//! Built-in agents use the same review queue and UI as ACP document edits.
use super::{LocalState, acp::edits::Edits};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ReviewQueue {
    pub edits: Arc<Edits>,
    pub context: Arc<Mutex<crate::context_updates::Inbox>>,
    pub session: Mutex<Option<std::path::PathBuf>>,
}

</storycode:copy>
<a id="f-ee38c84c9207-queue"></a>
### `queue`

<storycode:copy id="f-ee38c84c9207-queue">
pub fn queue(state: &LocalState, doc: &str) -> Arc<ReviewQueue> {
    state
        .agent
        .reviews
        .lock()
        .unwrap()
        .entry(doc.into())
        .or_default()
        .clone()
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/agent_review.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/agent_review.rs"><storycode:paste select="#f-ee38c84c9207-ReviewQueue"/><storycode:paste select="#f-ee38c84c9207-queue"/></storycode:file>

</storycode:doc>
