<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-09ea558208cd"></a>
## `crates/hickory-cli/src/serve/mod.rs`

Related contracts: [A desktop app bundles its own Python agent](../../../../../guarantees/agent/a-desktop-app-bundles-its-python-agent/contract.md), [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [A Provider Key Entered in Settings Stays on This Machine](../../../../../guarantees/agent/keys-are-stored-locally-and-never-leave/contract.md), [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A New Project Is A Recipe Commit, Made As One Act](../../../../../guarantees/authoring/a-new-project-is-a-recipe-commit/contract.md), [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [A Tree Row Hands Its File To The Rest Of The Machine](../../../../../guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md), [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md), [Keep divergent output until an explicit decision names the write](../../../../../guarantees/authoring/an-output-that-cannot-be-carried-back-is-held/contract.md), [The Desktop App And The CLI Are One Engine](../../../../../guarantees/authoring/the-app-and-the-cli-are-one-engine/contract.md), [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [The Git Pane Does The Daily Loop](../../../../../guarantees/collaboration/the-git-pane-does-the-daily-loop/contract.md), [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md), [One Language Server Session Per Workspace](../../../../../guarantees/editor-intelligence/one-language-server-per-workspace/contract.md), [Save Can Format First, With The File's Own Formatter](../../../../../guarantees/editor-intelligence/save-can-format-first/contract.md), [An Action Is Asked Of The Element, And Carried Out By The Host](../../../../../guarantees/language/an-action-is-asked-of-the-element/contract.md), [The History Lens Reads The Repository As A Story](../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-09ea558208cd-acp"></a>
### `acp`

The local document server — the engine behind the desktop app's window.
One process on the user's own machine answers the subset of the API a
document view needs (from files, not a database) and runs the rooms
`hickory-collab` defines. What that buys:
- the lineage ribbons for a `.md` file on your own disk, which nothing
else can show you;
- an editor whose durable state is your working tree, so your other editor
and `git diff` see every keystroke;
- execution on your hardware, under your executor.
## This is a library, not a command
There is no `hick serve`. The desktop app links this module and runs it
in-process on loopback, loading the React client Tauri bundles with it;
`hick up` is headless and never starts it. That is why nothing here serves
static files, and why there is no `--web-dist`.
## Why there is no authorization
This server binds `127.0.0.1` and answers exactly one person: the one whose
machine it is. There is no second principal to distinguish from the first,
so there is no token, no scope, and no capability link. A check that can
only ever pass is worse than no check, because it reads like protection.
Sharing is not a feature this product has — not disabled, not deferred.
See `docs/specs/freeform/local-only.md`.

<storycode:copy id="f-09ea558208cd-acp">
//! The local document server — the engine behind the desktop app's window.
//!
//! One process on the user's own machine answers the subset of the API a
//! document view needs (from files, not a database) and runs the rooms
//! `hickory-collab` defines. What that buys:
//!
//! - the lineage ribbons for a `.md` file on your own disk, which nothing
//!   else can show you;
//! - an editor whose durable state is your working tree, so your other editor
//!   and `git diff` see every keystroke;
//! - execution on your hardware, under your executor.
//!
//! ## This is a library, not a command
//!
//! There is no `hick serve`. The desktop app links this module and runs it
//! in-process on loopback, loading the React client Tauri bundles with it;
//! `hick up` is headless and never starts it. That is why nothing here serves
//! static files, and why there is no `--web-dist`.
//!
//! ## Why there is no authorization
//!
//! This server binds `127.0.0.1` and answers exactly one person: the one whose
//! machine it is. There is no second principal to distinguish from the first,
//! so there is no token, no scope, and no capability link. A check that can
//! only ever pass is worse than no check, because it reads like protection.
//!
//! Sharing is not a feature this product has — not disabled, not deferred.
//! See `docs/specs/freeform/local-only.md`.

pub mod acp;
</storycode:copy>
<a id="f-09ea558208cd-agent"></a>
### `agent`

<storycode:copy id="f-09ea558208cd-agent">
pub mod agent;
</storycode:copy>
<a id="f-09ea558208cd-agent_context"></a>
### `agent_context`

<storycode:copy id="f-09ea558208cd-agent_context">
pub(crate) mod agent_context;
</storycode:copy>
<a id="f-09ea558208cd-agent_projects"></a>
### `agent_projects`

<storycode:copy id="f-09ea558208cd-agent_projects">
mod agent_projects;
</storycode:copy>
<a id="f-09ea558208cd-agent_review"></a>
### `agent_review`

<storycode:copy id="f-09ea558208cd-agent_review">
mod agent_review;
</storycode:copy>
<a id="f-09ea558208cd-anchored"></a>
### `anchored`

<storycode:copy id="f-09ea558208cd-anchored">
pub mod anchored;
</storycode:copy>
<a id="f-09ea558208cd-api"></a>
### `api`

<storycode:copy id="f-09ea558208cd-api">
pub mod api;
</storycode:copy>
<a id="f-09ea558208cd-asset"></a>
### `asset`

<storycode:copy id="f-09ea558208cd-asset">
pub mod asset;
</storycode:copy>
<a id="f-09ea558208cd-audit_evidence"></a>
### `audit_evidence`

<storycode:copy id="f-09ea558208cd-audit_evidence">
mod audit_evidence;
</storycode:copy>
<a id="f-09ea558208cd-bisect"></a>
### `bisect`

<storycode:copy id="f-09ea558208cd-bisect">
pub mod bisect;
</storycode:copy>
<a id="f-09ea558208cd-changeset_drafts"></a>
### `changeset_drafts`

<storycode:copy id="f-09ea558208cd-changeset_drafts">
mod changeset_drafts;
</storycode:copy>
<a id="f-09ea558208cd-commit_reading"></a>
### `commit_reading`

<storycode:copy id="f-09ea558208cd-commit_reading">
pub mod commit_reading;
</storycode:copy>
<a id="f-09ea558208cd-dataset"></a>
### `dataset`

<storycode:copy id="f-09ea558208cd-dataset">
pub mod dataset;
</storycode:copy>
<a id="f-09ea558208cd-debug_bridge"></a>
### `debug_bridge`

<storycode:copy id="f-09ea558208cd-debug_bridge">
pub mod debug_bridge;
</storycode:copy>
<a id="f-09ea558208cd-private_prediction_acp"></a>
### `private_prediction_acp`

<storycode:copy id="f-09ea558208cd-private_prediction_acp">
mod private_prediction_acp;
</storycode:copy>
<a id="f-09ea558208cd-private_prediction_agent"></a>
### `private_prediction_agent`

<storycode:copy id="f-09ea558208cd-private_prediction_agent">
mod private_prediction_agent;
</storycode:copy>
<a id="f-09ea558208cd-private_prediction_debugger"></a>
### `private_prediction_debugger`

<storycode:copy id="f-09ea558208cd-private_prediction_debugger">
mod private_prediction_debugger;
</storycode:copy>
<a id="f-09ea558208cd-private_prediction_recovery"></a>
### `private_prediction_recovery`

<storycode:copy id="f-09ea558208cd-private_prediction_recovery">
pub(crate) mod private_prediction_recovery;
</storycode:copy>
<a id="f-09ea558208cd-selected_run"></a>
### `selected_run`

<storycode:copy id="f-09ea558208cd-selected_run">
mod selected_run;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration">
pub use private_prediction_debugger::Plan as PrivatePredictionDebuggerPlan;
</storycode:copy>
<a id="f-09ea558208cd-private_prediction_startup"></a>
### `private_prediction_startup`

<storycode:copy id="f-09ea558208cd-private_prediction_startup">
mod private_prediction_startup;
</storycode:copy>
<a id="f-09ea558208cd-private_predictions"></a>
### `private_predictions`

<storycode:copy id="f-09ea558208cd-private_predictions">
pub(crate) mod private_predictions;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-2">
pub use private_prediction_startup::Startup as PrivatePredictionStartup;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-3">
pub use private_predictions::Binding as PrivatePredictionBinding;
</storycode:copy>
<a id="f-09ea558208cd-capabilities"></a>
### `capabilities`

<storycode:copy id="f-09ea558208cd-capabilities">
mod capabilities;
</storycode:copy>
<a id="f-09ea558208cd-engineer_tour"></a>
### `engineer_tour`

<storycode:copy id="f-09ea558208cd-engineer_tour">
mod engineer_tour;
</storycode:copy>
<a id="f-09ea558208cd-environments"></a>
### `environments`

<storycode:copy id="f-09ea558208cd-environments">
pub mod environments;
</storycode:copy>
<a id="f-09ea558208cd-files_ops"></a>
### `files_ops`

<storycode:copy id="f-09ea558208cd-files_ops">
pub mod files_ops;
</storycode:copy>
<a id="f-09ea558208cd-find"></a>
### `find`

<storycode:copy id="f-09ea558208cd-find">
pub mod find;
</storycode:copy>
<a id="f-09ea558208cd-formula"></a>
### `formula`

<storycode:copy id="f-09ea558208cd-formula">
pub mod formula;
</storycode:copy>
<a id="f-09ea558208cd-git"></a>
### `git`

<storycode:copy id="f-09ea558208cd-git">
pub mod git;
</storycode:copy>
<a id="f-09ea558208cd-git_ops"></a>
### `git_ops`

<storycode:copy id="f-09ea558208cd-git_ops">
pub mod git_ops;
</storycode:copy>
<a id="f-09ea558208cd-github"></a>
### `github`

<storycode:copy id="f-09ea558208cd-github">
pub mod github;
</storycode:copy>
<a id="f-09ea558208cd-github_reviews"></a>
### `github_reviews`

<storycode:copy id="f-09ea558208cd-github_reviews">
mod github_reviews;
</storycode:copy>
<a id="f-09ea558208cd-history"></a>
### `history`

<storycode:copy id="f-09ea558208cd-history">
pub mod history;
</storycode:copy>
<a id="f-09ea558208cd-install"></a>
### `install`

<storycode:copy id="f-09ea558208cd-install">
pub mod install;
</storycode:copy>
<a id="f-09ea558208cd-lsp_bridge"></a>
### `lsp_bridge`

<storycode:copy id="f-09ea558208cd-lsp_bridge">
pub mod lsp_bridge;
</storycode:copy>
<a id="f-09ea558208cd-meeting_import"></a>
### `meeting_import`

<storycode:copy id="f-09ea558208cd-meeting_import">
mod meeting_import;
</storycode:copy>
<a id="f-09ea558208cd-meeting_recording"></a>
### `meeting_recording`

<storycode:copy id="f-09ea558208cd-meeting_recording">
mod meeting_recording;
</storycode:copy>
<a id="f-09ea558208cd-merged"></a>
### `merged`

<storycode:copy id="f-09ea558208cd-merged">
pub mod merged;
</storycode:copy>
<a id="f-09ea558208cd-merged_write"></a>
### `merged_write`

<storycode:copy id="f-09ea558208cd-merged_write">
mod merged_write;
</storycode:copy>
<a id="f-09ea558208cd-native_projects"></a>
### `native_projects`

<storycode:copy id="f-09ea558208cd-native_projects">
mod native_projects;
</storycode:copy>
<a id="f-09ea558208cd-output_resolution"></a>
### `output_resolution`

<storycode:copy id="f-09ea558208cd-output_resolution">
mod output_resolution;
</storycode:copy>
<a id="f-09ea558208cd-outputs"></a>
### `outputs`

<storycode:copy id="f-09ea558208cd-outputs">
pub mod outputs;
</storycode:copy>
<a id="f-09ea558208cd-performance"></a>
### `performance`

<storycode:copy id="f-09ea558208cd-performance">
pub mod performance;
</storycode:copy>
<a id="f-09ea558208cd-plain_file"></a>
### `plain_file`

<storycode:copy id="f-09ea558208cd-plain_file">
pub mod plain_file;
</storycode:copy>
<a id="f-09ea558208cd-private_practice"></a>
### `private_practice`

<storycode:copy id="f-09ea558208cd-private_practice">
mod private_practice;
</storycode:copy>
<a id="f-09ea558208cd-project_comparison"></a>
### `project_comparison`

<storycode:copy id="f-09ea558208cd-project_comparison">
mod project_comparison;
</storycode:copy>
<a id="f-09ea558208cd-project_incorporation"></a>
### `project_incorporation`

<storycode:copy id="f-09ea558208cd-project_incorporation">
mod project_incorporation;
</storycode:copy>
<a id="f-09ea558208cd-project_publish"></a>
### `project_publish`

<storycode:copy id="f-09ea558208cd-project_publish">
mod project_publish;
</storycode:copy>
<a id="f-09ea558208cd-python_agent"></a>
### `python_agent`

<storycode:copy id="f-09ea558208cd-python_agent">
mod python_agent;
</storycode:copy>
<a id="f-09ea558208cd-revision_projects"></a>
### `revision_projects`

<storycode:copy id="f-09ea558208cd-revision_projects">
mod revision_projects;
</storycode:copy>
<a id="f-09ea558208cd-runtime_runs"></a>
### `runtime_runs`

<storycode:copy id="f-09ea558208cd-runtime_runs">
mod runtime_runs;
</storycode:copy>
<a id="f-09ea558208cd-ui_actions"></a>
### `ui_actions`

<storycode:copy id="f-09ea558208cd-ui_actions">
pub(crate) mod ui_actions;
</storycode:copy>
<a id="f-09ea558208cd-ui_images"></a>
### `ui_images`

<storycode:copy id="f-09ea558208cd-ui_images">
pub(crate) mod ui_images;
</storycode:copy>
<a id="f-09ea558208cd-ui_verification"></a>
### `ui_verification`

<storycode:copy id="f-09ea558208cd-ui_verification">
mod ui_verification;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-4">
pub use python_agent::bundled_runtime;
</storycode:copy>
<a id="f-09ea558208cd-fleet_pairings"></a>
### `fleet_pairings`

<storycode:copy id="f-09ea558208cd-fleet_pairings">
mod fleet_pairings;
</storycode:copy>
<a id="f-09ea558208cd-launch_profiles"></a>
### `launch_profiles`

<storycode:copy id="f-09ea558208cd-launch_profiles">
mod launch_profiles;
</storycode:copy>
<a id="f-09ea558208cd-refactor"></a>
### `refactor`

<storycode:copy id="f-09ea558208cd-refactor">
pub mod refactor;
</storycode:copy>
<a id="f-09ea558208cd-representation"></a>
### `representation`

<storycode:copy id="f-09ea558208cd-representation">
pub mod representation;
</storycode:copy>
<a id="f-09ea558208cd-representation_store"></a>
### `representation_store`

<storycode:copy id="f-09ea558208cd-representation_store">
mod representation_store;
</storycode:copy>
<a id="f-09ea558208cd-representation_tools"></a>
### `representation_tools`

<storycode:copy id="f-09ea558208cd-representation_tools">
pub mod representation_tools;
</storycode:copy>
<a id="f-09ea558208cd-reveal"></a>
### `reveal`

<storycode:copy id="f-09ea558208cd-reveal">
pub mod reveal;
</storycode:copy>
<a id="f-09ea558208cd-reviews"></a>
### `reviews`

<storycode:copy id="f-09ea558208cd-reviews">
mod reviews;
</storycode:copy>
<a id="f-09ea558208cd-revision"></a>
### `revision`

<storycode:copy id="f-09ea558208cd-revision">
pub mod revision;
</storycode:copy>
<a id="f-09ea558208cd-sample"></a>
### `sample`

<storycode:copy id="f-09ea558208cd-sample">
pub mod sample;
</storycode:copy>
<a id="f-09ea558208cd-scaffold"></a>
### `scaffold`

<storycode:copy id="f-09ea558208cd-scaffold">
pub mod scaffold;
</storycode:copy>
<a id="f-09ea558208cd-sessions"></a>
### `sessions`

<storycode:copy id="f-09ea558208cd-sessions">
mod sessions;
</storycode:copy>
<a id="f-09ea558208cd-shell"></a>
### `shell`

<storycode:copy id="f-09ea558208cd-shell">
pub mod shell;
</storycode:copy>
<a id="f-09ea558208cd-socket"></a>
### `socket`

<storycode:copy id="f-09ea558208cd-socket">
pub mod socket;
</storycode:copy>
<a id="f-09ea558208cd-store"></a>
### `store`

<storycode:copy id="f-09ea558208cd-store">
pub mod store;
</storycode:copy>
<a id="f-09ea558208cd-story"></a>
### `story`

<storycode:copy id="f-09ea558208cd-story">
pub mod story;
</storycode:copy>
<a id="f-09ea558208cd-terminal"></a>
### `terminal`

<storycode:copy id="f-09ea558208cd-terminal">
pub mod terminal;
</storycode:copy>
<a id="f-09ea558208cd-test_coverage"></a>
### `test_coverage`

<storycode:copy id="f-09ea558208cd-test_coverage">
pub mod test_coverage;
</storycode:copy>
<a id="f-09ea558208cd-test_run"></a>
### `test_run`

<storycode:copy id="f-09ea558208cd-test_run">
pub mod test_run;
</storycode:copy>
<a id="f-09ea558208cd-watch"></a>
### `watch`

<storycode:copy id="f-09ea558208cd-watch">
pub mod watch;
</storycode:copy>
<a id="f-09ea558208cd-workspace"></a>
### `workspace`

<storycode:copy id="f-09ea558208cd-workspace">
pub mod workspace;
</storycode:copy>
<a id="f-09ea558208cd-workspace_fs"></a>
### `workspace_fs`

<storycode:copy id="f-09ea558208cd-workspace_fs">
pub mod workspace_fs;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-5">
use std::collections::HashMap;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-6">
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-7">
use std::path::{Path, PathBuf};
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-8">
use std::sync::{Arc, Mutex};

</storycode:copy>
<a id="f-09ea558208cd-use_declaration-9"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-9">
use anyhow::{Context as _, Result};
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-10"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-10">
use axum::Router;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-11"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-11">
use axum::routing::{delete, get, post, put};
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-12"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-12">
use hickory_collab::RoomRegistry;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-13"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-13">
use serde_json::{Value, json};

</storycode:copy>
<a id="f-09ea558208cd-use_declaration-14"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-14">
use crate::{ExecutorChoice, RunMode};
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-15"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-15">
use api::{ApiError, ApiResult};
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-16"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-16">
use store::{DocIndex, FileDocStore};

</storycode:copy>
<a id="f-09ea558208cd-use_declaration-17"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-17">
pub use shell::{OpenWhere, Shell, WindowOperation, WindowStatus};

</storycode:copy>
<a id="f-09ea558208cd-ServeOptions"></a>
### `ServeOptions`

How a local session was asked to run.
There is no `lan`, `scope`, `public`, or `web_dist` field, and there will
not be: this server binds loopback, answers one user — the one whose
machine it is — and has no UI of its own. The desktop app bundles the
client; the CLI does not serve HTML. See
`docs/specs/freeform/local-only.md`.

<storycode:copy id="f-09ea558208cd-ServeOptions">
/// How a local session was asked to run.
///
/// There is no `lan`, `scope`, `public`, or `web_dist` field, and there will
/// not be: this server binds loopback, answers one user — the one whose
/// machine it is — and has no UI of its own. The desktop app bundles the
/// client; the CLI does not serve HTML. See
/// `docs/specs/freeform/local-only.md`.
pub struct ServeOptions {
    /// The document, or the directory of documents, to serve.
    pub target: PathBuf,
    /// Port to listen on. 0 lets the kernel choose.
    pub port: u16,
    /// Parameter overrides passed to every weave and run.
    pub params: Vec<(String, String)>,
    /// Executor for runs (`HICKORY_EXECUTOR`).
    pub executor: ExecutorChoice,
    /// Where the desktop app's Settings page persists LLM provider keys
    /// (`<app config dir>/llm-keys.json`). `None` — the CLI and the tests —
    /// means no file: keys come from the environment alone, exactly as
    /// before this field existed.
    pub key_store_path: Option<PathBuf>,
    /// Where the desktop app's Settings page persists UI settings — today
    /// the custom window title (`<app config dir>/ui.json`, beside
    /// `llm-keys.json`). `None` — the CLI — means nothing persists: the
    /// routes still answer, with defaults.
    pub ui_settings_path: Option<PathBuf>,
}

</storycode:copy>
<a id="f-09ea558208cd-RunRecord"></a>
### `RunRecord`

One recorded run, in the shape `GET /api/runs/:id` returns.

<storycode:copy id="f-09ea558208cd-RunRecord">
/// One recorded run, in the shape `GET /api/runs/:id` returns.
#[derive(Clone)]
pub struct RunRecord {
    pub document: String,
    pub check_only: bool,
    pub requested_source_hash: Option<String>,
    pub error: Option<String>,
    pub source_hash: Option<String>,
    pub status: String,
    pub started_at: String,
    pub blocks: Value,
}

</storycode:copy>
<a id="f-09ea558208cd-LocalState"></a>
### `LocalState`

<storycode:copy id="f-09ea558208cd-LocalState">
/// Everything a request handler needs.
#[derive(Clone)]
pub struct LocalState {
    pub(crate) private_prediction: Option<Arc<private_predictions::Bound>>,
    pub(crate) ui_actions: Arc<ui_actions::Hub>,
    pub(crate) fleet_pairings: Arc<fleet_pairings::Pairings>,
    /// Text files explicitly opened outside the workspace.
    pub opened_files: Arc<std::sync::RwLock<std::collections::HashSet<PathBuf>>>,
    pub representations: representation::Representations,
    pub bisects: bisect::Bisects,
    /// Whether this view explicitly opened a folder, rather than one file.
    pub folder_open: bool,
    pub index: Arc<DocIndex>,
    pub rooms: Arc<RoomRegistry>,
    pub runs: Arc<Mutex<HashMap<String, RunRecord>>>,
    pub observations: Arc<std::sync::Mutex<Option<hick_observe::LocalStore>>>,
    pub params: Arc<Vec<(String, String)>>,
    pub executor: ExecutorChoice,
    /// The project search engine, built on first use and reused: reopening
    /// per request would re-parse the whole embedding index every query.
    /// std Mutex, always locked inside `spawn_blocking`.
    pub search: Arc<std::sync::Mutex<Option<hick_search::SearchEngine>>>,
    /// The in-app agent's conversation trees and test seam. See
    /// [`agent::AgentHub`].
    pub agent: Arc<agent::AgentHub>,
    /// The store behind the rooms, kept here for its echo test
    /// ([`store::FileDocStore::was_own_write`]) — the in-app up-loop must
    /// not mistake the rooms' own persists for external edits.
    pub store: Arc<FileDocStore>,
    /// The LLM provider keys the Settings page manages, and where they
    /// persist. See [`KeySettings`].
    pub keys: Arc<KeySettings>,
    /// The UI settings the Settings page manages, and where they persist.
    /// See [`UiSettings`].
    pub ui: Arc<UiSettings>,
    /// Pinned refactor baselines, one per document under restructuring.
    /// Session state, deliberately in memory — see [`refactor`].
    pub refactors: Arc<Mutex<HashMap<String, refactor::RefactorBaseline>>>,
    /// Every terminal session in the window, and the attention queue across
    /// them. Sessions outlive their panes, so they belong to the session
    /// state rather than to any one client. See [`terminal`].
    pub terminals: Arc<hick_term::Terminals>,
    pub environments: Arc<environments::Environments>,
    /// Which terminals are writing into which documents. See [`anchored`];
    /// empty is the normal state, because a terminal writes nothing until
    /// somebody anchors it.
    pub anchors: Arc<anchored::Anchors>,
    /// The workspace's one language-server session, shared by every socket.
    /// See [`lsp_bridge::LspHub`].
    pub lsp: Arc<lsp_bridge::LspHub>,
    /// Generated files the loop is holding — root-relative path → why —
    /// published by the loop after every batch so the tree and the pane can
    /// say so. See `docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md`.
    pub held: Arc<std::sync::Mutex<HashMap<String, DivergedOutput>>>,
    /// What became of every scaffold this session started, by the terminal
    /// session that ran it. The terminal shows the person; this is how the
    /// app finds out. See [`scaffold`].
    pub scaffolds: scaffold::Scaffolds,
    /// What the shell around this server can do that the server cannot.
    ///
    /// Set by the desktop app after [`prepare`], and `None` everywhere else —
    /// under `hick up` the "window" is a browser tab somebody else owns.
    /// Everything behind it is refused with a sentence when it is `None`,
    /// never guessed at. See [`Shell`].
    pub shell: Arc<Mutex<Option<Shell>>>,
    /// The Python runtime the app that started this engine shipped inside its
    /// own bundle, when it did. `None` — the CLI and the tests — falls back to
    /// a runtime discovered from the executable ([`bundled_runtime`]). A
    /// runtime named in Settings always wins. See [`python_agent`].
    pub python_runtime_path: Option<PathBuf>,
    /// The running loop's command inbox, set when the loop starts.
    pub up_commands:
        Arc<std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<crate::up::UpCommand>>>>,
    pub writes: Arc<tokio::sync::Mutex<()>>,
    pub window_slot: Option<String>,
    pub engine_woven: Option<Arc<tokio::sync::Mutex<crate::up::state::WovenState>>>,
}

</storycode:copy>
<a id="f-09ea558208cd-DivergedOutput"></a>
### `DivergedOutput`

One diverged produced file, as the tree and the pane read it. Axis 3 of
`docs/specs/freeform/three-axes.md`.

<storycode:copy id="f-09ea558208cd-DivergedOutput">
/// One diverged produced file, as the tree and the pane read it. Axis 3 of
/// `docs/specs/freeform/three-axes.md`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DivergedOutput {
    /// `held` (somebody wrote it and it could not be carried back) or
    /// `kept` (the document cannot reproduce it yet).
    pub kind: &'static str,
    pub reason: String,
    /// The last bytes both sides agreed on.
    pub base: String,
    /// What the document produces now; empty for `kept`.
    pub theirs: String,
}

</storycode:copy>
<a id="f-09ea558208cd-KeySettings"></a>
### `KeySettings`

The session's provider-key settings: the live store the agent route reads
(so a key saved in Settings works on the very next turn, no restart), and
the file it persists to (`None` = env-only, the CLI's mode).

<storycode:copy id="f-09ea558208cd-KeySettings">
/// The session's provider-key settings: the live store the agent route reads
/// (so a key saved in Settings works on the very next turn, no restart), and
/// the file it persists to (`None` = env-only, the CLI's mode).
pub struct KeySettings {
    pub store: std::sync::RwLock<hickory_agent::KeyStore>,
    pub path: Option<PathBuf>,
}

</storycode:copy>
<a id="f-09ea558208cd-UiSettings"></a>
### `UiSettings`

The session's UI settings — mirrors [`KeySettings`]: a live store the
routes read and write, and the file it persists to (`None` = in-memory
only, the CLI's mode).

<storycode:copy id="f-09ea558208cd-UiSettings">
/// The session's UI settings — mirrors [`KeySettings`]: a live store the
/// routes read and write, and the file it persists to (`None` = in-memory
/// only, the CLI's mode).
pub struct UiSettings {
    pub store: std::sync::RwLock<UiStore>,
    pub path: Option<PathBuf>,
}

</storycode:copy>
<a id="f-09ea558208cd-UiStore"></a>
### `UiStore`

What `ui.json` holds: `{"window_title": string|null}`. The desktop shell
reads this file directly at launch to name the native window before any
page has loaded; the page reads it over `GET /api/settings/ui` for the
in-window `document.title`.

<storycode:copy id="f-09ea558208cd-UiStore">
/// What `ui.json` holds: `{"window_title": string|null}`. The desktop shell
/// reads this file directly at launch to name the native window before any
/// page has loaded; the page reads it over `GET /api/settings/ui` for the
/// in-window `document.title`.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UiStore {
    /// Custom window title; `None` means the default (the folder's name).
    #[serde(default)]
    pub window_title: Option<String>,
    /// Run the file's formatter when Save is chosen. Off by default: a
    /// formatter rewriting a document nobody asked it to is a surprise, and
    /// this product's documents are prose as much as code.
    #[serde(default)]
    pub format_on_save: bool,
    /// Keep recovery drafts for buffers whose file already exists. Off by
    /// default; unnamed documents are always drafts and do not consult this.
    #[serde(default)]
    pub retain_unsaved_saved_files: bool,
    /// The keyboard profile and the person's own overrides, as the page
    /// keeps them (`{"profile": "vscode", "overrides": {"editor.format":
    /// "Ctrl+Alt+L", …}}`). Opaque here: the catalogue of actions and the
    /// profiles are the page's (apps/web/src/lib/keymap.ts), and the engine
    /// only carries them between launches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keymap: Option<serde_json::Value>,
    /// The resolved accelerator for each native menu item, by menu id, in
    /// the shell's spelling (`CmdOrCtrl+S`). Written by the page whenever
    /// the keymap changes; read by the desktop shell when it builds the
    /// menu bar at launch. An item absent here keeps its built-in key.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub native_accelerators: std::collections::BTreeMap<String, String>,
}

</storycode:copy>
<a id="f-09ea558208cd-impl-UiStore"></a>
### `impl-UiStore`

<storycode:copy id="f-09ea558208cd-impl-UiStore">
impl UiStore {
    /// Read the file, tolerating its absence (a fresh install has none).
    /// A file that exists but does not parse is an error naming the file —
    /// silently discarding a setting the user made is worse than failing
    /// where the cause is visible.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw)
                .with_context(|| format!("unreadable UI settings in {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    /// Persist, creating the parent directory on first save.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let body = serde_json::to_string_pretty(self).context("encoding UI settings")?;
        store::write_atomic(path, body.as_bytes())
            .with_context(|| format!("writing {}", path.display()))
    }
}

</storycode:copy>
<a id="f-09ea558208cd-impl-LocalState"></a>
### `impl-LocalState`

<storycode:copy id="f-09ea558208cd-impl-LocalState">
impl LocalState {
    pub fn executor_kind(&self) -> &'static str {
        self.executor.as_str()
    }

    pub fn read_source(&self, id: &str) -> ApiResult<String> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
        std::fs::read_to_string(&path)
            .map_err(|e| ApiError::internal(format!("reading {}: {e}", path.display())))
    }

    /// Write a document's source, leaving a local-history stop.
    ///
    /// Everything the server writes to a `.md` file goes through here, so
    /// this is the one place that has to remember to record — which is why
    /// the KIND is a parameter rather than a guess. "A person saved this" and
    /// "the agent rewrote this at a hashline anchor" are the two facts a
    /// person scanning their history needs told apart, and only the caller
    /// knows which it is.
    pub fn write_source_as(
        &self,
        id: &str,
        source: &str,
        kind: hickory_workspace::history::ActKind,
        detail: Option<String>,
    ) -> ApiResult<()> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
        crate::history::record(
            self.index.root(),
            kind,
            detail,
            &[(path.clone(), source.as_bytes().to_vec())],
        );
        store::write_atomic(&path, source.as_bytes())
            .map_err(|e| ApiError::internal(format!("writing {}: {e:#}", path.display())))
    }

    pub fn write_source(&self, id: &str, source: &str) -> ApiResult<()> {
        self.write_source_as(id, source, hickory_workspace::history::ActKind::Saved, None)
    }

    /// Resolve a document path as provenance names it.
    ///
    /// Provenance carries the path the weave used, which for the primary
    /// document is what the pipeline was handed. Both the absolute form and
    /// the root-relative form are accepted, and anything that escapes the
    /// served root is refused — a guest must not be able to steer an edit into
    /// the host's home directory.
    fn resolve_doc_path(&self, doc_path: &str) -> ApiResult<PathBuf> {
        let candidate = Path::new(doc_path);
        let joined = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.index.root().join(candidate)
        };
        let root = self
            .index
            .root()
            .canonicalize()
            .unwrap_or_else(|_| self.index.root().to_path_buf());
        let resolved = joined.canonicalize().unwrap_or(joined);
        if !resolved.starts_with(&root) {
            return Err(ApiError::forbidden(format!(
                "{doc_path} is outside the directory being served"
            )));
        }
        Ok(resolved)
    }

    pub fn read_source_by_doc_path(&self, doc_path: &str) -> ApiResult<String> {
        let path = self.resolve_doc_path(doc_path)?;
        std::fs::read_to_string(&path)
            .map_err(|e| ApiError::internal(format!("reading {}: {e}", path.display())))
    }

    /// Write a document named the way provenance names it.
    ///
    /// This is the reverse edit's path: an edit somebody made in a GENERATED
    /// file, carried back into the source that produces it. One of the
    /// writers with no way back at all until now.
    pub fn write_source_by_doc_path(&self, doc_path: &str, source: &str) -> ApiResult<()> {
        let path = self.resolve_doc_path(doc_path)?;
        crate::history::record(
            self.index.root(),
            hickory_workspace::history::ActKind::ReverseEdit,
            Some(doc_path.to_string()),
            &[(path.clone(), source.as_bytes().to_vec())],
        );
        store::write_atomic(&path, source.as_bytes())
            .map_err(|e| ApiError::internal(format!("writing {}: {e:#}", path.display())))
    }

    /// The room id for a document named the way provenance names it.
    pub fn id_for_doc_path(&self, doc_path: &str) -> String {
        let rel = self
            .resolve_doc_path(doc_path)
            .ok()
            .and_then(|p| {
                p.strip_prefix(self.index.root().canonicalize().ok()?)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            })
            .unwrap_or_else(|| doc_path.to_string());
        self.index.id_for_path(&rel)
    }

    /// Weave (never execute) a document, from the text the room is serving if
    /// one is live, else from disk.
    ///
    /// Reading through the room matters: a collaborator's unsaved keystrokes
    /// are in the CRDT for up to the persist debounce, and rendering the file
    /// instead would show everyone a document that is a moment out of date.
    pub async fn weave(&self, id: &str) -> ApiResult<crate::DocRun> {
        crate::engine::writes::during(&self.writes, self.weave_inner(id)).await
    }

    async fn weave_inner(&self, id: &str) -> ApiResult<crate::DocRun> {
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;

        // Flush the live text to disk first: the pipeline reads files (and
        // upstream documents) from the filesystem, so a weave of in-memory
        // text alone would silently ignore the room's newest edits.
        if let Some(room) = self.rooms.get(id).await {
            let live = room.text().await;
            if live != std::fs::read_to_string(&path).unwrap_or_default() {
                store::write_atomic(&path, live.as_bytes())
                    .map_err(|e| ApiError::internal(format!("{e:#}")))?;
            }
        }

        crate::run_doc(&path, &self.params, RunMode::Weave, ExecutorChoice::Local)
            .await
            .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
    }
}

</storycode:copy>
<a id="f-09ea558208cd-router"></a>
### `router`

Router
The routes a document view needs, and nothing else.
No authorization layer: this server binds loopback and answers the person
whose machine it is. There is no second principal to distinguish from the
first, so there is no token to check — and a check that can only ever pass
is worse than none, because it reads like protection.
No `/me`, `/billing/plans`, or `/analytics/capture` either. Those existed
because the React client was shared with a hosted product and asked for
them on load. It is not shared any more.

<storycode:copy id="f-09ea558208cd-router">
// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// The routes a document view needs, and nothing else.
///
/// No authorization layer: this server binds loopback and answers the person
/// whose machine it is. There is no second principal to distinguish from the
/// first, so there is no token to check — and a check that can only ever pass
/// is worse than none, because it reads like protection.
///
/// No `/me`, `/billing/plans`, or `/analytics/capture` either. Those existed
/// because the React client was shared with a hosted product and asked for
/// them on load. It is not shared any more.
pub(crate) fn router(state: LocalState) -> Router {
    let api = Router::new()
        .route("/prediction/state", get(private_predictions::status))
        .route("/prediction/history", get(private_predictions::history))
        .route(
            "/prediction/completion",
            get(private_predictions::completion),
        )
        .route("/prediction/evidence", get(private_predictions::evidence))
        .route("/prediction/tool", post(private_predictions::source_tool))
        .route("/prediction/seal", post(private_predictions::seal))
        .route(
            "/prediction/debugger",
            post(private_prediction_debugger::command),
        )
        .route("/prediction/promote", post(private_predictions::promote))
        .route(
            "/prediction/agent",
            get(private_prediction_agent::status).post(private_prediction_agent::start),
        )
        .route(
            "/prediction/agent/decision",
            post(private_prediction_agent::decide),
        )
        .route(
            "/prediction/agent/stop",
            post(private_prediction_agent::stop),
        )
        .route("/prediction/preview", post(private_predictions::preview))
        .merge(ui_actions::routes())
        .merge(fleet_pairings::routes())
        .route("/projects", get(api::projects))
        .route(
            "/projects/{id}/docs",
            get(api::project_docs).post(api::create_doc),
        )
        .route("/docs/{id}", get(api::get_doc).put(api::put_doc))
        .route("/docs/{id}/render", get(api::render_doc))
        .route("/docs/{id}/snapshot", get(api::snapshot_doc))
        .route("/docs/{id}/project", post(project_comparison::capture))
        .route("/docs/{id}/project/apply", post(project_comparison::apply))
        .route("/docs/{id}/project/undo", post(project_comparison::undo))
        .merge(changeset_drafts::routes())
        .route("/elements", get(api::elements))
        .route("/dataset", get(dataset::preview))
        .route("/revision-projects", post(revision_projects::capture))
        .route("/launch-profiles", post(launch_profiles::list))
        .route("/launch-profiles/run", post(launch_profiles::run))
        .route("/project-contexts", post(native_projects::register))
        .route("/project-contexts/file", get(native_projects::file))
        .route(
            "/project-contexts/{id}",
            axum::routing::delete(native_projects::remove),
        )
        .route("/docs/{id}/blocks/{at}/{action}", post(api::block_action))
        .route("/docs/{id}/outputs", get(outputs::list_outputs))
        .route("/docs/{id}/outputs/file", get(outputs::get_output_file))
        .route("/docs/{id}/outputs/edit", post(outputs::edit_outputs))
        .route("/docs/{id}/context", get(api::get_context))
        .route("/docs/{id}/cites", get(api::get_cites))
        .route("/docs/{id}/history", get(history::history))
        .route("/docs/{id}/replay", get(history::replay))
        .route("/sessions/view", get(api::session_view))
        .route("/sessions/audit", post(sessions::annotate))
        .route("/reviews", get(reviews::list).post(reviews::create))
        .route("/docs/{id}/refactor/begin", post(refactor::begin))
        .route("/docs/{id}/refactor/status", get(refactor::status))
        .route("/docs/{id}/refactor/end", post(refactor::end))
        .route("/adopt", post(refactor::adopt))
        .route(
            "/representations",
            get(representation::list).post(representation::create),
        )
        .route(
            "/representations/{id}/preview",
            post(representation::preview),
        )
        .route(
            "/representations/{id}",
            get(representation::get)
                .put(representation::edit)
                .delete(representation::discard),
        )
        .route(
            "/representations/{id}/refresh",
            post(representation::refresh),
        )
        .route("/representations/{id}/compare", get(revision::compare))
        .route(
            "/representations/{id}/arrange",
            post(representation_tools::arrange),
        )
        .route(
            "/representations/{id}/keep",
            post(representation_tools::save),
        )
        .route("/git/bisect", get(bisect::list).post(bisect::start))
        .route("/git/bisect/{id}/mark", post(bisect::mark))
        .route("/git/bisect/{id}/open", post(bisect::open))
        .route("/git/bisect/{id}/restore", post(bisect::restore))
        .route("/git/bisect/{id}", delete(bisect::finish))
        .route("/install", post(install::install))
        .route("/samples", post(sample::create))
        .route("/scaffold", post(scaffold::create))
        .route("/scaffold/templates", get(scaffold::templates))
        .route("/scaffold/options", get(scaffold::options))
        .route("/scaffold/preview", post(scaffold::preview))
        .route("/scaffold/result", get(scaffold::result))
        .route("/docs/{id}/run", post(api::run_doc))
        .route("/docs/{id}/check", post(api::check_doc))
        .merge(python_agent::routes())
        .route("/agents", get(acp::catalogue).put(acp::save_commands))
        .route("/agents/{id}/install", post(acp::install))
        .route(
            "/docs/{id}/agent/acp",
            get(acp::snapshot).post(acp::connect),
        )
        .route("/docs/{id}/agent/acp/authenticate", post(acp::authenticate))
        .route("/docs/{id}/agent/acp/configure", post(acp::configure))
        .route("/docs/{id}/agent/acp/permission", post(acp::permission))
        .route("/docs/{id}/agent/acp/edits", post(acp::edit_decision))
        .route("/docs/{id}/agent", post(agent::start_turn))
        .route("/docs/{id}/agent/stop", post(agent::stop_turn))
        .route("/docs/{id}/agent/turns", get(agent::list_turns))
        .route("/runs/{id}", get(api::get_run))
        .route(
            "/settings/keys",
            get(api::get_settings_keys).put(api::put_settings_keys),
        )
        .route(
            "/settings/ui",
            get(api::get_settings_ui).put(api::put_settings_ui),
        )
        // Continuity: off by default, and the whole feature on one switch.
        .route(
            "/settings/continuity",
            get(history::get_continuity).put(history::put_continuity),
        )
        .route("/files", get(api::files))
        .route("/files/op", post(files_ops::file_op))
        .route("/pick-folder", post(shell::pick_folder))
        .route("/save-file-dialog", post(shell::save_file_dialog))
        .route("/window/close", post(shell::close_window))
        .route("/reveal", post(reveal::reveal))
        .route("/open-external", post(reveal::open_external))
        .route("/open-file", post(plain_file::open_file))
        .route("/file", get(plain_file::get_file).put(plain_file::put_file))
        .route("/scratchpad", post(plain_file::post_scratchpad))
        .route("/v1/scratchpad", post(plain_file::post_assisted_scratchpad))
        .route("/tour/private-practice", post(private_practice::create))
        .route(
            "/tour/practice",
            axum::routing::get(engineer_tour::status).post(engineer_tour::create),
        )
        .route("/meetings/import", post(meeting_import::post))
        .merge(meeting_recording::routes())
        .route("/runtime/python", post(runtime_runs::python))
        .route(
            "/runtime/runs",
            get(runtime_runs::list).post(runtime_runs::save),
        )
        .route(
            "/runtime/runs/{id}",
            axum::routing::delete(runtime_runs::remove).patch(runtime_runs::pin),
        )
        .route("/asset", get(asset::get_asset).post(asset::post_asset))
        .route("/search", get(api::search))
        // What this project calls things — a different question from the
        // language server's, and shown beside it rather than instead of it.
        .route("/complete", get(api::complete))
        // Exhaustive, not ranked — see serve/find.rs for why that distinction
        // is the whole reason this is not `/search`.
        .route("/find", get(find::find))
        .route("/find/replace", post(find::replace))
        .route("/structure", get(api::structure))
        // Who last touched each line. Off by default in the editor, so this
        // is only ever asked for by someone who turned the column on.
        .route("/blame", get(api::blame))
        // History, read-only. Changing a repository is a thing people rightly
        // do deliberately, where the exact command is visible — and there is
        // a terminal on every row of the tree. See serve/git.rs.
        .route("/git/log", get(git::log))
        .route("/git/reading", get(commit_reading::read))
        .route("/git/status", get(git::status))
        // GitHub is reached by the person's own `gh` installation. Hickory
        // never receives or persists its token; a missing or signed-out CLI
        // is provider unavailability, not a broken filesystem tree.
        .route("/workspace/github/review-threads", get(github_reviews::get))
        .route("/workspace/github", get(github::workspace))
        .route("/workspace/github/pr/{number}", get(github::pull_request))
        .route("/workspace/github/issue/{number}", get(github::issue))
        .route("/workspace/github/issues", post(github::associate_issue))
        .route("/workspace/github/edit", post(github::edit))
        .route("/workspace/github/comment", post(github::comment))
        .route("/workspace/github/check-log", get(github::check_log))
        .route(
            "/workspace/github/notification/{thread}",
            post(github::mark_notification_read),
        )
        .route("/outputs/diverged", get(api::diverged_outputs))
        .route("/outputs/regenerate", post(api::regenerate_output))
        .route("/outputs/resolve", post(api::resolve_output))
        .route("/v1/outputs/resolve", post(api::resolve_output))
        .route("/v1/outputs/regenerate", post(api::regenerate_output))
        .route("/git/changes", get(git_ops::changes))
        .route("/git/diff", get(git_ops::diff))
        .route("/git/stage", post(git_ops::stage))
        .route("/git/unstage", post(git_ops::unstage))
        .route("/git/discard", post(git_ops::discard))
        // GET reads one commit as a card (the history lens); POST makes one.
        .route("/git/commit", get(git::commit).post(git_ops::commit))
        .route("/git/push", post(git_ops::push))
        .route("/git/pull", post(git_ops::pull))
        .route("/git/branches", get(git_ops::branches))
        .route("/git/checkout", post(git_ops::checkout))
        .route("/git/stash", post(git_ops::stash))
        // The publication floor and the merge-driver check: two facts about
        // the repository that the document panes need at open, and that no
        // amount of reading the log can answer.
        .route("/git/floor", get(history::floor))
        // The history lens's verbs (lenses.md steps 4–6): each one git
        // operation run as itself, refused in words, 409 with git's words
        // when a join stops. See serve/story.rs.
        .route("/git/replay", post(story::replay))
        .route("/git/recipe", post(story::emit))
        .route("/git/reword", post(story::reword))
        .route("/git/drop", post(story::drop))
        .route("/git/move", post(story::move_commit))
        // Captured worktree lens; v1 preserves literal target names and empty
        // membership. Reads do not grant the separate write authority below.
        .route("/worktrees", get(merged::list))
        .route("/merged", get(merged::merged))
        .route("/v1/merged", get(merged::merged_v1))
        // Writing through the view. Read-only is the default for any target
        // not explicitly opened for writing, a multi-target edit is not
        // atomic and reports per target, and undo across targets restores
        // from the recorded before-bytes of every one of them.
        .route("/merged/write", post(merged::write))
        .route("/merged/undo", post(merged::undo))
        .route(
            "/git/merge-driver",
            get(history::merge_driver).post(history::run_init),
        )
        // The fleet: a roster, not a presence list — nothing is reachable.
        .route("/fleet", get(history::fleet))
        // Enrolling and granting are done STANDING AT a machine: these are
        // deliberately unreachable over the peer channel, or a peer could
        // grant itself `execute` from inside the channel those grants bound.
        .route("/fleet/phrase", get(history::fleet_phrase))
        .route("/fleet/host", post(history::fleet_host))
        .route("/fleet/pair", post(history::fleet_pair))
        .route("/fleet/invite", get(history::fleet_invite))
        .route("/fleet/accept", post(history::fleet_accept))
        .route("/fleet/grant", axum::routing::put(history::fleet_grant))
        .route(
            "/v1/fleet/grant",
            axum::routing::put(history::fleet_grant_scoped),
        )
        .route("/v1/fleet/remove", post(history::fleet_remove_scoped))
        .route("/fleet/remove", post(history::fleet_remove))
        // Formulas: the host has already resolved references and worked out
        // the order by the time a backend sees anything. See serve/formula.rs.
        .route("/formula/evaluate", post(formula::evaluate))
        .route("/formula/trace", post(formula::trace))
        .route("/formula/languages", get(formula::languages))
        .route("/executor", get(api::executor))
        .route("/health", get(api::health))
        .route(
            "/workspace/ui",
            get(workspace::get_ui).put(workspace::put_ui),
        )
        .route(
            "/workspace/responses",
            get(workspace::get_response).put(workspace::put_response),
        )
        .route(
            "/workspace/drafts",
            get(workspace::list_drafts)
                .put(workspace::put_draft)
                .delete(workspace::discard_draft),
        )
        .route("/terminals", get(terminal::list).post(terminal::open))
        .route("/environments", get(environments::inspect))
        .route("/capabilities", get(capabilities::inspect))
        .route("/environments/actions", post(environments::act))
        .route("/environments/manager", put(environments::choose))
        .route("/tests/run", post(test_run::run))
        .route(
            "/performance",
            post(performance::action).layer(axum::extract::DefaultBodyLimit::max(36 * 1024 * 1024)),
        )
        .route(
            "/tests/coverage",
            get(test_coverage::list)
                .post(test_coverage::action)
                .layer(axum::extract::DefaultBodyLimit::max(18 * 1024 * 1024)),
        )
        .route("/terminals/turbo", put(terminal::set_turbo))
        .route("/terminals/anchors", get(terminal::anchors))
        .route("/terminals/ws", get(terminal::ws_handler))
        .route("/terminals/{id}", delete(terminal::close))
        .route("/terminals/{id}/input", post(terminal::input))
        .route("/terminals/{id}/output", get(terminal::output))
        .route("/terminals/{id}/resize", post(terminal::resize))
        .route("/terminals/{id}/interrupt", post(terminal::interrupt))
        .route("/terminals/{id}/answer", post(terminal::answer))
        .route(
            "/terminals/{id}/anchor",
            post(terminal::anchor).delete(terminal::unanchor),
        )
        .route(
            "/terminals/{id}/anchor/resume",
            post(terminal::resume_anchor),
        )
        .route("/ws", get(socket::ws_handler));

    Router::new()
        .nest("/api", api)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            private_predictions::quarantine,
        ))
        .with_state(state)
}

