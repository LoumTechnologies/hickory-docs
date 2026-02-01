use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{RwLock, mpsc, watch};
use tokio::task::JoinHandle;
use wasi_common::WasiCtx;
use wasmtime::{Engine, Module, Store};

use log::{debug, info, warn};

use bridge::NetworkBridge;
use connection_filter::{AllowAll, ConnectionFilter};
use net_config::NetworkConfig;
use network::http::HttpFilter;

use crate::pipe::{stderr_pipe, stdin_pipe, stdout_pipe};

const DEFAULT_PIPE_BUFFER_SIZE: usize = 65536;

// ---------------------------------------------------------------------------
// Resource tracking
// ---------------------------------------------------------------------------

/// Shared atomic counters for container resource monitoring.
pub(crate) struct ContainerCounters {
    pub epoch_ticks: AtomicU64,
    pub stdin_bytes: AtomicU64,
    pub stdout_bytes: AtomicU64,
}

impl ContainerCounters {
    fn new() -> Self {
        Self {
            epoch_ticks: AtomicU64::new(0),
            stdin_bytes: AtomicU64::new(0),
            stdout_bytes: AtomicU64::new(0),
        }
    }
}

/// Tracks pause/resume state for a container.
struct PauseState {
    is_paused: bool,
    last_paused_at: Option<Instant>,
    total_paused: Duration,
    pause_count: u32,
}

impl PauseState {
    fn new() -> Self {
        Self {
            is_paused: false,
            last_paused_at: None,
            total_paused: Duration::ZERO,
            pause_count: 0,
        }
    }
}

/// Snapshot of container resource usage.
#[derive(Debug, Clone)]
pub struct ContainerStats {
    /// Boot duration (set externally by the executor, if known).
    pub boot_duration: Option<Duration>,
    /// Time the container spent actually executing (uptime minus paused time).
    pub exec_duration: Duration,
    /// Number of epoch ticks delivered to this container's engine.
    pub epoch_ticks: u64,
    /// Total bytes written to the container's stdin.
    pub stdin_bytes: u64,
    /// Total bytes read from the container's stdout.
    pub stdout_bytes: u64,
    /// Number of times the container was paused.
    pub pause_count: u32,
    /// Total time spent paused.
    pub paused_duration: Duration,
}

/// Configuration for starting a [`Container`].
pub struct ContainerConfig {
    pub engine: Engine,
    pub module: Module,
    pub pipe_buffer_size: usize,
    pub networking: bool,
    pub connection_filter: Option<Arc<dyn ConnectionFilter>>,
    pub http_filter: Option<Arc<dyn HttpFilter>>,
    pub network_config: Option<NetworkConfig>,
    /// Shared bridge for inter-container networking.
    pub bridge: Option<Arc<RwLock<NetworkBridge>>>,
    /// Receiver for frames routed from other containers via the bridge.
    pub bridge_rx: Option<mpsc::Receiver<Vec<u8>>>,
}

impl ContainerConfig {
    pub fn new(engine: Engine, module: Module) -> Self {
        Self {
            engine,
            module,
            pipe_buffer_size: DEFAULT_PIPE_BUFFER_SIZE,
            networking: false,
            connection_filter: None,
            http_filter: None,
            network_config: None,
            bridge: None,
            bridge_rx: None,
        }
    }
}

/// A running WASM container with programmatic stdin/stdout/stderr I/O.
///
/// The WASM module runs in a background tokio task. The host communicates via
/// OS pipe pairs bridged to WASI through `wasi-common` [`ReadPipe`](wasi_common::pipe::ReadPipe)
/// / [`WritePipe`](wasi_common::pipe::WritePipe) implementations.
pub struct Container {
    stdin: UnixStream,
    stdout: BufReader<UnixStream>,
    stderr: BufReader<UnixStream>,
    task: JoinHandle<wasmtime::Result<()>>,
    epoch_task: JoinHandle<()>,
    network_shutdown: Option<watch::Sender<bool>>,
    engine: Engine,
    pause_tx: watch::Sender<bool>,
    counters: Arc<ContainerCounters>,
    started_at: Instant,
    pause_state: PauseState,
}

