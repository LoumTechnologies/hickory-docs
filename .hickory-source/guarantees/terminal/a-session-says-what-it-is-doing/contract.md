<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Session Says What It Is Doing

For: Authors running interactive tools alongside documents without losing session state.

Given any terminal session, when it is asked for its state, then it answers
exactly one of **needs-you**, **working**, **idle**, **finished**, or
**failed**, decided in this order:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-session-says-what-it-is-doing.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-47661a9d9424"></a>
## `crates/hick-term/src/classify.rs`

Related contracts: [A Session Says What It Is Doing](contract.md).

<a id="f-47661a9d9424-SessionState"></a>
### `SessionState`

What a session is doing, from what we can actually observe.
Five states, one pure function, no I/O — because this is the decision the
whole supervision loop rests on, and a decision that cannot be tested in
isolation is a decision nobody can trust. Everything that needs the
outside world (is the child alive, how long since a byte, did the agent
declare a prompt) is gathered by the caller and handed over as [`Signals`].
Protects `docs/guarantees/terminal/a-session-says-what-it-is-doing.md`.

<storycode:copy id="f-47661a9d9424-SessionState">
//! What a session is doing, from what we can actually observe.
//!
//! Five states, one pure function, no I/O — because this is the decision the
//! whole supervision loop rests on, and a decision that cannot be tested in
//! isolation is a decision nobody can trust. Everything that needs the
//! outside world (is the child alive, how long since a byte, did the agent
//! declare a prompt) is gathered by the caller and handed over as [`Signals`].
//!
//! Protects `docs/guarantees/terminal/a-session-says-what-it-is-doing.md`.

use serde::{Deserialize, Serialize};

/// The five states a session can be in.
///
/// The split that matters is *working* versus *needs you*: working sessions
/// are the ones you are supposed to leave alone, and every other state is a
/// claim on your attention of some size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    /// Blocked on a decision only a person can make.
    NeedsYou,
    /// Something is running. Do not interrupt.
    Working,
    /// Alive, at a prompt, with nothing to say.
    Idle,
    /// The command exited 0.
    Finished,
    /// The command exited non-zero.
    Failed,
}

</storycode:copy>
<a id="f-47661a9d9424-impl-SessionState"></a>
### `impl-SessionState`

<storycode:copy id="f-47661a9d9424-impl-SessionState">
impl SessionState {
    /// The name the API and the client use.
    pub fn as_str(self) -> &'static str {
        match self {
            SessionState::NeedsYou => "needs-you",
            SessionState::Working => "working",
            SessionState::Idle => "idle",
            SessionState::Finished => "finished",
            SessionState::Failed => "failed",
        }
    }

    /// Whether this state has any claim on a person's attention. Working and
    /// idle sessions do not: one is busy and one is done being busy, and
    /// neither is waiting on you.
    pub fn claims_attention(self) -> bool {
        !matches!(self, SessionState::Working | SessionState::Idle)
    }
}

</storycode:copy>
<a id="f-47661a9d9424-Signals"></a>
### `Signals`

Everything observable about a session at one instant.

<storycode:copy id="f-47661a9d9424-Signals">
/// Everything observable about a session at one instant.
#[derive(Debug, Clone, Copy, Default)]
pub struct Signals {
    /// The child's exit code, once it has one.
    pub exit: Option<i32>,
    /// A prompt is waiting for an answer.
    ///
    /// For `hickory-agent` this is structural — the agent protocol declares a
    /// question, with its own choices. For a third-party agent running in a
    /// plain shell it is a guess (see [`crate::prompt`]), and the guarantee
    /// says so.
    pub prompt_pending: bool,
    /// A foreground process other than the session's own shell is running.
    ///
    /// `None` where the platform will not tell us (Windows), which is why
    /// quiet time is also consulted rather than trusted alone.
    pub foreground_child: Option<bool>,
    /// Milliseconds since the PTY last produced a byte.
    pub quiet_ms: u64,
}

</storycode:copy>
<a id="f-47661a9d9424-IDLE_AFTER_MS"></a>
### `IDLE_AFTER_MS`

How long a silent session with no foreground child waits before it counts
as idle rather than working. Short enough that a finished `ls` settles
before you look away, long enough that a program printing a progress bar
every second never flickers.

<storycode:copy id="f-47661a9d9424-IDLE_AFTER_MS">
/// How long a silent session with no foreground child waits before it counts
/// as idle rather than working. Short enough that a finished `ls` settles
/// before you look away, long enough that a program printing a progress bar
/// every second never flickers.
pub const IDLE_AFTER_MS: u64 = 1_500;

</storycode:copy>
<a id="f-47661a9d9424-classify"></a>
### `classify`

The state a session is in.
Order matters and is deliberate:
1. An exited session is finished or failed — a dead process is not waiting
on your answer, however it looked a moment before it died.
2. A declared prompt outranks everything a live session could be doing.
3. A foreground child means work, however quiet it is: a compile that
prints nothing for a minute is not idle.
4. Otherwise silence decides, because on a platform that will not name the
foreground process that is all we have.

