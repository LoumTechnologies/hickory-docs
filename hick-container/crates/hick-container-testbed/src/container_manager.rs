//! Container lifecycle manager with named containers, switching, and bridge integration.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hick_net_bridge::NetworkBridge;
use hick_connection_filter::ConnectionFilter;
use hick_container::{Container, ContainerConfig};
use hick_net_config::NetworkConfig;
use tokio::sync::{RwLock, mpsc};
use wasmtime::{Engine, Module};

// ---------------------------------------------------------------------------
// Terminal state (per-container)
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
pub enum TestbedState {
    /// Kernel is booting — forward container output, detect first prompt.
    Booting,
    /// Shell is interactive — local echo, suppress container echo.
    Interactive,
}

/// Suppresses container echo of bytes we already displayed locally.
pub struct EchoSuppressor {
    queue: VecDeque<u8>,
}

impl EchoSuppressor {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, byte: u8) {
        self.queue.push_back(byte);
    }

    pub fn pop_last(&mut self) {
        self.queue.pop_back();
    }

    pub fn process(&mut self, src: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(src.len());
        let mut i = 0;
        while i < src.len() {
            if src[i] == 0x1b && i + 1 < src.len() && src[i + 1] == b'[' {
                out.push(src[i]);
                i += 1;
                while i < src.len() {
                    out.push(src[i]);
                    if src[i].is_ascii_alphabetic() || src[i] == b'~' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            if let Some(&expected) = self.queue.front() {
                if src[i] == expected {
                    self.queue.pop_front();
                } else {
                    self.queue.clear();
                    out.push(src[i]);
                }
            } else {
                out.push(src[i]);
            }
            i += 1;
        }
        out
    }
}

pub const PROMPT: &[u8] = b"/ # ";

/// Streaming pattern matcher for the BusyBox ash prompt `/ # `.
pub struct PromptMatcher {
    matched: usize,
    buffer: Vec<u8>,
}

impl PromptMatcher {
    pub fn new() -> Self {
        Self {
            matched: 0,
            buffer: Vec::new(),
        }
    }

    pub fn process(&mut self, src: &[u8]) -> (Vec<u8>, bool) {
        let mut out = Vec::with_capacity(src.len());
        let mut detected = false;

        for &byte in src {
            if byte == PROMPT[self.matched] {
                self.buffer.push(byte);
                self.matched += 1;
                if self.matched == PROMPT.len() {
                    self.buffer.clear();
                    self.matched = 0;
                    detected = true;
                }
            } else {
                if !self.buffer.is_empty() {
                    out.extend_from_slice(&self.buffer);
                    self.buffer.clear();
                }
                self.matched = 0;
                if byte == PROMPT[0] {
                    self.buffer.push(byte);
                    self.matched = 1;
                } else {
                    out.push(byte);
                }
            }
        }

        (out, detected)
    }
}

/// Per-container terminal state.
pub struct TerminalState {
    pub testbed_state: TestbedState,
    pub suppressor: EchoSuppressor,
    pub prompt_matcher: PromptMatcher,
    pub at_line_start: bool,
    pub in_meta: bool,
    pub meta_buf: Vec<u8>,
}

impl TerminalState {
    fn new() -> Self {
        Self {
            testbed_state: TestbedState::Booting,
            suppressor: EchoSuppressor::new(),
            prompt_matcher: PromptMatcher::new(),
            at_line_start: true,
            in_meta: false,
            meta_buf: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Managed container
// ---------------------------------------------------------------------------

pub struct ManagedContainer {
    pub id: u32,
    pub name: Option<String>,
    pub image: String,
    pub container: Container,
    pub terminal: TerminalState,
    pub network_config: NetworkConfig,
}

/// Summary of a container for `:ls` output.
pub struct ContainerInfo {
    pub id: u32,
    pub name: Option<String>,
    pub image: String,
    pub ip: String,
    pub status: &'static str,
    pub active: bool,
}

impl fmt::Display for ContainerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.active { "*" } else { " " };
        let name = self.name.as_deref().unwrap_or("-");
        write!(
            f,
            "{} {:>3}  {:<10} {:<10} {:<14} {}",
            marker, self.id, name, self.image, self.ip, self.status
        )
    }
}

// ---------------------------------------------------------------------------
// Container Manager
// ---------------------------------------------------------------------------

pub struct ContainerManager {
    containers: HashMap<u32, ManagedContainer>,
    active_id: Option<u32>,
    next_id: u32,
    next_subnet_index: u8,
    bridge: Arc<RwLock<NetworkBridge>>,
    engine: Engine,
    filter: Arc<dyn ConnectionFilter>,
}

impl ContainerManager {
    pub fn new(
        engine: Engine,
        filter: Arc<dyn ConnectionFilter>,
        bridge: Arc<RwLock<NetworkBridge>>,
    ) -> Self {
        Self {
            containers: HashMap::new(),
            active_id: None,
            next_id: 1,
            next_subnet_index: 0,
            bridge,
            engine,
            filter,
        }
    }

    /// Create a new container from an image name, optionally with a display name.
    pub async fn create(&mut self, image: &str, name: Option<&str>) -> anyhow::Result<u32> {
        let wasm_path = resolve_image_path(image);
        eprintln!("[testbed] Loading image: {}", wasm_path.display());

        let module = Module::from_file(&self.engine, &wasm_path)?;

        let net_cfg = NetworkConfig::for_container(self.next_subnet_index);
        self.next_subnet_index = self.next_subnet_index.wrapping_add(1);

        // Create bridge channel
        let (bridge_tx, bridge_rx) = mpsc::channel::<Vec<u8>>(256);

        // Register with bridge
        {
            let mut b = self.bridge.write().await;
            b.register(net_cfg.vm_ip_std(), bridge_tx, name);
        }

        let mut config = ContainerConfig::new(self.engine.clone(), module);
        config.networking = true;
        config.connection_filter = Some(self.filter.clone());
        config.network_config = Some(net_cfg.clone());
        config.bridge = Some(self.bridge.clone());
        config.bridge_rx = Some(bridge_rx);

        let container = Container::start(config).await?;

        let id = self.next_id;
        self.next_id += 1;

        let display_name = name.map(|n| n.to_string());
        eprintln!(
            "[testbed] Started container #{id}: image={image}, ip={}, name={}",
            net_cfg.vm_ip,
            display_name.as_deref().unwrap_or("-"),
        );

        self.containers.insert(
            id,
            ManagedContainer {
                id,
                name: display_name,
                image: image.to_string(),
                container,
                terminal: TerminalState::new(),
                network_config: net_cfg,
            },
        );

        self.active_id = Some(id);
        Ok(id)
    }

    /// Name or rename the active container. Updates bridge DNS.
    pub async fn name_active(&mut self, new_name: &str) -> bool {
        let Some(id) = self.active_id else {
            return false;
        };
        if let Some(mc) = self.containers.get_mut(&id) {
            let vm_ip = mc.network_config.vm_ip_std();
            mc.name = Some(new_name.to_string());
            let mut b = self.bridge.write().await;
            b.update_name(vm_ip, new_name);
            eprintln!("[testbed] Container #{id} named '{new_name}'");
            true
        } else {
            false
        }
    }

    /// Switch to a container by name.
    pub fn switch_to_name(&mut self, name: &str) -> bool {
        for (&id, mc) in &self.containers {
            if mc.name.as_deref() == Some(name) {
                self.active_id = Some(id);
                eprintln!("[testbed] Switched to container '{name}' (#{id})");
                return true;
            }
        }
        false
    }

    /// Get a mutable reference to the active container.
    pub fn active_mut(&mut self) -> Option<&mut ManagedContainer> {
        self.active_id.and_then(|id| self.containers.get_mut(&id))
    }

    /// Whether there is an active container.
    pub fn has_active(&self) -> bool {
        self.active_id.is_some()
    }

    /// Kill the active container and switch to another or go to testbed prompt.
    pub async fn kill_active(&mut self) {
        let Some(id) = self.active_id.take() else {
            return;
        };
        self.kill_by_id(id).await;

        // Switch to another container if one exists
        if let Some(&next_id) = self.containers.keys().next() {
            self.active_id = Some(next_id);
            let mc = &self.containers[&next_id];
            let label = mc
                .name
                .as_deref()
                .map(|n| format!("'{n}'"))
                .unwrap_or_else(|| format!("#{next_id}"));
            eprintln!("[testbed] Switched to container {label}");
        }
    }

    /// Kill a specific container by ID.
    async fn kill_by_id(&mut self, id: u32) {
        if let Some(mc) = self.containers.remove(&id) {
            let vm_ip = mc.network_config.vm_ip_std();
            mc.container.kill();
            let label = mc
                .name
                .as_deref()
                .map(|n| format!("'{n}'"))
                .unwrap_or_else(|| format!("#{id}"));
            eprintln!("[testbed] Killed container {label}");
            let mut b = self.bridge.write().await;
            b.unregister(vm_ip);
        }
    }

    /// Detach from the active container (leave it running in background).
    pub fn detach(&mut self) {
        if let Some(id) = self.active_id.take() {
            let mc = &self.containers[&id];
            let label = mc
                .name
                .as_deref()
                .map(|n| format!("'{n}'"))
                .unwrap_or_else(|| format!("#{id}"));
            eprintln!("[testbed] Detached from container {label}");
        }
    }

    /// List all containers.
    pub fn list(&self) -> Vec<ContainerInfo> {
        let mut infos: Vec<ContainerInfo> = self
            .containers
            .values()
            .map(|mc| {
                let status = match mc.terminal.testbed_state {
                    TestbedState::Booting => "booting",
                    TestbedState::Interactive => {
                        if self.active_id == Some(mc.id) {
                            "interactive"
                        } else {
                            "background"
                        }
                    }
                };
                ContainerInfo {
                    id: mc.id,
                    name: mc.name.clone(),
                    image: mc.image.clone(),
                    ip: format!("{}", mc.network_config.vm_ip),
                    status,
                    active: self.active_id == Some(mc.id),
                }
            })
            .collect();
        infos.sort_by_key(|i| i.id);
        infos
    }

    /// Kill all containers (cleanup).
    pub async fn kill_all(&mut self) {
        let ids: Vec<u32> = self.containers.keys().copied().collect();
        for id in ids {
            self.kill_by_id(id).await;
        }
        self.active_id = None;
    }
}

// ---------------------------------------------------------------------------
// Image path resolution
// ---------------------------------------------------------------------------

fn resolve_image_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let images_dir = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images");

    // 1. Check for exact file: {name}.wasm
    let exact = images_dir.join(format!("{name}.wasm"));
    if exact.is_file() {
        return exact;
    }

    // 2. Check for directory: {name}/, pick latest .wasm by lexicographic sort
    let dir = images_dir.join(name);
    if dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&dir)
    {
        let mut wasm_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "wasm"))
            .collect();
        wasm_files.sort();
        if let Some(latest) = wasm_files.last() {
            return latest.clone();
        }
    }

    // 3. Fallback: return path as-is (wasmtime gives clear "file not found" error)
    PathBuf::from(name)
}

// ---------------------------------------------------------------------------
// ANSI terminal query filtering
// ---------------------------------------------------------------------------

/// Remove Device Status Report queries (ESC[6n) from container output.
pub fn strip_dsr(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if i + 3 <= src.len()
            && src[i] == 0x1b
            && src[i + 1] == b'['
            && src[i + 2] == b'6'
            && i + 4 <= src.len()
            && src[i + 3] == b'n'
        {
            i += 4;
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}
