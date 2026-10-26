<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The App Installs Its Command For The User

For: Maintainers publishing installers and people checking what a download contains.

Given a packaged desktop app, then its bundle includes the real `hick` CLI.
When the user installs the command through Settings → Command line, then it
becomes available on the user's PATH in a new terminal, without requiring a
system PATH change. The command palette provides a route to those settings.
An existing conflicting command is identified and is never overwritten.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/the-app-installs-its-command-for-the-user.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-78310926e7db"></a>
## `apps/desktop/src-tauri/src/command_path.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [The App Installs Its Command For The User](contract.md).

<a id="f-78310926e7db-platform"></a>
### `platform`

User-level installation of the bundled CLI. No arbitrary paths over HTTP.

<storycode:copy id="f-78310926e7db-platform">
//! User-level installation of the bundled CLI. No arbitrary paths over HTTP.
use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[cfg(unix)]
#[path = "command_path_unix.rs"]
mod platform;
</storycode:copy>
<a id="f-78310926e7db-platform-2"></a>
### `platform`

<storycode:copy id="f-78310926e7db-platform-2">
#[cfg(windows)]
#[path = "command_path_windows.rs"]
mod platform;

</storycode:copy>
<a id="f-78310926e7db-Installer"></a>
### `Installer`

<storycode:copy id="f-78310926e7db-Installer">
#[derive(Clone)]
pub struct Installer {
    pub source: PathBuf,
    #[cfg(unix)]
    pub home: PathBuf,
    #[cfg(unix)]
    pub shell: String,
    #[cfg(unix)]
    pub appimage: Option<PathBuf>,
    pub search_path: std::ffi::OsString,
}

</storycode:copy>
<a id="f-78310926e7db-Status"></a>
### `Status`

<storycode:copy id="f-78310926e7db-Status">
#[derive(Serialize)]
pub struct Status {
    pub available: bool,
    pub installed: bool,
    pub can_remove: bool,
    pub command: PathBuf,
    pub source: PathBuf,
    pub conflict: Option<PathBuf>,
    pub message: String,
}

</storycode:copy>
<a id="f-78310926e7db-impl-Installer"></a>
### `impl-Installer`

<storycode:copy id="f-78310926e7db-impl-Installer">
impl Installer {
    fn discover() -> Result<Self> {
        let executable = std::env::current_exe()?;
        let source = executable
            .parent()
            .context("No application directory")?
            .join(if cfg!(windows) { "hick.exe" } else { "hick" });
        #[cfg(unix)]
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("No user home directory; cannot install the command")?;
        Ok(Self {
            source,
            #[cfg(unix)]
            home,
            #[cfg(unix)]
            shell: std::env::var("SHELL").unwrap_or_default(),
            #[cfg(unix)]
            appimage: std::env::var_os("APPIMAGE").map(PathBuf::from),
            search_path: std::env::var_os("PATH").unwrap_or_default(),
        })
    }

    pub fn status(&self) -> Result<Status> {
        platform::status(self)
    }
    pub fn install(&self) -> Result<Status> {
        platform::install(self).context("Cannot install the hick command. Check permissions on your user PATH directory and shell profiles, then retry")?;
        self.status()
    }
    pub fn remove(&self) -> Result<Status> {
        platform::remove(self).context("Cannot remove the hick command. Check permissions on your user PATH directory and shell profiles, then retry")?;
        self.status()
    }

    fn conflict(&self, destination: &Path) -> Option<PathBuf> {
        let path = &self.search_path;
        std::env::split_paths(path)
            .map(|dir| dir.join(if cfg!(windows) { "hick.exe" } else { "hick" }))
            .find(|path| {
                path.is_file()
                    && path != destination
                    && path.canonicalize().ok() != self.source.canonicalize().ok()
            })
    }
}

</storycode:copy>
<a id="f-78310926e7db-Action"></a>
### `Action`

<storycode:copy id="f-78310926e7db-Action">
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Install,
    Remove,
}
</storycode:copy>
<a id="f-78310926e7db-Request"></a>
### `Request`

