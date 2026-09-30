use super::Host;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::PathBuf;

/// The optional frontend must never silently fall back to an unmediated cwd.
pub fn availability() -> Value {
    #[cfg(target_os = "macos")]
    {
        let version = std::process::Command::new("/usr/bin/sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let extension = std::env::current_exe().ok().and_then(|p| {
            p.parent()?
                .parent()
                .map(|p| p.join("PlugIns/HickoryWorkspace.appex"))
        });
        let supported = version
            .split('.')
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .is_some_and(|v| v >= 26);
        json!({"supported":supported,"bundled":extension.is_some_and(|p| p.is_dir()),"message":"Enable Hickory Workspace in System Settings → General → Login Items & Extensions → File System Extensions. No kernel extension or Recovery-mode setup is used."})
    }
    #[cfg(not(target_os = "macos"))]
    json!({"supported":false,"bundled":false,"message":"The native workspace filesystem currently requires macOS 26 or later."})
}
pub struct Mount {
    pub path: PathBuf,
    _resource: tempfile::TempDir,
    _host: Host,
}
impl Mount {
    pub async fn exchange(&self, request: Value) -> Result<Value> {
        let response: Value = reqwest::Client::new()
            .post(&self._host.url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if let Some(error) = response["error"].as_str() {
            anyhow::bail!("{error}");
        }
        Ok(response["result"].clone())
    }
    pub async fn start(host: Host) -> Result<Self> {
        let available = availability();
        ensure!(
            available["supported"] == true,
            "native workspace filesystem requires macOS 26 or later"
        );
        ensure!(
            available["bundled"] == true,
            "this Hickory build does not include its workspace filesystem extension; install a build with Hickory Workspace bundled"
        );
        let resource = tempfile::Builder::new()
            .prefix("hickory-workspace-")
            .tempdir()?;
        let descriptor = resource.path().join("resource");
        std::fs::create_dir(&descriptor)?;
        let descriptor_file = descriptor.join("connection.json");
        super::super::store::write_atomic(
            &descriptor_file,
            serde_json::to_string(&json!({"url":host.url}))?.as_bytes(),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&descriptor_file, std::fs::Permissions::from_mode(0o600))?;
            std::fs::set_permissions(resource.path(), std::fs::Permissions::from_mode(0o700))?;
        }
        // Apple's FSPathURLResource mount interface grants the extension access
        // to this descriptor, not to the real repository.
        // A mountpoint must never be beneath TempDir: recursive cleanup after
        // a failed unmount could otherwise traverse the mounted workspace.
        let path = resource.path().with_extension("mount");
        std::fs::create_dir(&path)?;
        let mut command = tokio::process::Command::new("/sbin/mount");
        command.kill_on_drop(true);
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            command
                .args(["-t", "hickory"])
                .arg(&descriptor)
                .arg(&path)
                .output(),
        )
        .await
        .context("mounting Hickory Workspace timed out")?
        .context("mounting Hickory Workspace")?;
        ensure!(
            output.status.success(),
            "Could not mount Hickory Workspace: {}. Enable the bundled File System Extension in System Settings, then reconnect the agent.",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(Self {
            path,
            _resource: resource,
            _host: host,
        })
    }
}
impl Drop for Mount {
    fn drop(&mut self) {
        // Unmount before removing the resource or stopping the private host.
        // No force-unmount: pending rejected saves must remain visible as errors.
        if let Ok(mut child) = std::process::Command::new("/sbin/umount")
            .arg(&self.path)
            .spawn()
        {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            let _ = std::fs::remove_dir(&self.path);
                        }
                        break;
                    }
                    Ok(None) if std::time::Instant::now() < deadline => {
                        std::thread::sleep(std::time::Duration::from_millis(20))
                    }
                    _ => {
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                }
            }
        }
    }
}
