//! Shared test infrastructure for hick-agent integration tests.
//!
//! Provides:
//! - [`ScriptedLlm`]: deterministic LLM mock returning pre-programmed responses
//! - [`create_executor`]: create a WASM sandbox executor for a given workdir
//! - [`run_agent`]: drive a full ReactLoop with scripted responses and collect events
#![allow(dead_code)] // Each integration test binary uses only part of this module.
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::Stream;
use iron_peak::events::AgentEvent;
use iron_peak::llm::{ChatChunk, LlmClient, Message};
use hick_agent_script_first::ReactLoop;
use hick_agent_core::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
use hick_token::ContainerCapabilities;
use tokio::sync::Notify;

// ---------------------------------------------------------------------------
// ScriptedLlm — deterministic mock
// ---------------------------------------------------------------------------

/// Mock LLM that returns pre-programmed responses in order.
///
/// Each call to `complete()` pops the next response from the queue.
/// The last response should typically have no code block so the agent
/// transitions to `Done`.
pub struct ScriptedLlm {
    responses: Mutex<VecDeque<String>>,
}

impl ScriptedLlm {
    pub fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().map(String::from).collect()),
        }
    }
}

#[async_trait]
impl LlmClient for ScriptedLlm {
    async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
        let mut q = self.responses.lock().unwrap();
        q.pop_front()
            .ok_or_else(|| anyhow::anyhow!("ScriptedLlm: no more responses queued"))
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let response = self.complete(messages).await?;
        let stream = futures::stream::once(async move {
            Ok(ChatChunk {
                delta: response,
                finish_reason: Some("stop".into()),
            })
        });
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "scripted"
    }

    fn model_name(&self) -> &str {
        "scripted-test"
    }
}

// ---------------------------------------------------------------------------
// WASM image helpers
// ---------------------------------------------------------------------------

/// Ensure a `.wasm` file exists by extracting from the sibling `.wasm.zip`.
pub fn ensure_wasm_extracted(wasm_path: &Path) {
    if wasm_path.exists() {
        return;
    }
    let zip_path = wasm_path.with_extension("wasm.zip");
    assert!(
        zip_path.exists(),
        "Neither {wasm_path:?} nor {zip_path:?} found. \
         Run `git lfs pull` to fetch the fixture."
    );
    let file = std::fs::File::open(&zip_path)
        .unwrap_or_else(|e| panic!("failed to open {zip_path:?}: {e}"));
    let mut archive = zip::ZipArchive::new(file)
        .unwrap_or_else(|e| panic!("failed to read zip {zip_path:?}: {e}"));
    let dest_dir = wasm_path.parent().unwrap();
    archive
        .extract(dest_dir)
        .unwrap_or_else(|e| panic!("failed to extract {zip_path:?}: {e}"));
    assert!(
        wasm_path.exists(),
        "Extraction succeeded but {wasm_path:?} still missing"
    );
}

/// Repo-root-relative path to compiled container images (`c2w` output).
///
/// Matches `detect_images_dir` in `hick-agent` `main.rs`: walk upward from this
/// crate until `native2wasm/crates/c2w/converted_images` exists.
pub fn images_dir() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel = Path::new("native2wasm/crates/c2w/converted_images");
    let mut dir = manifest.to_path_buf();
    loop {
        let candidate = dir.join(rel);
        if candidate.is_dir() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "Could not find `{}` by walking up from {}. Expected the Hick monorepo checkout with native2wasm artifacts.",
        rel.display(),
        manifest.display()
    );
}

