use std::fmt;

use crate::detect::DetectedBinary;
use native2wasm_core::target::CpuArch;

/// Transpilation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Strategy {
    /// Automatically select the best strategy.
    Auto,
    /// AOT: Remill lift → wasm32 (fastest, static binaries only).
    Aot,
    /// EMU: User-mode emulator (dynamic binaries, most cases).
    Emu,
    /// VM: Full Linux VM in WASM (universal fallback).
    Vm,
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Strategy::Auto => write!(f, "auto"),
            Strategy::Aot => write!(f, "aot"),
            Strategy::Emu => write!(f, "emu"),
            Strategy::Vm => write!(f, "vm"),
        }
    }
}

/// Check if EMU strategy is available for the given binary.
fn emu_available(detected: &DetectedBinary) -> bool {
    // EMU is available for all x86_64 binaries via blink:
    //   - Linux ELF: direct blink execution
    //   - macOS Mach-O: blink + Mach-O loader shim + XNU syscall translation
    //   - Windows PE: blink + PE loader shim + Win32 API translation
    detected.cpu_arch == CpuArch::X86_64
}

/// Select the best strategy for a detected binary.
///
/// Auto-selection logic:
///   1. Static → AOT (fastest, Remill lift)
///   2. Dynamic + EMU available → EMU (user-mode emulator)
///   3. Otherwise → VM (full VM fallback)
pub fn select_strategy(detected: &DetectedBinary, requested: Strategy) -> Strategy {
    match requested {
        Strategy::Auto => {
            if detected.is_static {
                log::info!("auto-selected strategy: AOT (static binary)");
                Strategy::Aot
            } else if emu_available(detected) {
                log::info!(
                    "auto-selected strategy: EMU (dynamic {} {} binary)",
                    detected.cpu_arch,
                    detected.os_target,
                );
                Strategy::Emu
            } else {
                log::info!(
                    "auto-selected strategy: VM (no EMU support for {} {})",
                    detected.cpu_arch,
                    detected.os_target,
                );
                Strategy::Vm
            }
        }
        other => {
            log::info!("using requested strategy: {other}");
            other
        }
    }
}
