use anyhow::{Context, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::io::AsyncReadExt;
pub const LIMIT: usize = 1_048_576;

pub fn command(
    binary: &Path,
    scratch: &Path,
    roots: &[PathBuf],
    files: &[PathBuf],
    denied: &[PathBuf],
) -> Result<tokio::process::Command> {
    command_with_helpers(binary, scratch, roots, files, denied, &[])
}

/// Trusted credential utilities are fixed by the adapter, never by model input.
pub fn command_with_helpers(
    binary: &Path,
    scratch: &Path,
    roots: &[PathBuf],
    files: &[PathBuf],
    denied: &[PathBuf],
    helpers: &[PathBuf],
) -> Result<tokio::process::Command> {
    ensure!(
        cfg!(target_os = "macos") && Path::new("/usr/bin/sandbox-exec").is_file(),
        "Subscription inference requires a macOS workspace server with sandbox-exec. Connect to that server and retry; no API fallback was attempted."
    );
    let binary = binary.canonicalize()?;
    let scratch = scratch.canonicalize()?;
    let quote = |path: &Path| -> Result<String> {
        Ok(serde_json::to_string(
            path.to_str()
                .context("Subscription process paths must be UTF-8")?,
        )?)
    };
    let mut reads = String::new();
    for path in [
        Path::new("/usr"),
        Path::new("/System"),
        Path::new("/Library"),
        Path::new("/opt/homebrew/lib"),
        scratch.as_path(),
    ]
    .into_iter()
    .chain(roots.iter().map(PathBuf::as_path))
    {
        reads.push_str(&format!("(require-not (subpath {}))", quote(path)?));
    }
    for path in [
        Path::new("/"),
        Path::new("/dev/null"),
        Path::new("/dev/urandom"),
        Path::new("/dev/random"),
        Path::new("/private/etc/resolv.conf"),
        Path::new("/private/etc/hosts"),
        binary.as_path(),
    ]
    .into_iter()
    .chain(files.iter().map(PathBuf::as_path))
    {
        reads.push_str(&format!("(require-not (literal {}))", quote(path)?));
    }
    let mut writes = format!(
        "(require-not (subpath {}))(require-not (literal \"/dev/null\"))",
        quote(&scratch)?
    );
    for root in roots {
        writes.push_str(&format!("(require-not (subpath {}))", quote(root)?));
    }
    for file in files {
        writes.push_str(&format!("(require-not (literal {}))", quote(file)?));
    }
    let mut executions = format!("(require-not (literal {}))", quote(&binary)?);
    for helper in helpers {
        executions.push_str(&format!(
            "(require-not (literal {}))",
            quote(&helper.canonicalize()?)?
        ));
    }
    let mut profile = format!(
        "(version 1)(allow default)(deny process-exec (require-all {}))(deny file-read-data (require-all {}))(deny file-write* (require-all {}))",
        executions, reads, writes
    );
    for path in denied {
        profile.push_str(&format!("(deny file-read* (subpath {}))", quote(path)?));
    }
    let mut command = tokio::process::Command::new("/usr/bin/sandbox-exec");
    command
        .args(["-p", &profile])
        .arg(binary)
        .current_dir(scratch)
        .kill_on_drop(true);
    Ok(command)
}
struct Running {
    child: tokio::process::Child,
    pid: u32,
}
impl Drop for Running {
    fn drop(&mut self) {
        #[cfg(unix)]
        // SAFETY: capture() started this child in its own process group.
        unsafe {
            libc::kill(-(self.pid as i32), libc::SIGKILL);
        }
        let _ = self.child.start_kill();
    }
}
pub async fn capture(command: tokio::process::Command) -> Result<Vec<u8>> {
    capture_input(command, None).await
}
pub async fn capture_input(
    mut command: tokio::process::Command,
    input: Option<Vec<u8>>,
) -> Result<Vec<u8>> {
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("The native CLI could not start. Install/update it and retry.")?;
    let stdin = child.stdin.take();
    let stdout = child
        .stdout
        .take()
        .context("Native CLI stdout is missing")?;
    let mut running = Running {
        pid: child.id().context("Native CLI process id missing")?,
        child,
    };
    let mut bytes = Vec::new();
    let write = async move {
        use tokio::io::AsyncWriteExt;
        if let (Some(mut stdin), Some(input)) = (stdin, input) {
            stdin.write_all(&input).await?;
            stdin.shutdown().await?;
        }
        Ok::<_, std::io::Error>(())
    };
    let read = async {
        stdout
            .take((LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .await
    };
    tokio::try_join!(write, read)?;
    ensure!(
        bytes.len() <= LIMIT,
        "Native CLI output exceeded its safety limit. No model action was admitted."
    );
    ensure!(
        running.child.wait().await?.success(),
        "Native CLI request failed. Check sign-in, connection and usage limits, then retry. No API billing fallback was attempted."
    );
    Ok(bytes)
}
