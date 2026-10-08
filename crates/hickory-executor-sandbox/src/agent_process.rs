//! A separate allowlist for trusted Python orchestration, not the cell profile.
//! Workspace and provider access happen only through the parent process's pipe.
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::policy::Sandbox;

fn quoted(path: &Path) -> Result<String> {
    let path = path.to_str().context("Agent runtime paths must be UTF-8")?;
    Ok(format!(
        "\"{}\"",
        path.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

/// Resolve an installed venv, not an editable checkout or a shell command.
pub fn runtime_roots(python: &Path) -> Result<Vec<PathBuf>> {
    ensure!(
        python.is_absolute(),
        "Pass an absolute path to a venv Python"
    );
    let bin = python.parent().context("Python has no parent")?;
    let venv = bin
        .parent()
        .context("Python has no environment directory")?;
    ensure!(
        venv.join("pyvenv.cfg").is_file(),
        "Python must belong to an installed virtual environment"
    );
    ensure!(
        bin.join("monty").is_file(),
        "Install the locked Hickory agent environment first (Monty is missing)"
    );
    let real_python = python.canonicalize()?;
    let base = real_python
        .parent()
        .and_then(Path::parent)
        .context("Python has no runtime root")?;
    let roots = vec![venv.canonicalize()?, base.to_path_buf()];
    for root in &roots {
        ensure!(
            root != Path::new("/") && root != Path::new("/Users") && root != Path::new("/home"),
            "Runtime read root is too broad"
        );
    }
    Ok(roots)
}

/// No shell, inherited environment, inherited cwd, network grant or fallback.
pub fn command(python: &Path, scratch: &Path) -> Result<Command> {
    let roots = runtime_roots(python)?;
    if let Some(home) = std::env::var_os("HOME") {
        for root in &roots {
            ensure!(
                !Path::new(&home).starts_with(root),
                "Runtime root would expose the user's home"
            );
        }
    }
    let scratch = scratch.canonicalize()?;
    let args = ["-I", "-B", "-m", "hickory_agent_worker"];
    let mut command = match Sandbox::detect() {
        Sandbox::Seatbelt => {
            let mut profile = String::from(
                "(version 1)(deny default)(allow process*)(allow signal (target self))\
                 (allow sysctl-read)(allow file-read-metadata)\
                 (allow file-read* (subpath \"/usr\") (subpath \"/System/Library\")\
                 (subpath \"/System/Cryptexes/OS\") (subpath \"/System/Volumes/Preboot/Cryptexes/OS\")\
                 (subpath \"/Library/Apple\")\
                 (literal \"/\") (literal \"/dev/null\") (literal \"/dev/urandom\") (literal \"/dev/random\"))\
                 (allow file-write* (literal \"/dev/null\"))",
            );
            for root in roots {
                profile.push_str(&format!("(allow file-read* (subpath {}))", quoted(&root)?));
            }
            profile.push_str(&format!(
                "(allow file-read* file-write* (subpath {}))",
                quoted(&scratch)?
            ));
            let mut cmd = Command::new("/usr/bin/sandbox-exec");
            cmd.args(["-p", &profile]).arg(python).args(args);
            cmd
        }
        Sandbox::Bubblewrap => {
            let mut cmd = Command::new("bwrap");
            cmd.args([
                "--die-with-parent",
                "--new-session",
                "--unshare-all",
                "--clearenv",
                "--proc",
                "/proc",
                "--dev",
                "/dev",
                "--tmpfs",
                "/tmp",
            ]);
            for path in ["/usr", "/lib", "/lib64", "/bin", "/etc/ld.so.cache"] {
                if Path::new(path).exists() {
                    cmd.arg("--ro-bind").arg(path).arg(path);
                }
            }
            for root in roots {
                cmd.arg("--ro-bind").arg(&root).arg(&root);
            }
            cmd.arg("--bind")
                .arg(&scratch)
                .arg(&scratch)
                .arg("--chdir")
                .arg(&scratch)
                .args(["--setenv", "LANG", "C.UTF-8", "--setenv", "HOME"])
                .arg(&scratch)
                .args(["--setenv", "TMPDIR"])
                .arg(&scratch)
                .arg("--")
                .arg(python)
                .args(args);
            cmd
        }
        Sandbox::AppContainer => bail!(
            "The Python agent process profile is not implemented on Windows; refusing unconfined execution"
        ),
        Sandbox::None => {
            bail!("No working OS sandbox for the Python agent; refusing unconfined execution")
        }
    };
    command
        .env_clear()
        .env("LANG", "C.UTF-8")
        .env("HOME", &scratch)
        .env("TMPDIR", &scratch)
        .current_dir(&scratch);
    // Descendants inherit confinement. The supervisor kills this process group
    // on every exit, including protocol errors and timeout, not just Python.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
    #[test]
    fn a_general_python_or_relative_command_is_not_a_runtime() {
        assert!(runtime_roots(Path::new("python3")).is_err());
        assert!(runtime_roots(Path::new("/usr/bin/python3")).is_err());
    }
    #[test]
    fn profile_paths_cannot_inject_rules() {
        assert_eq!(
            quoted(Path::new("/tmp/a\"b\\c")).unwrap(),
            "\"/tmp/a\\\"b\\\\c\""
        );
    }
}
