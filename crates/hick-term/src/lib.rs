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
pub mod attention;
pub mod classify;
pub mod command;
pub mod config;
pub mod git;
pub mod prompt;
pub mod registry;
pub mod screen;
pub mod session;
pub mod shell_integration;
pub mod turbo;

pub use attention::{Claim, attention_order};
pub use classify::{IDLE_AFTER_MS, SessionState, Signals, classify};
pub use config::TermConfig;
pub use git::{GitFacts, add_worktree};
pub use prompt::{looks_like_a_question, question_in};
pub use registry::Terminals;
pub use screen::Screen;
pub use session::{Choice, Prompt, PromptSource, Session, SessionSpec, SessionSummary};
pub use shell_integration::Integration;
pub use turbo::turbo_choice;