/// Image family passed to [`hick::executor::resolve_image_path`] (e.g. `ubuntu` → `ubuntu/24.04.wasm`).
///
/// Prefer `ubuntu` when that fixture exists (full Python toolchain in typical images); otherwise
/// use `alpine` so shell-based integration tests still run in minimal checkouts.
///
/// Note: `execution_python` needs `python3` on `PATH` (usually `ubuntu`/`debian` fixtures). A bare
/// `alpine` image will fail those tests unless you extend them to `apk add python3` first.
fn integration_image_family(images_root: &Path) -> String {
    for name in ["ubuntu", "debian", "fedora", "alpine"] {
        let path = hick::executor::resolve_image_path(images_root, name);
        if path.is_file() {
            return name.to_string();
        }
    }
    panic!(
        "No runnable `.wasm` image under {} (tried ubuntu, debian, fedora, alpine). \
         Convert or fetch images into that directory (see native2wasm / `git lfs pull`).",
        images_root.display()
    );
}

// ---------------------------------------------------------------------------
// Executor creation
// ---------------------------------------------------------------------------

/// Type alias for the executor handle shared between tests and the ReactLoop.
pub type SharedExecutor = Arc<tokio::sync::Mutex<dyn CommandExecutor>>;

pub fn create_executor(workdir: &Path) -> (SharedExecutor, String) {
    create_executor_with_guest_network(workdir, true)
}

/// Like [`create_executor`], but allows toggling the virtio guest network bootstrap.
///
/// Pass `guest_network: true` (the default for production) to boot with `--net=socket` so
/// in-guest networking (`apk`, `pip`, etc.) works. The FD-3 collision fix ensures that
/// host directory preopens are still visible at `/workspace` even with networking on.
///
/// Pass `guest_network: false` only when specifically testing the no-network path or when
/// the test environment cannot reach the network.
pub fn create_executor_with_guest_network(
    workdir: &Path,
    guest_network: bool,
) -> (SharedExecutor, String) {
    create_executor_with_capabilities(workdir, guest_network, ContainerCapabilities::default())
}

/// Like [`create_executor_with_guest_network`], but with explicit capabilities.
///
/// Use this to test the `CapabilityFilter` path (deny-all + specific allow list),
/// which is what the live agent uses by default.
pub fn create_executor_with_capabilities(
    workdir: &Path,
    guest_network: bool,
    capabilities: ContainerCapabilities,
) -> (SharedExecutor, String) {
    let images_root = images_dir();
    let image_family = integration_image_family(&images_root);
    let wasm_path = hick::executor::resolve_image_path(&images_root, &image_family);
    ensure_wasm_extracted(&wasm_path);

    let config = WasmSandboxConfig {
        images_dir: images_root,
        workdir: workdir.to_path_buf(),
        default_image: image_family.clone(),
        capabilities,
        auth: None,
        guest_network_enabled: guest_network,
    };

    let executor = WasmSandboxExecutor::new(config);
    let shared: SharedExecutor = Arc::new(tokio::sync::Mutex::new(executor));
    (shared, image_family)
}

// ---------------------------------------------------------------------------
// Agent runner
// ---------------------------------------------------------------------------

/// Drive a full ReactLoop with scripted LLM responses and a WASM executor.
///
/// Returns all events emitted during the run.
pub async fn run_agent(
    workdir: &Path,
    executor: SharedExecutor,
    image: &str,
    responses: Vec<&str>,
) -> Vec<AgentEvent> {
    let llm = ScriptedLlm::new(responses);

    let react = ReactLoop::new(Box::new(llm), workdir.to_path_buf())
        .with_executor(executor, image.to_string())
        .with_agent_dir(".hick");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let interrupt = Notify::new();

    react
        .run("test", &tx, &interrupt)
        .await
        .expect("ReactLoop::run failed");

    // Drain events
    drop(tx);
    let mut events = Vec::new();
    while let Some(ev) = rx.recv().await {
        events.push(ev);
    }
    events
}

/// Collect all `ScriptFinished` events from a run.
pub fn finished_results(events: &[AgentEvent]) -> Vec<&iron_peak::tools::ScriptResult> {
    events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::ScriptFinished(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// Get the Done message from events, if any.
pub fn done_message(events: &[AgentEvent]) -> Option<&str> {
    events.iter().find_map(|e| match e {
        AgentEvent::Done(msg) => Some(msg.as_str()),
        _ => None,
    })
}
