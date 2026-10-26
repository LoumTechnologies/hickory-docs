<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Install a supported missing debugger and resume the requested launch

For: Authors debugging generated code while navigating its document source.

Choose Debug on a supported program with no adapter installed. Setup should continue that launch without requiring another Debug click, and dismissal must block a late launch.

## What must happen

A typed catalogue-supported missing-adapter result starts the existing confined installer and then retries the originating program. Setup exposes actual progress and diagnostics. Failure offers an explicit retry; repeated missing-adapter results do not cause an automatic installation loop. Dismissed or replaced setup cannot start a late session. Requests for the same adapter share installation. Unsupported adapters retain their ecosystem-specific guidance. Installer paths are shell-quoted and redirected home applies to the complete compound install.

## Limits of the evidence

The UI witness uses a controlled installer response. It does not download an adapter, establish platform confinement or certify a packaged debugger.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-missing-debugger-is-a-button.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The current Debug action authorizes prerequisite setup and supersedes the former separate-install-button decision. Keep retry, cancellation and typed-error boundaries in the current statement.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-943c146869d8"></a>
## `crates/hickory-cli/src/serve/install.rs`

Related contracts: [Install a supported missing debugger and resume the requested launch](contract.md).

<a id="f-943c146869d8-InstallRequest"></a>
### `InstallRequest`

Fetching a missing tool from inside the app: `POST /api/install`.
The app can already tell you a debug adapter is missing. Until now that
was where it stopped — a sentence naming a command, and a person expected
to leave, find a terminal, and come back. Every other editor makes this a
button, and the gap between "here is a command" and "yes, do that" is
most of what separates an IDE from a tool with a CLI attached.
It installs through exactly the same catalogue and the same confinement
`hick dap install` uses, into the same prefix, so a tool fetched here and
a tool fetched in a terminal are the same tool. This route adds an
affordance, never a second mechanism.
**Only what a catalogue can install is offered.** The failure carries
`offer_install` only when `hick dap install <language>` is a real command
(`hick_dap::MissingAdapter::installable`), so a button never appears for a
language whose adapter comes from its own ecosystem — those say so in
prose, which is all there is to say.

<storycode:copy id="f-943c146869d8-InstallRequest">
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

</storycode:copy>
<a id="f-943c146869d8-install"></a>
### `install`

`POST /api/install` — fetch a tool the project needs.
Blocking, and deliberately so: a download with a pinned hash either lands
or does not, it takes seconds rather than minutes, and a progress stream
would be a second protocol for a wait that is already short. The client
says "Installing…" and gets one answer.

<storycode:copy id="f-943c146869d8-install">
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/install.rs"><storycode:paste select="#f-943c146869d8-InstallRequest"/><storycode:paste select="#f-943c146869d8-install"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [apps/web/src/debug/DebugStrip.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-09306f4ec1a5)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)
- [crates/hick-dap/src/program.rs](../../../mechanisms/crates/hick-dap/src/program.rs.md#output-dd5fefdc2d90)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)
- [crates/hickory-cli/src/serve/install.rs](contract.md#output-943c146869d8)
- [crates/hickory-cli/src/tool_install.rs](../an-adapter-that-ships-as-an-archive-installs-like-any-other/contract.md#output-2e9d7402a43b)

### Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/debug/useDebugger.test.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-101aa3d58173)

### Focused witnesses

- `installs a missing debugger and resumes without another click` in `apps/web/src/debug/DebugStrip.test.tsx`.
- `does not start a session after setup is dismissed` in `apps/web/src/debug/DebugStrip.test.tsx`.

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
