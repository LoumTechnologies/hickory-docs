//! Ratatui-based terminal frontend for Hick agents driven by [`hick_agent_sdk::UiEvent`] /
//! [`hick_agent_sdk::UiCommand`].
//!
//! Binary crates multiplex core agent events (and auxiliary prompts) onto a shared
//! [`hick_agent_sdk::UiEvent`] channel; this crate renders the unified stream.

mod run;

pub use run::{run_tui, RatingSubmission, TabSource, TuiConfig};
