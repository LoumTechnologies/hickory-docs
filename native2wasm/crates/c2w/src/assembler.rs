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

    // 2a. Run optional setup commands inside the extracted rootfs
    if !cli.setup_cmd.is_empty() {
        log::info!("running {} setup command(s) in rootfs", cli.setup_cmd.len());
        run_setup_in_rootfs(&rootfs_dir, &cli.setup_cmd, image_ref)
            .context("rootfs setup commands failed")?;
    }

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
        let packed_wasm = work.join("packed-pre-opt.wasm");
        run_wasi_vfs_pack(&wizered_wasm, &minpack_dir, &packed_wasm)?;

        // Wizer produces one data segment per modified memory page, which can
        // exceed wasmtime's 100k-segment parse limit. Coalesce with wasm-opt.
        run_wasm_opt(&packed_wasm, output_path)?;
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
/// Run `wasm-opt --limit-segments` to coalesce Wizer-generated data segments.
///
/// Wizer emits one segment per modified memory page; without this pass the
/// segment count easily exceeds wasmtime's 100 000-segment parse limit.
fn run_wasm_opt(input_wasm: &Path, output_wasm: &Path) -> Result<()> {
    let wasm_opt = which::which("wasm-opt").context(
        "wasm-opt not found in PATH; install binaryen (e.g. `apt install binaryen`)",
    )?;

    log::info!("running wasm-opt --limit-segments on {:?}", input_wasm);

    let status = Command::new(&wasm_opt)
        .arg("--limit-segments")
        .arg("--enable-bulk-memory")
        .arg(input_wasm)
        .arg("-o")
        .arg(output_wasm)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to execute wasm-opt")?;

    if !status.success() {
        bail!("wasm-opt failed");
    }

    log::info!("wasm-opt complete: {:?}", output_wasm);
    Ok(())
}

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

/// Run setup commands inside the extracted container rootfs.
///
/// Tries `unshare --user --map-root-user --mount` first (no root required on
/// Linux with unprivileged user namespaces enabled). Falls back to Docker with
/// `--cap-add SYS_CHROOT` when unshare is unavailable or restricted.
fn run_setup_in_rootfs(rootfs_dir: &Path, cmds: &[String], image_ref: &str) -> Result<()> {
    // Write the inner setup script into the rootfs so we avoid shell-escaping
    // every command individually.
    write_inner_setup_script(rootfs_dir, cmds)?;

    let result = if unshare_userns_available() {
        log::info!("running rootfs setup via unshare+chroot");
        run_setup_unshare(rootfs_dir)
    } else if which::which("docker").is_ok() {
        log::info!("unshare unavailable; falling back to docker chroot");
        run_setup_docker(rootfs_dir, image_ref)
    } else {
        bail!(
            "cannot run rootfs setup: neither `unshare --user` (Linux user namespaces) \
             nor `docker` is available.\n\
             Enable user namespaces: `sysctl -w kernel.unprivileged_userns_clone=1`\n\
             Or install Docker."
        )
    };

    // Always clean up the inner script regardless of success/failure.
    fs::remove_file(rootfs_dir.join("_hick_setup.sh")).ok();

    result
}

/// Write the combined setup commands as a shell script inside the rootfs.
fn write_inner_setup_script(rootfs_dir: &Path, cmds: &[String]) -> Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::PermissionsExt;

    let path = rootfs_dir.join("_hick_setup.sh");
    let mut f = fs::File::create(&path).context("create inner setup script")?;
    writeln!(f, "#!/bin/sh\nset -e")?;
    for cmd in cmds {
        writeln!(f, "{cmd}")?;
    }
    f.set_permissions(fs::Permissions::from_mode(0o755))
        .context("chmod inner setup script")?;
    Ok(())
}

