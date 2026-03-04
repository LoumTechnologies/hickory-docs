use std::fmt;

use crate::detect::DetectedBinary;

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

/// Select the best strategy for a detected binary.
///
/// Auto-selection logic:
///   1. Static + Remill available → AOT (fastest)
///   2. Dynamic → EMU (user-mode emulator)
///   3. EMU unsupported → VM (full VM fallback)
///
/// Currently only AOT is implemented (Phase 1).
pub fn select_strategy(detected: &DetectedBinary, requested: Strategy) -> Strategy {
    match requested {
        Strategy::Auto => {
            if detected.is_static {
                // Static binaries can use AOT (Remill lift).
                log::info!("auto-selected strategy: AOT (static binary)");
                Strategy::Aot
            } else {
                // Dynamic binaries need emulation or VM.
                // For now, fall back to VM since EMU is not yet implemented.
                log::info!("auto-selected strategy: VM (dynamic binary, EMU not yet available)");
                Strategy::Vm
            }
        }
        other => {
            log::info!("using requested strategy: {other}");
            other
        }
    }
}
