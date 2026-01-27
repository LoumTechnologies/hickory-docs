use std::path::Path;

use anyhow::{Context, Result};

/// Configuration for Wizer pre-initialization.
///
/// Wizer snapshots a WebAssembly module's state after running an initialization
/// function. For container2wasm, this means booting the Linux kernel inside the
/// CPU emulator, so the resulting module starts instantly at a responsive shell
/// instead of going through the full boot sequence every time.
pub struct WizerOpts {
    /// WASI directory mappings (guest_path, host_path).
    ///
    /// For container2wasm the emulator needs `/pack` mapped to the directory
    /// containing the kernel image and container root filesystem.
    pub map_dirs: Vec<(String, String)>,

    /// Name of the initialization function export to call.
    /// Default: `"wizer.initialize"`
    pub init_func: String,

    /// Function renames to apply in the output module (old_name, new_name).
    ///
    /// For container2wasm the standard rename is `_start` -> `wizer.resume`
    /// so the pre-initialized module resumes from the snapshot instead of
    /// restarting.
    pub func_renames: Vec<(String, String)>,

    /// Whether to keep the initialization function exported in the output.
    pub keep_init_func: bool,

    /// Enable WebAssembly bulk memory operations for efficient snapshots.
    pub wasm_bulk_memory: bool,
}

impl Default for WizerOpts {
    fn default() -> Self {
        Self {
            map_dirs: Vec::new(),
            init_func: "wizer.initialize".to_string(),
            func_renames: vec![("_start".to_string(), "wizer.resume".to_string())],
            keep_init_func: false,
            wasm_bulk_memory: true,
        }
    }
}

impl WizerOpts {
    /// Build options from CLI arguments, applying container2wasm defaults.
    pub fn from_cli(cli: &crate::cli::Cli) -> Self {
        let map_dirs = cli
            .wizer_map_dir
            .iter()
            .filter_map(|mapping| {
                let parts: Vec<&str> = mapping.splitn(2, "::").collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    log::warn!("ignoring invalid --wizer-map-dir {:?} (expected guest::host)", mapping);
                    None
                }
            })
            .collect();

        Self {
            map_dirs,
            init_func: cli.wizer_init_func.clone(),
            // container2wasm default: rename _start to wizer.resume
            func_renames: vec![("_start".to_string(), "wizer.resume".to_string())],
            keep_init_func: false,
            wasm_bulk_memory: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Library-based pre-initialization (via the `wizer` crate)
// ---------------------------------------------------------------------------

/// Pre-initialize a WebAssembly module using the wizer crate directly.
///
/// This avoids needing the `wizer` CLI binary installed, but pulls in wasmtime
/// as a transitive dependency. Enable via the `wizer-lib` Cargo feature.
#[cfg(feature = "wizer-lib")]
pub fn pre_initialize(input: &Path, output: &Path, opts: &WizerOpts) -> Result<()> {
    log::info!("pre-initializing {:?} with Wizer (library mode)", input);

    let wasm_bytes = std::fs::read(input)
        .with_context(|| format!("failed to read wasm file {:?}", input))?;

    let mut w = wizer::Wizer::new();
    w.allow_wasi(true)
        .context("failed to enable WASI support in Wizer")?;
    w.wasm_bulk_memory(opts.wasm_bulk_memory);
    w.init_func(&opts.init_func);
    w.keep_init_func(opts.keep_init_func);

    for (guest, host) in &opts.map_dirs {
        w.map_dir(guest, host);
    }

    for (old_name, new_name) in &opts.func_renames {
        w.func_rename(new_name, old_name);
    }

    let output_wasm = w.run(&wasm_bytes)
        .context("Wizer pre-initialization failed")?;

    std::fs::write(output, &output_wasm)
        .with_context(|| format!("failed to write pre-initialized wasm to {:?}", output))?;

    log::info!(
        "pre-initialization complete: {} -> {} bytes",
        wasm_bytes.len(),
        output_wasm.len()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// CLI-based pre-initialization (shells out to the `wizer` binary)
// ---------------------------------------------------------------------------

/// Pre-initialize a WebAssembly module by invoking the `wizer` CLI binary.
///
/// This is the default mode that does not require the `wizer-lib` feature.
/// The `wizer` binary must be installed and available in PATH.
/// Install with: `cargo install wizer --all-features`
#[cfg(not(feature = "wizer-lib"))]
pub fn pre_initialize(input: &Path, output: &Path, opts: &WizerOpts) -> Result<()> {
    use std::process::{Command, Stdio};

    log::info!("pre-initializing {:?} with Wizer (CLI mode)", input);

    let wizer_bin = which::which("wizer").context(
        "wizer binary not found in PATH. Install with: cargo install wizer --all-features",
    )?;

    let mut cmd = Command::new(&wizer_bin);

    cmd.arg("--allow-wasi");

    if opts.wasm_bulk_memory {
        cmd.arg("--wasm-bulk-memory=true");
    }

    cmd.arg("--init-func").arg(&opts.init_func);

    if opts.keep_init_func {
        cmd.arg("--keep-init-func");
    }

    for (guest, host) in &opts.map_dirs {
        cmd.arg("--mapdir")
            .arg(format!("{}::{}", guest, host));
    }

    for (old_name, new_name) in &opts.func_renames {
        cmd.arg("-r")
            .arg(format!("{}={}", old_name, new_name));
    }

    cmd.arg("-o").arg(output);
    cmd.arg(input);

    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    log::info!("running: {:?}", cmd);

    let status = cmd
        .status()
        .context("failed to execute wizer binary")?;

    if !status.success() {
        anyhow::bail!(
            "wizer pre-initialization failed (exit code: {:?})",
            status.code()
        );
    }

    log::info!("pre-initialization complete: {:?} -> {:?}", input, output);
    Ok(())
}
