<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Attention Queue Ranks By Claim

For: Authors running interactive tools alongside documents without losing session state.

Given every terminal session in the window, when the attention queue is read,
then it contains exactly the sessions making a claim on a person, ordered:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/the-attention-queue-ranks-by-claim.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-08fbab2e70a9"></a>
## `crates/hick-term/src/attention.rs`

Related contracts: [The Attention Queue Ranks By Claim](contract.md).

<a id="f-08fbab2e70a9-Claim"></a>
### `Claim`

The order in which sessions get to interrupt you.
One queue for the whole window, so "what needs me?" is a question with a
single answer instead of a scan across panes. Pure, and tested, because
the ordering *is* the feature: a queue that puts a clean success ahead of
a blocked agent is worse than no queue, since you learn to stop trusting
the top of it.
Protects `docs/guarantees/terminal/the-attention-queue-ranks-by-claim.md`.

<storycode:copy id="f-08fbab2e70a9-Claim">
//! The order in which sessions get to interrupt you.
//!
//! One queue for the whole window, so "what needs me?" is a question with a
//! single answer instead of a scan across panes. Pure, and tested, because
//! the ordering *is* the feature: a queue that puts a clean success ahead of
//! a blocked agent is worse than no queue, since you learn to stop trusting
//! the top of it.
//!
//! Protects `docs/guarantees/terminal/the-attention-queue-ranks-by-claim.md`.

use crate::classify::SessionState;

/// One session's claim on your attention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub id: String,
    pub state: SessionState,
    /// The working tree has uncommitted changes.
    pub dirty: bool,
    /// When this session entered the state it is in, in milliseconds since the
    /// epoch. Older waiters go first within a band.
    pub since_ms: u64,
}

</storycode:copy>
<a id="f-08fbab2e70a9-band"></a>
### `band`

Which band a claim belongs to. Lower sorts first; `None` means the session
makes no claim at all and does not enter the queue.
Finished-with-uncommitted-changes ranks above a clean finish because it
still has a claim on you: there is a decision left (commit, amend, throw
away) that nobody else can make. A clean finish is only news.

<storycode:copy id="f-08fbab2e70a9-band">
/// Which band a claim belongs to. Lower sorts first; `None` means the session
/// makes no claim at all and does not enter the queue.
///
/// Finished-with-uncommitted-changes ranks above a clean finish because it
/// still has a claim on you: there is a decision left (commit, amend, throw
/// away) that nobody else can make. A clean finish is only news.
fn band(claim: &Claim) -> Option<u8> {
    match claim.state {
        SessionState::NeedsYou => Some(0),
        SessionState::Failed => Some(1),
        SessionState::Finished if claim.dirty => Some(2),
        SessionState::Finished => Some(3),
        SessionState::Working | SessionState::Idle => None,
    }
}

</storycode:copy>
<a id="f-08fbab2e70a9-attention_order"></a>
### `attention_order`

The ids of every session claiming attention, most-claiming first.
Within a band the oldest waiter leads, so nothing starves behind a stream
of fresher interruptions; ties break on id so the order never depends on
the order sessions happen to be stored in.

<storycode:copy id="f-08fbab2e70a9-attention_order">
/// The ids of every session claiming attention, most-claiming first.
///
/// Within a band the oldest waiter leads, so nothing starves behind a stream
/// of fresher interruptions; ties break on id so the order never depends on
/// the order sessions happen to be stored in.
pub fn attention_order(claims: &[Claim]) -> Vec<String> {
    let mut ranked: Vec<(u8, u64, &str)> = claims
        .iter()
        .filter_map(|c| band(c).map(|b| (b, c.since_ms, c.id.as_str())))
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(b.2)));
    ranked
        .into_iter()
        .map(|(_, _, id)| id.to_string())
        .collect()
}

</storycode:copy>
<a id="f-08fbab2e70a9-tests"></a>
### `tests`

<storycode:copy id="f-08fbab2e70a9-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn claim(id: &str, state: SessionState, dirty: bool, since_ms: u64) -> Claim {
        Claim {
            id: id.to_string(),
            state,
            dirty,
            since_ms,
        }
    }

    #[test]
    fn blocked_then_failed_then_dirty_success_then_clean_success() {
        let claims = vec![
            claim("clean", SessionState::Finished, false, 1),
            claim("dirty", SessionState::Finished, true, 1),
            claim("failed", SessionState::Failed, false, 1),
            claim("blocked", SessionState::NeedsYou, false, 1),
        ];
        assert_eq!(
            attention_order(&claims),
            vec!["blocked", "failed", "dirty", "clean"]
        );
    }

    #[test]
    fn uncommitted_changes_outrank_a_clean_finish_however_much_older_it_is() {
        let claims = vec![
            claim("clean", SessionState::Finished, false, 1),
            claim("dirty", SessionState::Finished, true, 9_999),
        ];
        assert_eq!(attention_order(&claims), vec!["dirty", "clean"]);
    }

    #[test]
    fn the_oldest_waiter_leads_its_band() {
        let claims = vec![
            claim("new", SessionState::NeedsYou, false, 300),
            claim("old", SessionState::NeedsYou, false, 100),
            claim("middle", SessionState::NeedsYou, false, 200),
        ];
        assert_eq!(attention_order(&claims), vec!["old", "middle", "new"]);
    }

    #[test]
    fn working_and_idle_sessions_never_enter_the_queue() {
        let claims = vec![
            claim("busy", SessionState::Working, true, 1),
            claim("resting", SessionState::Idle, true, 1),
        ];
        assert!(attention_order(&claims).is_empty());
    }

    #[test]
    fn a_tie_breaks_on_id_rather_than_on_storage_order() {
        let forward = vec![
            claim("a", SessionState::Failed, false, 5),
            claim("b", SessionState::Failed, false, 5),
        ];
        let backward: Vec<Claim> = forward.iter().cloned().rev().collect();
        assert_eq!(attention_order(&forward), attention_order(&backward));
    }
}
</storycode:copy>

### Assemble `crates/hick-term/src/attention.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/src/attention.rs"><storycode:paste select="#f-08fbab2e70a9-Claim"/><storycode:paste select="#f-08fbab2e70a9-band"/><storycode:paste select="#f-08fbab2e70a9-attention_order"/><storycode:paste select="#f-08fbab2e70a9-tests"/></storycode:file>

## Implementation review

### Implementation

- apps/web/src/lib/attentionCursor.ts
- [crates/hick-term/src/attention.rs](contract.md#output-08fbab2e70a9)
- [crates/hick-term/src/registry.rs](../../../mechanisms/crates/hick-term/src/registry.rs.md#output-8678d496098a)
- [crates/hickory-cli/src/serve/terminal.rs](../../../mechanisms/crates/hickory-cli/src/serve/terminal.rs.md#output-e67fa7430800)

### Tests and fixtures

- apps/web/src/lib/attentionCursor.test.ts
- [crates/hickory-cli/tests/serve_terminals.rs](../../../mechanisms/crates/hickory-cli/tests/serve_terminals.rs.md#output-5d8946e53299)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
