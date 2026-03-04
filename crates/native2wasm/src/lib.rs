pub mod detect;
pub mod strategy;

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::detect::{BinaryFormat, DetectedBinary};
use emu_core::pipeline::{EmulationConfig, run_emulation};
use native2wasm_core::target::{CpuArch, OsTarget};
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

/// Run the EMU strategy: user-mode emulation via blink (x86_64) or
/// QEMU TCI (ARM64, Phase 4).
///
/// For x86_64 binaries, selects the appropriate provider based on OS:
/// - Linux ELF → BlinkProvider (direct blink execution)
/// - macOS Mach-O → BlinkMachoProvider (blink + Mach-O loader + XNU shims)
/// - Windows PE → BlinkPeProvider (blink + PE loader + Win32 shims)
pub fn run_emu(
    input: &Path,
    detected: &DetectedBinary,
    output: &Path,
    emulator_path: Option<&Path>,
) -> Result<()> {
    let provider: Box<dyn emu_core::provider::EmulatorProvider> = match detected.cpu_arch {
        CpuArch::X86_64 => {
            match (detected.format, detected.os_target) {
                (BinaryFormat::Elf, OsTarget::Linux) => {
                    if let Some(path) = emulator_path {
                        Box::new(emu_x86::BlinkProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_x86::BlinkProvider::new())
                    }
                }
                (BinaryFormat::MachO, OsTarget::Darwin) => {
                    log::info!(
                        "macOS x86_64 EMU: blink + Mach-O loader shim \
                         (basic POSIX calls only; complex binaries may need --strategy vm)"
                    );
                    if let Some(path) = emulator_path {
                        Box::new(emu_x86::BlinkMachoProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_x86::BlinkMachoProvider::new())
                    }
                }
                (BinaryFormat::Pe, OsTarget::Windows) => {
                    log::info!(
                        "Windows x86_64 EMU: blink + PE loader shim \
                         (MSVC CRT console only; complex binaries may need --strategy vm)"
                    );
                    if let Some(path) = emulator_path {
                        Box::new(emu_x86::BlinkPeProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_x86::BlinkPeProvider::new())
                    }
                }
                _ => {
                    bail!(
                        "unsupported format/OS combination for x86_64 EMU: {} {}",
                        detected.format,
                        detected.os_target,
                    )
                }
            }
        }
        CpuArch::AArch64 => {
            bail!(
                "ARM64 user-mode emulation not yet implemented (Phase 4).\n\
                 Use --strategy vm for ARM64 dynamic binaries."
            )
        }
    };

    let config = EmulationConfig {
        input,
        output,
        emulator_path,
    };

    run_emulation(provider.as_ref(), &config)
}

/// Run the VM strategy (not yet implemented).
pub fn run_vm(
    _input: &Path,
    _detected: &DetectedBinary,
    _output: &Path,
) -> Result<()> {
    bail!("VM strategy not yet implemented (Phase 5)")
}
