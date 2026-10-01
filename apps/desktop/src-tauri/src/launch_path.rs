//! Recover the shell's executable search path, once, before desktop startup.
//! Only PATH is imported: profile output and credentials are not environment.
use std::os::unix::ffi::OsStringExt;
use std::{
    ffi::OsString,
    io::{Read, Seek},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const MARKER: &[u8] = b"\0HICKORY_PATH\0";
const LIMIT: u64 = 64 * 1024;

pub fn load() -> Result<OsString, String> {
    let shell = std::env::var_os("SHELL").unwrap_or_else(|| "/bin/zsh".into());
    let home = std::env::var_os("HOME").ok_or("No home directory for shell PATH discovery")?;
    let mut command = shell_command(Path::new(&shell), Path::new(&home));
    let recovered = capture(&mut command, Duration::from_secs(5))?;
    merge(&recovered, std::env::var_os("PATH").as_deref())
}

fn shell_command(shell: &Path, home: &Path) -> Command {
    let mut command = Command::new(shell);
    // Interactive as well as login: people put PATH in .zshrc/.bashrc too.
    // Fish's PATH is a list rather than a colon-separated scalar.
    let script = if shell.file_name().is_some_and(|name| name == "fish") {
        "printf '\\0HICKORY_PATH\\0%s\\0' (string join : $PATH)"
    } else {
        "printf '\\0HICKORY_PATH\\0%s\\0' \"$PATH\""
    };
    command.args(["-ilc", script]).current_dir(home);
    command
}

fn capture(command: &mut Command, timeout: Duration) -> Result<OsString, String> {
    // A file avoids pipe-buffer deadlocks on noisy profiles and lets us bound
    // both waiting and reading without starting threads before set_var.
    let mut stdout = tempfile::tempfile().map_err(|e| e.to_string())?;
    let mut child = command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(stdout.try_clone().map_err(|e| e.to_string())?)
        .spawn()
        .map_err(|e| format!("Could not read shell PATH: {e}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) => return Err("Shell PATH discovery failed".into()),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(e) => e.to_string(),
                    _ => "Shell PATH discovery timed out".into(),
                });
            }
        }
    }
    stdout.rewind().map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    stdout
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("Shell PATH discovery produced too much output".into());
    }
    parse(&bytes).ok_or_else(|| "Shell did not return a valid PATH".into())
}

fn parse(bytes: &[u8]) -> Option<OsString> {
    let start = bytes
        .windows(MARKER.len())
        .rposition(|part| part == MARKER)?
        + MARKER.len();
    let value = bytes[start..].split(|byte| *byte == 0).next()?;
    // Require the terminator; a truncated profile is not a PATH.
    if value.is_empty() || bytes.get(start + value.len()) != Some(&0) {
        return None;
    }
    Some(OsString::from_vec(value.to_vec()))
}

fn merge(
    recovered: &std::ffi::OsStr,
    inherited: Option<&std::ffi::OsStr>,
) -> Result<OsString, String> {
    let mut dirs = Vec::new();
    for path in [Some(recovered), inherited].into_iter().flatten() {
        for dir in std::env::split_paths(path) {
            // Never add a workspace-relative executable directory to a GUI
            // process. Preserve shell order and append inherited-only entries.
            if dir.is_absolute() && !dirs.contains(&dir) {
                dirs.push(dir);
            }
        }
    }
    if dirs.is_empty() {
        return Err("Shell PATH contains no absolute directories".into());
    }
    std::env::join_paths(dirs).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    // Protects docs/guarantees/integrations/the-desktop-finds-tools-from-the-shell-path.md
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn shell_startup_path_can_launch_gh_with_a_gui_environment() {
        let home = tempfile::tempdir().unwrap();
        let bin = home.path().join("tools with spaces");
        std::fs::create_dir(&bin).unwrap();
        let gh = bin.join("gh");
        std::fs::write(&gh, "#!/bin/sh\nprintf 'fixture gh'\n").unwrap();
        std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
        // .zshrc is only read in interactive shells; banner output must not
        // become part of the path. This is a real shell, not a provider mock.
        std::fs::write(
            home.path().join(".zshrc"),
            format!(
                "printf 'profile banner\\n'\nexport PATH='{}':$PATH\n",
                bin.display()
            ),
        )
        .unwrap();
        let mut command = shell_command(Path::new("/bin/zsh"), home.path());
        command
            .env("ZDOTDIR", home.path())
            .env("HOME", home.path())
            .env("PATH", "/usr/bin:/bin");
        let path = capture(&mut command, Duration::from_secs(5)).unwrap();
        let path = merge(&path, Some(std::ffi::OsStr::new("/usr/bin:/bin"))).unwrap();
        let output = Command::new("gh")
            .env("PATH", path)
            .current_dir(home.path())
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"fixture gh");
    }

    #[test]
    fn path_order_is_preserved_and_relative_entries_are_excluded() {
        let path = merge(
            std::ffi::OsStr::new("/custom:/bin:.:relative:/custom"),
            Some(std::ffi::OsStr::new("/bin:/inherited")),
        )
        .unwrap();
        assert_eq!(path, "/custom:/bin:/inherited");
    }

    #[test]
    fn framing_preserves_non_utf8_and_ignores_profile_noise() {
        assert_eq!(
            parse(b"banner\n\0HICKORY_PATH\0/bin:/x\xff\0after"),
            Some(OsString::from_vec(b"/bin:/x\xff".to_vec()))
        );
        for bytes in [
            b"/bin".as_slice(),
            b"\0HICKORY_PATH\0",
            b"\0HICKORY_PATH\0/bin",
            b"\0HICKORY_PATH\0\0",
        ] {
            assert_eq!(parse(bytes), None);
        }
    }

    #[test]
    fn failed_and_hanging_profiles_do_not_block_startup() {
        let mut failure = Command::new("/bin/sh");
        failure.args(["-c", "exit 1"]);
        assert!(capture(&mut failure, Duration::from_secs(1)).is_err());
        let mut hang = Command::new("/bin/sh");
        hang.args(["-c", "exec /bin/sleep 10"]);
        let start = Instant::now();
        assert!(
            capture(&mut hang, Duration::from_millis(50))
                .unwrap_err()
                .contains("timed out")
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
