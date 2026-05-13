//! Orchestrate the full pure Rust container-to-WASM pipeline.
//!
//! This replaces the Docker build path when `--no-docker` is set or Docker
//! is not available. The pipeline:
//!
//! 1. Resolve/cache emulator template (emulator WASM + kernel + guest binaries)
//! 2. Pull OCI image via HTTP and extract rootfs
//! 3. Generate OCI spec files (image.json, spec.json, initconfig.json)
//! 4. Assemble VM rootfs (template binaries + container rootfs + specs)
//! 5. Create ISO 9660 image (rootfs.bin)
//! 6. Compose /pack directory (kernel + BIOS + rootfs.bin + config)
//! 7. wasi-vfs pack (embed /pack or /minpack into emulator WASM)
//! 8. Wizer pre-initialization (if enabled)

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::cli::Cli;
use crate::template;
use crate::wizer;

/// Run the full pure Rust pipeline without Docker.
pub async fn build_without_docker(image_ref: &str, output_path: &Path, cli: &Cli) -> Result<()> {
    let tmpdir = tempfile::tempdir().context("failed to create temp dir")?;
    let work = tmpdir.path();

    // 1. Resolve template
    let template_dir = template::resolve_template(
        cli.template.as_deref(),
        &cli.target_arch,
        cli.template_version
            .as_deref()
            .unwrap_or(template::DEFAULT_TEMPLATE_VERSION),
        cli.cache_dir.as_deref(),
    )?;

    let layout = template::layout_for_arch(&cli.target_arch)?;

    // 2. Pull OCI image and extract rootfs
    let rootfs_dir = work.join("container-rootfs");
    log::info!(
        "pulling OCI image {:?} for arch {:?}",
        image_ref,
        cli.target_arch
    );

    let auth = oci_pull::auth::resolve_auth(
        oci_pull::reference::ImageReference::parse(image_ref)?.registry(),
        cli.registry_username.as_deref(),
        cli.registry_password.as_deref(),
    )?;

    let image_config =
        oci_pull::pull_and_extract(image_ref, &cli.target_arch, &rootfs_dir, &auth).await?;

    // Serialize image config for spec generation
    let image_config_json =
        serde_json::to_string_pretty(&image_config).context("failed to serialize image config")?;

    // 3. Generate spec files
    let spec_dir = work.join("specs");
    crate::spec::generate_specs(
        &image_config_json,
        &cli.target_arch,
        cli.debug_image,
        cli.wizer,
        &spec_dir,
    )?;

    // 4. Assemble VM rootfs
    let vm_rootfs_dir = work.join("vm-rootfs");
    crate::rootfs::assemble_vm_rootfs(
        &template::guest_binaries_dir(&template_dir),
        &rootfs_dir,
        &spec_dir,
        &vm_rootfs_dir,
    )?;

    // 5. Create ISO 9660 image
    let rootfs_bin = work.join("rootfs.bin");
    crate::iso::create_iso(&vm_rootfs_dir, &rootfs_bin)?;

    // 6. Compose /pack directory
    let pack_dir = work.join("pack");
    compose_pack_dir(&template_dir, &rootfs_bin, layout, &pack_dir)?;

    // 7 & 8. wasi-vfs pack + optional Wizer
    let emulator_wasm = template::emulator_path(&template_dir, &cli.target_arch)?;

    if cli.wizer {
        // Wizer mode: pre-initialize first (with full /pack), then pack
        // only the minimal files (rootfs.bin + boot.iso) via wasi-vfs.
        let wizered_wasm = work.join("emulator-wizered.wasm");
        let wizer_opts = build_wizer_opts_with_pack(cli, &pack_dir);
        wizer::pre_initialize(&emulator_wasm, &wizered_wasm, &wizer_opts)?;

        // Create minpack with only the files needed post-wizer
        let minpack_dir = work.join("minpack");
        compose_minpack_dir(&pack_dir, layout, &minpack_dir)?;

        // Pack minpack into wizered WASM
        run_wasi_vfs_pack(&wizered_wasm, &minpack_dir, output_path)?;
    } else {
        // Native mode: pack full /pack into emulator WASM
        run_wasi_vfs_pack(&emulator_wasm, &pack_dir, output_path)?;
    }

    log::info!("output written to {:?}", output_path);
    Ok(())
}

