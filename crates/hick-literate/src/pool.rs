//! Container pre-warming pool for fast pipeline startup.
//!
//! Boots containers ahead of time, pauses them, and hands them to the
//! executor with dynamically-applied capabilities via [`SwappableFilter`].
//!
//! The pool is **optional** — everything works without it. Resource cost
//! of pooled containers is RAM only (~256MB per container for WASM linear
//! memory). Zero CPU, zero disk, zero I/O when paused.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use log::{debug, info, warn};
use tokio::sync::{RwLock, mpsc};
use wasmtime::Module;

use hick_net_bridge::NetworkBridge;
use hick_connection_filter::{AllowAll, ConnectionFilter, SwappableFilter};
use hick_container::{Container, ContainerConfig, create_async_engine};
use hick_token::ContainerCapabilities;
use hick_net_config::NetworkConfig;

use crate::executor::{PromptMatcher, resolve_image_path};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the container pool.
pub struct PoolConfig {
    /// Maximum number of warm containers to keep in the pool.
    pub max_warm: usize,
    /// Directory containing pre-converted `.wasm` images.
    pub images_dir: PathBuf,
    /// Eviction timeout for idle warm containers.
    pub max_idle: Duration,
}

impl PoolConfig {
    pub fn new(images_dir: PathBuf) -> Self {
        Self {
            max_warm: 4,
            images_dir,
            max_idle: Duration::from_secs(300),
        }
    }
}

// ---------------------------------------------------------------------------
// Pool statistics
// ---------------------------------------------------------------------------

/// Snapshot of pool state for reporting.
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Number of warm containers currently in the pool.
    pub warm_count: usize,
    /// Warm containers per image.
    pub per_image: HashMap<String, usize>,
    /// Estimated memory usage (warm_count * 256MB).
    pub estimated_memory_bytes: u64,
    /// Total containers served from the pool.
    pub containers_served: u64,
    /// Total containers evicted due to idle timeout.
    pub containers_evicted: u64,
    /// How long the pool has been alive.
    pub uptime: Duration,
    /// Peak memory used by pool containers (high-water mark).
    pub peak_memory_bytes: u64,
}

// ---------------------------------------------------------------------------
// SharedPool type alias
// ---------------------------------------------------------------------------

/// Thread-safe handle to a shared container pool.
///
/// Pool operations (take, warm) are infrequent and short, so a tokio
/// Mutex avoids blocking the runtime.
pub type SharedPool = Arc<tokio::sync::Mutex<ContainerPool>>;

/// Estimated RAM per warm container (WASM linear memory).
const ESTIMATED_MEMORY_PER_CONTAINER: u64 = 256 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Warm container
// ---------------------------------------------------------------------------

/// A pre-booted, paused container ready for assignment.
struct WarmContainer {
    container: Container,
    /// Handle to swap the filter from AllowAll to real capabilities.
    swappable_filter: Arc<SwappableFilter>,
    prompt_matcher: PromptMatcher,
    network_config: NetworkConfig,
    /// How long the container took to boot (for stats).
    boot_duration: Duration,
    /// When this container was added to the pool.
    warmed_at: Instant,
}

/// A container taken from the pool, ready for use by an executor.
pub(crate) struct TakenContainer {
    pub container: Container,
    pub prompt_matcher: PromptMatcher,
    pub network_config: NetworkConfig,
    pub boot_duration: Duration,
    pub image_name: String,
}

// ---------------------------------------------------------------------------
// ContainerPool
// ---------------------------------------------------------------------------

/// In-process pool of pre-warmed WASM containers.
pub struct ContainerPool {
    config: PoolConfig,
    /// Warm containers keyed by image name.
    warm: HashMap<String, Vec<WarmContainer>>,
    /// Shared WASM module cache (image name → serialized module bytes).
    module_cache: HashMap<String, Vec<u8>>,
    /// Shared bridge for inter-container networking.
    bridge: Arc<RwLock<NetworkBridge>>,
    /// Next subnet index for IP allocation.
    next_subnet: u8,
    /// Lifetime counters.
    containers_served: u64,
    containers_evicted: u64,
    /// When the pool was created (for uptime tracking).
    created_at: Instant,
    /// High-water mark of estimated memory usage.
    peak_memory_bytes: u64,
}

