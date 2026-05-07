//! Emulator artifact caching.
//!
//! Pre-compiled emulator WASM binaries are cached at
//! `~/.cache/native2wasm/emulators/<name>-<version>/emulator.wasm`.
//!
//! This follows the same pattern as `c2w/src/template.rs`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Default cache directory name.
const CACHE_SUBDIR: &str = "native2wasm/emulators";

/// Get the cache directory for a specific emulator.
///
/// Returns `~/.cache/native2wasm/emulators/<name>/`.
pub fn emulator_cache_dir(name: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(CACHE_SUBDIR)
        .join(name)
}

/// Check if a cached emulator WASM exists and return its path.
pub fn cached_emulator(name: &str) -> Option<PathBuf> {
    let dir = emulator_cache_dir(name);
    let wasm_path = dir.join("emulator.wasm");
    if wasm_path.exists() {
        log::debug!("found cached emulator at {}", wasm_path.display());
        Some(wasm_path)
    } else {
        None
    }
}

/// Store an emulator WASM binary in the cache.
pub fn store_emulator(name: &str, source: &Path) -> Result<PathBuf> {
    let dir = emulator_cache_dir(name);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create cache dir: {}", dir.display()))?;

    let dest = dir.join("emulator.wasm");
    fs::copy(source, &dest)
        .with_context(|| format!("failed to cache emulator to {}", dest.display()))?;

    log::info!("cached emulator at {}", dest.display());
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_dir_has_expected_structure() {
        let dir = emulator_cache_dir("blink");
        let path_str = dir.to_string_lossy();
        assert!(path_str.contains("native2wasm"));
        assert!(path_str.contains("emulators"));
        assert!(path_str.contains("blink"));
    }
}