impl Container {
    /// Start a WASM module in a background task.
    pub async fn start(config: ContainerConfig) -> anyhow::Result<Self> {
        let engine = config.engine;
        let module = config.module;
        let networking = config.networking;
        let connection_filter = config.connection_filter;
        let http_filter = config.http_filter;
        let network_config = config.network_config;
        let bridge = config.bridge;
        let bridge_rx = config.bridge_rx;

        // Create OS pipe pairs for stdin, stdout, stderr.
        let (host_stdin, wasi_stdin) = stdin_pipe()?;
        let (host_stdout, wasi_stdout) = stdout_pipe()?;
        let (host_stderr, wasi_stderr) = stderr_pipe()?;

        // Build WASI context using the sync builder. The sync implementation
        // uses POSIX poll() for poll_oneoff and blocking accept() for
        // sock_accept, which is how wasmtime CLI handles it. The tokio builder
        // wraps sync files in a way that breaks async readiness for listeners.
        let mut builder = wasi_common::sync::WasiCtxBuilder::new();
        builder
            .stdin(Box::new(wasi_stdin))
            .stdout(Box::new(wasi_stdout))
            .stderr(Box::new(wasi_stderr));

        // Set up networking if enabled
        let mut network_shutdown = None;
        let mut network_connect_addr = None;
        if networking {
            // Create a TCP listener that WASM will use to accept connections.
            // The kernel calls sock_accept on this fd to establish the network channel.
            // Always bind to port 0 so multiple containers can coexist.
            let std_listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            // Set non-blocking like wasmtime CLI does
            std_listener.set_nonblocking(true)?;
            let listen_addr = std_listener.local_addr()?;
            debug!(
                "[container] Created listener (non-blocking), local_addr = {:?}",
                listen_addr
            );

            // Use wasi_common::sync::TcpListener like wasmtime CLI does.
            // The sync implementation uses POSIX poll() for readiness which
            // properly blocks until a connection arrives.
            let wasi_listener = wasi_common::sync::TcpListener::from_std(std_listener);
            builder.preopened_socket(3, wasi_listener)?;

            // Pass args and env so the container2wasm kernel knows to use networking.
            // argv[0] must be a program name (like wasmtime CLI passes the module basename).
            // LISTEN_FDS=1 tells the kernel there's one preopened socket (at fd=3).
            builder.arg("container.wasm")?;
            builder.arg("--net=socket")?;
            builder.env("LISTEN_FDS", "1")?;
            builder.env("LISTEN_PID", "1")?;

            // Store the address for connecting after WASM starts
            let (shutdown_tx, shutdown_rx) = watch::channel(false);
            network_shutdown = Some(shutdown_tx);
            network_connect_addr = Some((listen_addr, shutdown_rx));
        }

        // Create pause channel (starts in running state)
        let (pause_tx, pause_rx) = watch::channel(true);

        // Shared counters for resource monitoring
        let counters = Arc::new(ContainerCounters::new());

        // Start epoch incrementer for async cooperative scheduling
        let engine_for_epoch = engine.clone();
        let counters_for_epoch = counters.clone();
        let epoch_task = tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
                engine_for_epoch.increment_epoch();
                counters_for_epoch
                    .epoch_ticks
                    .fetch_add(1, Ordering::Relaxed);
            }
        });

        // Spawn the WASM module in a background task with pause support
        let engine_for_task = engine.clone();
        let task = tokio::task::spawn(async move {
            let mut pause_rx = pause_rx;

            let mut linker = wasmtime::Linker::<WasiCtx>::new(&engine_for_task);
            wasi_common::sync::add_to_linker(&mut linker, |cx| cx)?;

            let wasi = builder.build();
            let mut store = Store::new(&engine_for_task, wasi);
            store.epoch_deadline_async_yield_and_update(1);

            linker.module_async(&mut store, "", &module).await?;

            let func = linker.get_default(&mut store, "")?;
            let typed = func.typed::<(), ()>(&store)?;

            // Wrap call_async in a pause-aware loop. When paused, the future
            // is not polled and the epoch task is stopped, so zero CPU is used.
            let mut wasm_future = std::pin::pin!(typed.call_async(&mut store, ()));

            loop {
                tokio::select! {
                    biased;
                    result = &mut wasm_future => {
                        return result;
                    }
                    result = pause_rx.changed() => {
                        if result.is_err() {
                            // Sender dropped — container is being destroyed
                            return Ok(());
                        }
                        // If paused, wait until resumed
                        while !*pause_rx.borrow() {
                            if pause_rx.changed().await.is_err() {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        });

        // Spawn the network task - it will connect to the WASM listener
        if let Some((listen_addr, shutdown_rx)) = network_connect_addr {
            let filter: Arc<dyn ConnectionFilter> =
                connection_filter.unwrap_or_else(|| Arc::new(AllowAll));
            let net_cfg = network_config.unwrap_or_default();
            debug!(
                "[container] Spawning network task to connect to {:?}...",
                listen_addr
            );
            tokio::spawn(run_network_connect(
                listen_addr,
                shutdown_rx,
                filter,
                http_filter,
                net_cfg,
                bridge,
                bridge_rx,
            ));
        }

        Ok(Self {
            stdin: host_stdin,
            stdout: BufReader::new(host_stdout),
            stderr: BufReader::new(host_stderr),
            task,
            epoch_task,
            network_shutdown,
            engine,
            pause_tx,
            counters,
            started_at: Instant::now(),
            pause_state: PauseState::new(),
        })
    }

    /// Write raw bytes to the module's stdin.
    pub async fn write_stdin(&mut self, data: &[u8]) -> io::Result<()> {
        self.counters
            .stdin_bytes
            .fetch_add(data.len() as u64, Ordering::Relaxed);
        self.stdin.write_all(data).await
    }

    /// Write a line (with trailing newline) to the module's stdin.
    pub async fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.counters
            .stdin_bytes
            .fetch_add(line.len() as u64 + 1, Ordering::Relaxed);
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await
    }

    /// Read a chunk of bytes from the module's stdout.
    pub async fn read_stdout(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.stdout.read(buf).await?;
        self.counters
            .stdout_bytes
            .fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }

    /// Read a chunk of bytes from the module's stderr.
    pub async fn read_stderr(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stderr.read(buf).await
    }

    /// Read from stdout until the given delimiter byte is found.
    pub async fn read_stdout_until(&mut self, delim: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.stdout.read_until(delim, buf).await
    }

    /// Read from stderr until the given delimiter byte is found.
    pub async fn read_stderr_until(&mut self, delim: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.stderr.read_until(delim, buf).await
    }

    /// Send Ctrl+C (0x03 ETX) to the module's stdin, which may trigger SIGINT
    /// in the emulated guest OS.
    pub async fn send_ctrl_c(&mut self) -> io::Result<()> {
        self.stdin.write_all(&[0x03]).await
    }

    /// Pause the container. Stops the epoch task and signals the WASM task
    /// to suspend. No-op if already paused.
    pub fn pause(&mut self) {
        if self.pause_state.is_paused {
            return;
        }
        self.pause_tx.send(false).ok();
        self.epoch_task.abort();
        self.pause_state.is_paused = true;
        self.pause_state.last_paused_at = Some(Instant::now());
        self.pause_state.pause_count += 1;
    }

    /// Resume a paused container. Spawns a new epoch task and signals the
    /// WASM task to continue. No-op if already running.
    pub fn resume(&mut self) {
        if !self.pause_state.is_paused {
            return;
        }
        // Accumulate paused duration
        if let Some(paused_at) = self.pause_state.last_paused_at.take() {
            self.pause_state.total_paused += paused_at.elapsed();
        }
        // Spawn a fresh epoch task
        let engine_for_epoch = self.engine.clone();
        let counters_for_epoch = self.counters.clone();
        self.epoch_task = tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
                engine_for_epoch.increment_epoch();
                counters_for_epoch
                    .epoch_ticks
                    .fetch_add(1, Ordering::Relaxed);
            }
        });
        self.pause_tx.send(true).ok();
        self.pause_state.is_paused = false;
    }

    /// Returns `true` if the container is currently paused.
    pub fn is_paused(&self) -> bool {
        self.pause_state.is_paused
    }

    /// Snapshot current resource counters and pause state.
    pub fn stats(&self) -> ContainerStats {
        let uptime = self.started_at.elapsed();
        let mut paused_duration = self.pause_state.total_paused;
        if let Some(paused_at) = self.pause_state.last_paused_at {
            paused_duration += paused_at.elapsed();
        }
        ContainerStats {
            boot_duration: None,
            exec_duration: uptime.saturating_sub(paused_duration),
            epoch_ticks: self.counters.epoch_ticks.load(Ordering::Relaxed),
            stdin_bytes: self.counters.stdin_bytes.load(Ordering::Relaxed),
            stdout_bytes: self.counters.stdout_bytes.load(Ordering::Relaxed),
            pause_count: self.pause_state.pause_count,
            paused_duration,
        }
    }

    /// Total wall-clock time since the container was started.
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Check whether the WASM task is still running.
    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }

    /// Force-stop the WASM module by aborting the background task.
    pub fn kill(&self) {
        self.task.abort();
    }

    /// Await completion of the WASM module.
    pub async fn wait(mut self) -> anyhow::Result<()> {
        self.epoch_task.abort();
        if let Some(tx) = &self.network_shutdown {
            let _ = tx.send(true);
        }

        let result = Pin::new(&mut self.task).await;
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(join_err) if join_err.is_cancelled() => Ok(()),
            Err(join_err) => Err(anyhow::anyhow!("wasm task panicked: {join_err}")),
        }
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        self.epoch_task.abort();
        if let Some(tx) = &self.network_shutdown {
            let _ = tx.send(true);
        }
    }
}

