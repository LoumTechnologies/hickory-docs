//! Pre-compile the WASM module using our wasmtime version.
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    config.async_support(true);
    config.epoch_interruption(true);
    let engine = wasmtime::Engine::new(&config)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");
    let cwasm_path = wasm_path.with_extension("v41.cwasm");

    eprintln!("Compiling {:?} ...", wasm_path);
    let module = wasmtime::Module::from_file(&engine, &wasm_path)?;
    std::fs::write(&cwasm_path, module.serialize()?)?;
    eprintln!("Written to {:?}", cwasm_path);
    Ok(())
}
