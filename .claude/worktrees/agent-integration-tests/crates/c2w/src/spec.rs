//! Generate OCI runtime spec files for the VM guest.
//!
//! Produces three files that the guest init process reads at boot:
//! - `image.json` — raw OCI image configuration
//! - `spec.json` — OCI runtime spec (process args, env, user, mounts)
//! - `initconfig.json` — boot configuration (mounts, runc command, etc.)

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;

/// Write OCI spec files to the output directory.
///
/// # Arguments
/// * `image_config_json` - Raw image config JSON string
/// * `target_arch` - Guest architecture (e.g. "amd64", "riscv64")
/// * `debug` - Enable debug output in guest
/// * `is_wizer` - Whether Wizer pre-initialization will be used
/// * `output_dir` - Directory to write spec files into
pub fn generate_specs(
    image_config_json: &str,
    target_arch: &str,
    debug: bool,
    is_wizer: bool,
    output_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create spec output dir {:?}", output_dir))?;

    // Parse the image config to extract process settings
    let image_config: ImageConfig =
        serde_json::from_str(image_config_json).context("failed to parse image config")?;

    // 1. Write image.json (raw image config)
    fs::write(output_dir.join("image.json"), image_config_json)
        .context("failed to write image.json")?;

    // 2. Generate and write spec.json (OCI runtime spec)
    let spec = generate_runtime_spec(&image_config, target_arch)?;
    let spec_json = serde_json::to_string_pretty(&spec)?;
    fs::write(output_dir.join("spec.json"), &spec_json).context("failed to write spec.json")?;

    // 3. Generate and write initconfig.json (boot config)
    let init_config = generate_init_config(debug, is_wizer)?;
    let init_json = serde_json::to_string_pretty(&init_config)?;
    fs::write(output_dir.join("initconfig.json"), &init_json)
        .context("failed to write initconfig.json")?;

    log::info!("wrote spec files to {:?}", output_dir);
    Ok(())
}

