//! Simple non-interactive container test

use container::{Container, ContainerConfig, create_async_engine};
use std::path::Path;
use std::time::Duration;
use wasmtime::Module;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("[test] Starting...");

    let engine = create_async_engine()?;
    eprintln!("[test] Engine created");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");
    eprintln!("[test] Loading module from {:?}", wasm_path);

    let module = Module::from_file(&engine, &wasm_path)?;
    eprintln!("[test] Module loaded");

    let mut config = ContainerConfig::new(engine, module);
    config.networking = true;

    eprintln!("[test] Starting container with networking=true...");
    let mut container = Container::start(config).await?;
    eprintln!("[test] Container started!");

    // Wait for boot and read some output
    eprintln!("[test] Waiting for output...");
    let mut buf = [0u8; 4096];

    for i in 0..20 {
        tokio::select! {
            result = container.read_stdout(&mut buf) => {
                match result {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buf[..n]);
                        eprint!("[test stdout] {}", output);
                    }
                    Ok(_) => {
                        eprintln!("[test] stdout EOF");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[test] stdout error: {e}");
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(3)) => {
                eprintln!("[test] tick {i}...");
            }
        }
    }

    eprintln!("[test] Killing container...");
    container.kill();
    eprintln!("[test] Done");

    Ok(())
}
