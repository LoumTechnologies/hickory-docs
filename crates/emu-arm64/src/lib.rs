//! ARM64 user-mode emulation via QEMU TCI.
//!
//! QEMU's Tiny Code Interpreter (TCI) is a portable CPU emulator backend
//! that interprets guest instructions without requiring a JIT. When QEMU
//! is compiled to wasm32-wasi with TCI as the backend, it can emulate
//! ARM64 Linux user-mode binaries in any WASI runtime.
//!
//! This crate implements `EmulatorProvider` for AArch64 by:
//! 1. Locating a pre-compiled `qemu-aarch64.wasm` (from cache or explicit path)
//! 2. Preparing a guest filesystem with the target binary at `/guest/binary`
//! 3. The emulation pipeline (in emu-core) packs everything via wasi-vfs
//!
//! ## Building qemu-aarch64.wasm
//!
//! QEMU TCI must be cross-compiled to wasm32-wasi:
//!
//! ```bash
//! # Using the container2wasm approach (recommended):
//! # See: https://github.com/nicktehrany/qemu-wasm
//! git clone https://github.com/nicktehrany/qemu-wasm
//! cd qemu-wasm
//! # Follow build instructions for wasm32-wasi target
//!
//! # Or build from QEMU source (10.1+ has upstream WASM support):
//! git clone https://gitlab.com/qemu-project/qemu.git
//! cd qemu
//! ./configure --target-list=aarch64-linux-user \
//!     --enable-tcg-interpreter --cross-prefix=wasm32-wasi-
//! make -j$(nproc)
//! # Output: qemu-aarch64 → rename to qemu-aarch64.wasm
//! ```
//!
//! Then cache it:
//! ```bash
//! mkdir -p ~/.cache/native2wasm/emulators/qemu-tci-aarch64
//! cp qemu-aarch64 ~/.cache/native2wasm/emulators/qemu-tci-aarch64/emulator.wasm
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use emu_core::cache;
use emu_core::provider::EmulatorProvider;
use native2wasm_core::target::{CpuArch, OsTarget};

/// Emulator provider name used for cache directory.
const PROVIDER_NAME: &str = "qemu-tci-aarch64";

/// QEMU TCI-based ARM64 Linux user-mode emulator provider.
pub struct QemuTciProvider {
    /// Explicit path to qemu-aarch64.wasm (overrides cache lookup).
    explicit_path: Option<PathBuf>,
}

impl QemuTciProvider {
    /// Create a new QEMU TCI provider.
    pub fn new() -> Self {
        Self {
            explicit_path: None,
        }
    }

    /// Create a QEMU TCI provider with an explicit emulator WASM path.
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            explicit_path: Some(path),
        }
    }
}

impl Default for QemuTciProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorProvider for QemuTciProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn supported_arch(&self) -> CpuArch {
        CpuArch::AArch64
    }

    fn supported_os(&self) -> OsTarget {
        OsTarget::Linux
    }

    fn emulator_wasm(&self) -> Result<PathBuf> {
        // 1. Explicit path
        if let Some(ref path) = self.explicit_path {
            if !path.exists() {
                bail!("qemu-aarch64.wasm not found at {}", path.display());
            }
            return Ok(path.clone());
        }

        // 2. Check cache
        if let Some(cached) = cache::cached_emulator(PROVIDER_NAME) {
            return Ok(cached);
        }

        // 3. Not found — provide instructions
        let cache_dir = cache::emulator_cache_dir(PROVIDER_NAME);
        bail!(
            "qemu-aarch64.wasm not found. To set up ARM64 emulation:\n\n\
             Option 1: Build QEMU TCI for WASM\n\
               # Using qemu-wasm (recommended):\n\
               git clone https://github.com/nicktehrany/qemu-wasm\n\
               cd qemu-wasm && follow build instructions\n\n\
               # Or from QEMU source (10.1+):\n\
               git clone https://gitlab.com/qemu-project/qemu.git\n\
               cd qemu\n\
               ./configure --target-list=aarch64-linux-user \\\n\
                 --enable-tcg-interpreter\n\
               make -j$(nproc)\n\
               mkdir -p {cache_dir}\n\
               cp qemu-aarch64 {cache_dir}/emulator.wasm\n\n\
             Option 2: Use --emulator-path to point to an existing qemu-aarch64.wasm\n\
               native2wasm input -o out.wasm --strategy emu \\\n\
                 --emulator-path /path/to/qemu-aarch64.wasm\n",
            cache_dir = cache_dir.display(),
        )
    }

    fn prepare_guest_fs(&self, binary: &Path, workdir: &Path) -> Result<PathBuf> {
        let guest_dir = workdir.join("guest");
        fs::create_dir_all(&guest_dir)
            .context("failed to create guest directory")?;

        // Copy the target binary into the guest filesystem.
        let dest = guest_dir.join("binary");
        fs::copy(binary, &dest)
            .with_context(|| format!(
                "failed to copy {} to guest filesystem",
                binary.display()
            ))?;

        // Make the binary executable (on platforms that support it).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
        }

        // Create a minimal /lib directory for ARM64 dynamic linker/libs.
        let lib_dir = guest_dir.join("lib");
        fs::create_dir_all(&lib_dir)
            .context("failed to create guest lib directory")?;

        log::info!(
            "packed ARM64 guest binary: {} -> /guest/binary ({} bytes)",
            binary.display(),
            fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        );

        Ok(guest_dir)
    }

    fn emulator_args(&self) -> Vec<String> {
        // QEMU user-mode: qemu-aarch64 /guest/binary [args...]
        vec!["/guest/binary".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_basics() {
        let provider = QemuTciProvider::new();
        assert_eq!(provider.name(), "qemu-tci-aarch64");
        assert_eq!(provider.supported_arch(), CpuArch::AArch64);
        assert_eq!(provider.supported_os(), OsTarget::Linux);
        assert_eq!(provider.emulator_args(), vec!["/guest/binary"]);
    }

    #[test]
    fn prepare_guest_fs_copies_binary() {
        let provider = QemuTciProvider::new();
        let tmpdir = tempfile::tempdir().unwrap();

        // Create a fake binary.
        let fake_binary = tmpdir.path().join("test_arm64");
        fs::write(&fake_binary, b"fake ARM64 ELF content").unwrap();

        let workdir = tmpdir.path().join("work");
        fs::create_dir(&workdir).unwrap();

        let guest_dir = provider.prepare_guest_fs(&fake_binary, &workdir).unwrap();
        let packed = guest_dir.join("binary");
        assert!(packed.exists());
        assert_eq!(fs::read(&packed).unwrap(), b"fake ARM64 ELF content");

        // lib directory should exist for dynamic linker/libs.
        assert!(guest_dir.join("lib").is_dir());
    }
}
