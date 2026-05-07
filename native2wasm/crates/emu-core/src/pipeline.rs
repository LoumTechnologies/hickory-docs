//! Emulation pipeline: emulator WASM + guest binary → packed WASM module.
//!
//! The pipeline:
//! 1. Resolve the emulator WASM (from cache or provider)
//! 2. Prepare guest filesystem (embed binary + support files)
//! 3. Pack everything into a single WASM module using wasi-vfs
//!
//! The result is a self-contained WASM module that, when executed in a WASI
//! runtime, runs the emulator which interprets the guest binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::provider::EmulatorProvider;

/// Configuration for the emulation pipeline.
pub struct EmulationConfig<'a> {
    /// Path to the input native binary.
    pub input: &'a Path,
    /// Output .wasm file path.
    pub output: &'a Path,
    /// Explicit path to emulator WASM (overrides provider/cache).
    pub emulator_path: Option<&'a Path>,
}

/// Run the emulation pipeline.
///
/// 1. Resolve emulator WASM binary
/// 2. Prepare guest filesystem
/// 3. Pack via wasi-vfs into final output
pub fn run_emulation(provider: &dyn EmulatorProvider, config: &EmulationConfig<'_>) -> Result<()> {
    // 1. Resolve emulator WASM.
    let emulator_wasm = if let Some(explicit) = config.emulator_path {
        if !explicit.exists() {
            bail!(
                "emulator WASM not found at {} (specified via --emulator-path)",
                explicit.display()
            );
        }
        log::info!("using explicit emulator: {}", explicit.display());
        explicit.to_path_buf()
    } else {
        log::info!("resolving {} emulator WASM...", provider.name());
        provider
            .emulator_wasm()
            .with_context(|| format!("failed to resolve {} emulator", provider.name()))?
    };

    log::info!("emulator WASM: {}", emulator_wasm.display());

    // 2. Prepare guest filesystem.
    let workdir = tempfile::tempdir().context("failed to create temp dir")?;

    log::info!("preparing guest filesystem...");
    let guest_dir = provider
        .prepare_guest_fs(config.input, workdir.path())
        .context("failed to prepare guest filesystem")?;

    log::info!("guest filesystem at {}", guest_dir.display());

    // 3. Pack via wasi-vfs.
    pack_with_wasi_vfs(&emulator_wasm, &guest_dir, config.output)
        .context("wasi-vfs packing failed")?;

    let wasm_size = fs::metadata(config.output).map(|m| m.len()).unwrap_or(0);
    log::info!(
        "done: {} ({} bytes, strategy=emu/{})",
        config.output.display(),
        wasm_size,
        provider.name(),
    );

    Ok(())
}

/// Pack an emulator WASM with a guest filesystem using wasi-vfs.
///
/// wasi-vfs creates a self-contained WASM module by embedding a virtual
/// filesystem into the module's data section. The emulator accesses the
/// guest binary via WASI filesystem operations, which wasi-vfs intercepts
/// and serves from the embedded data.
fn pack_with_wasi_vfs(emulator_wasm: &Path, guest_dir: &Path, output: &Path) -> Result<()> {
    let wasi_vfs = find_wasi_vfs()?;

    let mut cmd = Command::new(&wasi_vfs);
    cmd.arg("pack")
        .arg(emulator_wasm)
        .arg("--mapdir")
        .arg(format!("/guest::{}", guest_dir.display()))
        .arg("-o")
        .arg(output);

    log::debug!("wasi-vfs: {:?}", cmd);

    let result = cmd.output().map_err(|e| {
        anyhow::anyhow!(
            "failed to run wasi-vfs: {e}. \
             Install it with: cargo install wasi-vfs-cli"
        )
    })?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        bail!("wasi-vfs pack failed: {stderr}");
    }

    Ok(())
}

/// Find the wasi-vfs tool.
fn find_wasi_vfs() -> Result<PathBuf> {
    which::which("wasi-vfs").map_err(|_| {
        anyhow::anyhow!(
            "wasi-vfs not found in PATH. Install it with:\n  \
             cargo install wasi-vfs-cli\n\n\
             Or download from: https://github.com/aspect-build/aspect-workflows"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_accepts_paths() {
        let config = EmulationConfig {
            input: Path::new("/tmp/test"),
            output: Path::new("/tmp/out.wasm"),
            emulator_path: None,
        };
        assert_eq!(config.input.display().to_string(), "/tmp/test");
    }
}
