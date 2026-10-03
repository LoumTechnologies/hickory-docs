//! `hick open` — the CLI opening the app, the way `code .` does.
//!
//! The desktop installer bundles the CLI; a CLI-only archive is also shipped.
//! Launch the copy enclosing or beside this CLI before searching other installs.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

/// Name the app's executable answers to on `PATH` and beside `hick`.
const EXE: &str = if cfg!(windows) {
    "hickory-desktop.exe"
} else {
    "hickory-desktop"
};

/// What macOS calls the bundle, from `tauri.conf.json`'s `productName`.
const MAC_APP: &str = "Hickory Docs";

/// Development overrides describe the app process `just dev` started. A
/// second process launched to open an explicit folder must not inherit them:
/// that would replace its path with the seed project, compete for the dev
/// port, and point its UI proxy at the first process's engine.
const SESSION_ENV: &[&str] = &[
    "HICKORY_PROJECT_DIR",
    "HICKORY_SERVE_PORT",
    "HICKORY_UI_ORIGIN",
    "HICKORY_API_ORIGIN",
];

/// Where the app was found, so the failure can say which route was taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum App {
    /// A plain executable to run with the path as its argument.
    Exe(PathBuf),
    /// A macOS bundle, launched through `open -n -a`.
    Bundle(PathBuf),
}

/// Find the desktop app.
///
/// In order, and the order is the point: an explicit variable wins, then the
/// copy sitting beside this `hick` — which is the one a single install put
/// there, and the one a developer just built — and only then the platform's
/// conventional locations. Preferring `PATH` first would launch a different
/// build than the `hick` being run, which is the kind of mismatch nobody
/// thinks to check.
pub fn find(
    lookup: impl Fn(&str) -> Option<String>,
    exists: impl Fn(&Path) -> bool,
) -> Option<App> {
    if let Some(named) = lookup("HICKORY_DESKTOP").filter(|v| !v.trim().is_empty()) {
        let path = PathBuf::from(named.trim());
        // An explicit path is honoured even if it does not exist: the refusal
        // then names what was asked for, which is more useful than silently
        // falling through to a different app than the one that was named.
        return Some(if path.extension().is_some_and(|e| e == "app") {
            App::Bundle(path)
        } else {
            App::Exe(path)
        });
    }

    if let Ok(me) = std::env::current_exe().map(|me| me.canonicalize().unwrap_or(me))
        && let Some(dir) = me.parent()
    {
        // Already inside the bundle: `…/Hickory Docs.app/Contents/MacOS/x`.
        // Answered as the bundle rather than as the executable beside us,
        // because launching a Mach-O out of a bundle directly bypasses
        // LaunchServices — which is how you get a second copy with no dock
        // icon that never comes to the front. This is the case when the app
        // asks for a window of its own.
        if let Some(bundle) = enclosing_bundle(&me) {
            return Some(App::Bundle(bundle));
        }
        let beside = dir.join(EXE);
        if exists(&beside) {
            return Some(App::Exe(beside));
        }
        // A macOS install puts `hick` inside the bundle's Resources; the
        // executable is two levels up in MacOS/.
        let sibling_bundle = dir.join(format!("{MAC_APP}.app"));
        if exists(&sibling_bundle) {
            return Some(App::Bundle(sibling_bundle));
        }
    }

    if cfg!(target_os = "macos") {
        for base in ["/Applications", "~/Applications"] {
            let base = if let Some(rest) = base.strip_prefix("~/") {
                match lookup("HOME") {
                    Some(home) => PathBuf::from(home).join(rest),
                    None => continue,
                }
            } else {
                PathBuf::from(base)
            };
            let bundle = base.join(format!("{MAC_APP}.app"));
            if exists(&bundle) {
                return Some(App::Bundle(bundle));
            }
        }
    }

    if cfg!(windows) {
        for (variable, relative) in [
            ("ProgramFiles", "Hickory Docs"),
            ("LOCALAPPDATA", "Programs/Hickory Docs"),
        ] {
            if let Some(base) = lookup(variable) {
                let exe = PathBuf::from(base).join(relative).join(EXE);
                if exists(&exe) {
                    return Some(App::Exe(exe));
                }
            }
        }
    }

    // Last: whatever `PATH` has.
    let path = lookup("PATH")?;
    let sep = if cfg!(windows) { ';' } else { ':' };
    path.split(sep)
        .filter(|p| !p.is_empty())
        .map(|dir| Path::new(dir).join(EXE))
        .find(|candidate| exists(candidate))
        .map(App::Exe)
}

