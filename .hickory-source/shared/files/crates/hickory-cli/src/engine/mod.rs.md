<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/engine/mod.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A desktop app bundles its own Python agent](../../../../../../guarantees/agent/a-desktop-app-bundles-its-python-agent/contract.md)
- [One loop owns a directory and never reacts to itself](../../../../../../guarantees/authoring/one-loop-owns-a-directory/contract.md)
- [The daemon exits with its last client](../../../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `3c2384eecd7c161edd973e9d2e9fd75ef5ebc153119519ada4c19a1d4a12ab79`.

<book:entry path="crates/hickory-cli/src/engine/mod.rs">
//! One engine per user, with independently lived desktop and CLI clients.
//! The rendezvous files are machine-local discovery, never provenance.
mod client;
mod daemon;
pub(crate) mod mcp;
pub(crate) mod private_policy;
mod proxy;
mod startup;
mod watch;
pub(crate) mod writes;

pub use client::{ClientGuard, Connection, connect, forward_cli, up};
pub use daemon::run_argv;
pub use proxy::client_router;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// A protocol version, not a release version: compatible releases may attach.
const PROTOCOL: u32 = 2;
const PREDICTION_PROTOCOL: &str = "private_prediction_v1";
pub(crate) const WORKER: &str = "HICKORY_ENGINE_WORKER";
const HEARTBEAT: std::time::Duration = std::time::Duration::from_secs(2);
const LEASE: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Clone, Serialize, Deserialize)]
struct Endpoint {
    url: String,
    token: String,
    protocol: u32,
    pid: u32,
}

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

#[derive(Serialize, Deserialize)]
struct Attached {
    #[serde(default)]
    prediction: Option<String>,
    slot: String,
    log_offset: u64,
}

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

pub fn window_token() -> String {
    secret()
}

pub(crate) fn secret() -> String {
    // OS randomness, with no dependency on wall time, process IDs or counters.
    // tempfile already uses getrandom; obtain the same OS RNG directly.
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("operating system randomness unavailable");
    hex::encode(bytes)
}

pub(super) fn load_keys(path: &std::path::Path) -> Result<hickory_agent::KeyStore> {
    hickory_agent::KeyStore::load(path)
}

pub(crate) use client::{private_prediction_active, require_ordinary_agent_context_at};
</book:entry>

</book:doc>