<storycode:copy id="f-47661a9d9424-classify">
/// The state a session is in.
///
/// Order matters and is deliberate:
///
/// 1. An exited session is finished or failed — a dead process is not waiting
///    on your answer, however it looked a moment before it died.
/// 2. A declared prompt outranks everything a live session could be doing.
/// 3. A foreground child means work, however quiet it is: a compile that
///    prints nothing for a minute is not idle.
/// 4. Otherwise silence decides, because on a platform that will not name the
///    foreground process that is all we have.
pub fn classify(signals: Signals) -> SessionState {
    if let Some(code) = signals.exit {
        return if code == 0 {
            SessionState::Finished
        } else {
            SessionState::Failed
        };
    }
    if signals.prompt_pending {
        return SessionState::NeedsYou;
    }
    if signals.foreground_child == Some(true) {
        return SessionState::Working;
    }
    if signals.quiet_ms < IDLE_AFTER_MS {
        SessionState::Working
    } else {
        SessionState::Idle
    }
}

</storycode:copy>
<a id="f-47661a9d9424-tests"></a>
### `tests`

<storycode:copy id="f-47661a9d9424-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn alive() -> Signals {
        Signals {
            quiet_ms: IDLE_AFTER_MS + 1,
            ..Signals::default()
        }
    }

    #[test]
    fn a_clean_exit_is_finished_and_a_dirty_one_failed() {
        assert_eq!(
            classify(Signals {
                exit: Some(0),
                ..alive()
            }),
            SessionState::Finished
        );
        assert_eq!(
            classify(Signals {
                exit: Some(1),
                ..alive()
            }),
            SessionState::Failed
        );
    }

    #[test]
    fn an_exited_session_is_not_waiting_on_you_even_with_a_prompt_on_screen() {
        // The agent asked something and then died. The prompt is moot; what
        // you need to know is that it failed.
        assert_eq!(
            classify(Signals {
                exit: Some(2),
                prompt_pending: true,
                ..alive()
            }),
            SessionState::Failed
        );
    }

    #[test]
    fn a_declared_prompt_outranks_a_busy_looking_session() {
        assert_eq!(
            classify(Signals {
                prompt_pending: true,
                foreground_child: Some(true),
                quiet_ms: 0,
                ..Signals::default()
            }),
            SessionState::NeedsYou
        );
    }

    #[test]
    fn a_quiet_foreground_child_is_still_working() {
        // The case a pure quiet-timer gets wrong: a long compile.
        assert_eq!(
            classify(Signals {
                foreground_child: Some(true),
                quiet_ms: 600_000,
                ..Signals::default()
            }),
            SessionState::Working
        );
    }

    #[test]
    fn a_shell_at_its_prompt_goes_idle_once_the_output_stops() {
        assert_eq!(
            classify(Signals {
                foreground_child: Some(false),
                quiet_ms: IDLE_AFTER_MS,
                ..Signals::default()
            }),
            SessionState::Idle
        );
        assert_eq!(
            classify(Signals {
                foreground_child: Some(false),
                quiet_ms: IDLE_AFTER_MS - 1,
                ..Signals::default()
            }),
            SessionState::Working
        );
    }

    #[test]
    fn without_a_foreground_answer_silence_decides() {
        // Windows: portable-pty cannot name the foreground process, so the
        // quiet timer carries the classification alone.
        assert_eq!(classify(alive()), SessionState::Idle);
        assert_eq!(
            classify(Signals {
                quiet_ms: 0,
                ..Signals::default()
            }),
            SessionState::Working
        );
    }

    #[test]
    fn working_and_idle_make_no_claim_on_attention() {
        assert!(!SessionState::Working.claims_attention());
        assert!(!SessionState::Idle.claims_attention());
        assert!(SessionState::NeedsYou.claims_attention());
        assert!(SessionState::Failed.claims_attention());
        assert!(SessionState::Finished.claims_attention());
    }
}
</storycode:copy>

### Assemble `crates/hick-term/src/classify.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/src/classify.rs"><storycode:paste select="#f-47661a9d9424-SessionState"/><storycode:paste select="#f-47661a9d9424-impl-SessionState"/><storycode:paste select="#f-47661a9d9424-Signals"/><storycode:paste select="#f-47661a9d9424-IDLE_AFTER_MS"/><storycode:paste select="#f-47661a9d9424-classify"/><storycode:paste select="#f-47661a9d9424-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-term/src/classify.rs](contract.md#output-47661a9d9424)
- [crates/hick-term/src/registry.rs](../../../mechanisms/crates/hick-term/src/registry.rs.md#output-8678d496098a)

### Tests and fixtures

- [crates/hickory-cli/tests/serve_terminals.rs](../../../mechanisms/crates/hickory-cli/tests/serve_terminals.rs.md#output-5d8946e53299)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
