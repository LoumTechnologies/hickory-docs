<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-term/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2cfc624164ae"></a>
## `crates/hick-term/src/lib.rs`

<a id="f-2cfc624164ae-anchor"></a>
### `anchor`

Terminals that keep track of themselves.
A terminal in this app is not a pane that happens to have a shell in it.
It is a **session**: a named piece of work, running in a directory, on a
branch, that knows whether it is busy, blocked, or done — and that keeps
knowing while its tab is closed.
That distinction is the whole feature. Once a session can say what it is
doing, the window can put the ones that need a person in one queue and
leave the working ones alone, which is the only way running several
agents at once stops being a game of checking on them.
## The parts
- [`classify`] — five states from what we can observe. Pure.
- [`attention`] — the order in which sessions may interrupt you. Pure.
- [`turbo`] — which prompts may be answered without you. Pure, and mostly
about refusing.
- [`prompt`] — recognising a question on a stranger's screen. A guess, and
quarantined as one.
- [`session`] — one PTY, its scrollback, and its state.
- [`registry`] — every session in the window, and the one queue.
- [`git`] — branch and dirty, which is what separates "finished" from
"finished, and now what?".
- [`config`] — `HICKORY_SHELL` and `HICKORY_TERM_SCROLLBACK`, typed and
validated at startup.
- [`shell_integration`] — how a shell is told to report its directory,
since on macOS none of them does by default.
## What is not here
Nothing that reaches off this machine. There is no relay, no pairing, and
no companion app: a session is visible to the person whose machine it runs
on, which is the same person the server already answers. See
`docs/specs/freeform/local-only.md`.

<storycode:copy id="f-2cfc624164ae-anchor">
//! Terminals that keep track of themselves.
//!
//! A terminal in this app is not a pane that happens to have a shell in it.
//! It is a **session**: a named piece of work, running in a directory, on a
//! branch, that knows whether it is busy, blocked, or done — and that keeps
//! knowing while its tab is closed.
//!
//! That distinction is the whole feature. Once a session can say what it is
//! doing, the window can put the ones that need a person in one queue and
//! leave the working ones alone, which is the only way running several
//! agents at once stops being a game of checking on them.
//!
//! ## The parts
//!
//! - [`classify`] — five states from what we can observe. Pure.
//! - [`attention`] — the order in which sessions may interrupt you. Pure.
//! - [`turbo`] — which prompts may be answered without you. Pure, and mostly
//!   about refusing.
//! - [`prompt`] — recognising a question on a stranger's screen. A guess, and
//!   quarantined as one.
//! - [`session`] — one PTY, its scrollback, and its state.
//! - [`registry`] — every session in the window, and the one queue.
//! - [`git`] — branch and dirty, which is what separates "finished" from
//!   "finished, and now what?".
//! - [`config`] — `HICKORY_SHELL` and `HICKORY_TERM_SCROLLBACK`, typed and
//!   validated at startup.
//! - [`shell_integration`] — how a shell is told to report its directory,
//!   since on macOS none of them does by default.
//!
//! ## What is not here
//!
//! Nothing that reaches off this machine. There is no relay, no pairing, and
//! no companion app: a session is visible to the person whose machine it runs
//! on, which is the same person the server already answers. See
//! `docs/specs/freeform/local-only.md`.

pub mod anchor;
</storycode:copy>
<a id="f-2cfc624164ae-attention"></a>
### `attention`

<storycode:copy id="f-2cfc624164ae-attention">
pub mod attention;
</storycode:copy>
<a id="f-2cfc624164ae-classify"></a>
### `classify`

<storycode:copy id="f-2cfc624164ae-classify">
pub mod classify;
</storycode:copy>
<a id="f-2cfc624164ae-command"></a>
### `command`

<storycode:copy id="f-2cfc624164ae-command">
pub mod command;
</storycode:copy>
<a id="f-2cfc624164ae-config"></a>
### `config`

<storycode:copy id="f-2cfc624164ae-config">
pub mod config;
</storycode:copy>
<a id="f-2cfc624164ae-git"></a>
### `git`

<storycode:copy id="f-2cfc624164ae-git">
pub mod git;
</storycode:copy>
<a id="f-2cfc624164ae-prompt"></a>
### `prompt`

<storycode:copy id="f-2cfc624164ae-prompt">
pub mod prompt;
</storycode:copy>
<a id="f-2cfc624164ae-registry"></a>
### `registry`

<storycode:copy id="f-2cfc624164ae-registry">
pub mod registry;
</storycode:copy>
<a id="f-2cfc624164ae-screen"></a>
### `screen`

<storycode:copy id="f-2cfc624164ae-screen">
pub mod screen;
</storycode:copy>
<a id="f-2cfc624164ae-session"></a>
### `session`

<storycode:copy id="f-2cfc624164ae-session">
pub mod session;
</storycode:copy>
<a id="f-2cfc624164ae-shell_integration"></a>
### `shell_integration`

<storycode:copy id="f-2cfc624164ae-shell_integration">
pub mod shell_integration;
</storycode:copy>
<a id="f-2cfc624164ae-turbo"></a>
### `turbo`

<storycode:copy id="f-2cfc624164ae-turbo">
pub mod turbo;

</storycode:copy>
<a id="f-2cfc624164ae-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration">
pub use attention::{Claim, attention_order};
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-2">
pub use classify::{IDLE_AFTER_MS, SessionState, Signals, classify};
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-3">
pub use config::TermConfig;
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-4">
pub use git::{GitFacts, add_worktree};
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-5">
pub use prompt::{looks_like_a_question, question_in};
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-6">
pub use registry::Terminals;
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-7">
pub use screen::Screen;
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-8">
pub use session::{Choice, Prompt, PromptSource, Session, SessionSpec, SessionSummary};
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-9"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-9">
pub use shell_integration::Integration;
</storycode:copy>
<a id="f-2cfc624164ae-use_declaration-10"></a>
### `use_declaration`

<storycode:copy id="f-2cfc624164ae-use_declaration-10">
pub use turbo::turbo_choice;
</storycode:copy>

### Assemble `crates/hick-term/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/src/lib.rs"><storycode:paste select="#f-2cfc624164ae-anchor"/><storycode:paste select="#f-2cfc624164ae-attention"/><storycode:paste select="#f-2cfc624164ae-classify"/><storycode:paste select="#f-2cfc624164ae-command"/><storycode:paste select="#f-2cfc624164ae-config"/><storycode:paste select="#f-2cfc624164ae-git"/><storycode:paste select="#f-2cfc624164ae-prompt"/><storycode:paste select="#f-2cfc624164ae-registry"/><storycode:paste select="#f-2cfc624164ae-screen"/><storycode:paste select="#f-2cfc624164ae-session"/><storycode:paste select="#f-2cfc624164ae-shell_integration"/><storycode:paste select="#f-2cfc624164ae-turbo"/><storycode:paste select="#f-2cfc624164ae-use_declaration"/><storycode:paste select="#f-2cfc624164ae-use_declaration-2"/><storycode:paste select="#f-2cfc624164ae-use_declaration-3"/><storycode:paste select="#f-2cfc624164ae-use_declaration-4"/><storycode:paste select="#f-2cfc624164ae-use_declaration-5"/><storycode:paste select="#f-2cfc624164ae-use_declaration-6"/><storycode:paste select="#f-2cfc624164ae-use_declaration-7"/><storycode:paste select="#f-2cfc624164ae-use_declaration-8"/><storycode:paste select="#f-2cfc624164ae-use_declaration-9"/><storycode:paste select="#f-2cfc624164ae-use_declaration-10"/></storycode:file>

</storycode:doc>
