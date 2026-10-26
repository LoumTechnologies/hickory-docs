<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/engine/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e55203129482"></a>
## `crates/hickory-cli/src/engine/mod.rs`

Related contracts: [A desktop app bundles its own Python agent](../../../../../guarantees/agent/a-desktop-app-bundles-its-python-agent/contract.md), [One loop owns a directory and never reacts to itself](../../../../../guarantees/authoring/one-loop-owns-a-directory/contract.md), [The daemon exits with its last client](../../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-e55203129482-client"></a>
### `client`

One engine per user, with independently lived desktop and CLI clients.
The rendezvous files are machine-local discovery, never provenance.

<storycode:copy id="f-e55203129482-client">
//! One engine per user, with independently lived desktop and CLI clients.
//! The rendezvous files are machine-local discovery, never provenance.
mod client;
</storycode:copy>
<a id="f-e55203129482-daemon"></a>
### `daemon`

<storycode:copy id="f-e55203129482-daemon">
mod daemon;
</storycode:copy>
<a id="f-e55203129482-mcp"></a>
### `mcp`

<storycode:copy id="f-e55203129482-mcp">
pub(crate) mod mcp;
</storycode:copy>
<a id="f-e55203129482-private_policy"></a>
### `private_policy`

<storycode:copy id="f-e55203129482-private_policy">
pub(crate) mod private_policy;
</storycode:copy>
<a id="f-e55203129482-proxy"></a>
### `proxy`

<storycode:copy id="f-e55203129482-proxy">
mod proxy;
</storycode:copy>
<a id="f-e55203129482-startup"></a>
### `startup`

<storycode:copy id="f-e55203129482-startup">
mod startup;
</storycode:copy>
<a id="f-e55203129482-watch"></a>
### `watch`

<storycode:copy id="f-e55203129482-watch">
mod watch;
</storycode:copy>
<a id="f-e55203129482-writes"></a>
### `writes`

<storycode:copy id="f-e55203129482-writes">
pub(crate) mod writes;

</storycode:copy>
<a id="f-e55203129482-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration">
pub use client::{ClientGuard, Connection, connect, forward_cli, up};
</storycode:copy>
<a id="f-e55203129482-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration-2">
pub use daemon::run_argv;
</storycode:copy>
<a id="f-e55203129482-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration-3">
pub use proxy::client_router;

</storycode:copy>
<a id="f-e55203129482-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration-4">
use anyhow::{Context, Result};
</storycode:copy>
<a id="f-e55203129482-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration-5">
use serde::{Deserialize, Serialize};
</storycode:copy>
<a id="f-e55203129482-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration-6">
use std::path::PathBuf;

</storycode:copy>
<a id="f-e55203129482-PROTOCOL"></a>
### `PROTOCOL`

A protocol version, not a release version: compatible releases may attach.

<storycode:copy id="f-e55203129482-PROTOCOL">
// A protocol version, not a release version: compatible releases may attach.
const PROTOCOL: u32 = 2;
</storycode:copy>
<a id="f-e55203129482-PREDICTION_PROTOCOL"></a>
### `PREDICTION_PROTOCOL`

<storycode:copy id="f-e55203129482-PREDICTION_PROTOCOL">
const PREDICTION_PROTOCOL: &str = "private_prediction_v1";
</storycode:copy>
<a id="f-e55203129482-WORKER"></a>
### `WORKER`

<storycode:copy id="f-e55203129482-WORKER">
pub(crate) const WORKER: &str = "HICKORY_ENGINE_WORKER";
</storycode:copy>
<a id="f-e55203129482-HEARTBEAT"></a>
### `HEARTBEAT`

<storycode:copy id="f-e55203129482-HEARTBEAT">
const HEARTBEAT: std::time::Duration = std::time::Duration::from_secs(2);
</storycode:copy>
<a id="f-e55203129482-LEASE"></a>
### `LEASE`

<storycode:copy id="f-e55203129482-LEASE">
const LEASE: std::time::Duration = std::time::Duration::from_secs(10);

</storycode:copy>
<a id="f-e55203129482-Endpoint"></a>
### `Endpoint`

<storycode:copy id="f-e55203129482-Endpoint">
#[derive(Clone, Serialize, Deserialize)]
struct Endpoint {
    url: String,
    token: String,
    protocol: u32,
    pid: u32,
}

</storycode:copy>
<a id="f-e55203129482-Attach"></a>
### `Attach`

<storycode:copy id="f-e55203129482-Attach">
#[derive(Clone, Serialize, Deserialize)]
pub struct Attach {
    /// Trusted host startup profile; inherited by every client of this workspace.
    #[serde(default)]
    pub prediction: Option<crate::serve::PrivatePredictionStartup>,
    pub target: PathBuf,
    pub params: Vec<(String, String)>,
    pub executor: crate::ExecutorChoice,
    pub run: bool,
    /// Desktop editors can boot before the initial weave. File operations
    /// still wait for output ownership; CLI clients keep synchronous startup.
    #[serde(default)]
    pub background: bool,
    pub key_store_path: Option<PathBuf>,
    pub ui_settings_path: Option<PathBuf>,
    /// A runtime the attaching app ships inside its own bundle, if it does.
    /// Sent over the wire because the engine may have been started by another
    /// program (the CLI) and would otherwise have no way to know about it.
    #[serde(default)]
    pub python_runtime_path: Option<PathBuf>,
    pub callback: Option<String>,
    pub callback_token: String,
    pub client: String,
}

</storycode:copy>
<a id="f-e55203129482-impl-Attach"></a>
### `impl-Attach`

<storycode:copy id="f-e55203129482-impl-Attach">
impl Attach {
    pub fn new(target: PathBuf, executor: crate::ExecutorChoice) -> Self {
        Self {
            prediction: None,
            target,
            executor,
            params: Vec::new(),
            run: false,
            background: false,
            key_store_path: None,
            ui_settings_path: None,
            python_runtime_path: None,
            callback: None,
            callback_token: String::new(),
            client: secret(),
        }
    }
}

</storycode:copy>
<a id="f-e55203129482-Attached"></a>
### `Attached`

<storycode:copy id="f-e55203129482-Attached">
#[derive(Serialize, Deserialize)]
struct Attached {
    #[serde(default)]
    prediction: Option<String>,
    slot: String,
    log_offset: u64,
}

</storycode:copy>
<a id="f-e55203129482-directory"></a>
### `directory`

<storycode:copy id="f-e55203129482-directory">
fn directory() -> Result<PathBuf> {
    let dir = hickory_workspace::data_root()?.join("engine");
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir)
}

