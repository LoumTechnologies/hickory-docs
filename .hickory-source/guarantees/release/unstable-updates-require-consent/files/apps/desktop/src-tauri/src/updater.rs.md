<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/desktop/src-tauri/src/updater.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Unstable Updates Require Consent](../../../../../contract.md)

Captured SHA-256: `13c161d8e53ffe38ba5021c50e8eebd357bbd554239c78ad4e9ec449a48b5979`.

<book:entry path="apps/desktop/src-tauri/src/updater.rs">
//! Only packaged Unstable Apple Silicon builds opt into network update checks.

fn is_unstable(version: &str) -> bool {
    version
        .split('+')
        .next()
        .unwrap_or_default()
        .split_once('-')
        .is_some_and(|(_, pre)| pre.split('.').next() == Some("unstable"))
}

#[cfg(any(test, all(target_os = "macos", target_arch = "aarch64")))]
fn should_offer(current: &semver::Version, remote: &semver::Version) -> bool {
    is_unstable(&remote.to_string()) && remote > current
}

pub(super) fn start(app: tauri::AppHandle) {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    if !cfg!(debug_assertions) && is_unstable(&app.package_info().version.to_string()) {
        macos::start(app);
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let _ = (app, is_unstable as fn(&str) -> bool);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod macos {
    use fs2::FileExt;
    use std::{fs::OpenOptions, time::Duration};
    use tauri::{AppHandle, Manager};
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    use tauri_plugin_updater::UpdaterExt;

    pub(super) fn start(app: AppHandle) {
        if let Err(error) = app.plugin(tauri_plugin_updater::Builder::new().build()) {
            log::warn!("Could not initialize Unstable updater: {error}");
            return;
        }
        // Native blocking dialogs belong on a worker, never the event loop.
        std::thread::spawn(move || {
            // Startup may still be opening the local server and editor window.
            while app.get_webview_window("main").is_none() {
                std::thread::sleep(Duration::from_secs(1));
            }
            let mut offered = String::new();
            loop {
                if let Err(error) = check(&app, &mut offered) {
                    // Offline startup and a not-yet-published channel are nonfatal.
                    log::warn!("Unstable update check: {error:#}");
                }
                std::thread::sleep(Duration::from_secs(30 * 60));
            }
        });
    }

    fn check(app: &AppHandle, offered: &mut String) -> anyhow::Result<()> {
        let config = app.path().app_config_dir()?;
        std::fs::create_dir_all(&config)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(config.join("unstable-update.lock"))?;
        if lock.try_lock_exclusive().is_err() {
            return Ok(()); // Another folder's process is offering/installing it.
        }
        let updater = app
            .updater_builder()
            .timeout(Duration::from_secs(120))
            .version_comparator(|current, remote| super::should_offer(&current, &remote.version))
            .build()?;
        let Some(mut update) = tauri::async_runtime::block_on(updater.check())? else {
            return Ok(());
        };
        // Avoid repeated prompts, including from another open folder process.
        let installed = config.join("unstable-installed-version");
        if update.version == *offered
            || std::fs::read_to_string(&installed).ok().as_deref() == Some(update.version.as_str())
        {
            return Ok(());
        }
        *offered = update.version.clone();
        let consent = app.dialog().message(format!(
            "Unstable {} is available. Download and install it now?\n\nYou can keep working. The update takes effect after you save your work, quit all Hickory Docs windows, and reopen the app.", update.version
        )).title("Hickory Docs update")
            .buttons(MessageDialogButtons::OkCancelCustom("Download and install".into(), "Later".into()))
            .blocking_show();
        if !consent {
            return Ok(());
        }
        // The bundled Python runtime can make an update a large download.
        update.timeout = Some(Duration::from_secs(15 * 60));
        let result = tauri::async_runtime::block_on(update.download_and_install(|_, _| {}, || {}));
        match result {
            Ok(()) => {
                // Installation replaces the bundle on disk; never bypass the editor's
                // save/discard or workspace detach gates by forcing a process restart.
                if let Err(error) = std::fs::write(installed, &update.version) {
                    log::warn!("Could not record installed update: {error}");
                }
                app.dialog().message("Update installed. Save your work, quit all Hickory Docs windows, then reopen the app to use it.")
                    .title("Hickory Docs update").blocking_show();
            }
            Err(error) => {
                offered.clear(); // Allow a retry at the next check.
                app.dialog().message(format!("The update could not be installed. You can keep using this version.\n\n{error}"))
                    .title("Hickory Docs update").kind(MessageDialogKind::Error).blocking_show();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::is_unstable;
    // Guarantee: docs/guarantees/release/unstable-updates-require-consent.md
    #[test]
    fn newer_timestamp_updates_but_duplicates_downgrades_and_other_channels_do_not() {
        let current =
            semver::Version::parse("0.1.0-unstable.20261025123456789012.gffffffff").unwrap();
        let newer =
            semver::Version::parse("0.1.0-unstable.20261025123456789013.g00000000").unwrap();
        assert!(super::should_offer(&current, &newer));
        assert!(!super::should_offer(&current, &current));
        assert!(!super::should_offer(&newer, &current));
        for other in ["0.1.0", "0.2.0-beta.1", "0.2.0-unstableish.1"] {
            assert!(!super::should_offer(
                &current,
                &semver::Version::parse(other).unwrap()
            ));
        }
    }

    // Guarantee: docs/guarantees/release/unstable-updates-require-consent.md
    #[test]
    fn only_the_explicit_unstable_channel_opts_in() {
        assert!(is_unstable("0.1.0-unstable.20261025123456789012.abcdef01"));
        assert!(is_unstable("0.1.0-unstable.abcdef01"));
        for version in [
            "0.1.0",
            "0.1.0-beta.1",
            "0.1.0+unstable",
            "0.1.0+build-unstable",
            "0.1.0-unstableish.1",
        ] {
            assert!(!is_unstable(version), "{version}");
        }
    }
}
</book:entry>

</book:doc>
