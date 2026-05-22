//! Hick-literate-backed WASM container backend for the SDK's `WasmSandboxExecutor`.
//!
//! Bridges `hick_literate::executor::ContainerExecutor` (which knows about images,
//! capabilities, and auth) to the SDK's `ContainerBackend` trait (which only knows
//! about start/execute primitives).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hick_agent_sdk::command::ContainerBackend;
use hick_literate::auth::AuthStack;
use hick_literate::executor::ContainerExecutor;
use hick_token::ContainerCapabilities;

/// `ContainerBackend` implementation backed by `hick_literate::executor::ContainerExecutor`.
///
/// Carries the hick-specific configuration (`images_dir`, `capabilities`, `auth`)
/// that the generic `WasmSandboxConfig` intentionally omits.
pub struct HickWasmBackend {
    images_dir: PathBuf,
    workdir: PathBuf,
    capabilities: ContainerCapabilities,
    auth: Option<Arc<AuthStack>>,
    guest_network_enabled: bool,
    executor: Option<ContainerExecutor>,
}

impl HickWasmBackend {
    pub fn new(
        images_dir: PathBuf,
        workdir: PathBuf,
        capabilities: ContainerCapabilities,
        auth: Option<Arc<AuthStack>>,
        guest_network_enabled: bool,
    ) -> Self {
        Self {
            images_dir,
            workdir,
            capabilities,
            auth,
            guest_network_enabled,
            executor: None,
        }
    }
}

#[async_trait]
impl ContainerBackend for HickWasmBackend {
    async fn ensure_started(&mut self, container: &str, image: &str) -> anyhow::Result<()> {
        if self.executor.is_none() {
            let mut caps_map = HashMap::new();
            caps_map.insert(container.to_string(), self.capabilities.clone());
            let mut exec = ContainerExecutor::new(self.images_dir.clone(), caps_map)?;
            exec.set_preopened_dirs(vec![(self.workdir.clone(), "/workspace".to_string())]);
            exec.set_guest_network_enabled(self.guest_network_enabled);
            if let Some(ref auth) = self.auth {
                exec.set_auth(auth.clone());
            }
            self.executor = Some(exec);
        }
        self.executor
            .as_mut()
            .unwrap()
            .ensure_started(container, image)
            .await
    }

    async fn execute_raw(&mut self, container: &str, command: &str) -> anyhow::Result<String> {
        self.executor
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("executor not initialized"))?
            .execute(container, command)
            .await
    }
}
