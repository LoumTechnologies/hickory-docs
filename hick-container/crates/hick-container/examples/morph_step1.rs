//! Step 1: Replicate wasmtime CLI behavior in-process.
//!
//! This uses the EXACT same approach as wasmtime CLI:
//! - sync WasiCtxBuilder
//! - sync add_to_linker
//! - sync TcpListener via preopened_socket
//! - async engine with epoch interruption
//!
//! The network connect runs in a separate std::thread to avoid any
//! tokio scheduling issues.

use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use wasi_common::WasiCtx;
use wasmtime::{Engine, Module, Store};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    eprintln!("[step1] Starting in-process wasmtime clone...");

    // Create engine exactly like wasmtime CLI
    let mut config = wasmtime::Config::new();
    config.async_support(true);
    config.epoch_interruption(true);
    let engine = Engine::new(&config)?;
    eprintln!("[step1] Engine created");

    // Load pre-compiled module for speed
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cwasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.v41.cwasm");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");

    // Try pre-compiled first, fall back to source compilation
    let use_cwasm = std::env::var("USE_CWASM").is_ok();
    let module = if use_cwasm && cwasm_path.exists() {
        eprintln!("[step1] Loading pre-compiled module from {:?}", cwasm_path);
        unsafe { Module::deserialize_file(&engine, &cwasm_path)? }
    } else {
        eprintln!(
            "[step1] Compiling module from {:?} (this takes a while...)",
            wasm_path
        );
        Module::from_file(&engine, &wasm_path)?
    };
    eprintln!("[step1] Module loaded");

    // Create TCP listener like wasmtime CLI's compute_preopen_sockets()
    let std_listener = std::net::TcpListener::bind("127.0.0.1:1234")
        .or_else(|_| std::net::TcpListener::bind("127.0.0.1:0"))?;
    std_listener.set_nonblocking(true)?;
    let listen_addr = std_listener.local_addr()?;
    eprintln!("[step1] Listener on {:?}", listen_addr);

    // Build WASI context using sync builder (like wasmtime CLI).
    // Use pipes instead of inherit_stdio to avoid macOS poll() ENODEV on
    // certain fd types (e.g., when stdin is a Unix domain socket).
    let mut builder = wasi_common::sync::WasiCtxBuilder::new();

    // Create UnixStream pipe pairs for stdin/stdout/stderr.
    // We use the PipeReader/PipeWriter types which implement pollable().
    let (_stdin_host, stdin_wasi) = std::os::unix::net::UnixStream::pair()?;
    stdin_wasi.set_nonblocking(true)?;
    let (_stdout_host, stdout_wasi) = std::os::unix::net::UnixStream::pair()?;
    let (_stderr_host, stderr_wasi) = std::os::unix::net::UnixStream::pair()?;

    builder
        .stdin(Box::new(hick_container::pipe::PipeReader::from_unix_stream(
            stdin_wasi,
        )))
        .stdout(Box::new(hick_container::pipe::PipeWriter::from_unix_stream(
            stdout_wasi,
        )))
        .stderr(Box::new(hick_container::pipe::PipeWriter::from_unix_stream(
            stderr_wasi,
        )));
    builder.arg("container.wasm")?;
    builder.arg("--net=socket")?;
    builder.env("LISTEN_FDS", "1")?;

    let wasi_listener = wasi_common::sync::TcpListener::from_std(std_listener);
    builder.preopened_socket(3, wasi_listener)?;
    eprintln!("[step1] Listener added as fd=3");

    // Spawn network connect on a real OS thread (not tokio) to avoid any
    // scheduling interaction with the WASM execution
    let addr = listen_addr;
    let net_thread = std::thread::spawn(move || {
        eprintln!("[net-thread] Waiting 2s for kernel...");
        std::thread::sleep(Duration::from_secs(2));

        for attempt in 1..=10 {
            eprintln!("[net-thread] Connect attempt {}...", attempt);
            match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
                Ok(stream) => {
                    eprintln!("[net-thread] Connected on attempt {}!", attempt);
                    // Read frames from the WASM kernel
                    read_frames(stream);
                    return;
                }
                Err(e) => {
                    eprintln!("[net-thread] Attempt {} failed: {e}", attempt);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
        eprintln!("[net-thread] Failed to connect after 10 attempts");
    });

    // Start epoch incrementer on another OS thread
    let engine_for_epoch = engine.clone();
    let epoch_thread = std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(10));
            engine_for_epoch.increment_epoch();
        }
    });

    // Run WASM using tokio runtime (like wasmtime CLI with async support)
    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        let mut linker = wasmtime::Linker::<WasiCtx>::new(&engine);
        wasi_common::sync::add_to_linker(&mut linker, |cx| cx)?;

        let wasi = builder.build();
        let mut store = Store::new(&engine, wasi);
        store.epoch_deadline_async_yield_and_update(1);

        linker.module_async(&mut store, "", &module).await?;

        let func = linker.get_default(&mut store, "")?;
        let typed = func.typed::<(), ()>(&store)?;
        typed.call_async(&mut store, ()).await
    });

    eprintln!("[step1] WASM exited: {:?}", result);
    let _ = net_thread.join();
    drop(epoch_thread); // runs forever, that's fine

    Ok(())
}

fn read_frames(mut stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    let mut frame_count = 0u64;
    let start = std::time::Instant::now();
    let mut header = [0u8; 4];

    while start.elapsed() < Duration::from_secs(30) {
        // Read 4-byte length prefix (QEMU protocol)
        match stream.read_exact(&mut header) {
            Ok(()) => {
                let len = u32::from_be_bytes(header) as usize;
                frame_count += 1;

                // Read the frame payload
                let mut payload = vec![0u8; len];
                match stream.read_exact(&mut payload) {
                    Ok(()) => {
                        if len >= 14 {
                            let ethertype = u16::from_be_bytes([payload[12], payload[13]]);
                            eprintln!(
                                "[net-thread] Frame #{}: {} bytes, ethertype=0x{:04x}",
                                frame_count, len, ethertype
                            );
                        } else {
                            eprintln!("[net-thread] Frame #{}: {} bytes (short)", frame_count, len);
                        }
                    }
                    Err(e) => {
                        eprintln!("[net-thread] Payload read error: {e}");
                        break;
                    }
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // timeout, try again
                continue;
            }
            Err(e) => {
                eprintln!("[net-thread] Header read error: {e}");
                break;
            }
        }
    }

    eprintln!(
        "[net-thread] Done. {} frames in {:?}",
        frame_count,
        start.elapsed()
    );
}
