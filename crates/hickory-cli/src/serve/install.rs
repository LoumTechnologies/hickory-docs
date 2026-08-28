//! Fetching a missing tool from inside the app: `POST /api/install`.
//!
//! The app can already tell you a debug adapter is missing. Until now that
//! was where it stopped — a sentence naming a command, and a person expected
//! to leave, find a terminal, and come back. Every other editor makes this a
//! button, and the gap between "here is a command" and "yes, do that" is
//! most of what separates an IDE from a tool with a CLI attached.
//!
//! It installs through exactly the same catalogue and the same confinement
//! `hick dap install` uses, into the same prefix, so a tool fetched here and
//! a tool fetched in a terminal are the same tool. This route adds an
//! affordance, never a second mechanism.
//!
//! **Only what a catalogue can install is offered.** The failure carries
//! `offer_install` only when `hick dap install <language>` is a real command
//! (`hick_dap::MissingAdapter::installable`), so a button never appears for a
//! language whose adapter comes from its own ecosystem — those say so in
//! prose, which is all there is to say.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct InstallRequest {
    /// What kind of tool. `dap` today; `lsp` and `index` are the same
    /// gesture and are named here so the client's shape does not have to
    /// change when they arrive.
    pub kind: String,
    pub language: String,
}

/// `POST /api/install` — fetch a tool the project needs.
///
/// Blocking, and deliberately so: a download with a pinned hash either lands
/// or does not, it takes seconds rather than minutes, and a progress stream
/// would be a second protocol for a wait that is already short. The client
/// says "Installing…" and gets one answer.
pub async fn install(
    State(state): State<LocalState>,
    Json(body): Json<InstallRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let language = body.language.clone();
    match body.kind.as_str() {
        "dap" => {
            // A blocking install inside an async handler would hold a
            // runtime worker for the length of a download.
            let path =
                tokio::task::spawn_blocking(move || crate::dap_install::install(&root, &language))
                    .await
                    .map_err(|e| {
                        ApiError::internal(format!("the install task did not finish: {e}"))
                    })?
                    .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
            Ok(Json(json!({
                "installed": body.language,
                "path": path.to_string_lossy(),
            })))
        }
        other => Err(ApiError::bad_request(format!(
            "`{other}` is not something this can install. Today: `dap`."
        ))),
    }
}
