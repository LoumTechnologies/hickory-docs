<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Turbo Never Answers A Prompt It Did Not Parse

For: Authors running interactive tools alongside documents without losing session state.

Given turbo is on, when a session is waiting on a prompt, then that prompt is
answered automatically **only** if all three hold:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-c05b5ea8cc95"></a>
## `crates/hick-term/src/prompt.rs`

Related contracts: [Turbo Never Answers A Prompt It Did Not Parse](contract.md).

<a id="f-c05b5ea8cc95-ASKING"></a>
### `ASKING`

Guessing that a stranger's program is waiting for an answer.
`hickory-agent` never needs this: its protocol declares a question, with
the choices it will accept, and the attention card renders them. This
module is for the other case — someone else's coding agent running in a
plain shell, where the only evidence is what is on the screen.
It is therefore **best-effort, and stays quarantined here**. Two rules
keep a guess from doing damage:
- a guessed prompt may raise a session to *needs you*, which only ever
costs you a glance;
- a guessed prompt may never be auto-answered by turbo, which would cost
you an answer you did not give. See
`docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md`.
There are two shapes, and they need different windows onto the screen.
**A line-oriented question** — `Delete build/? [y/N]` — is only live while
it is the last thing printed. Once it is answered the answer and its
output follow, so looking at the last non-empty line alone is what stops a
long-answered `[y/N]` from pinning a session at *needs you* forever.
**A drawn menu** — the `❯ 1. Yes` list that Claude Code, Codex and their
siblings put up — is not the last line: below it sit a blank row and a
footer like `Enter to confirm · Esc to cancel`. It is also redrawn rather
than scrolled, so it leaves the screen the moment it is answered. That one
is looked for across the last few lines.
Substrings that, at the end of a session's output, mean a program has
stopped to ask. Lowercased before matching.

<storycode:copy id="f-c05b5ea8cc95-ASKING">
//! Guessing that a stranger's program is waiting for an answer.
//!
//! `hickory-agent` never needs this: its protocol declares a question, with
//! the choices it will accept, and the attention card renders them. This
//! module is for the other case — someone else's coding agent running in a
//! plain shell, where the only evidence is what is on the screen.
//!
//! It is therefore **best-effort, and stays quarantined here**. Two rules
//! keep a guess from doing damage:
//!
//! - a guessed prompt may raise a session to *needs you*, which only ever
//!   costs you a glance;
//! - a guessed prompt may never be auto-answered by turbo, which would cost
//!   you an answer you did not give. See
//!   `docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md`.
//!
//! There are two shapes, and they need different windows onto the screen.
//!
//! **A line-oriented question** — `Delete build/? [y/N]` — is only live while
//! it is the last thing printed. Once it is answered the answer and its
//! output follow, so looking at the last non-empty line alone is what stops a
//! long-answered `[y/N]` from pinning a session at *needs you* forever.
//!
//! **A drawn menu** — the `❯ 1. Yes` list that Claude Code, Codex and their
//! siblings put up — is not the last line: below it sit a blank row and a
//! footer like `Enter to confirm · Esc to cancel`. It is also redrawn rather
//! than scrolled, so it leaves the screen the moment it is answered. That one
//! is looked for across the last few lines.

/// Substrings that, at the end of a session's output, mean a program has
/// stopped to ask. Lowercased before matching.
const ASKING: &[&str] = &[
    "[y/n]",
    "(y/n)",
    "[yes/no]",
    "(yes/no)",
    "? (y)",
    "press enter to continue",
    "do you want to proceed",
    "do you want to continue",
    "overwrite?",
];

</storycode:copy>
<a id="f-c05b5ea8cc95-FOOTERS"></a>
### `FOOTERS`

Footers a drawn menu puts under its choices.

<storycode:copy id="f-c05b5ea8cc95-FOOTERS">
/// Footers a drawn menu puts under its choices.
const FOOTERS: &[&str] = &[
    "enter to confirm",
    "to confirm · esc",
    "press enter to select",
    "esc to cancel",
];

