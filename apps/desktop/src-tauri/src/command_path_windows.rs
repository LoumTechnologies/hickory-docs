use super::*;
use std::os::windows::process::CommandExt;

// Environment variables carry paths, never interpolated PowerShell source.
const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$directory = $env:HICKORY_COMMAND_DIR
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @($userPath -split ';' | Where-Object { $_ -ne '' })
$has = @($entries | Where-Object { $_.TrimEnd('\') -ieq $directory.TrimEnd('\') }).Count -gt 0
$system = [Environment]::GetEnvironmentVariable('Path', 'Machine')
$systemHas = @($system -split ';' | Where-Object { $_.TrimEnd('\') -ieq $directory.TrimEnd('\') }).Count -gt 0
$changed = $false
if ($env:HICKORY_COMMAND_ACTION -eq 'install' -and !$has -and !$systemHas) {
    $entries += $directory
    $changed = $true
}
if ($env:HICKORY_COMMAND_ACTION -eq 'remove' -and $has) {
    $entries = @($entries | Where-Object { $_.TrimEnd('\') -ine $directory.TrimEnd('\') })
    $changed = $true
}
if ($changed) {
    [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class HickoryEnvironment { [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] public static extern IntPtr SendMessageTimeout(IntPtr h, uint m, UIntPtr w, string l, uint f, uint t, out UIntPtr r); }'
    $result = [UIntPtr]::Zero
    [void][HickoryEnvironment]::SendMessageTimeout([IntPtr]0xffff, 0x1a, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result)
}
$has = @($entries | Where-Object { $_.TrimEnd('\') -ieq $directory.TrimEnd('\') }).Count -gt 0
@{ installed = ($has -or $systemHas); can_remove = $has } | ConvertTo-Json -Compress
"#;

#[derive(Deserialize)]
struct EnvironmentStatus {
    installed: bool,
    can_remove: bool,
}
fn environment(installer: &Installer, action: &str) -> Result<EnvironmentStatus> {
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            SCRIPT,
        ])
        .env(
            "HICKORY_COMMAND_DIR",
            installer.source.parent().context("No command directory")?,
        )
        .env("HICKORY_COMMAND_ACTION", action)
        .creation_flags(0x08000000)
        .output()
        .context("Cannot run Windows PATH setup; check that PowerShell is available")?;
    if !output.status.success() {
        bail!(
            "Windows could not update your user PATH: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

pub fn status(installer: &Installer) -> Result<Status> {
    let state = environment(installer, "status")?;
    Ok(Status {
        available: installer.source.is_file(),
        installed: state.installed,
        can_remove: state.can_remove,
        command: installer.source.clone(),
        source: installer.source.clone(),
        conflict: installer.conflict(&installer.source),
        message: if state.installed {
            "Installed. Open a new terminal to use hick."
        } else {
            "Add the bundled hick command to your user PATH."
        }
        .into(),
    })
}
pub fn install(installer: &Installer) -> Result<()> {
    require_available(&status(installer)?)?;
    environment(installer, "install")?;
    Ok(())
}
pub fn remove(installer: &Installer) -> Result<()> {
    environment(installer, "remove")?;
    Ok(())
}
