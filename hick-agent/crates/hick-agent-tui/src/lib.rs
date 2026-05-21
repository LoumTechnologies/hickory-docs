//! Ratatui-based terminal frontend for Hick agents driven by [`hick_agent_sdk::UiEvent`] /
//! [`hick_agent_sdk::UiCommand`].
//!
//! Binary crates multiplex core agent events (and auxiliary prompts) onto a shared
//! [`hick_agent_sdk::UiEvent`] channel; this crate renders the unified stream.

mod run;
mod setup;

pub use run::{run_tui, PruneAction, RatingSubmission, TabSource, TuiConfig};
pub use setup::{SetupResult, run_setup_tui};
