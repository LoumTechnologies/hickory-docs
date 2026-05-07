use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use filetime::FileTime;
use walkdir::WalkDir;

pub fn prepare_source_img(
    builder_path: &str,
    img_name: &str,
    tmpdir: &Path,
    target_arch: &str,
) -> Result<()> {
    log::info!("saving {:?} to {:?}", img_name, tmpdir);

    let mut needs_pull = false;

    match Command::new(builder_path)
        .args(["image", "inspect", img_name])
        .output()
    {
        Ok(output) if output.status.success() => {
            if !target_arch.is_empty() {
                let inspect_data: Vec<serde_json::Value> =
                    serde_json::from_slice(&output.stdout)
                        .context("failed to parse image inspect JSON")?;

                let image_arch = inspect_data
                    .first()
                    .and_then(|v| v.get("Architecture"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if image_arch != target_arch {
                    log::info!(
                        "unexpected architecture {:?} (target: {:?}). \
                         Try \"--target-arch\" when specifying an architecture.",
                        image_arch,
                        target_arch
                    );
                    needs_pull = true;
                }
            }
        }
        _ => {
            needs_pull = true;
        }
    }

    if needs_pull {
        let mut args = vec!["pull".to_string()];
        if !target_arch.is_empty() {
            args.push(format!("--platform=linux/{}", target_arch));
        }
        args.push(img_name.to_string());

        log::info!(
            "cannot get image {:?} locally; pulling it from the registry...",
            img_name
        );

        let status = Command::new(builder_path)
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to execute pull command")?;

        if !status.success() {
            bail!(
                "failed to pull the image. Try \"--target-arch\" when specifying an architecture"
            );
        }
    }

    // docker save | tar extract
    let mut save_cmd = Command::new(builder_path)
        .args(["save", img_name])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start docker save")?;

    let stdout = save_cmd
        .stdout
        .take()
        .context("failed to capture docker save stdout")?;

    let mut archive = tar::Archive::new(stdout);
    archive
        .unpack(tmpdir)
        .context("failed to extract tar archive")?;

    let status = save_cmd.wait().context("failed to wait for docker save")?;
    if !status.success() {
        bail!("docker save failed");
    }

    // Update timestamps so that BuildKit can prioritize them over cached files
    let now = FileTime::now();
    for entry in WalkDir::new(tmpdir) {
        let entry = entry?;
        filetime::set_file_times(entry.path(), now, now)
            .with_context(|| format!("failed to update timestamp for {:?}", entry.path()))?;
    }

    Ok(())
}