</storycode:copy>
<a id="f-09ea558208cd-bootstrap"></a>
### `bootstrap`

<storycode:copy id="f-09ea558208cd-bootstrap">
mod bootstrap;
</storycode:copy>
<a id="f-09ea558208cd-use_declaration-18"></a>
### `use_declaration`

<storycode:copy id="f-09ea558208cd-use_declaration-18">
pub use bootstrap::{Prepared, prepare, prepare_with_prediction, prepare_without_folder, serve};

</storycode:copy>
<a id="f-09ea558208cd-git_head"></a>
### `git_head`

Small helpers
The commit the work sits on, or `None` outside a repository.
Context for a correspondence recorded in the working tree: both its
endpoints are uncommitted, so the entry names what they were uncommitted
*from*.

<storycode:copy id="f-09ea558208cd-git_head">
// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// The commit the work sits on, or `None` outside a repository.
///
/// Context for a correspondence recorded in the working tree: both its
/// endpoints are uncommitted, so the entry names what they were uncommitted
/// *from*.
pub(crate) fn git_head(root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!sha.is_empty()).then_some(sha)
}

</storycode:copy>
<a id="f-09ea558208cd-now_rfc3339_public"></a>
### `now_rfc3339_public`

[`now_rfc3339`], for callers outside this module (the broker's log).

<storycode:copy id="f-09ea558208cd-now_rfc3339_public">
/// [`now_rfc3339`], for callers outside this module (the broker's log).
pub fn now_rfc3339_public() -> String {
    now_rfc3339()
}

</storycode:copy>
<a id="f-09ea558208cd-now_rfc3339"></a>
### `now_rfc3339`

<storycode:copy id="f-09ea558208cd-now_rfc3339">
pub(crate) fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rest = secs % 86_400;
    let (y, m, d) = api::civil_from_days_public(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

</storycode:copy>
<a id="f-09ea558208cd-rand_id"></a>
### `rand_id`

A run id. Uniqueness within one process is all this needs.

<storycode:copy id="f-09ea558208cd-rand_id">
/// A run id. Uniqueness within one process is all this needs.
pub(crate) fn rand_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    now.rotate_left(17) ^ n
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/mod.rs"><storycode:paste select="#f-09ea558208cd-acp"/><storycode:paste select="#f-09ea558208cd-agent"/><storycode:paste select="#f-09ea558208cd-agent_context"/><storycode:paste select="#f-09ea558208cd-agent_projects"/><storycode:paste select="#f-09ea558208cd-agent_review"/><storycode:paste select="#f-09ea558208cd-anchored"/><storycode:paste select="#f-09ea558208cd-api"/><storycode:paste select="#f-09ea558208cd-asset"/><storycode:paste select="#f-09ea558208cd-audit_evidence"/><storycode:paste select="#f-09ea558208cd-bisect"/><storycode:paste select="#f-09ea558208cd-changeset_drafts"/><storycode:paste select="#f-09ea558208cd-commit_reading"/><storycode:paste select="#f-09ea558208cd-dataset"/><storycode:paste select="#f-09ea558208cd-debug_bridge"/><storycode:paste select="#f-09ea558208cd-private_prediction_acp"/><storycode:paste select="#f-09ea558208cd-private_prediction_agent"/><storycode:paste select="#f-09ea558208cd-private_prediction_debugger"/><storycode:paste select="#f-09ea558208cd-private_prediction_recovery"/><storycode:paste select="#f-09ea558208cd-selected_run"/><storycode:paste select="#f-09ea558208cd-use_declaration"/><storycode:paste select="#f-09ea558208cd-private_prediction_startup"/><storycode:paste select="#f-09ea558208cd-private_predictions"/><storycode:paste select="#f-09ea558208cd-use_declaration-2"/><storycode:paste select="#f-09ea558208cd-use_declaration-3"/><storycode:paste select="#f-09ea558208cd-capabilities"/><storycode:paste select="#f-09ea558208cd-engineer_tour"/><storycode:paste select="#f-09ea558208cd-environments"/><storycode:paste select="#f-09ea558208cd-files_ops"/><storycode:paste select="#f-09ea558208cd-find"/><storycode:paste select="#f-09ea558208cd-formula"/><storycode:paste select="#f-09ea558208cd-git"/><storycode:paste select="#f-09ea558208cd-git_ops"/><storycode:paste select="#f-09ea558208cd-github"/><storycode:paste select="#f-09ea558208cd-github_reviews"/><storycode:paste select="#f-09ea558208cd-history"/><storycode:paste select="#f-09ea558208cd-install"/><storycode:paste select="#f-09ea558208cd-lsp_bridge"/><storycode:paste select="#f-09ea558208cd-meeting_import"/><storycode:paste select="#f-09ea558208cd-meeting_recording"/><storycode:paste select="#f-09ea558208cd-merged"/><storycode:paste select="#f-09ea558208cd-merged_write"/><storycode:paste select="#f-09ea558208cd-native_projects"/><storycode:paste select="#f-09ea558208cd-output_resolution"/><storycode:paste select="#f-09ea558208cd-outputs"/><storycode:paste select="#f-09ea558208cd-performance"/><storycode:paste select="#f-09ea558208cd-plain_file"/><storycode:paste select="#f-09ea558208cd-private_practice"/><storycode:paste select="#f-09ea558208cd-project_comparison"/><storycode:paste select="#f-09ea558208cd-project_incorporation"/><storycode:paste select="#f-09ea558208cd-project_publish"/><storycode:paste select="#f-09ea558208cd-python_agent"/><storycode:paste select="#f-09ea558208cd-revision_projects"/><storycode:paste select="#f-09ea558208cd-runtime_runs"/><storycode:paste select="#f-09ea558208cd-ui_actions"/><storycode:paste select="#f-09ea558208cd-ui_images"/><storycode:paste select="#f-09ea558208cd-ui_verification"/><storycode:paste select="#f-09ea558208cd-use_declaration-4"/><storycode:paste select="#f-09ea558208cd-fleet_pairings"/><storycode:paste select="#f-09ea558208cd-launch_profiles"/><storycode:paste select="#f-09ea558208cd-refactor"/><storycode:paste select="#f-09ea558208cd-representation"/><storycode:paste select="#f-09ea558208cd-representation_store"/><storycode:paste select="#f-09ea558208cd-representation_tools"/><storycode:paste select="#f-09ea558208cd-reveal"/><storycode:paste select="#f-09ea558208cd-reviews"/><storycode:paste select="#f-09ea558208cd-revision"/><storycode:paste select="#f-09ea558208cd-sample"/><storycode:paste select="#f-09ea558208cd-scaffold"/><storycode:paste select="#f-09ea558208cd-sessions"/><storycode:paste select="#f-09ea558208cd-shell"/><storycode:paste select="#f-09ea558208cd-socket"/><storycode:paste select="#f-09ea558208cd-store"/><storycode:paste select="#f-09ea558208cd-story"/><storycode:paste select="#f-09ea558208cd-terminal"/><storycode:paste select="#f-09ea558208cd-test_coverage"/><storycode:paste select="#f-09ea558208cd-test_run"/><storycode:paste select="#f-09ea558208cd-watch"/><storycode:paste select="#f-09ea558208cd-workspace"/><storycode:paste select="#f-09ea558208cd-workspace_fs"/><storycode:paste select="#f-09ea558208cd-use_declaration-5"/><storycode:paste select="#f-09ea558208cd-use_declaration-6"/><storycode:paste select="#f-09ea558208cd-use_declaration-7"/><storycode:paste select="#f-09ea558208cd-use_declaration-8"/><storycode:paste select="#f-09ea558208cd-use_declaration-9"/><storycode:paste select="#f-09ea558208cd-use_declaration-10"/><storycode:paste select="#f-09ea558208cd-use_declaration-11"/><storycode:paste select="#f-09ea558208cd-use_declaration-12"/><storycode:paste select="#f-09ea558208cd-use_declaration-13"/><storycode:paste select="#f-09ea558208cd-use_declaration-14"/><storycode:paste select="#f-09ea558208cd-use_declaration-15"/><storycode:paste select="#f-09ea558208cd-use_declaration-16"/><storycode:paste select="#f-09ea558208cd-use_declaration-17"/><storycode:paste select="#f-09ea558208cd-ServeOptions"/><storycode:paste select="#f-09ea558208cd-RunRecord"/><storycode:paste select="#f-09ea558208cd-LocalState"/><storycode:paste select="#f-09ea558208cd-DivergedOutput"/><storycode:paste select="#f-09ea558208cd-KeySettings"/><storycode:paste select="#f-09ea558208cd-UiSettings"/><storycode:paste select="#f-09ea558208cd-UiStore"/><storycode:paste select="#f-09ea558208cd-impl-UiStore"/><storycode:paste select="#f-09ea558208cd-impl-LocalState"/><storycode:paste select="#f-09ea558208cd-router"/><storycode:paste select="#f-09ea558208cd-bootstrap"/><storycode:paste select="#f-09ea558208cd-use_declaration-18"/><storycode:paste select="#f-09ea558208cd-git_head"/><storycode:paste select="#f-09ea558208cd-now_rfc3339_public"/><storycode:paste select="#f-09ea558208cd-now_rfc3339"/><storycode:paste select="#f-09ea558208cd-rand_id"/></storycode:file>

</storycode:doc>
