//! x86_64 user-mode emulation via blink.
//!
//! blink (github.com/jart/blink) is a 116KB x86_64 Linux user-mode emulator
//! that handles 500+ instructions and 150+ syscalls. When compiled to WASM,
//! it runs as a pure interpreter (no JIT).
//!
//! This crate implements `EmulatorProvider` for x86_64 by:
//! 1. Locating a pre-compiled `blink.wasm` (from cache or explicit path)
//! 2. Preparing a guest filesystem with the target binary at `/guest/binary`
//! 3. The emulation pipeline (in emu-core) packs everything via wasi-vfs
//!
//! ## Building blink.wasm
//!
//! blink must be cross-compiled to wasm32-wasi with JIT disabled:
//!
//! ```bash
//! git clone https://github.com/jart/blink
//! cd blink
//! # Using wasi-sdk:
//! CC=/opt/wasi-sdk/bin/clang ./configure --target=wasm32-wasi \
//!     --disable-jit --disable-fork --disable-signals
//! make -j$(nproc)
//! # Output: blink → rename to blink.wasm
//! ```
//!
//! Then cache it:
//! ```bash
//! mkdir -p ~/.cache/native2wasm/emulators/blink
//! cp blink ~/.cache/native2wasm/emulators/blink/emulator.wasm
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use emu_core::cache;
use emu_core::provider::EmulatorProvider;
use native2wasm_core::target::CpuArch;

/// Emulator provider name used for cache directory.
const PROVIDER_NAME: &str = "blink";

/// blink-based x86_64 Linux user-mode emulator provider.
pub struct BlinkProvider {
    /// Explicit path to blink.wasm (overrides cache lookup).
    explicit_path: Option<PathBuf>,
}

impl BlinkProvider {
    /// Create a new blink provider.
    pub fn new() -> Self {
        Self {
            explicit_path: None,
        }
    }

    /// Create a blink provider with an explicit emulator WASM path.
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            explicit_path: Some(path),
        }
    }
}

impl Default for BlinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorProvider for BlinkProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn supported_arch(&self) -> CpuArch {
        CpuArch::X86_64
    }

    fn emulator_wasm(&self) -> Result<PathBuf> {
        // 1. Explicit path
        if let Some(ref path) = self.explicit_path {
            if !path.exists() {
                bail!("blink.wasm not found at {}", path.display());
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
            "blink.wasm not found. To set up x86_64 emulation:\n\n\
             Option 1: Build from source\n\
               git clone https://github.com/jart/blink\n\
               cd blink\n\
               CC=/opt/wasi-sdk/bin/clang ./configure \\\n\
                 --target=wasm32-wasi --disable-jit\n\
               make -j$(nproc)\n\
               mkdir -p {cache_dir}\n\
               cp blink {cache_dir}/emulator.wasm\n\n\
             Option 2: Use --emulator-path to point to an existing blink.wasm\n\
               native2wasm input -o out.wasm --strategy emu \\\n\
                 --emulator-path /path/to/blink.wasm\n",
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
            let perms = fs::Permissions::from_mode(0o755);
            let _ = fs::set_permissions(&dest, perms);
        }

        log::info!(
            "packed guest binary: {} -> /guest/binary ({} bytes)",
            binary.display(),
            fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        );

        Ok(guest_dir)
    }

    fn emulator_args(&self) -> Vec<String> {
        // blink executes: blink /guest/binary [args...]
        // The guest binary path is the first argument to blink.
        vec!["/guest/binary".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_basics() {
        let provider = BlinkProvider::new();
        assert_eq!(provider.name(), "blink");
        assert_eq!(provider.supported_arch(), CpuArch::X86_64);
    }

    #[test]
    fn prepare_guest_fs_copies_binary() {
        let provider = BlinkProvider::new();
        let tmpdir = tempfile::tempdir().unwrap();

        // Create a fake binary.
        let fake_binary = tmpdir.path().join("test_binary");
        fs::write(&fake_binary, b"fake ELF content").unwrap();

        let workdir = tmpdir.path().join("work");
        fs::create_dir(&workdir).unwrap();

        let guest_dir = provider.prepare_guest_fs(&fake_binary, &workdir).unwrap();
        let packed = guest_dir.join("binary");
        assert!(packed.exists());
        assert_eq!(fs::read(&packed).unwrap(), b"fake ELF content");
    }
}
