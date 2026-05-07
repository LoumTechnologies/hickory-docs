use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::binary::ParsedBinary;
use crate::lifter;
use crate::linker;
use crate::semantics;
use crate::shim_compiler;

/// Configuration for the transpilation pipeline.
pub struct PipelineConfig {
    /// Output .wasm file path.
    pub output: PathBuf,
    /// Optimization level (0-3) passed to llc.
    pub opt_level: u8,
    /// Emit LLVM IR (.ll) instead of .wasm — useful for debugging.
    pub emit_llvm: bool,
    /// Explicit path to Remill semantics directory.
    pub semantics_dir: Option<PathBuf>,
    /// Path to wasi-sysroot for libc headers/libraries.
    pub wasi_sysroot: Option<PathBuf>,
    /// OS-specific header files: `&[(filename, content)]`.
    pub os_headers: &'static [(&'static str, &'static [u8])],
    /// OS-specific C source files: `&[(filename, content)]`.
    pub os_sources: &'static [(&'static str, &'static [u8])],
}

/// Run the full 9-step transpilation pipeline.
///
/// 1. Resolve semantics directory
/// 2. Lift x86_64 → LLVM IR via Remill
/// 3. (Optional) Write .ll and return early
/// 4. Write lifted bitcode to temp file
/// 5. Compile syscall shims
/// 6. Optimize bitcode (retarget to wasm32, stub x86_fp80, inject _start)
/// 7. Compile LLVM bitcode → wasm32 object via llc
/// 8. Link everything into final .wasm
pub fn run_pipeline(binary: &dyn ParsedBinary, config: &PipelineConfig) -> Result<()> {
    let os_name = binary.os_target().display_name();

    // 1. Resolve semantics directory.
    let semantics_dir = semantics::resolve_semantics_dir(config.semantics_dir.as_deref())?;

    // 2. Lift native code → LLVM IR via Remill.
    let cpu_arch = binary.cpu_arch();
    log::info!("lifting {} code via Remill...", cpu_arch);
    let lifted = lifter::lift(binary, &semantics_dir).context("remill lifting failed")?;

    // 3. If --emit-llvm, write .ll and return early.
    if config.emit_llvm {
        let ll_path = config.output.with_extension("ll");
        log::info!("writing LLVM IR to {}", ll_path.display());
        lifted.write_ll(&ll_path)?;
        return Ok(());
    }

    // 4. Write lifted bitcode to temp file.
    let tmp_dir = tempfile::tempdir().context("failed to create temp dir")?;
    let bc_path = tmp_dir.path().join("lifted.bc");
    lifted.write_bc(&bc_path)?;

    // 5. Compile shims.
    log::info!("compiling {os_name} syscall shims...");
    let shim_objs = shim_compiler::compile_shims(
        tmp_dir.path(),
        config.wasi_sysroot.as_deref(),
        config.os_headers,
        config.os_sources,
    )
    .context("shim compilation failed")?;

    // 6. Optimize bitcode: retarget to wasm32, strip dead semantic functions
    //    (e.g. x87 BCD ops that use float80_t and can't be lowered to wasm32).
    log::info!("optimizing bitcode (stripping dead functions)...");
    let opt_bc_path = tmp_dir.path().join("lifted_opt.bc");
    linker::optimize_bc(&bc_path, &opt_bc_path, binary.entry_point(), cpu_arch)
        .context("bitcode optimization failed")?;

    // 7. Compile LLVM bitcode → wasm32 object via llc.
    log::info!("compiling LLVM IR → wasm32 object...");
    let lifted_obj = tmp_dir.path().join("lifted.o");
    linker::compile_bc_to_obj(&opt_bc_path, &lifted_obj, config.opt_level)
        .context("llc compilation failed")?;

    // 8. Link everything into final .wasm.
    log::info!("linking → {}", config.output.display());
    linker::link_wasm(
        &lifted_obj,
        &shim_objs,
        &config.output,
        config.wasi_sysroot.as_deref(),
    )
    .context("wasm-ld linking failed")?;

    let wasm_size = std::fs::metadata(&config.output)
        .map(|m| m.len())
        .unwrap_or(0);
    log::info!("done: {} ({} bytes)", config.output.display(), wasm_size);

    Ok(())
}
