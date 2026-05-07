//! Loader shim caching.
//!
//! Pre-compiled loader binaries (static Linux x86_64 ELFs) are cached at
//! `~/.cache/native2wasm/loaders/<name>`.
//!
//! These loaders run inside blink to handle non-Linux binary formats:
//! - `macho_loader`: loads Mach-O binaries, patches XNU syscalls
//! - `pe_loader`: loads PE binaries, resolves IAT with Linux thunks

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Default cache directory name.
const CACHE_SUBDIR: &str = "native2wasm/loaders";

/// Get the cache directory for loaders.
///
/// Returns `~/.cache/native2wasm/loaders/`.
pub fn loader_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(CACHE_SUBDIR)
}

/// Check if a cached loader exists and return its path.
pub fn cached_loader(name: &str) -> Option<PathBuf> {
    let dir = loader_cache_dir();
    let path = dir.join(name);
    if path.exists() {
        log::debug!("found cached loader at {}", path.display());
        Some(path)
    } else {
        None
    }
}

/// Store a loader binary in the cache.
pub fn store_loader(name: &str, source: &Path) -> Result<PathBuf> {
    let dir = loader_cache_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create loader cache dir: {}", dir.display()))?;

    let dest = dir.join(name);
    fs::copy(source, &dest)
        .with_context(|| format!("failed to cache loader to {}", dest.display()))?;

    log::info!("cached loader at {}", dest.display());
    Ok(dest)
}

/// Locate a loader by name, returning its path or an error with build instructions.
pub fn locate_loader(name: &str) -> Result<PathBuf> {
    if let Some(path) = cached_loader(name) {
        return Ok(path);
    }

    let cache_dir = loader_cache_dir();
    anyhow::bail!(
        "loader '{name}' not found at {cache_dir}/{name}\n\n\
         Build it with:\n\
           scripts/build-loaders.sh\n\n\
         Or manually:\n\
           x86_64-linux-musl-gcc -static -Os -o {cache_dir}/{name} \\\n\
             crates/emu-x86/shims/{name}.c",
        name = name,
        cache_dir = cache_dir.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_cache_dir_has_expected_structure() {
        let dir = loader_cache_dir();
        let path_str = dir.to_string_lossy();
        assert!(path_str.contains("native2wasm"));
        assert!(path_str.contains("loaders"));
    }
}