</storycode:copy>
<a id="f-e55203129482-window_token"></a>
### `window_token`

<storycode:copy id="f-e55203129482-window_token">
pub fn window_token() -> String {
    secret()
}

</storycode:copy>
<a id="f-e55203129482-secret"></a>
### `secret`

<storycode:copy id="f-e55203129482-secret">
pub(crate) fn secret() -> String {
    // OS randomness, with no dependency on wall time, process IDs or counters.
    // tempfile already uses getrandom; obtain the same OS RNG directly.
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("operating system randomness unavailable");
    hex::encode(bytes)
}

</storycode:copy>
<a id="f-e55203129482-load_keys"></a>
### `load_keys`

<storycode:copy id="f-e55203129482-load_keys">
pub(super) fn load_keys(path: &std::path::Path) -> Result<hickory_agent::KeyStore> {
    hickory_agent::KeyStore::load(path)
}

</storycode:copy>
<a id="f-e55203129482-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-e55203129482-use_declaration-7">
pub(crate) use client::{private_prediction_active, require_ordinary_agent_context_at};
</storycode:copy>

### Assemble `crates/hickory-cli/src/engine/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/engine/mod.rs"><storycode:paste select="#f-e55203129482-client"/><storycode:paste select="#f-e55203129482-daemon"/><storycode:paste select="#f-e55203129482-mcp"/><storycode:paste select="#f-e55203129482-private_policy"/><storycode:paste select="#f-e55203129482-proxy"/><storycode:paste select="#f-e55203129482-startup"/><storycode:paste select="#f-e55203129482-watch"/><storycode:paste select="#f-e55203129482-writes"/><storycode:paste select="#f-e55203129482-use_declaration"/><storycode:paste select="#f-e55203129482-use_declaration-2"/><storycode:paste select="#f-e55203129482-use_declaration-3"/><storycode:paste select="#f-e55203129482-use_declaration-4"/><storycode:paste select="#f-e55203129482-use_declaration-5"/><storycode:paste select="#f-e55203129482-use_declaration-6"/><storycode:paste select="#f-e55203129482-PROTOCOL"/><storycode:paste select="#f-e55203129482-PREDICTION_PROTOCOL"/><storycode:paste select="#f-e55203129482-WORKER"/><storycode:paste select="#f-e55203129482-HEARTBEAT"/><storycode:paste select="#f-e55203129482-LEASE"/><storycode:paste select="#f-e55203129482-Endpoint"/><storycode:paste select="#f-e55203129482-Attach"/><storycode:paste select="#f-e55203129482-impl-Attach"/><storycode:paste select="#f-e55203129482-Attached"/><storycode:paste select="#f-e55203129482-directory"/><storycode:paste select="#f-e55203129482-window_token"/><storycode:paste select="#f-e55203129482-secret"/><storycode:paste select="#f-e55203129482-load_keys"/><storycode:paste select="#f-e55203129482-use_declaration-7"/></storycode:file>

</storycode:doc>
