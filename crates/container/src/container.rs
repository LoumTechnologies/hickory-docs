use std::io;
use std::pin::Pin;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use wasi_common::WasiCtx;
use wasmtime::{Engine, Module, Store};

use log::{debug, info, warn};

use crate::pipe::{stderr_pipe, stdin_pipe, stdout_pipe};

const DEFAULT_PIPE_BUFFER_SIZE: usize = 65536;

/// Configuration for starting a [`Container`].
pub struct ContainerConfig {
    pub engine: Engine,
    pub module: Module,
    pub pipe_buffer_size: usize,
    pub networking: bool,
}

impl ContainerConfig {
    pub fn new(engine: Engine, module: Module) -> Self {
        Self {
            engine,
            module,
            pipe_buffer_size: DEFAULT_PIPE_BUFFER_SIZE,
            networking: false,
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
    #[allow(dead_code)]
    engine: Engine,
}

impl Container {
    /// Start a WASM module in a background task.
    pub async fn start(config: ContainerConfig) -> anyhow::Result<Self> {
        let engine = config.engine;
        let module = config.module;
        let networking = config.networking;

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
            // Create a TCP listener that WASM will use to accept connections
            // The kernel calls sock_accept on this fd to establish the network channel
            // Use the same default port (1234) that c2w-net uses
            let std_listener = std::net::TcpListener::bind("127.0.0.1:1234")
                .or_else(|_| std::net::TcpListener::bind("127.0.0.1:0"))?;
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

        // Start epoch incrementer for async cooperative scheduling
        let engine_for_epoch = engine.clone();
        let epoch_task = tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
                engine_for_epoch.increment_epoch();
            }
        });

        // Spawn the WASM module in a background task
        let engine_for_task = engine.clone();
        let task = tokio::task::spawn(async move {
            let mut linker = wasmtime::Linker::<WasiCtx>::new(&engine_for_task);
            wasi_common::sync::add_to_linker(&mut linker, |cx| cx)?;

            let wasi = builder.build();
            let mut store = Store::new(&engine_for_task, wasi);
            store.epoch_deadline_async_yield_and_update(1);

            linker.module_async(&mut store, "", &module).await?;

            let func = linker.get_default(&mut store, "")?;
            let typed = func.typed::<(), ()>(&store)?;
            typed.call_async(&mut store, ()).await
        });

        // Spawn the network task with the pre-connected stream
        // Spawn the network task - it will connect to the WASM listener
        if let Some((listen_addr, shutdown_rx)) = network_connect_addr {
            debug!(
                "[container] Spawning network task to connect to {:?}...",
                listen_addr
            );
            tokio::spawn(run_network_connect(listen_addr, shutdown_rx));
        }

        Ok(Self {
            stdin: host_stdin,
            stdout: BufReader::new(host_stdout),
            stderr: BufReader::new(host_stderr),
            task,
            epoch_task,
            network_shutdown,
            engine,
        })
    }

    /// Write raw bytes to the module's stdin.
    pub async fn write_stdin(&mut self, data: &[u8]) -> io::Result<()> {
        self.stdin.write_all(data).await
    }

    /// Write a line (with trailing newline) to the module's stdin.
    pub async fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await
    }

    /// Read a chunk of bytes from the module's stdout.
    pub async fn read_stdout(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdout.read(buf).await
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
async fn run_network_connect(addr: std::net::SocketAddr, shutdown: watch::Receiver<bool>) {
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
    network::network_loop::run_network(stream, shutdown).await;
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