/// Run the network stack by connecting to the WASM module's listener.
///
/// The WASM module has a preopened listener socket on fd=3. When the kernel
/// starts up and calls sock_accept, it will block waiting for a connection.
/// We connect from the host side to complete the handshake.
async fn run_network_connect(
    addr: std::net::SocketAddr,
    shutdown: watch::Receiver<bool>,
    filter: Arc<dyn ConnectionFilter>,
    http_filter: Option<Arc<dyn HttpFilter>>,
    config: NetworkConfig,
    bridge: Option<Arc<RwLock<NetworkBridge>>>,
    bridge_rx: Option<mpsc::Receiver<Vec<u8>>>,
) {
    // Wait for kernel to initialize before attempting to connect
    // In c2w-net, the first connection attempt fails because the kernel
    // hasn't called sock_accept yet
    debug!("[network] Waiting 2s for kernel to initialize...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Connect with retries like c2w-net does
    debug!("[network] Connecting to {:?} with retries...", addr);

    let mut stream = None;
    for attempt in 1..=10 {
        debug!("[network] Connection attempt {}...", attempt);
        match tokio::net::TcpStream::connect(addr).await {
            Ok(s) => {
                info!("[network] Connected on attempt {}!", attempt);
                stream = Some(s);
                break;
            }
            Err(e) => {
                warn!("[network] Attempt {} failed: {e}", attempt);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    let stream = match stream {
        Some(s) => s,
        None => {
            warn!("[network] Failed to connect after 10 attempts");
            return;
        }
    };

    info!("[network] Starting network loop...");
    match (bridge, bridge_rx) {
        (Some(b), Some(rx)) => {
            network::network_loop::run_network_bridged(stream, shutdown, filter, config, b, rx)
                .await;
        }
        _ => {
            network::network_loop::run_network_with_config(
                stream,
                shutdown,
                filter,
                config,
                http_filter,
            )
            .await;
        }
    }
    debug!("[network] Network loop exited");
}

/// Create a [`wasmtime::Engine`] configured for async operation with epoch
/// interruption support.
pub fn create_async_engine() -> wasmtime::Result<Engine> {
    let mut config = wasmtime::Config::new();
    config.async_support(true);
    config.epoch_interruption(true);
    Engine::new(&config)
}
