pub mod detect;
pub mod strategy;

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::detect::{BinaryFormat, DetectedBinary};
use native2wasm_core::pipeline::{PipelineConfig, run_pipeline};

/// Run the AOT transpilation pipeline for the detected binary.
///
/// Dispatches to the appropriate format-specific parser (elf2wasm, mach2wasm,
/// pe2wasm) and then runs the shared pipeline.
pub fn run_aot(
    input: &Path,
    detected: &DetectedBinary,
    output: &Path,
    opt_level: u8,
    emit_llvm: bool,
    semantics_dir: Option<&Path>,
    wasi_sysroot: Option<&Path>,
) -> Result<()> {
    match detected.format {
        BinaryFormat::Elf => {
            let binary = elf2wasm::elf::parse(input)
                .with_context(|| format!("failed to parse ELF: {}", input.display()))?;

            log::info!(
                "ELF: arch={}, entry=0x{:x}, {} code, {} data, {} syms",
                binary.arch,
                binary.entry_point,
                binary.code_sections.len(),
                binary.data_sections.len(),
                binary.symbols.len(),
            );

            let config = PipelineConfig {
                output: output.to_path_buf(),
                opt_level,
                emit_llvm,
                semantics_dir: semantics_dir.map(|p| p.to_path_buf()),
                wasi_sysroot: wasi_sysroot.map(|p| p.to_path_buf()),
                os_headers: elf2wasm::os_headers(),
                os_sources: elf2wasm::os_sources(),
            };
            run_pipeline(&binary, &config)
        }
        BinaryFormat::MachO => {
            let binary = mach2wasm::macho::parse(input)
                .with_context(|| format!("failed to parse Mach-O: {}", input.display()))?;

            log::info!(
                "Mach-O: arch={}, entry=0x{:x}, {} code, {} data, {} syms",
                binary.arch,
                binary.entry_point,
                binary.code_sections.len(),
                binary.data_sections.len(),
                binary.symbols.len(),
            );

            let config = PipelineConfig {
                output: output.to_path_buf(),
                opt_level,
                emit_llvm,
                semantics_dir: semantics_dir.map(|p| p.to_path_buf()),
                wasi_sysroot: wasi_sysroot.map(|p| p.to_path_buf()),
                os_headers: mach2wasm::os_headers(),
                os_sources: mach2wasm::os_sources(),
            };
            run_pipeline(&binary, &config)
        }
        BinaryFormat::Pe => {
            let binary = pe2wasm::pe::parse(input)
                .with_context(|| format!("failed to parse PE: {}", input.display()))?;

            log::info!(
                "PE: arch={}, entry=0x{:x}, {} code, {} data, {} syms",
                binary.arch,
                binary.entry_point,
                binary.code_sections.len(),
                binary.data_sections.len(),
                binary.symbols.len(),
            );

            let config = PipelineConfig {
                output: output.to_path_buf(),
                opt_level,
                emit_llvm,
                semantics_dir: semantics_dir.map(|p| p.to_path_buf()),
                wasi_sysroot: wasi_sysroot.map(|p| p.to_path_buf()),
                os_headers: pe2wasm::os_headers(),
                os_sources: pe2wasm::os_sources(),
            };
            run_pipeline(&binary, &config)
        }
    }
}

/// Run the EMU strategy (not yet implemented).
pub fn run_emu(
    _input: &Path,
    _detected: &DetectedBinary,
    _output: &Path,
) -> Result<()> {
    bail!("EMU strategy not yet implemented (Phase 2)")
}

/// Run the VM strategy (not yet implemented).
pub fn run_vm(
    _input: &Path,
    _detected: &DetectedBinary,
    _output: &Path,
) -> Result<()> {
    bail!("VM strategy not yet implemented (Phase 5)")
}
