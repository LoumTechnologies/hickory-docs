<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/agent_review.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Conversation edits use the client's review policy](../../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md)

Captured SHA-256: `408403d28232ca3be2a57bf2e680ec3f5f8f01ccee39683477619f932823cb26`.

<book:entry path="crates/hickory-cli/src/serve/agent_review.rs">
//! Built-in agents use the same review queue and UI as ACP document edits.
use super::{LocalState, acp::edits::Edits};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ReviewQueue {
    pub edits: Arc<Edits>,
    pub context: Arc<Mutex<crate::context_updates::Inbox>>,
    pub session: Mutex<Option<std::path::PathBuf>>,
}

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
</book:entry>

</book:doc>
