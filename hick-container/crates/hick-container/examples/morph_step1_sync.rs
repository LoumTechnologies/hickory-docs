//! Step 1 variant: Fully synchronous wasmtime (no async, no epoch).
//!
//! Tests whether the issue is with async support breaking sync WASI.

use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use wasi_common::WasiCtx;
use wasmtime::{Engine, Module, Store};

fn main() -> anyhow::Result<()> {
    eprintln!("[sync] Starting FULLY SYNC test...");

    // Create engine WITHOUT async support
    let engine = Engine::default();
    eprintln!("[sync] Engine created (no async)");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");

    eprintln!("[sync] Compiling module (this takes a while)...");
    let module = Module::from_file(&engine, &wasm_path)?;
    eprintln!("[sync] Module loaded");

    // Create TCP listener
    let std_listener = std::net::TcpListener::bind("127.0.0.1:1234")
        .or_else(|_| std::net::TcpListener::bind("127.0.0.1:0"))?;
    std_listener.set_nonblocking(true)?;
    let listen_addr = std_listener.local_addr()?;
    eprintln!("[sync] Listener on {:?}", listen_addr);

    // Build WASI context
    let mut builder = wasi_common::sync::WasiCtxBuilder::new();
    builder.inherit_stdio();
    builder.arg("--net=socket")?;
    builder.env("LISTEN_FDS", "1")?;

    let wasi_listener = wasi_common::sync::TcpListener::from_std(std_listener);
    builder.preopened_socket(3, wasi_listener)?;

    // Spawn network connect on a separate thread
    let addr = listen_addr;
    std::thread::spawn(move || {
        eprintln!("[net] Waiting 2s...");
        std::thread::sleep(Duration::from_secs(2));
        for attempt in 1..=10 {
            eprintln!("[net] Connect attempt {}...", attempt);
            match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
                Ok(stream) => {
                    eprintln!("[net] Connected on attempt {}!", attempt);
                    read_frames(stream);
                    return;
                }
                Err(e) => {
                    eprintln!("[net] Attempt {} failed: {e}", attempt);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });

    // Run WASM synchronously (no async, no epoch)
    let mut linker = wasmtime::Linker::<WasiCtx>::new(&engine);
    wasi_common::sync::add_to_linker(&mut linker, |cx| cx)?;

    let wasi = builder.build();
    let mut store = Store::new(&engine, wasi);

    linker.module(&mut store, "", &module)?;

    let func = linker.get_default(&mut store, "")?;
    let typed = func.typed::<(), ()>(&store)?;
    typed.call(&mut store, ())?;

    eprintln!("[sync] WASM exited");
    Ok(())
}

fn read_frames(stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut header = [0u8; 4];
    let mut frame_count = 0u64;
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(30) {
        match (&stream).read_exact(&mut header) {
            Ok(()) => {
                let len = u32::from_be_bytes(header) as usize;
                frame_count += 1;
                let mut payload = vec![0u8; len];
                let _ = (&stream).read_exact(&mut payload);
                if len >= 14 {
                    let et = u16::from_be_bytes([payload[12], payload[13]]);
                    eprintln!("[net] Frame #{frame_count}: {len}B, ethertype=0x{et:04x}");
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(e) => {
                eprintln!("[net] Read error: {e}");
                break;
            }
        }
    }
    eprintln!("[net] Total: {frame_count} frames in {:?}", start.elapsed());
}
