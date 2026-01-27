use std::io;
use std::pin::Pin;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, DuplexStream};
use tokio::task::JoinHandle;
use wasmtime::{Engine, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{self, WasiP1Ctx};

use crate::pipe::{ChannelStdin, ChannelStdout};

const DEFAULT_PIPE_BUFFER_SIZE: usize = 65536;

/// Configuration for starting a [`Container`].
pub struct ContainerConfig {
    pub engine: Engine,
    pub module: Module,
    pub pipe_buffer_size: usize,
}

impl ContainerConfig {
    pub fn new(engine: Engine, module: Module) -> Self {
        Self {
            engine,
            module,
            pipe_buffer_size: DEFAULT_PIPE_BUFFER_SIZE,
        }
    }
}

/// A running WASM container with programmatic stdin/stdout/stderr I/O.
///
/// The WASM module runs in a background tokio task. The host communicates via
/// [`DuplexStream`] pairs bridged to WASI through custom
/// [`StdinStream`](wasmtime_wasi::cli::StdinStream) /
/// [`StdoutStream`](wasmtime_wasi::cli::StdoutStream) implementations.
pub struct Container {
    stdin: DuplexStream,
    stdout: BufReader<DuplexStream>,
    stderr: BufReader<DuplexStream>,
    task: JoinHandle<wasmtime::Result<()>>,
    epoch_task: JoinHandle<()>,
    #[allow(dead_code)]
    engine: Engine,
}

impl Container {
    /// Start a WASM module in a background task.
    pub async fn start(config: ContainerConfig) -> anyhow::Result<Self> {
        let buf = config.pipe_buffer_size;
        let engine = config.engine;
        let module = config.module;

        // Create duplex stream pairs for stdin, stdout, stderr.
        // For each pair: host uses one end, module uses the other.
        let (host_stdin, module_stdin) = tokio::io::duplex(buf);
        let (module_stdout, host_stdout) = tokio::io::duplex(buf);
        let (module_stderr, host_stderr) = tokio::io::duplex(buf);

        // Wrap module-side streams in WASI-compatible types
        let channel_stdin = ChannelStdin::new(module_stdin);
        let channel_stdout = ChannelStdout::new(module_stdout);
        let channel_stderr = ChannelStdout::new(module_stderr);

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
            let mut linker = wasmtime::Linker::<WasiP1Ctx>::new(&engine_for_task);
            p1::add_to_linker_async(&mut linker, |cx| cx)?;

            let wasi = WasiCtxBuilder::new()
                .stdin(channel_stdin)
                .stdout(channel_stdout)
                .stderr(channel_stderr)
                .build_p1();

            let mut store = Store::new(&engine_for_task, wasi);
            store.epoch_deadline_async_yield_and_update(1);

            linker.module_async(&mut store, "", &module).await?;

            let func = linker.get_default(&mut store, "")?;
            let typed = func.typed::<(), ()>(&store)?;
            typed.call_async(&mut store, ()).await?;

            Ok(())
        });

        Ok(Self {
            stdin: host_stdin,
            stdout: BufReader::new(host_stdout),
            stderr: BufReader::new(host_stderr),
            task,
            epoch_task,
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
    }
}

/// Create a [`wasmtime::Engine`] configured for async operation with epoch
/// interruption support.
pub fn create_async_engine() -> wasmtime::Result<Engine> {
    let mut config = wasmtime::Config::new();
    config.async_support(true);
    config.epoch_interruption(true);
    Engine::new(&config)
}
