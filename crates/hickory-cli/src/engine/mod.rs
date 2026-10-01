//! One engine per user, with independently lived desktop and CLI clients.
//! The rendezvous files are machine-local discovery, never provenance.
mod client;
mod daemon;
pub(crate) mod mcp;
mod proxy;
mod watch;
pub(crate) mod writes;

pub use client::{ClientGuard, Connection, connect, forward_cli, up};
pub use daemon::run_argv;
pub use proxy::client_router;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// A protocol version, not a release version: compatible releases may attach.
const PROTOCOL: u32 = 1;
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
    pub target: PathBuf,
    pub params: Vec<(String, String)>,
    pub executor: crate::ExecutorChoice,
    pub run: bool,
    pub key_store_path: Option<PathBuf>,
    pub ui_settings_path: Option<PathBuf>,
    pub callback: Option<String>,
    pub callback_token: String,
    pub client: String,
}

impl Attach {
    pub fn new(target: PathBuf, executor: crate::ExecutorChoice) -> Self {
        Self {
            target,
            executor,
            params: Vec::new(),
            run: false,
            key_store_path: None,
            ui_settings_path: None,
            callback: None,
            callback_token: String::new(),
            client: secret(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Attached {
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
