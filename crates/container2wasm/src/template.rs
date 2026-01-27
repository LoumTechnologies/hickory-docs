//! Template artifact management for pre-built emulator + kernel + guest binaries.
//!
//! The "template" is the set of container-image-independent artifacts needed
//! to build a WASM module:
//! - Emulator WASM binary (Bochs for x86, TinyEMU for RISC-V)
//! - Linux kernel image
//! - Boot files (BIOS, GRUB ISO, bootloader)
//! - Guest OS binaries (busybox, init, runc, tini, vmtouch)
//! - Emulator config file
//!
//! These can be obtained by:
//! 1. Extracting from a Docker build via `--target=bochs-dev-common` (one-time)
//! 2. Pointing `--template` at a local directory
//! 3. Downloading from a URL (if a template distribution is available)
//!
//! Templates are cached at `~/.cache/c2w/templates/<arch>-<version>/`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Default upstream container2wasm version for template compatibility.
pub const DEFAULT_TEMPLATE_VERSION: &str = "v0.8.3";

/// Expected files in a template directory, keyed by target architecture.
pub struct TemplateLayout {
    /// Emulator WASM binary (the core artifact)
    pub emulator_wasm: &'static str,
    /// Linux kernel image
    pub kernel: &'static str,
    /// Boot ISO (GRUB for Bochs/x86)
    pub boot_iso: Option<&'static str>,
    /// BIOS files
    pub bios_files: &'static [&'static str],
    /// Emulator config file
    pub config_file: &'static str,
    /// Guest OS binaries
    pub guest_binaries: &'static [&'static str],
}

/// Template layout for x86_64/amd64 (Bochs emulator).
pub const AMD64_LAYOUT: TemplateLayout = TemplateLayout {
    emulator_wasm: "bochs.wasm",
    kernel: "bzImage",
    boot_iso: Some("boot.iso"),
    bios_files: &["BIOS-bochs-latest", "VGABIOS-lgpl-latest"],
    config_file: "bochsrc",
    guest_binaries: &["busybox", "init", "runc", "tini", "vmtouch"],
};

/// Template layout for RISC-V (TinyEMU emulator).
pub const RISCV64_LAYOUT: TemplateLayout = TemplateLayout {
    emulator_wasm: "temu.wasm",
    kernel: "Image",
    boot_iso: None,
    bios_files: &["bbl.bin"],
    config_file: "config",
    guest_binaries: &["busybox", "init", "runc", "tini", "vmtouch"],
};

/// Get the template layout for a given target architecture.
pub fn layout_for_arch(arch: &str) -> Result<&'static TemplateLayout> {
    match arch {
        "amd64" | "x86_64" => Ok(&AMD64_LAYOUT),
        "riscv64" => Ok(&RISCV64_LAYOUT),
        _ => bail!("unsupported architecture for template: {}", arch),
    }
}

/// Resolve the template directory, creating/populating it if needed.
///
/// Priority:
/// 1. Explicit `--template` path from CLI
/// 2. Cached template at `~/.cache/c2w/templates/<arch>-<version>/`
/// 3. Extract from Docker build (requires Docker)
pub fn resolve_template(
    explicit_path: Option<&Path>,
    arch: &str,
    version: &str,
    cache_dir: Option<&Path>,
) -> Result<PathBuf> {
    // 1. Explicit path
    if let Some(path) = explicit_path {
        validate_template_dir(path, arch)?;
        return Ok(path.to_path_buf());
    }

    // 2. Check cache
    let cache = template_cache_dir(cache_dir, arch, version);
    if cache.exists() {
        match validate_template_dir(&cache, arch) {
            Ok(()) => {
                log::info!("using cached template at {:?}", cache);
                return Ok(cache);
            }
            Err(e) => {
                log::warn!("cached template invalid ({:?}), will re-extract", e);
            }
        }
    }

    // 3. Try to extract from Docker
    extract_template_from_docker(arch, &cache)?;
    validate_template_dir(&cache, arch)?;

    Ok(cache)
}

