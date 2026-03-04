//! Create ISO 9660 images from a directory tree.
//!
//! The VM rootfs is packaged as an ISO 9660 image (`rootfs.bin`) that the
//! emulator mounts as its root filesystem at boot.

use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// Create an ISO 9660 image from a directory.
///
/// Tries `mkisofs` first (used by upstream container2wasm), falling back
/// to `genisoimage` or `xorriso`. These are standard tools available on
/// most Linux systems and via Homebrew on macOS.
///
/// # Arguments
/// * `source_dir` - Directory tree to pack into the ISO
/// * `output_path` - Path to write the ISO image to
pub fn create_iso(source_dir: &Path, output_path: &Path) -> Result<()> {
    log::info!(
        "creating ISO 9660 image: {:?} → {:?}",
        source_dir,
        output_path
    );

    // Try mkisofs first (matches upstream container2wasm Dockerfile)
    if let Ok(path) = which::which("mkisofs") {
        return run_mkisofs(&path, source_dir, output_path);
    }

    // Try genisoimage (Debian/Ubuntu alternative)
    if let Ok(path) = which::which("genisoimage") {
        return run_mkisofs(&path, source_dir, output_path);
    }

    // Try xorriso in mkisofs compatibility mode
    if let Ok(path) = which::which("xorriso") {
        return run_xorriso(&path, source_dir, output_path);
    }

    bail!(
        "no ISO 9660 creation tool found. Install one of: mkisofs, genisoimage, xorriso\n\
         On macOS: brew install cdrtools\n\
         On Debian/Ubuntu: apt install genisoimage"
    );
}

fn run_mkisofs(bin: &Path, source_dir: &Path, output_path: &Path) -> Result<()> {
    let status = Command::new(bin)
        .args([
            "-l", // allow long filenames
            "-J", // Joliet extensions
            "-R", // Rock Ridge extensions (preserves Unix permissions/symlinks)
            "-D", // do not use deep directory relocation
            "-o",
        ])
        .arg(output_path)
        .arg(source_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute {:?}", bin))?;

    if !status.success() {
        bail!("mkisofs/genisoimage failed");
    }

    log::info!("ISO image created: {:?}", output_path);
    Ok(())
}

fn run_xorriso(bin: &Path, source_dir: &Path, output_path: &Path) -> Result<()> {
    let status = Command::new(bin)
        .args(["-as", "mkisofs", "-l", "-J", "-R", "-D", "-o"])
        .arg(output_path)
        .arg(source_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute {:?}", bin))?;

    if !status.success() {
        bail!("xorriso failed");
    }

    log::info!("ISO image created: {:?}", output_path);
    Ok(())
}
