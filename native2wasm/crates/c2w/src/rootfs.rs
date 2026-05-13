//! Assemble the VM rootfs directory tree from template binaries,
//! the container rootfs, and OCI spec files.
//!
//! The VM rootfs is later packed into an ISO 9660 image (`rootfs.bin`)
//! that the emulator mounts as its root filesystem.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result};

/// Standard directories that must exist in the VM rootfs.
const VM_DIRS: &[&str] = &[
    "bin",
    "sbin",
    "usr/bin",
    "usr/sbin",
    "etc",
    "proc",
    "sys",
    "dev",
    "tmp",
    "mnt",
    "run",
    "var",
    "oci",
    "oci/rootfs",
];

/// Assemble the full VM rootfs directory.
///
/// Layout:
/// ```text
/// bin/busybox, sh, ls...    (from template)
/// sbin/init, runc, tini     (from template)
/// bin/vmtouch               (from template)
/// oci/rootfs/               (container filesystem)
/// oci/image.json            (OCI image config)
/// oci/spec.json             (OCI runtime spec)
/// oci/initconfig.json       (boot config)
/// proc/ sys/ dev/ etc.      (empty mount points)
/// ```
pub fn assemble_vm_rootfs(
    template_dir: &Path,
    container_rootfs: &Path,
    spec_dir: &Path,
    output_dir: &Path,
) -> Result<()> {
    log::info!("assembling VM rootfs at {:?}", output_dir);

    // Create directory structure
    for dir in VM_DIRS {
        fs::create_dir_all(output_dir.join(dir))
            .with_context(|| format!("failed to create VM rootfs dir {:?}", dir))?;
    }

    // Copy template binaries (busybox, init, runc, tini, vmtouch)
    copy_template_binaries(template_dir, output_dir)?;

    // Copy container rootfs
    copy_dir_recursive(container_rootfs, &output_dir.join("oci/rootfs"))?;

    // Ensure /workspace exists as a mount point for WASI preopened directories.
    // Container2wasm's init enumerates WASI preopened FDs and mounts them via
    // virtio-9p. The mount silently fails if the destination does not exist in
    // the guest rootfs, so we create it unconditionally here.
    fs::create_dir_all(output_dir.join("oci/rootfs/workspace"))
        .context("failed to create /workspace mount point in OCI rootfs")?;

    // Copy spec files
    for name in &["image.json", "spec.json", "initconfig.json"] {
        let src = spec_dir.join(name);
        if src.exists() {
            fs::copy(&src, output_dir.join("oci").join(name))
                .with_context(|| format!("failed to copy {:?}", name))?;
        }
    }

    // Create /dev/null as a regular file (the kernel will set up proper devices)
    let dev_null = output_dir.join("dev/null");
    if !dev_null.exists() {
        fs::write(&dev_null, b"").context("failed to create dev/null placeholder")?;
    }

    log::info!("VM rootfs assembly complete");
    Ok(())
}

/// Copy template binaries into the VM rootfs.
///
/// Expected template directory layout:
/// ```text
/// busybox          → output/bin/busybox + symlinks
/// init             → output/sbin/init
/// runc             → output/sbin/runc
/// tini             → output/sbin/tini
/// vmtouch          → output/bin/vmtouch
/// ```
fn copy_template_binaries(template_dir: &Path, output_dir: &Path) -> Result<()> {
    // Map of template file → destination in VM rootfs
    let binaries = [
        ("busybox", "bin/busybox"),
        ("init", "sbin/init"),
        ("runc", "sbin/runc"),
        ("tini", "sbin/tini"),
        ("vmtouch", "bin/vmtouch"),
    ];

    for (name, dest) in &binaries {
        let src = template_dir.join(name);
        if src.exists() {
            let dst = output_dir.join(dest);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst)
                .with_context(|| format!("failed to copy template binary {:?}", name))?;
            // Ensure executable
            fs::set_permissions(&dst, fs::Permissions::from_mode(0o755))?;
        } else {
            log::warn!("template binary {:?} not found in {:?}", name, template_dir);
        }
    }

    // Create busybox symlinks for common applets
    let busybox_dst = output_dir.join("bin/busybox");
    if busybox_dst.exists() {
        create_busybox_symlinks(output_dir)?;
    }

    Ok(())
}

/// Create symlinks for all busybox applets by querying the binary itself.
///
/// Runs `busybox --list` (same approach as container2wasm's Dockerfile) so
/// every applet the compiled busybox supports gets a symlink — including
/// `stty`, `udhcpc`, etc. that the guest init process expects.
///
/// Falls back to a minimal hardcoded list if the binary cannot be executed
/// (e.g. cross-architecture builds without binfmt support).
fn create_busybox_symlinks(output_dir: &Path) -> Result<()> {
    let busybox_bin = output_dir.join("bin/busybox");
    let bin_dir = output_dir.join("bin");
    let sbin_dir = output_dir.join("sbin");

    let applets = query_busybox_applets(&busybox_bin);

    // sbin applets that busybox places under /sbin
    let sbin_set: std::collections::HashSet<&str> = [
        "halt", "ifconfig", "ip", "mdev", "modprobe", "pivot_root", "poweroff", "reboot",
        "route", "sysctl", "umount", "iptables", "ip6tables", "udhcpc", "udhcpd",
    ]
    .into_iter()
    .collect();

    for applet in &applets {
        let in_sbin = sbin_set.contains(applet.as_str());
        let (link, target) = if in_sbin {
            (sbin_dir.join(applet), "../bin/busybox")
        } else {
            (bin_dir.join(applet), "busybox")
        };
        if !link.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(target, &link)
                .with_context(|| format!("failed to create busybox symlink for {:?}", applet))?;
        }
    }

    Ok(())
}

/// Run `busybox --list` on the host to get every supported applet name.
/// Falls back to a minimal hardcoded set if execution fails.
fn query_busybox_applets(busybox_bin: &Path) -> Vec<String> {
    match std::process::Command::new(busybox_bin)
        .arg("--list")
        .output()
    {
        Ok(out) if out.status.success() => {
            let list: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            log::info!("busybox reports {} applets", list.len());
            list
        }
        _ => {
            log::warn!("could not run busybox --list; falling back to minimal applet set");
            [
                "sh", "ash", "cat", "cp", "dd", "df", "echo", "env", "grep", "hostname", "id",
                "kill", "ln", "ls", "mkdir", "mknod", "mount", "mv", "ping", "ps", "pwd", "rm",
                "rmdir", "sed", "sleep", "stat", "stty", "sync", "tar", "touch", "true", "false",
                "uname", "umount", "vi", "wget", "chmod", "chown", "date", "dmesg", "du", "find",
                "free", "head", "ifconfig", "ip", "less", "more", "netstat", "nslookup", "route",
                "tail", "top", "wc", "which", "xargs",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect()
        }
    }
}

/// Recursively copy a directory tree.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        log::warn!("source directory {:?} does not exist, skipping copy", src);
        return Ok(());
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src).with_context(|| format!("failed to read dir {:?}", src))? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_symlink() {
            let target = fs::read_link(&src_path)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &dst_path)
                .with_context(|| format!("failed to copy symlink {:?}", src_path))?;
        } else {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("failed to copy {:?}", src_path))?;
        }
    }

    Ok(())
}