/// Validate that a template directory contains all required files.
pub fn validate_template_dir(path: &Path, arch: &str) -> Result<()> {
    let layout = layout_for_arch(arch)?;

    if !path.is_dir() {
        bail!("template path {:?} is not a directory", path);
    }

    // Check emulator WASM
    let emulator = path.join(layout.emulator_wasm);
    if !emulator.exists() {
        bail!(
            "template missing emulator WASM: {:?} (expected at {:?})",
            layout.emulator_wasm,
            emulator
        );
    }

    // Check kernel
    let kernel = path.join(layout.kernel);
    if !kernel.exists() {
        bail!(
            "template missing kernel: {:?} (expected at {:?})",
            layout.kernel,
            kernel
        );
    }

    // Check config
    let config = path.join(layout.config_file);
    if !config.exists() {
        bail!(
            "template missing config: {:?} (expected at {:?})",
            layout.config_file,
            config
        );
    }

    // Check guest binaries
    let guest_dir = path.join("guest");
    for bin in layout.guest_binaries {
        let bin_path = guest_dir.join(bin);
        if !bin_path.exists() {
            bail!(
                "template missing guest binary: {:?} (expected at {:?})",
                bin,
                bin_path
            );
        }
    }

    log::info!("template at {:?} validated for arch {:?}", path, arch);
    Ok(())
}

fn template_cache_dir(override_dir: Option<&Path>, arch: &str, version: &str) -> PathBuf {
    if let Some(dir) = override_dir {
        return dir.join(format!("{}-{}", arch, version));
    }

    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("c2w")
        .join("templates")
        .join(format!("{}-{}", arch, version))
}

/// Extract template artifacts from a Docker build.
///
/// This runs a targeted Docker build to extract just the emulator, kernel,
/// and guest binaries — without processing any container image.
fn extract_template_from_docker(arch: &str, output_dir: &Path) -> Result<()> {
    let _docker = which::which("docker").context(
        "Docker is required for initial template extraction. \
                  Alternatively, provide --template pointing to a pre-built template directory.",
    )?;

    let layout = layout_for_arch(arch)?;

    // Determine which Docker build target to use
    let target_stage = match arch {
        "amd64" | "x86_64" => "bochs-dev-common",
        "riscv64" => "tinyemu-dev-common",
        _ => bail!("unsupported arch for Docker template extraction: {}", arch),
    };

    log::info!(
        "extracting template for {:?} from Docker build (target: {}). \
         This is a one-time operation.",
        arch,
        target_stage
    );

    fs::create_dir_all(output_dir)?;

    // For now, document that the template must be manually extracted.
    // A full Docker extraction would require the upstream Dockerfile
    // and cloning the container2wasm repo.
    bail!(
        "Automatic template extraction from Docker is not yet implemented.\n\
         \n\
         To create a template manually for {arch}:\n\
         \n\
         1. Clone the upstream container2wasm repo:\n\
            git clone https://github.com/ktock/container2wasm\n\
         \n\
         2. Build and extract the emulator stage:\n\
            docker buildx build --target={target_stage} \\\n\
              --output type=local,dest=./emulator-out .\n\
         \n\
         3. Build and extract the kernel + guest binaries:\n\
            docker buildx build --target=vm-{arch}-dev \\\n\
              --output type=local,dest=./vm-out .\n\
         \n\
         4. Assemble the template directory:\n\
            mkdir -p {output_dir:?}/guest\n\
            cp emulator-out/.../{emulator} {output_dir:?}/\n\
            cp vm-out/pack/* {output_dir:?}/\n\
            cp <guest-binaries> {output_dir:?}/guest/\n\
         \n\
         5. Re-run c2w with --template {output_dir:?}\n",
        arch = arch,
        target_stage = target_stage,
        output_dir = output_dir.display(),
        emulator = layout.emulator_wasm,
    );
}

/// Get the path to the emulator WASM binary within a template directory.
pub fn emulator_path(template_dir: &Path, arch: &str) -> Result<PathBuf> {
    let layout = layout_for_arch(arch)?;
    Ok(template_dir.join(layout.emulator_wasm))
}

/// Get the path to the guest binaries directory within a template.
pub fn guest_binaries_dir(template_dir: &Path) -> PathBuf {
    template_dir.join("guest")
}