/// Probe whether `unshare --user --map-root-user` works on this host.
fn unshare_userns_available() -> bool {
    Command::new("unshare")
        .args(["--user", "--map-root-user", "--", "true"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Install packages via `unshare --user --map-root-user --mount --fork`.
fn run_setup_unshare(rootfs_dir: &Path) -> Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::PermissionsExt;

    let rootfs = rootfs_dir
        .to_str()
        .context("rootfs path is not valid UTF-8")?;

    // Ensure DNS resolution and virtual filesystem mount points exist.
    let etc = rootfs_dir.join("etc");
    fs::create_dir_all(&etc).ok();
    let resolv = etc.join("resolv.conf");
    if !resolv.exists() {
        fs::write(&resolv, "nameserver 8.8.8.8\nnameserver 8.8.4.4\n").ok();
    }
    for dir in &["proc", "sys", "dev"] {
        fs::create_dir_all(rootfs_dir.join(dir)).ok();
    }

    let outer_script = format!(
        "#!/bin/sh\nset -e\n\
        mount --bind /proc '{rootfs}/proc'\n\
        mount --bind /sys  '{rootfs}/sys'\n\
        mount --bind /dev  '{rootfs}/dev'\n\
        chroot '{rootfs}' /bin/sh /_hick_setup.sh\n\
        STATUS=$?\n\
        umount '{rootfs}/dev'  2>/dev/null || true\n\
        umount '{rootfs}/sys'  2>/dev/null || true\n\
        umount '{rootfs}/proc' 2>/dev/null || true\n\
        exit $STATUS\n"
    );

    let mut tmp = tempfile::NamedTempFile::new().context("create outer script")?;
    tmp.write_all(outer_script.as_bytes())
        .context("write outer script")?;
    tmp.as_file()
        .set_permissions(fs::Permissions::from_mode(0o755))
        .context("chmod outer script")?;
    let (_, script_path) = tmp.keep().context("keep outer script")?;

    let status = Command::new("unshare")
        .args(["--user", "--map-root-user", "--mount", "--fork", "--"])
        .arg("sh")
        .arg(&script_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run `unshare`")?;

    fs::remove_file(&script_path).ok();

    if !status.success() {
        bail!("unshare setup failed (see stderr above)");
    }
    Ok(())
}

/// Install packages by bind-mounting the rootfs into a privileged Docker
/// container and running the setup script via `chroot`.
///
/// `--privileged` is required so that the container can bind-mount `/proc`,
/// `/sys`, and `/dev` into the chroot — Alpine's apk post-install scripts
/// hang without these virtual filesystems.
fn run_setup_docker(rootfs_dir: &Path, image_ref: &str) -> Result<()> {
    // Resolve to an absolute path so the Docker bind mount works regardless of cwd.
    let rootfs_abs =
        fs::canonicalize(rootfs_dir).unwrap_or_else(|_| rootfs_dir.to_path_buf());
    let rootfs_str = rootfs_abs
        .to_str()
        .context("rootfs path is not valid UTF-8")?;

    // Bind-mount virtual filesystems inside the chroot so apk post-install
    // scripts that read /proc or /dev work correctly, then chroot and run the
    // setup script, then unmount regardless of outcome.
    //
    // After unmounting, wipe /tmp in the rootfs: package managers (npm) leave
    // root-owned compile-cache files there that our non-root user can't read
    // when the c2w pipeline later copies the rootfs into the VM image.
    let shell_cmd = "\
        cp /etc/resolv.conf /rootfs/etc/resolv.conf 2>/dev/null || true && \
        mkdir -p /rootfs/proc /rootfs/sys /rootfs/dev /rootfs/dev/pts && \
        mount --bind /proc    /rootfs/proc    && \
        mount --bind /sys     /rootfs/sys     && \
        mount --bind /dev     /rootfs/dev     && \
        mount --bind /dev/pts /rootfs/dev/pts && \
        chroot /rootfs /bin/sh /_hick_setup.sh; \
        STATUS=$?; \
        umount /rootfs/dev/pts 2>/dev/null || true; \
        umount /rootfs/dev     2>/dev/null || true; \
        umount /rootfs/sys     2>/dev/null || true; \
        umount /rootfs/proc    2>/dev/null || true; \
        rm -rf /rootfs/tmp/* 2>/dev/null || true; \
        chmod -R a+rX /rootfs 2>/dev/null || true; \
        exit $STATUS";

    let status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--privileged",
            "-v",
            &format!("{rootfs_str}:/rootfs"),
            image_ref,
            "sh",
            "-c",
            shell_cmd,
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run docker for rootfs setup")?;

    if !status.success() {
        bail!("docker chroot setup failed (see stderr above)");
    }
    Ok(())
}