<storycode:copy id="f-78310926e7db-Request">
#[derive(Deserialize)]
struct Request {
    action: Action,
}

</storycode:copy>
<a id="f-78310926e7db-routes"></a>
### `routes`

Shared by folderless and workspace windows, at the desktop's own origin.

<storycode:copy id="f-78310926e7db-routes">
/// Shared by folderless and workspace windows, at the desktop's own origin.
pub fn routes(router: Router, ui_origin: Option<String>) -> Router {
    let installer = Arc::new(Mutex::new(Installer::discover()));
    let read = installer.clone();
    router.route(
        "/api/settings/command-path",
        get(move || {
            let read = read.clone();
            async move { perform(read, None).await }
        })
        .post(move |headers: HeaderMap, Json(request): Json<Request>| {
            let installer = installer.clone();
            let ui_origin = ui_origin.clone();
            async move {
                let own = format!(
                    "http://{}",
                    headers
                        .get("host")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default()
                );
                if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok())
                    && origin != own
                    && ui_origin.as_deref() != Some(origin)
                {
                    return StatusCode::FORBIDDEN.into_response();
                }
                perform(installer, Some(request.action)).await
            }
        }),
    )
}

</storycode:copy>
<a id="f-78310926e7db-perform"></a>
### `perform`

<storycode:copy id="f-78310926e7db-perform">
async fn perform(installer: Arc<Mutex<Result<Installer>>>, action: Option<Action>) -> Response {
    let outcome = tokio::task::spawn_blocking(move || -> Result<Status> {
        let guard = installer
            .lock()
            .map_err(|_| anyhow::anyhow!("Command installer is busy; retry"))?;
        let installer = guard
            .as_ref()
            .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        match action {
            None => installer.status(),
            Some(Action::Install) => installer.install(),
            Some(Action::Remove) => installer.remove(),
        }
    })
    .await;
    match outcome {
        Ok(Ok(status)) => Json(status).into_response(),
        other => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": match other {
                Ok(Err(error)) => format!("{error:#}"),
                Err(error) => format!("Command setup did not finish: {error}"),
                _ => unreachable!(),
            }})),
        )
            .into_response(),
    }
}

</storycode:copy>
<a id="f-78310926e7db-require_available"></a>
### `require_available`

<storycode:copy id="f-78310926e7db-require_available">
fn require_available(status: &Status) -> Result<()> {
    if !status.available {
        bail!(
            "This build has no bundled hick CLI at {}. Install a packaged desktop release.",
            status.source.display()
        );
    }
    if let Some(conflict) = &status.conflict {
        bail!(
            "Another hick command is installed at {}. Remove that installation or adjust PATH before installing this one.",
            conflict.display()
        );
    }
    Ok(())
}

</storycode:copy>
<a id="f-78310926e7db-tests"></a>
### `tests`

<storycode:copy id="f-78310926e7db-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
    #[tokio::test]
    async fn desktop_status_is_readable_and_other_origins_cannot_install() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, routes(Router::new(), None))
                .await
                .unwrap();
        });
        let client = reqwest::Client::new();
        let endpoint = format!("{url}/api/settings/command-path");
        let status: serde_json::Value = client
            .get(&endpoint)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(status["available"].is_boolean());
        assert!(status["command"].is_string());
        let blocked = client
            .post(&endpoint)
            .header("Origin", "https://other.example")
            .json(&serde_json::json!({"action":"install"}))
            .send()
            .await
            .unwrap();
        assert_eq!(blocked.status(), StatusCode::FORBIDDEN);
        let invalid = client
            .post(&endpoint)
            .json(&serde_json::json!({"action":"replace"}))
            .send()
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
        server.abort();
    }
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/src/command_path.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/src/command_path.rs"><storycode:paste select="#f-78310926e7db-platform"/><storycode:paste select="#f-78310926e7db-platform-2"/><storycode:paste select="#f-78310926e7db-Installer"/><storycode:paste select="#f-78310926e7db-Status"/><storycode:paste select="#f-78310926e7db-impl-Installer"/><storycode:paste select="#f-78310926e7db-Action"/><storycode:paste select="#f-78310926e7db-Request"/><storycode:paste select="#f-78310926e7db-routes"/><storycode:paste select="#f-78310926e7db-perform"/><storycode:paste select="#f-78310926e7db-require_available"/><storycode:paste select="#f-78310926e7db-tests"/></storycode:file>

