pub mod cli;
pub mod error;
pub mod lifter;
pub mod linker;
pub mod macho;
pub mod syscall_shim;

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::cli::Cli;

/// Run the full CLI pipeline.
pub fn run(cli: Cli) -> Result<()> {
    // 1. Parse Mach-O.
    log::info!("parsing Mach-O: {}", cli.input.display());
    let binary = macho::parse(&cli.input)
        .with_context(|| format!("failed to parse {}", cli.input.display()))?;

    log::info!(
        "arch={}, entry=0x{:x}, {} code sections, {} data sections, {} symbols",
        binary.arch,
        binary.entry_point,
        binary.code_sections.len(),
        binary.data_sections.len(),
        binary.symbols.len(),
    );

    if !binary.is_static {
        log::warn!(
            "binary links against {} dylibs (only static binaries fully supported in Phase 0):",
            binary.dylib_paths.len()
        );
        for dylib in &binary.dylib_paths {
            log::warn!("  {dylib}");
        }
    }

    // 2. Resolve semantics directory.
    let semantics_dir = resolve_semantics_dir(&cli)?;

    // 3. Lift x86_64 → LLVM IR via Remill.
    log::info!("lifting x86_64 code via Remill...");
    let lifted = lifter::lift(&binary, &semantics_dir)
        .context("remill lifting failed")?;

    // 4. If --emit-llvm, write .ll and return early.
    if cli.emit_llvm {
        let ll_path = cli.output.with_extension("ll");
        log::info!("writing LLVM IR to {}", ll_path.display());
        lifted.write_ll(&ll_path)?;
        return Ok(());
    }

    // 5. Write lifted bitcode to temp file.
    let tmp_dir = tempfile::tempdir().context("failed to create temp dir")?;
    let bc_path = tmp_dir.path().join("lifted.bc");
    lifted.write_bc(&bc_path)?;

    // 6. Compile shims.
    log::info!("compiling Darwin syscall shims...");
    let shim_objs =
        syscall_shim::compile_shims(tmp_dir.path(), cli.wasi_sysroot.as_deref())
            .context("shim compilation failed")?;

    // 7. Optimize bitcode: strip dead semantic functions (e.g. x87 BCD ops
    //    that use float80_t and can't be lowered to wasm32).
    log::info!("optimizing bitcode (stripping dead functions)...");
    let opt_bc_path = tmp_dir.path().join("lifted_opt.bc");
    linker::optimize_bc(&bc_path, &opt_bc_path, binary.entry_point)
        .context("bitcode optimization failed")?;

    // 8. Compile LLVM bitcode → wasm32 object via llc.
    log::info!("compiling LLVM IR → wasm32 object...");
    let lifted_obj = tmp_dir.path().join("lifted.o");
    linker::compile_bc_to_obj(&opt_bc_path, &lifted_obj, cli.opt_level)
        .context("llc compilation failed")?;

    // 9. Link everything into final .wasm.
    log::info!("linking → {}", cli.output.display());
    linker::link_wasm(
        &lifted_obj,
        &shim_objs,
        &cli.output,
        cli.wasi_sysroot.as_deref(),
    )
    .context("wasm-ld linking failed")?;

    let wasm_size = std::fs::metadata(&cli.output)
        .map(|m| m.len())
        .unwrap_or(0);
    log::info!("done: {} ({} bytes)", cli.output.display(), wasm_size);

    Ok(())
}

/// Library API for programmatic use.
pub fn transpile(input: &Path, output: &Path, opt_level: u8) -> Result<()> {
    let cli = Cli {
        input: input.to_path_buf(),
        output: output.to_path_buf(),
        opt_level,
        emit_llvm: false,
        semantics_dir: None,
        wasi_sysroot: None,
        verbose: false,
    };
    run(cli)
}

fn resolve_semantics_dir(cli: &Cli) -> Result<std::path::PathBuf> {
    if let Some(ref dir) = cli.semantics_dir {
        return Ok(dir.clone());
    }

    // Check standard Remill install locations.
    let candidates = [
        "/usr/local/share/remill/semantics",
        "/usr/share/remill/semantics",
    ];
    for candidate in &candidates {
        let p = Path::new(candidate);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }

    // Check REMILL_PREFIX env var.
    if let Ok(prefix) = std::env::var("REMILL_PREFIX") {
        let base = Path::new(&prefix).join("share/remill");
        // Try direct path first, then versioned subdirs (e.g. share/remill/21/semantics).
        let direct = base.join("semantics");
        if direct.exists() {
            return Ok(direct);
        }
        // Scan for versioned subdirectories (numerically named).
        if let Ok(entries) = std::fs::read_dir(&base) {
            let mut versioned: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
                })
                .collect();
            // Sort descending so we pick the highest version.
            versioned.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            for entry in versioned {
                let sem = entry.path().join("semantics");
                if sem.exists() {
                    log::info!("Found Remill semantics at {}", sem.display());
                    return Ok(sem);
                }
            }
        }
    }

    bail!(
        "Remill semantics directory not found. Set --semantics-dir or REMILL_SEMANTICS_DIR.\n\
         Expected locations: /usr/local/share/remill/semantics or $REMILL_PREFIX/share/remill/<version>/semantics"
    );
}
