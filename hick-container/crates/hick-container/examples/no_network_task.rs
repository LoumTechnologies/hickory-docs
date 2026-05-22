//! Test container with socket but no network task connecting
//!
//! This tests what happens when the kernel tries to use networking
//! but nothing connects to the preopened socket.

use hick_container::{Container, ContainerConfig, create_async_engine};
use std::path::Path;
use std::time::Duration;
use wasmtime::Module;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("[test] Starting without network task...");

    let engine = create_async_engine()?;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");
    eprintln!("[test] Loading module from {:?}", wasm_path);

    let module = Module::from_file(&engine, &wasm_path)?;
    eprintln!("[test] Module loaded");

    // Start container WITHOUT networking to compare behavior
    let mut config = ContainerConfig::new(engine, module);
    config.networking = false; // No networking

    eprintln!("[test] Starting container WITHOUT networking...");
    let mut container = Container::start(config).await?;
    eprintln!("[test] Container started!");

    // Wait and collect output
    let mut buf = [0u8; 4096];
    for i in 0..10 {
        tokio::select! {
            result = container.read_stdout(&mut buf) => {
                match result {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buf[..n]);
                        eprint!("[stdout] {}", output);
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                eprintln!("[test] tick {i}...");
            }
        }
    }

    eprintln!("[test] Killing container...");
    container.kill();
    eprintln!("[test] Done");

    Ok(())
}