<a id="output-6435d8a2c822"></a>
## `apps/desktop/src-tauri/src/command_path_unix.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [The App Installs Its Command For The User](contract.md).

<a id="f-6435d8a2c822-START"></a>
### `START`

<storycode:copy id="f-6435d8a2c822-START">
use super::*;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

const START: &str = "# >>> Hickory Docs command PATH >>>";
</storycode:copy>
<a id="f-6435d8a2c822-END"></a>
### `END`

<storycode:copy id="f-6435d8a2c822-END">
const END: &str = "# <<< Hickory Docs command PATH <<<";

</storycode:copy>
<a id="f-6435d8a2c822-command"></a>
### `command`

<storycode:copy id="f-6435d8a2c822-command">
fn command(installer: &Installer) -> PathBuf {
    installer.home.join(".local/bin/hick")
}
</storycode:copy>
<a id="f-6435d8a2c822-quote"></a>
### `quote`

<storycode:copy id="f-6435d8a2c822-quote">
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
</storycode:copy>
<a id="f-6435d8a2c822-wrapper"></a>
### `wrapper`

<storycode:copy id="f-6435d8a2c822-wrapper">
fn wrapper(installer: &Installer) -> Option<String> {
    installer.appimage.as_ref().map(|image| format!("#!/bin/sh\n# Hickory Docs command launcher\nexport HICKORY_DESKTOP={}\nexec {} --hick-cli \"$@\"\n", quote(image), quote(image)))
}
</storycode:copy>
<a id="f-6435d8a2c822-owned"></a>
### `owned`

<storycode:copy id="f-6435d8a2c822-owned">
fn owned(installer: &Installer) -> bool {
    let target = command(installer);
    if let Some(text) = wrapper(installer) {
        !target.is_symlink() && fs::read_to_string(target).ok().as_deref() == Some(&text)
    } else {
        fs::read_link(target).ok().as_ref() == Some(&installer.source)
    }
}

</storycode:copy>
<a id="f-6435d8a2c822-profiles"></a>
### `profiles`

<storycode:copy id="f-6435d8a2c822-profiles">
fn profiles(installer: &Installer) -> Result<Vec<(PathBuf, String)>> {
    let shell = Path::new(&installer.shell)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("zsh");
    let bin = quote(command(installer).parent().unwrap());
    // Bash reads only the first existing login profile. Creating .bash_profile
    // over an existing .profile would silently stop loading the person's setup.
    let login = [".bash_profile", ".bash_login", ".profile"]
        .into_iter()
        .find(|name| installer.home.join(name).exists())
        .unwrap_or(".profile");
    let (names, line) = match shell {
        "fish" => (
            vec![".config/fish/conf.d/hickory-docs.fish"],
            format!("fish_add_path --path {bin}"),
        ),
        "bash" => (
            vec![".bashrc", login],
            format!("export PATH={bin}:\"$PATH\""),
        ),
        "zsh" | "" => (vec![".zshrc"], format!("export PATH={bin}:\"$PATH\"")),
        "sh" | "dash" | "ksh" => (vec![".profile"], format!("export PATH={bin}:\"$PATH\"")),
        _ => bail!(
            "Automatic PATH setup is unavailable for {shell}. Add {} to your shell's PATH, then retry.",
            command(installer).parent().unwrap().display()
        ),
    };
    Ok(names
        .into_iter()
        .map(|name| {
            (
                installer.home.join(name),
                format!("{START}\n{line}\n{END}\n"),
            )
        })
        .collect())
}

</storycode:copy>
<a id="f-6435d8a2c822-status"></a>
### `status`