impl ContainerPool {
    /// Create a new container pool.
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            warm: HashMap::new(),
            module_cache: HashMap::new(),
            bridge: Arc::new(RwLock::new(NetworkBridge::new())),
            next_subnet: 128, // Start at 128 to avoid collision with executor's 0-based range
            containers_served: 0,
            containers_evicted: 0,
            created_at: Instant::now(),
            peak_memory_bytes: 0,
        }
    }

    /// How long the pool has been alive.
    pub fn uptime(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Boot `count` containers for the given image and add them to the pool.
    ///
    /// Each container boots with an `AllowAll` filter (via `SwappableFilter`)
    /// and is paused once the shell prompt is detected.
    pub async fn warm(&mut self, image: &str, count: usize) -> Result<()> {
        let current = self.warm.get(image).map_or(0, |v| v.len());
        let total = current + count;
        if total > self.config.max_warm {
            let allowed = self.config.max_warm.saturating_sub(current);
            if allowed == 0 {
                debug!(
                    "Pool at capacity ({} warm), skipping warm for '{image}'",
                    current
                );
                return Ok(());
            }
            info!(
                "Pool: warming {allowed} of {count} requested for '{image}' (capacity: {})",
                self.config.max_warm
            );
            return self.warm_n(image, allowed).await;
        }
        self.warm_n(image, count).await
    }

    /// Internal: boot exactly `n` containers for the given image.
    async fn warm_n(&mut self, image: &str, count: usize) -> Result<()> {
        for i in 0..count {
            let wc = self.boot_warm_container(image).await.with_context(|| {
                format!("failed to warm container {}/{count} for '{image}'", i + 1)
            })?;
            info!(
                "Pool: warmed '{image}' container (boot: {:?}), {} total warm",
                wc.boot_duration,
                self.warm_count() + 1
            );
            self.warm.entry(image.to_string()).or_default().push(wc);

            // Update peak memory high-water mark
            let current_memory = self.warm_count() as u64 * ESTIMATED_MEMORY_PER_CONTAINER;
            if current_memory > self.peak_memory_bytes {
                self.peak_memory_bytes = current_memory;
            }
        }
        Ok(())
    }

    /// Boot a single warm container: compile module, start, wait for prompt, pause.
    async fn boot_warm_container(&mut self, image: &str) -> Result<WarmContainer> {
        let wasm_path = resolve_image_path(&self.config.images_dir, image);

        let engine = create_async_engine()?;

        let module = if let Some(serialized) = self.module_cache.get(image) {
            info!("Pool: loading cached module for '{image}'");
            unsafe { Module::deserialize(&engine, serialized) }
                .with_context(|| format!("failed to deserialize cached module for '{image}'"))?
        } else {
            info!("Pool: compiling image '{image}': {}", wasm_path.display());
            let module = Module::from_file(&engine, &wasm_path)
                .with_context(|| format!("failed to load WASM image: {}", wasm_path.display()))?;
            let serialized = module
                .serialize()
                .with_context(|| format!("failed to serialize module for '{image}'"))?;
            self.module_cache.insert(image.to_string(), serialized);
            module
        };

        let net_cfg = NetworkConfig::for_container(self.next_subnet);
        self.next_subnet = self.next_subnet.wrapping_add(1);

        // Create bridge channel
        let (bridge_tx, bridge_rx) = mpsc::channel::<Vec<u8>>(256);
        {
            let mut b = self.bridge.write().await;
            b.register(net_cfg.vm_ip_std(), bridge_tx, Some(image));
        }

        // Create swappable filter starting with AllowAll
        let swappable = Arc::new(SwappableFilter::new(Arc::new(AllowAll)));
        let filter: Arc<dyn ConnectionFilter> = swappable.clone();

        let mut config = ContainerConfig::new(engine, module);
        config.networking = true;
        config.connection_filter = Some(filter);
        config.network_config = Some(net_cfg.clone());
        config.bridge = Some(self.bridge.clone());
        config.bridge_rx = Some(bridge_rx);

        let boot_start = Instant::now();
        let mut container = Container::start(config)
            .await
            .with_context(|| format!("failed to start warm container for '{image}'"))?;

        // Wait for boot prompt
        let mut prompt_matcher = PromptMatcher::new();
        wait_for_boot_prompt(&mut container, &mut prompt_matcher).await?;

        let boot_duration = boot_start.elapsed();

        // Pause — zero CPU while waiting in pool
        container.pause();

        Ok(WarmContainer {
            container,
            swappable_filter: swappable,
            prompt_matcher,
            network_config: net_cfg,
            boot_duration,
            warmed_at: Instant::now(),
        })
    }

    /// Take a warm container from the pool for the given image.
    ///
    /// Swaps the filter from `AllowAll` to a `CapabilityFilter` built from
    /// the provided capabilities. The container remains paused — the
    /// executor resumes it when the first command runs.
    pub(crate) fn take(
        &mut self,
        image: &str,
        caps: &ContainerCapabilities,
    ) -> Option<TakenContainer> {
        let entries = self.warm.get_mut(image)?;
        let wc = entries.pop()?;

        // Clean up empty vec
        if entries.is_empty() {
            self.warm.remove(image);
        }

        // Swap the filter to the real capability filter
        let cap_filter: Arc<dyn ConnectionFilter> =
            Arc::new(hick_connection_filter::CapabilityFilter::new(caps.clone()));
        wc.swappable_filter.swap(cap_filter);

        self.containers_served += 1;

        Some(TakenContainer {
            container: wc.container,
            prompt_matcher: wc.prompt_matcher,
            network_config: wc.network_config,
            boot_duration: wc.boot_duration,
            image_name: image.to_string(),
        })
    }

    /// Parse `.hick` sources for `<container>` and `<exec>` tags and warm
    /// one container per unique image found.
    pub async fn warm_from_sources(&mut self, sources: &[(&str, &str)]) -> Result<()> {
        let mut images = std::collections::HashSet::new();

        for (_name, source) in sources {
            if let Ok(doc) = hick_lang::parse(source) {
                collect_images_from_nodes(&doc.nodes, &mut images);
            }
        }

        for image in &images {
            let already_warm = self.warm.get(image.as_str()).map_or(0, |v| v.len());
            if already_warm == 0 {
                info!("Pool: pre-warming 1 container for image '{image}'");
                if let Err(e) = self.warm(image, 1).await {
                    warn!("Pool: failed to pre-warm '{image}': {e}");
                }
            }
        }

        Ok(())
    }

    /// Evict containers that have been idle beyond `max_idle`.
    pub fn evict_stale(&mut self) {
        let max_idle = self.config.max_idle;
        let mut evicted = 0u64;

        for entries in self.warm.values_mut() {
            let before = entries.len();
            entries.retain(|wc| {
                let idle = wc.warmed_at.elapsed() < max_idle;
                if !idle {
                    wc.container.kill();
                }
                idle
            });
            evicted += (before - entries.len()) as u64;
        }

        // Remove empty image entries
        self.warm.retain(|_, v| !v.is_empty());

        if evicted > 0 {
            self.containers_evicted += evicted;
            info!("Pool: evicted {evicted} stale container(s)");
        }
    }

    /// Snapshot of current pool state.
    pub fn stats(&self) -> PoolStats {
        let warm_count = self.warm_count();
        let per_image: HashMap<String, usize> = self
            .warm
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();

        PoolStats {
            warm_count,
            per_image,
            estimated_memory_bytes: warm_count as u64 * ESTIMATED_MEMORY_PER_CONTAINER,
            containers_served: self.containers_served,
            containers_evicted: self.containers_evicted,
            uptime: self.created_at.elapsed(),
            peak_memory_bytes: self.peak_memory_bytes,
        }
    }

    /// Total number of warm containers across all images.
    fn warm_count(&self) -> usize {
        self.warm.values().map(|v| v.len()).sum()
    }

    /// Reference to the shared module cache. Executors can seed their own
    /// cache from this to avoid recompilation.
    pub fn module_cache(&self) -> &HashMap<String, Vec<u8>> {
        &self.module_cache
    }

    /// Reference to the shared bridge.
    pub fn bridge(&self) -> &Arc<RwLock<NetworkBridge>> {
        &self.bridge
    }

    /// The next available subnet index. The executor should continue
    /// allocating from this value to avoid IP collisions.
    pub fn next_subnet(&self) -> u8 {
        self.next_subnet
    }

    /// Advance the subnet counter (called by the executor when it allocates
    /// a fresh container while sharing this pool's bridge).
    pub fn advance_subnet(&mut self) {
        self.next_subnet = self.next_subnet.wrapping_add(1);
    }

    /// Kill all warm containers and release resources.
    pub async fn shutdown(&mut self) {
        let mut killed = 0;
        for (_image, entries) in self.warm.drain() {
            for wc in entries {
                let vm_ip = wc.network_config.vm_ip_std();
                wc.container.kill();
                let mut b = self.bridge.write().await;
                b.unregister(vm_ip);
                killed += 1;
            }
        }
        if killed > 0 {
            info!("Pool: shut down {killed} warm container(s)");
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Wait for the BusyBox ash prompt (`/ # `) during container boot.
async fn wait_for_boot_prompt(
    container: &mut Container,
    prompt_matcher: &mut PromptMatcher,
) -> Result<()> {
    use crate::executor::strip_dsr;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
    let mut buf = [0u8; 4096];

    loop {
        let read_result = tokio::time::timeout_at(deadline, container.read_stdout(&mut buf)).await;

        match read_result {
            Err(_) => anyhow::bail!("timed out waiting for boot prompt"),
            Ok(Err(e)) => anyhow::bail!("read error during boot: {e}"),
            Ok(Ok(0)) => anyhow::bail!("container exited during boot"),
            Ok(Ok(n)) => {
                let cleaned = strip_dsr(&buf[..n]);
                let (_passthrough, detected) = prompt_matcher.process(&cleaned);
                if detected {
                    return Ok(());
                }
            }
        }
    }
}

/// Recursively scan hick nodes for `<container>` and `<exec>` tags and
/// collect unique image names.
fn collect_images_from_nodes(
    nodes: &[hick_lang::HickNode],
    images: &mut std::collections::HashSet<String>,
) {
    for node in nodes {
        if let hick_lang::HickNode::Tag(tag) = node {
            match tag.name.as_str() {
                "container" => {
                    if let Some(image) = tag.get_attribute("image") {
                        images.insert(image.to_string());
                    }
                    // Containers without explicit image use their name as image
                    else if let Some(name) = tag.get_attribute("name") {
                        images.insert(name.to_string());
                    }
                }
                "exec" => {
                    if let Some(image) = tag.get_attribute("image") {
                        images.insert(image.to_string());
                    }
                }
                _ => {}
            }
            collect_images_from_nodes(&tag.children, images);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_config_defaults() {
        let config = PoolConfig::new(PathBuf::from("/tmp/images"));
        assert_eq!(config.max_warm, 4);
        assert_eq!(config.max_idle, Duration::from_secs(300));
    }

    #[test]
    fn pool_stats_empty() {
        let pool = ContainerPool::new(PoolConfig::new(PathBuf::from("/tmp/images")));
        let stats = pool.stats();
        assert_eq!(stats.warm_count, 0);
        assert_eq!(stats.estimated_memory_bytes, 0);
        assert_eq!(stats.containers_served, 0);
        assert_eq!(stats.containers_evicted, 0);
        assert!(stats.per_image.is_empty());
        assert_eq!(stats.peak_memory_bytes, 0);
        // uptime should be very small (just created)
        assert!(stats.uptime < Duration::from_secs(1));
    }

    #[test]
    fn take_from_empty_pool_returns_none() {
        let mut pool = ContainerPool::new(PoolConfig::new(PathBuf::from("/tmp/images")));
        let caps = ContainerCapabilities::new();
        assert!(pool.take("alpine", &caps).is_none());
    }

    #[test]
    fn collect_images_from_hick_source() {
        let source = r#"<hick:doc>
<hick:container name="web" image="alpine" />
<hick:container name="db" image="python" />
<hick:file path="out.txt">
  <hick:exec container="web" image="alpine">echo hello</hick:exec>
</hick:file>
</hick:doc>"#;

        if let Ok(doc) = hick_lang::parse(source) {
            let mut images = std::collections::HashSet::new();
            collect_images_from_nodes(&doc.nodes, &mut images);
            assert!(images.contains("alpine"));
            assert!(images.contains("python"));
        }
    }

    #[test]
    fn evict_stale_empty_pool() {
        let mut pool = ContainerPool::new(PoolConfig::new(PathBuf::from("/tmp/images")));
        pool.evict_stale(); // Should not panic
        assert_eq!(pool.stats().warm_count, 0);
    }
}