/// The `.app` this executable is inside, if it is inside one.
///
/// `…/Hickory Docs.app/Contents/MacOS/hickory-desktop` -> `…/Hickory Docs.app`.
/// Purely a path shape, so it is testable everywhere and simply never matches
/// off macOS.
fn enclosing_bundle(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    if !matches!(macos.file_name()?.to_str()?, "MacOS" | "Resources") {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()? != "Contents" {
        return None;
    }
    let bundle = contents.parent()?;
    (bundle.extension()? == "app").then(|| bundle.to_path_buf())
}

/// The message for a machine that has the CLI and not the app.
///
/// It leads with the fact that they are separate downloads, because "the app
/// is not installed" is surprising to somebody who installed *Hickory Docs*
/// and reasonably assumed that was one thing.
pub fn not_installed() -> String {
    format!(
        "Hickory Docs desktop is not installed. This installation has only the CLI.\n  \
         Install the desktop app from https://hickorydocs.com, or set \
         HICKORY_DESKTOP=/path/to/{EXE} to use a copy already on this machine.\n  \
         CLI commands still work: `hick --help` lists them; `hick up <folder>` \
         runs the engine with your own editor."
    )
}

/// Open `target` in the app, and return without waiting for it.
///
/// Detached on purpose: a terminal that blocks until the editor is closed is
/// `git commit` behaviour, not `code .` behaviour, and the request here was
/// explicitly the latter.
pub fn open(app: &App, target: &Path) -> Result<()> {
    // Absolute, because the app is launched detached and inherits nothing
    // useful — an app started from a dock or Finder gets `/` as its working
    // directory, so a relative path would resolve somewhere nobody meant.
    let target = std::fs::canonicalize(target)
        .with_context(|| format!("{} does not exist", target.display()))?;

    let mut command = match app {
        App::Bundle(bundle) => {
            let mut c = std::process::Command::new("open");
            // `-n` is load-bearing, not caution: without it `open -a` hands
            // the arguments to a copy that is already running, which here
            // would be a process holding a *different* folder's directory
            // lock and watcher. A session is a process in this product, so
            // "open that folder" always means a new one.
            c.arg("-n").arg("-a").arg(bundle).arg("--args").arg(&target);
            c
        }
        App::Exe(exe) => {
            if !exe.exists() {
                bail!(
                    "HICKORY_DESKTOP names {}, which is not there.\n  \
                     Point it at the app's executable, or unset it to let \
                     `hick open` look in the usual places.",
                    exe.display()
                );
            }
            let mut c = std::process::Command::new(exe);
            c.arg(&target);
            c
        }
    };
    // `target` is this launcher's explicit instruction. In particular, File →
    // Open Folder from a blank window must win over `just dev`'s inherited
    // `.dev/project` setting.
    for name in SESSION_ENV {
        command.env_remove(name);
    }
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    let mut child = command
        .spawn()
        .with_context(|| format!("could not start the desktop app for {}", target.display()))?;

    // `open -a` returns immediately once the app is launched, so it is the one
    // case worth reaping — leaving it would make a zombie of a process that
    // has already done its job.
    if matches!(app, App::Bundle(_)) {
        let status = child
            .wait()
            .context("waiting for macOS to launch Hickory Docs")?;
        if !status.success() {
            bail!(
                "macOS could not launch Hickory Docs ({status}). Open the app from Applications to check the installation."
            );
        }
    }
    Ok(())
}

/// Start a second app process with no workspace open.
///
/// This intentionally takes no path: a blank window must not be smuggled into
/// existence by creating a default folder or by reopening the last one.
pub fn open_blank(app: &App) -> Result<()> {
    let mut command = match app {
        App::Bundle(bundle) => {
            let mut c = std::process::Command::new("open");
            c.arg("-n")
                .arg("-a")
                .arg(bundle)
                .arg("--args")
                .arg("--blank-window");
            c
        }
        App::Exe(exe) => {
            if !exe.exists() {
                bail!(
                    "HICKORY_DESKTOP names {}, which is not there.\n  \
                     Point it at the app's executable, or unset it to let \
                     Hickory Docs find the installed app.",
                    exe.display()
                );
            }
            let mut c = std::process::Command::new(exe);
            c.arg("--blank-window");
            c
        }
    };
    for name in SESSION_ENV {
        command.env_remove(name);
    }
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child = command
        .spawn()
        .context("could not start a blank Hickory Docs window")?;
    if matches!(app, App::Bundle(_)) {
        let status = child
            .wait()
            .context("waiting for macOS to launch Hickory Docs")?;
        if !status.success() {
            bail!(
                "macOS could not launch Hickory Docs ({status}). Open the app from Applications to check the installation."
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn an_executable_inside_a_bundle_is_answered_as_the_bundle() {
        // Launching the Mach-O directly bypasses LaunchServices, which is how
        // a second copy ends up with no dock icon and never comes forward.
        // The app asks for a window of its own by this route, so the shape
        // has to be recognised — and it is only a path shape, which is why it
        // is testable on a machine that has never seen a bundle.
        assert_eq!(
            enclosing_bundle(Path::new(
                "/Applications/Hickory Docs.app/Contents/MacOS/hickory-desktop"
            )),
            Some(PathBuf::from("/Applications/Hickory Docs.app"))
        );
        assert_eq!(
            enclosing_bundle(Path::new("/usr/local/bin/hickory-desktop")),
            None
        );
        // The shape has to be the whole shape: MacOS/, under Contents/, under
        // something ending `.app`.
        assert_eq!(
            enclosing_bundle(Path::new("/x/Thing.app/MacOS/hickory-desktop")),
            None
        );
        assert_eq!(
            enclosing_bundle(Path::new("/x/Thing/Contents/MacOS/hickory-desktop")),
            None
        );
    }

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k: &str| map.get(k).cloned()
    }

    #[test]
    fn an_explicit_variable_wins_and_is_honoured_even_when_wrong() {
        // Falling through to a different app than the one somebody named is
        // how you debug the wrong binary for an hour.
        let found = find(
            env(&[("HICKORY_DESKTOP", "/opt/nope/hickory-desktop")]),
            |_| false,
        );
        assert_eq!(
            found,
            Some(App::Exe(PathBuf::from("/opt/nope/hickory-desktop")))
        );
    }

    #[test]
    fn a_dot_app_named_explicitly_is_treated_as_a_bundle() {
        let found = find(
            env(&[("HICKORY_DESKTOP", "/Apps/Hickory Docs.app")]),
            |_| true,
        );
        assert_eq!(
            found,
            Some(App::Bundle(PathBuf::from("/Apps/Hickory Docs.app")))
        );
    }

    #[test]
    fn path_is_searched_last_and_entry_order_is_respected() {
        let sep = if cfg!(windows) { ";" } else { ":" };
        let first = PathBuf::from("/usr/local/bin").join(EXE);
        let wanted = first.clone();
        let found = find(
            env(&[("PATH", &format!("/usr/local/bin{sep}/usr/bin"))]),
            move |p| p == wanted,
        );
        assert_eq!(found, Some(App::Exe(first)));
    }

    #[test]
    fn nothing_anywhere_is_none_rather_than_a_guess() {
        assert_eq!(find(env(&[("PATH", "/nowhere")]), |_| false), None);
        assert_eq!(find(env(&[]), |_| false), None);
    }

    #[test]
    fn the_refusal_explains_a_cli_only_install_and_names_the_alternative() {
        // Somebody who installed "Hickory Docs" reasonably assumed that was
        // one thing, so "not installed" needs the reason attached.
        let text = not_installed();
        assert!(text.contains("only the CLI"), "{text}");
        assert!(text.contains("HICKORY_DESKTOP"), "{text}");
        // And `hick up` is named as the pairing, never as a consolation.
        assert!(text.contains("hick up"), "{text}");
        assert!(text.contains("hick --help"), "{text}");
    }
}