<storycode:copy id="f-6435d8a2c822-status">
pub fn status(installer: &Installer) -> Result<Status> {
    let target = command(installer);
    let own = owned(installer);
    let available =
        installer.source.is_file() && installer.appimage.as_ref().is_none_or(|p| p.is_file());
    let conflict = if fs::symlink_metadata(&target).is_ok() && !own {
        Some(target.clone())
    } else {
        installer.conflict(&target)
    };
    let on_path = installer.source.exists()
        && std::env::split_paths(&installer.search_path)
            .any(|p| p.join("hick").canonicalize().ok() == installer.source.canonicalize().ok());
    let profiles_ready = profiles(installer).is_ok_and(|profiles| {
        profiles
            .iter()
            .all(|(path, block)| read_profile(path).is_ok_and(|text| text.contains(block)))
    });
    Ok(Status {
        available,
        installed: (own && profiles_ready) || on_path,
        can_remove: own,
        command: target,
        source: installer.source.clone(),
        conflict,
        message: if (own && profiles_ready) || on_path {
            "Installed for your user. Open a new terminal to use hick."
        } else if !available {
            "The command is available in packaged desktop releases."
        } else {
            "Install hick for your user to open documents and use the CLI from a terminal."
        }
        .into(),
    })
}

</storycode:copy>
<a id="f-6435d8a2c822-install"></a>
### `install`

<storycode:copy id="f-6435d8a2c822-install">
pub fn install(installer: &Installer) -> Result<()> {
    let status = status(installer)?;
    require_available(&status)?;
    let profiles = profiles(installer)?;
    // Validate every profile before creating anything. An edited block is the
    // person's text and must not be replaced or deleted by this installer.
    for (path, block) in &profiles {
        let text = read_profile(path)?;
        if text.contains(START) && !text.contains(block) {
            bail!(
                "{} contains an edited Hickory Docs PATH block. Remove that block before retrying.",
                path.display()
            );
        }
    }
    fs::create_dir_all(status.command.parent().unwrap())?;
    if !owned(installer) {
        if let Some(text) = wrapper(installer) {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&status.command)?;
            file.write_all(text.as_bytes())?;
            file.set_permissions(fs::Permissions::from_mode(0o755))?;
        } else {
            symlink(&installer.source, &status.command)?;
        }
    }
    for (path, block) in profiles {
        let mut text = read_profile(&path)?;
        if !text.contains(&block) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&block);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, text)?;
        }
    }
    Ok(())
}

</storycode:copy>
<a id="f-6435d8a2c822-read_profile"></a>
### `read_profile`

<storycode:copy id="f-6435d8a2c822-read_profile">
fn read_profile(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error)
            .with_context(|| format!("Cannot read {}; check its permissions", path.display())),
    }
}

</storycode:copy>
<a id="f-6435d8a2c822-remove"></a>
### `remove`

<storycode:copy id="f-6435d8a2c822-remove">
pub fn remove(installer: &Installer) -> Result<()> {
    if !owned(installer) {
        bail!(
            "{} is not this app's command. Remove it through the installer that created it.",
            command(installer).display()
        );
    }
    for (path, block) in profiles(installer)? {
        let text = read_profile(&path)?;
        if text.contains(&block) {
            fs::write(path, text.replace(&block, ""))?;
        }
    }
    fs::remove_file(command(installer))?;
    Ok(())
}

</storycode:copy>
<a id="f-6435d8a2c822-tests"></a>
### `tests`

