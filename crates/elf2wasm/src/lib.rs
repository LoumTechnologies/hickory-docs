pub mod cli;
pub mod elf;

use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::Cli;
use native2wasm_core::pipeline::{PipelineConfig, run_pipeline};

/// OS-specific shim sources for Linux.
static OS_HEADERS: &[(&str, &[u8])] = &[(
    "linux_syscall.h",
    include_bytes!("../shims/linux_syscall.h"),
)];

static OS_SOURCES: &[(&str, &[u8])] = &[
    (
        "syscall_dispatch.c",
        include_bytes!("../shims/syscall_dispatch.c"),
    ),
    ("shim_io.c", include_bytes!("../shims/shim_io.c")),
    ("shim_memory.c", include_bytes!("../shims/shim_memory.c")),
    ("shim_misc.c", include_bytes!("../shims/shim_misc.c")),
];

/// Access OS-specific headers for use by the unified CLI.
pub fn os_headers() -> &'static [(&'static str, &'static [u8])] {
    OS_HEADERS
}

/// Access OS-specific sources for use by the unified CLI.
pub fn os_sources() -> &'static [(&'static str, &'static [u8])] {
    OS_SOURCES
}

/// Run the full CLI pipeline.
pub fn run(cli: Cli) -> Result<()> {
    // 1. Parse ELF.
    log::info!("parsing ELF: {}", cli.input.display());
    let binary = elf::parse(&cli.input)
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
            "binary links {} shared libraries (only static binaries fully supported in Phase 0):",
            binary.shared_libs.len()
        );
        for lib in &binary.shared_libs {
            log::warn!("  {lib}");
        }
    }

    // 2. Run the shared pipeline.
    let config = PipelineConfig {
        output: cli.output,
        opt_level: cli.opt_level,
        emit_llvm: cli.emit_llvm,
        semantics_dir: cli.semantics_dir,
        wasi_sysroot: cli.wasi_sysroot,
        os_headers: OS_HEADERS,
        os_sources: OS_SOURCES,
    };

    run_pipeline(&binary, &config)
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
