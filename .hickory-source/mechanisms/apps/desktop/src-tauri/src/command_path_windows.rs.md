<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/desktop/src-tauri/src/command_path_windows.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1ae18fc903f6"></a>
## `apps/desktop/src-tauri/src/command_path_windows.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

<a id="f-1ae18fc903f6-SCRIPT"></a>
### `SCRIPT`

<storycode:copy id="f-1ae18fc903f6-SCRIPT">
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

</storycode:copy>
<a id="f-1ae18fc903f6-EnvironmentStatus"></a>
### `EnvironmentStatus`

<storycode:copy id="f-1ae18fc903f6-EnvironmentStatus">
#[derive(Deserialize)]
struct EnvironmentStatus {
    installed: bool,
    can_remove: bool,
}
</storycode:copy>
<a id="f-1ae18fc903f6-environment"></a>
### `environment`

<storycode:copy id="f-1ae18fc903f6-environment">
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

</storycode:copy>
<a id="f-1ae18fc903f6-status"></a>
### `status`

<storycode:copy id="f-1ae18fc903f6-status">
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
</storycode:copy>
<a id="f-1ae18fc903f6-install"></a>
### `install`

<storycode:copy id="f-1ae18fc903f6-install">
pub fn install(installer: &Installer) -> Result<()> {
    require_available(&status(installer)?)?;
    environment(installer, "install")?;
    Ok(())
}
</storycode:copy>
<a id="f-1ae18fc903f6-remove"></a>
### `remove`

<storycode:copy id="f-1ae18fc903f6-remove">
pub fn remove(installer: &Installer) -> Result<()> {
    environment(installer, "remove")?;
    Ok(())
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/src/command_path_windows.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/src/command_path_windows.rs"><storycode:paste select="#f-1ae18fc903f6-SCRIPT"/><storycode:paste select="#f-1ae18fc903f6-EnvironmentStatus"/><storycode:paste select="#f-1ae18fc903f6-environment"/><storycode:paste select="#f-1ae18fc903f6-status"/><storycode:paste select="#f-1ae18fc903f6-install"/><storycode:paste select="#f-1ae18fc903f6-remove"/></storycode:file>

</storycode:doc>