/// Compose the /pack directory with kernel, BIOS, rootfs, and config.
fn compose_pack_dir(
    template_dir: &Path,
    rootfs_bin: &Path,
    layout: &template::TemplateLayout,
    pack_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(pack_dir)?;

    // Kernel
    let kernel_src = template_dir.join(layout.kernel);
    if kernel_src.exists() {
        fs::copy(&kernel_src, pack_dir.join(layout.kernel))
            .context("failed to copy kernel to pack dir")?;
    }

    // Boot ISO
    if let Some(boot_iso) = layout.boot_iso {
        let iso_src = template_dir.join(boot_iso);
        if iso_src.exists() {
            fs::copy(&iso_src, pack_dir.join(boot_iso))
                .context("failed to copy boot ISO to pack dir")?;
        }
    }

    // BIOS files
    for bios in layout.bios_files {
        let src = template_dir.join(bios);
        if src.exists() {
            fs::copy(&src, pack_dir.join(bios)).context("failed to copy BIOS file to pack dir")?;
        }
    }

    // Config file
    let config_src = template_dir.join(layout.config_file);
    if config_src.exists() {
        fs::copy(&config_src, pack_dir.join(layout.config_file))
            .context("failed to copy config to pack dir")?;
    }

    // Rootfs ISO
    fs::copy(rootfs_bin, pack_dir.join("rootfs.bin"))
        .context("failed to copy rootfs.bin to pack dir")?;

    log::info!("pack directory composed at {:?}", pack_dir);
    Ok(())
}

/// Compose the minimal /minpack directory (post-Wizer).
/// Only includes rootfs.bin and boot.iso (if applicable).
fn compose_minpack_dir(
    pack_dir: &Path,
    layout: &template::TemplateLayout,
    minpack_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(minpack_dir)?;

    // Always include rootfs.bin
    fs::copy(pack_dir.join("rootfs.bin"), minpack_dir.join("rootfs.bin"))
        .context("failed to copy rootfs.bin to minpack")?;

    // Include boot.iso for Bochs/x86
    if let Some(boot_iso) = layout.boot_iso {
        let src = pack_dir.join(boot_iso);
        if src.exists() {
            fs::copy(&src, minpack_dir.join(boot_iso))
                .context("failed to copy boot ISO to minpack")?;
        }
    }

    Ok(())
}

/// Run `wasi-vfs pack` to embed a directory into a WASM module.
fn run_wasi_vfs_pack(input_wasm: &Path, pack_dir: &Path, output_wasm: &Path) -> Result<()> {
    let wasi_vfs = which::which("wasi-vfs")
        .context("wasi-vfs binary not found in PATH. Install with: cargo install wasi-vfs-cli")?;

    log::info!(
        "packing {:?} into {:?} → {:?}",
        pack_dir,
        input_wasm,
        output_wasm
    );

    let status = Command::new(&wasi_vfs)
        .arg("pack")
        .arg(input_wasm)
        .arg("--dir")
        .arg(format!("{}::/pack", pack_dir.display()))
        .arg("-o")
        .arg(output_wasm)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to execute wasi-vfs")?;

    if !status.success() {
        bail!("wasi-vfs pack failed");
    }

    Ok(())
}

/// Build Wizer options that map /pack to the composed pack directory.
fn build_wizer_opts_with_pack(cli: &Cli, pack_dir: &Path) -> wizer::WizerOpts {
    let mut opts = wizer::WizerOpts::from_cli(cli);

    // Add /pack mapping so the emulator can access kernel/rootfs during pre-init
    opts.map_dirs
        .push(("/pack".to_string(), pack_dir.to_string_lossy().to_string()));

    opts
}
