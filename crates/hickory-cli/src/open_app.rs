//! `hick open` — the CLI opening the app, the way `code .` does.
//!
//! The two halves of this product ship as **separate artifacts**: the archive
//! carries `hick`, the licence and `examples/`, while the desktop app arrives
//! as a `.dmg`, an `AppImage` or an `.msi`
//! (`continuous-delivery-downloadable.md`). So this is a launcher, not a
//! front end — it finds an app that may not be installed, and says so plainly
//! when it is not, rather than reporting "not found" about a binary the user
//! has never heard of.
//!
//! The app already takes a folder or a document as its first argument
//! (`server::named_dir`), and handles either. Nothing new had to be taught to
//! it; what was missing was only a way to say so from a terminal.

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

/// Where the app was found, so the failure can say which route was taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum App {
    /// A plain executable to run with the path as its argument.
    Exe(PathBuf),
    /// A macOS bundle, launched through `open -a`.
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

    if let Ok(me) = std::env::current_exe()
        && let Some(dir) = me.parent()
    {
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

    // Last: whatever `PATH` has.
    let path = lookup("PATH")?;
    let sep = if cfg!(windows) { ';' } else { ':' };
    path.split(sep)
        .filter(|p| !p.is_empty())
        .map(|dir| Path::new(dir).join(EXE))
        .find(|candidate| exists(candidate))
        .map(App::Exe)
}

/// The message for a machine that has the CLI and not the app.
///
/// It leads with the fact that they are separate downloads, because "the app
/// is not installed" is surprising to somebody who installed *Hickory Docs*
/// and reasonably assumed that was one thing.
pub fn not_installed() -> String {
    format!(
        "the desktop app is not on this machine — and that is not the same as \
         a broken install.\n  \
         `hick` and the app ship as separate downloads: the archive you have \
         carries the command line, and the app arrives as a .dmg, an AppImage \
         or an .msi from the releases page.\n  \
         Next steps: install it, or point this at a copy you already have with \
         HICKORY_DESKTOP=/path/to/{EXE}.\n  \
         Without it, `hick up <folder>` gives you the same engine with your own \
         editor over the top — that is the pairing the product is built around, \
         not a fallback."
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
            c.arg("-a").arg(bundle).arg("--args").arg(&target);
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
        let _ = child.wait();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
    fn the_refusal_says_they_are_separate_downloads_and_names_the_alternative() {
        // Somebody who installed "Hickory Docs" reasonably assumed that was
        // one thing, so "not installed" needs the reason attached.
        let text = not_installed();
        assert!(text.contains("separate downloads"), "{text}");
        assert!(text.contains("HICKORY_DESKTOP"), "{text}");
        // And `hick up` is named as the pairing, never as a consolation.
        assert!(text.contains("hick up"), "{text}");
        assert!(text.contains("not a fallback"), "{text}");
    }
}
