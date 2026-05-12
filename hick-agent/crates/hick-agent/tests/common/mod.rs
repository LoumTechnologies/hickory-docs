//! Shared test infrastructure for hick-agent integration tests.
//!
//! Provides:
//! - [`ScriptedLlm`]: deterministic LLM mock returning pre-programmed responses
//! - [`create_executor`]: create a WASM sandbox executor for a given workdir
//! - [`run_agent`]: drive a full ReactLoop with scripted responses and collect events

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::Stream;
use hick_agent::events::AgentEvent;
use hick_agent::llm::{ChatChunk, LlmClient, Message};
use hick_agent::{Agent, AgentConfig, ReactLoop};
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

/// Path to the `c2w/converted_images` directory.
pub fn images_dir() -> PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest)
        .parent()
        .unwrap()
        .join("c2w/converted_images")
}

/// Path to the ubuntu 24.04 WASM image.
pub fn ubuntu_wasm_path() -> PathBuf {
    images_dir().join("ubuntu/24.04.wasm")
}

// ---------------------------------------------------------------------------
// Executor creation
// ---------------------------------------------------------------------------

/// Type alias for the executor handle shared between tests and the ReactLoop.
pub type SharedExecutor = Arc<tokio::sync::Mutex<dyn CommandExecutor>>;

/// Create a WASM sandbox executor backed by the ubuntu image.
///
/// The `workdir` is mounted at `/workspace` inside the container.
pub fn create_executor(workdir: &Path) -> (SharedExecutor, String) {
    let wasm_path = ubuntu_wasm_path();
    ensure_wasm_extracted(&wasm_path);

    let config = WasmSandboxConfig {
        images_dir: images_dir(),
        workdir: workdir.to_path_buf(),
        default_image: "ubuntu".to_string(),
        capabilities: ContainerCapabilities::default(),
    };

    let executor = WasmSandboxExecutor::new(config);
    let shared: SharedExecutor = Arc::new(tokio::sync::Mutex::new(executor));
    (shared, "ubuntu".to_string())
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

    let config = AgentConfig {
        project_dir: workdir.to_path_buf(),
        images_dir: images_dir(),
        default_image: image.to_string(),
    };

    let agent = Agent::new(config, Box::new(llm));

    let react =
        ReactLoop::new(agent, workdir.to_path_buf()).with_executor(executor, image.to_string());

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let interrupt = Notify::new();

    let mut history = Vec::new();
    react
        .run("test", &mut history, &tx, &interrupt)
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
pub fn finished_results(events: &[AgentEvent]) -> Vec<&hick_agent::tools::ScriptResult> {
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