/// Subset of OCI image config we need to extract process settings.
#[derive(Debug, Deserialize)]
struct ImageConfig {
    #[serde(default)]
    config: Option<ImageConfigDetails>,
    #[allow(dead_code)]
    #[serde(default)]
    architecture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageConfigDetails {
    #[serde(rename = "Entrypoint", default)]
    entrypoint: Option<Vec<String>>,
    #[serde(rename = "Cmd", default)]
    cmd: Option<Vec<String>>,
    #[serde(rename = "Env", default)]
    env: Option<Vec<String>>,
    #[serde(rename = "WorkingDir", default)]
    working_dir: Option<String>,
    #[serde(rename = "User", default)]
    user: Option<String>,
}

/// Generate an OCI runtime spec matching the upstream create-spec logic.
fn generate_runtime_spec(
    image_config: &ImageConfig,
    target_arch: &str,
) -> Result<serde_json::Value> {
    let config = image_config.config.as_ref();

    // Process args = entrypoint + cmd
    let mut args: Vec<String> = Vec::new();
    if let Some(ep) = config.and_then(|c| c.entrypoint.as_ref()) {
        args.extend(ep.iter().cloned());
    }
    if let Some(cmd) = config.and_then(|c| c.cmd.as_ref()) {
        args.extend(cmd.iter().cloned());
    }
    if args.is_empty() {
        args.push("/bin/sh".to_string());
    }

    // Environment variables
    let mut env: Vec<String> = config
        .and_then(|c| c.env.as_ref())
        .cloned()
        .unwrap_or_default();

    // Ensure PATH is set
    if !env.iter().any(|e| e.starts_with("PATH=")) {
        env.push("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string());
    }

    let cwd = config
        .and_then(|c| c.working_dir.as_ref())
        .filter(|w| !w.is_empty())
        .cloned()
        .unwrap_or_else(|| "/".to_string());

    // User
    let (uid, gid) = parse_user(config.and_then(|c| c.user.as_deref()).unwrap_or("0"));

    // Map target arch to OCI platform arch string
    let platform_arch = match target_arch {
        "amd64" => "amd64",
        "riscv64" => "riscv64",
        "arm64" | "aarch64" => "arm64",
        other => other,
    };

    let spec = json!({
        "ociVersion": "1.0.2-dev",
        "process": {
            "terminal": true,
            "user": {
                "uid": uid,
                "gid": gid
            },
            "args": args,
            "env": env,
            "cwd": cwd,
            "capabilities": default_capabilities(),
            "noNewPrivileges": false,
            "rlimits": [
                {
                    "type": "RLIMIT_NOFILE",
                    "hard": 1024,
                    "soft": 1024
                }
            ]
        },
        "root": {
            "path": "/run/rootfs",
            "readonly": false
        },
        "hostname": "localhost",
        "mounts": runtime_mounts(),
        "linux": {
            "namespaces": [
                { "type": "pid" },
                { "type": "ipc" },
                { "type": "uts" },
                { "type": "mount" }
            ],
            "resources": {
                "devices": [
                    { "allow": true, "access": "rwm" }
                ]
            }
        },
        "platform": {
            "os": "linux",
            "arch": platform_arch
        }
    });

    Ok(spec)
}

fn default_capabilities() -> serde_json::Value {
    let caps = vec![
        "CAP_CHOWN",
        "CAP_DAC_OVERRIDE",
        "CAP_FSETID",
        "CAP_FOWNER",
        "CAP_MKNOD",
        "CAP_NET_RAW",
        "CAP_SETGID",
        "CAP_SETUID",
        "CAP_SETFCAP",
        "CAP_SETPCAP",
        "CAP_NET_BIND_SERVICE",
        "CAP_SYS_CHROOT",
        "CAP_KILL",
        "CAP_AUDIT_WRITE",
    ];

    json!({
        "bounding": caps,
        "effective": caps,
        "inheritable": caps,
        "permitted": caps,
        "ambient": caps
    })
}

fn runtime_mounts() -> Vec<serde_json::Value> {
    vec![
        json!({
            "destination": "/proc",
            "type": "proc",
            "source": "proc"
        }),
        json!({
            "destination": "/dev",
            "type": "tmpfs",
            "source": "tmpfs",
            "options": ["nosuid", "strictatime", "mode=755", "size=65536k"]
        }),
        json!({
            "destination": "/dev/pts",
            "type": "devpts",
            "source": "devpts",
            "options": ["nosuid", "noexec", "newinstance", "ptmxmode=0666", "mode=0620"]
        }),
        json!({
            "destination": "/dev/shm",
            "type": "tmpfs",
            "source": "shm",
            "options": ["nosuid", "noexec", "nodev", "mode=1777", "size=65536k"]
        }),
        json!({
            "destination": "/dev/mqueue",
            "type": "mqueue",
            "source": "mqueue",
            "options": ["nosuid", "noexec", "nodev"]
        }),
        json!({
            "destination": "/sys",
            "type": "sysfs",
            "source": "sysfs",
            "options": ["nosuid", "noexec", "nodev", "ro"]
        }),
    ]
}

fn parse_user(user_str: &str) -> (u32, u32) {
    if user_str.is_empty() || user_str == "0" || user_str == "root" {
        return (0, 0);
    }

    // Try "uid:gid" format
    if let Some((uid_s, gid_s)) = user_str.split_once(':') {
        let uid = uid_s.parse().unwrap_or(0);
        let gid = gid_s.parse().unwrap_or(0);
        return (uid, gid);
    }

    // Try numeric uid
    if let Ok(uid) = user_str.parse::<u32>() {
        return (uid, uid);
    }

    // Non-numeric user — would need passwd file lookup in container rootfs.
    // Default to root for now; the guest init will handle resolution.
    log::warn!(
        "non-numeric user {:?} specified; defaulting to root (guest init will resolve)",
        user_str
    );
    (0, 0)
}

/// Boot configuration read by the guest init process.
fn generate_init_config(debug: bool, is_wizer: bool) -> Result<serde_json::Value> {
    let debug_str = if debug { "true" } else { "false" };

    // Pre-cache binaries into memory for faster startup
    let cmd_pre_run: Vec<Vec<String>> = vec![vec![
        "vmtouch".to_string(),
        "-tv".to_string(),
        "/sbin/runc".to_string(),
        "/sbin/init".to_string(),
    ]];

    let config = json!({
        "cmd": [
            "/sbin/runc",
            "run",
            "-b", "/run/bundle",
            "foo"
        ],
        "cmd_pre_run": cmd_pre_run,
        "debug": debug_str == "true",
        "debug_init": is_wizer,
        "container": {
            "bundle_path": "/run/bundle",
            "image_config_path": "/oci/image.json",
            "runtime_config_path": "/oci/spec.json",
            "image_rootfs_path": "/oci/rootfs"
        },
        "mounts": init_mounts()
    });

    Ok(config)
}

fn init_mounts() -> Vec<serde_json::Value> {
    vec![
        json!({
            "source": "proc",
            "destination": "/proc",
            "type": "proc",
            "options": []
        }),
        json!({
            "source": "tmpfs",
            "destination": "/run",
            "type": "tmpfs",
            "options": []
        }),
        json!({
            "source": "tmpfs",
            "destination": "/sys",
            "type": "tmpfs",
            "options": []
        }),
        json!({
            "source": "tmpfs",
            "destination": "/tmp",
            "type": "tmpfs",
            "options": []
        }),
        json!({
            "source": "tmpfs",
            "destination": "/var",
            "type": "tmpfs",
            "options": []
        }),
        json!({
            "source": "tmpfs",
            "destination": "/mnt",
            "type": "tmpfs",
            "options": []
        }),
        json!({
            "source": "tmpfs",
            "destination": "/etc",
            "type": "tmpfs",
            "options": []
        }),
        json!({
            "source": "cgroup2",
            "destination": "/sys/fs/cgroup",
            "type": "cgroup2",
            "options": []
        }),
        // Overlay mount for the container rootfs
        json!({
            "source": "overlay",
            "destination": "/run/rootfs",
            "type": "overlay",
            "options": [
                "lowerdir=/oci/rootfs",
                "upperdir=/run/rootfs-upper",
                "workdir=/run/rootfs-work"
            ]
        }),
    ]
}