</storycode:copy>
<a id="f-c05b5ea8cc95-MENU_WINDOW"></a>
### `MENU_WINDOW`

How far up to look for a drawn menu. Enough for the choices plus a footer
and a blank line, not so far that ordinary output resembles one.

<storycode:copy id="f-c05b5ea8cc95-MENU_WINDOW">
/// How far up to look for a drawn menu. Enough for the choices plus a footer
/// and a blank line, not so far that ordinary output resembles one.
const MENU_WINDOW: usize = 8;

</storycode:copy>
<a id="f-c05b5ea8cc95-question_in"></a>
### `question_in`

The question the visible screen is asking, if it is asking one.
Give it the screen, not the scrollback: what matters is what a person
would see if they looked now. The text handed back is what the attention
card shows — for a line-oriented question that is the line; for a drawn
menu it is the choices and their footer, because the choices ARE the
question.

<storycode:copy id="f-c05b5ea8cc95-question_in">
/// The question the visible screen is asking, if it is asking one.
///
/// Give it the screen, not the scrollback: what matters is what a person
/// would see if they looked now. The text handed back is what the attention
/// card shows — for a line-oriented question that is the line; for a drawn
/// menu it is the choices and their footer, because the choices ARE the
/// question.
pub fn question_in(screen: &str) -> Option<String> {
    let live: Vec<&str> = screen
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect();
    let last = live.last()?;
    if ASKING
        .iter()
        .any(|needle| last.to_lowercase().contains(needle))
    {
        return Some(last.trim().to_string());
    }

    let from = live.len().saturating_sub(MENU_WINDOW);
    let tail = &live[from..];
    let first_choice = tail
        .iter()
        .position(|line| is_choice(&line.to_lowercase()))?;
    let has_footer = tail.iter().any(|line| {
        let line = line.to_lowercase();
        FOOTERS.iter().any(|f| line.contains(f))
    });
    if !has_footer {
        return None;
    }
    Some(
        tail[first_choice..]
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

</storycode:copy>
<a id="f-c05b5ea8cc95-looks_like_a_question"></a>
### `looks_like_a_question`

Whether the visible screen looks like an unanswered question.

<storycode:copy id="f-c05b5ea8cc95-looks_like_a_question">
/// Whether the visible screen looks like an unanswered question.
pub fn looks_like_a_question(screen: &str) -> bool {
    question_in(screen).is_some()
}

</storycode:copy>
<a id="f-c05b5ea8cc95-is_choice"></a>
### `is_choice`

A selectable option in a drawn menu: a marker or a number, then a label.
`❯ 1. Yes, I trust this folder` and `  2. No, exit` both count.

<storycode:copy id="f-c05b5ea8cc95-is_choice">
/// A selectable option in a drawn menu: a marker or a number, then a label.
/// `❯ 1. Yes, I trust this folder` and `  2. No, exit` both count.
fn is_choice(line: &str) -> bool {
    let line = line.trim_start();
    let rest = line
        .strip_prefix('❯')
        .or_else(|| line.strip_prefix('>'))
        .unwrap_or(line)
        .trim_start();
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_digit() {
        return false;
    }
    // "1." or "1)" — and something after it, so a bare number in a table is
    // not a choice.
    matches!(chars.next(), Some('.') | Some(')')) && chars.as_str().trim().len() > 1
}

</storycode:copy>
<a id="f-c05b5ea8cc95-tests"></a>
### `tests`

<storycode:copy id="f-c05b5ea8cc95-tests">
#[cfg(test)]
mod tests {
    use super::*;

    /// What Claude Code actually draws on its first run in a new folder,
    /// captured from a real PTY session (tests/full_screen_apps.rs is the
    /// harness that produced it). The exact case this feature exists for.
    const CODING_AGENT_TRUST_PROMPT: &str = "\
────────────────────────────────────────────────────
 Accessing workspace:

 /tmp/.tmprwHQ9S

 Quick safety check: Is this a project you created or one you trust?

 Security guide

 ❯ 1. Yes, I trust this folder
   2. No, exit

 Enter to confirm · Esc to cancel";

    #[test]
    fn a_coding_agents_drawn_menu_reads_as_asking() {
        assert!(looks_like_a_question(CODING_AGENT_TRUST_PROMPT));
    }

    #[test]
    fn the_question_it_reports_is_the_choices_not_the_footer() {
        // The card shows this. "Enter to confirm · Esc to cancel" on its own
        // would tell a person nothing about what they are confirming.
        let question = question_in(CODING_AGENT_TRUST_PROMPT).unwrap();
        assert!(
            question.starts_with("❯ 1. Yes, I trust this folder"),
            "{question}"
        );
        assert!(question.contains("2. No, exit"), "{question}");
    }

    #[test]
    fn a_line_question_reports_itself() {
        assert_eq!(
            question_in("Delete build/? [y/N] ").unwrap(),
            "Delete build/? [y/N]"
        );
    }

    #[test]
    fn a_yes_no_question_on_the_last_line_reads_as_asking() {
        assert!(looks_like_a_question("Delete build/? [y/N] "));
        assert!(looks_like_a_question("Overwrite?"));
        assert!(looks_like_a_question("Press ENTER to continue"));
    }

    #[test]
    fn an_answered_question_further_up_does_not_hold_the_session_hostage() {
        assert!(!looks_like_a_question(
            "Delete build/? [y/N] y\nDeleted.\n$ "
        ));
    }

    #[test]
    fn choices_without_a_footer_are_just_a_list() {
        // A numbered list in ordinary output — release notes, a table of
        // contents — must not put a session in the queue.
        assert!(!looks_like_a_question(
            "Changes:\n 1. faster weaves\n 2. fewer bugs\n"
        ));
    }

    #[test]
    fn a_footer_without_choices_is_not_a_menu_either() {
        assert!(!looks_like_a_question("some prose\nesc to cancel"));
    }

    #[test]
    fn ordinary_output_is_not_a_question() {
        assert!(!looks_like_a_question("Compiling hick-term v0.1.0\n"));
        assert!(!looks_like_a_question(""));
    }

    #[test]
    fn a_bare_number_is_not_a_choice() {
        assert!(!is_choice("  42"));
        assert!(!is_choice("1."));
        assert!(is_choice("❯ 1. Yes, I trust this folder"));
        assert!(is_choice("  2) No, exit"));
    }
}
</storycode:copy>

### Assemble `crates/hick-term/src/prompt.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/src/prompt.rs"><storycode:paste select="#f-c05b5ea8cc95-ASKING"/><storycode:paste select="#f-c05b5ea8cc95-FOOTERS"/><storycode:paste select="#f-c05b5ea8cc95-MENU_WINDOW"/><storycode:paste select="#f-c05b5ea8cc95-question_in"/><storycode:paste select="#f-c05b5ea8cc95-looks_like_a_question"/><storycode:paste select="#f-c05b5ea8cc95-is_choice"/><storycode:paste select="#f-c05b5ea8cc95-tests"/></storycode:file>

<a id="output-aded4ac572de"></a>
## `crates/hick-term/src/turbo.rs`

Related contracts: [Turbo Never Answers A Prompt It Did Not Parse](contract.md).

<a id="f-aded4ac572de-turbo_choice"></a>
### `turbo_choice`

Answering the routine prompts for you — and refusing to answer the rest.
Turbo is off until you turn it on, and even then it is deliberately timid.
The value of an attention queue is that the things in it are real; an
auto-answer that guesses wrong destroys that in one keystroke you never
saw. So two refusals are absolute:
- a **guessed** prompt is never answered. We recognised the shape of a
question on a screen; we do not know what the answers mean.
- a **destructive** choice is never taken, and an ambiguous prompt (no
safe choice, or more than one) is left for a person.
Protects `docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md`.

<storycode:copy id="f-aded4ac572de-turbo_choice">
//! Answering the routine prompts for you — and refusing to answer the rest.
//!
//! Turbo is off until you turn it on, and even then it is deliberately timid.
//! The value of an attention queue is that the things in it are real; an
//! auto-answer that guesses wrong destroys that in one keystroke you never
//! saw. So two refusals are absolute:
//!
//! - a **guessed** prompt is never answered. We recognised the shape of a
//!   question on a screen; we do not know what the answers mean.
//! - a **destructive** choice is never taken, and an ambiguous prompt (no
//!   safe choice, or more than one) is left for a person.
//!
//! Protects `docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md`.

use crate::session::{Choice, Prompt, PromptSource};

/// The choice turbo would take, if it may take one at all.
pub fn turbo_choice(prompt: &Prompt) -> Option<&Choice> {
    if prompt.source != PromptSource::Declared {
        return None;
    }
    let mut safe = prompt.choices.iter().filter(|c| !c.destructive);
    let only = safe.next()?;
    // A second safe answer means the program is asking something with more
    // than one harmless outcome — which is a decision, not a formality.
    safe.next().is_none().then_some(only)
}

</storycode:copy>
<a id="f-aded4ac572de-tests"></a>
### `tests`

<storycode:copy id="f-aded4ac572de-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn choice(label: &str, destructive: bool) -> Choice {
        Choice {
            label: label.to_string(),
            send: "y\n".to_string(),
            destructive,
        }
    }

    fn declared(choices: Vec<Choice>) -> Prompt {
        Prompt {
            question: "Read package.json?".to_string(),
            choices,
            source: PromptSource::Declared,
        }
    }

    #[test]
    fn one_safe_choice_on_a_declared_prompt_is_answered() {
        let prompt = declared(vec![choice("Allow", false), choice("Delete it", true)]);
        assert_eq!(turbo_choice(&prompt).unwrap().label, "Allow");
    }

    #[test]
    fn a_guessed_prompt_is_never_answered() {
        let prompt = Prompt {
            source: PromptSource::Guessed,
            ..declared(vec![choice("Allow", false)])
        };
        assert!(turbo_choice(&prompt).is_none());
    }

    #[test]
    fn an_all_destructive_prompt_is_left_alone() {
        let prompt = declared(vec![choice("Force push", true), choice("Reset hard", true)]);
        assert!(turbo_choice(&prompt).is_none());
    }

    #[test]
    fn a_choice_between_two_harmless_outcomes_is_still_yours_to_make() {
        let prompt = declared(vec![choice("Python", false), choice("Rust", false)]);
        assert!(turbo_choice(&prompt).is_none());
    }

    #[test]
    fn a_prompt_with_no_choices_is_not_answerable() {
        assert!(turbo_choice(&declared(vec![])).is_none());
    }
}
</storycode:copy>

### Assemble `crates/hick-term/src/turbo.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/src/turbo.rs"><storycode:paste select="#f-aded4ac572de-turbo_choice"/><storycode:paste select="#f-aded4ac572de-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/terminal/AttentionCard.tsx](../../../mechanisms/apps/web/src/terminal/AttentionCard.tsx.md#output-da1d3639d06b)
- [crates/hick-term/src/prompt.rs](contract.md#output-c05b5ea8cc95)
- [crates/hick-term/src/registry.rs](../../../mechanisms/crates/hick-term/src/registry.rs.md#output-8678d496098a)
- [crates/hick-term/src/turbo.rs](contract.md#output-aded4ac572de)

### Tests and fixtures

- [apps/web/src/terminal/AttentionCard.test.tsx](../../../mechanisms/apps/web/src/terminal/AttentionCard.tsx.md#output-b44600c6bdc4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
