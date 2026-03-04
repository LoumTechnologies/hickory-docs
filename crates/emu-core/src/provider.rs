use std::path::{Path, PathBuf};

use anyhow::Result;
use native2wasm_core::target::{CpuArch, OsTarget};

/// Trait for user-mode emulator providers.
///
/// Each provider wraps a specific emulator (blink for x86_64, QEMU TCI for
/// ARM64, etc.) and knows how to:
/// - Locate the pre-compiled emulator WASM binary
/// - Prepare a guest filesystem with the target binary embedded
pub trait EmulatorProvider: Send + Sync {
    /// Human-readable name of this emulator (e.g., "blink", "blink-macho").
    fn name(&self) -> &str;

    /// CPU architecture this emulator handles.
    fn supported_arch(&self) -> CpuArch;

    /// Guest OS this provider targets (Linux, Darwin, Windows).
    fn supported_os(&self) -> OsTarget;

    /// Path to the pre-compiled emulator WASM binary.
    ///
    /// The provider should check the cache first, then download or build
    /// the emulator if not found. Returns an error if the emulator cannot
    /// be obtained.
    fn emulator_wasm(&self) -> Result<PathBuf>;

    /// Prepare a guest filesystem directory containing the target binary
    /// and any required support files (dynamic linker, libraries, etc.).
    ///
    /// Returns the path to the prepared directory. The directory will be
    /// packed into the final WASM module using wasi-vfs.
    fn prepare_guest_fs(&self, binary: &Path, workdir: &Path) -> Result<PathBuf>;

    /// Command-line arguments the emulator uses to run the guest binary.
    ///
    /// These are baked into the WASM module's entry point.
    /// Default: `["/guest/binary"]`
    fn emulator_args(&self) -> Vec<String> {
        vec!["/guest/binary".to_string()]
    }
}