<storycode:copy id="f-6435d8a2c822-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn installer(home: &Path) -> Installer {
        let source = home.join("Hickory Docs.app/Contents/MacOS/hick");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "cli").unwrap();
        Installer {
            source,
            home: home.into(),
            shell: "/bin/zsh".into(),
            appimage: None,
            search_path: Default::default(),
        }
    }
    // Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
    #[test]
    fn install_is_idempotent_and_removal_preserves_the_profile() {
        let home = tempfile::tempdir().unwrap();
        let setup = installer(home.path());
        let profile = home.path().join(".zshrc");
        fs::write(&profile, "# my settings\n").unwrap();
        install(&setup).unwrap();
        install(&setup).unwrap();
        assert!(owned(&setup));
        assert_eq!(
            fs::read_to_string(&profile).unwrap().matches(START).count(),
            1
        );
        remove(&setup).unwrap();
        assert_eq!(fs::read_to_string(profile).unwrap(), "# my settings\n");
        assert!(!command(&setup).exists());
    }
    #[test]
    fn existing_command_and_edited_blocks_are_preserved() {
        let home = tempfile::tempdir().unwrap();
        let setup = installer(home.path());
        fs::create_dir_all(command(&setup).parent().unwrap()).unwrap();
        fs::write(command(&setup), "someone else's command").unwrap();
        assert!(install(&setup).is_err());
        assert!(remove(&setup).is_err());
        assert_eq!(
            fs::read_to_string(command(&setup)).unwrap(),
            "someone else's command"
        );
        fs::remove_file(command(&setup)).unwrap();
        fs::write(
            home.path().join(".zshrc"),
            format!("{START}\ncustom\n{END}\n"),
        )
        .unwrap();
        assert!(install(&setup).is_err());
        assert!(!command(&setup).exists());
    }
    #[test]
    fn portable_launcher_uses_the_saved_appimage() {
        let home = tempfile::tempdir().unwrap();
        let mut setup = installer(home.path());
        let image = home.path().join("Hickory's Docs.AppImage");
        fs::write(&image, "appimage").unwrap();
        setup.appimage = Some(image);
        install(&setup).unwrap();
        let text = fs::read_to_string(command(&setup)).unwrap();
        assert!(text.contains("--hick-cli"));
        assert!(text.contains("'\\''"));
        assert!(!text.contains("Contents/MacOS"));
        remove(&setup).unwrap();
    }
}

</storycode:copy>
<a id="f-6435d8a2c822-bash_tests"></a>
### `bash_tests`

<storycode:copy id="f-6435d8a2c822-bash_tests">
#[cfg(test)]
mod bash_tests {
    use super::*;
    // Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
    #[test]
    fn bash_keeps_the_existing_login_profile_in_use() {
        let home = tempfile::tempdir().unwrap();
        let source = home.path().join("hick");
        fs::write(&source, "cli").unwrap();
        fs::write(home.path().join(".profile"), "# login setup\n").unwrap();
        let installer = Installer {
            source,
            home: home.path().into(),
            shell: "/bin/bash".into(),
            appimage: None,
            search_path: Default::default(),
        };
        install(&installer).unwrap();
        assert!(!home.path().join(".bash_profile").exists());
        assert!(
            fs::read_to_string(home.path().join(".profile"))
                .unwrap()
                .starts_with("# login setup\n")
        );
        remove(&installer).unwrap();
        assert_eq!(
            fs::read_to_string(home.path().join(".profile")).unwrap(),
            "# login setup\n"
        );
    }
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/src/command_path_unix.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/src/command_path_unix.rs"><storycode:paste select="#f-6435d8a2c822-START"/><storycode:paste select="#f-6435d8a2c822-END"/><storycode:paste select="#f-6435d8a2c822-command"/><storycode:paste select="#f-6435d8a2c822-quote"/><storycode:paste select="#f-6435d8a2c822-wrapper"/><storycode:paste select="#f-6435d8a2c822-owned"/><storycode:paste select="#f-6435d8a2c822-profiles"/><storycode:paste select="#f-6435d8a2c822-status"/><storycode:paste select="#f-6435d8a2c822-install"/><storycode:paste select="#f-6435d8a2c822-read_profile"/><storycode:paste select="#f-6435d8a2c822-remove"/><storycode:paste select="#f-6435d8a2c822-tests"/><storycode:paste select="#f-6435d8a2c822-bash_tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/command_path.rs](contract.md#output-78310926e7db)
- [apps/desktop/src-tauri/src/command_path_unix.rs](contract.md#output-6435d8a2c822)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)
- [scripts/install.sh](../../../mechanisms/scripts/install.sh.md#output-e076e43cb817)

### Tests and fixtures

- [apps/web/src/views/CommandPathSettings.test.tsx](../../../mechanisms/apps/web/src/views/CommandPathSettings.tsx.md#output-7f450949f2a8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
