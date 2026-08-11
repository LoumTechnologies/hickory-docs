//! `hickory login` — GitHub OAuth, device flow.
//!
//! The tool itself needs no account. This exists for exactly one thing: opening
//! a tunnel on the relay, which forwards bytes to a stranger's browser and
//! therefore has to be attributable to somebody (`docs/specs/freeform/relay.md`).
//!
//! **Device flow, not the redirect flow.** A CLI has no browser to redirect
//! back to, and the person running it may be on a remote machine over SSH.
//! Device flow is built for that: the CLI polls while the human authenticates
//! wherever their browser already is. No callback listener, no localhost port,
//! and no client secret on anyone's machine — a public client does not have
//! one to leak.
//!
//! **No scopes requested.** The default grants a token that can read a public
//! profile and nothing else: no repositories, no email, no organisations. The
//! relay needs one fact — which account is this — and asking for more would be
//! asking for trust the feature does not need.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};

/// GitHub's device-flow endpoints, overridable so the flow can be tested
/// against a local stand-in rather than a real browser and a real account.
pub const GITHUB_BASE: &str = "https://github.com";
pub const GITHUB_API_BASE: &str = "https://api.github.com";

/// The OAuth app the CLI identifies itself as.
///
/// Public by construction — device flow has no client secret, and the id is
/// visible in every request the CLI makes. It is *not* compiled in as a
/// constant because this repository has no registered app yet: whoever
/// registers one sets `HICKORY_GH_CLIENT_ID`, and until then the failure says
/// exactly that rather than sending people at a 404.
pub fn client_id() -> Result<String> {
    match std::env::var("HICKORY_GH_CLIENT_ID") {
        Ok(id) if !id.trim().is_empty() => Ok(id.trim().to_string()),
        _ => bail!(
            "no GitHub OAuth app is configured for this build.\n\
             Set HICKORY_GH_CLIENT_ID to the client id of a GitHub OAuth app with device \
             flow enabled (GitHub → Settings → Developer settings → OAuth Apps → \
             \"Enable Device Flow\").\n\
             This is only needed to use the relay; everything else works signed out."
        ),
    }
}

// ---------------------------------------------------------------------------
// Stored credentials
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// A GitHub access token. Held so the relay can be told who is calling.
    pub access_token: String,
    /// The GitHub login it belongs to, resolved once at sign-in so `hickory
    /// whoami` costs nothing.
    pub login: String,
}

/// Where credentials live: `$XDG_CONFIG_HOME/hickory/credentials.json`, else
/// `~/.config/hickory/credentials.json`.
///
/// Never inside a repository — a token in a working tree is a token in a commit
/// eventually.
pub fn credentials_path() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("HICKORY_CONFIG_DIR")
        && !dir.trim().is_empty()
    {
        return Ok(PathBuf::from(dir).join("credentials.json"));
    }
    let base = match std::env::var("XDG_CONFIG_HOME") {
        Ok(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .context("cannot find your home directory to store credentials in")?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(base.join("hickory").join("credentials.json"))
}

/// Read stored credentials, if any. A missing or unreadable file is `None`:
/// being signed out is a normal state, not an error.
pub fn load() -> Option<Credentials> {
    let path = credentials_path().ok()?;
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Write credentials, readable only by this user.
pub fn save(credentials: &Credentials) -> Result<PathBuf> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(credentials)?;
    std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;

    // 0600 before anyone else can read it. On Windows the file inherits the
    // user profile's ACL, which is the equivalent protection.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("restricting permissions on {}", path.display()))?;
    }
    Ok(path)
}

/// Forget stored credentials. Returns whether there were any.
pub fn forget() -> Result<bool> {
    let path = credentials_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
}

// ---------------------------------------------------------------------------
// The device flow
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default = "default_interval")]
    interval: u64,
    #[serde(default)]
    expires_in: u64,
}

fn default_interval() -> u64 {
    5
}

#[derive(Debug, Deserialize)]
struct TokenAnswer {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    interval: Option<u64>,
}

/// Where the flow talks to. Split out so a test can point it at a stand-in.
pub struct Endpoints {
    pub github: String,
    pub api: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            github: std::env::var("HICKORY_GITHUB_BASE").unwrap_or_else(|_| GITHUB_BASE.into()),
            api: std::env::var("HICKORY_GITHUB_API_BASE")
                .unwrap_or_else(|_| GITHUB_API_BASE.into()),
        }
    }
}

/// Run the device flow to completion, printing instructions as it goes.
///
/// `announce` receives the two things the human must see — where to go and what
/// to type. It is a callback so the flow can be driven by a test without
/// scraping stdout.
pub async fn device_flow(
    endpoints: &Endpoints,
    client_id: &str,
    announce: &mut dyn FnMut(&str, &str),
) -> Result<Credentials> {
    let http = reqwest::Client::new();

    let device: DeviceCode = http
        .post(format!("{}/login/device/code", endpoints.github))
        .header("accept", "application/json")
        .form(&[("client_id", client_id)])
        .send()
        .await
        .context("asking GitHub to start a device login")?
        .json()
        .await
        .context("GitHub's device-code answer was not what we expected")?;

    announce(&device.verification_uri, &device.user_code);

    let deadline = std::time::Instant::now()
        + Duration::from_secs(if device.expires_in == 0 {
            900
        } else {
            device.expires_in
        });
    let mut interval = Duration::from_secs(device.interval.max(1));

    loop {
        if std::time::Instant::now() > deadline {
            bail!("the code expired before it was entered — run `hickory login` again");
        }
        tokio::time::sleep(interval).await;

        let answer: TokenAnswer = http
            .post(format!("{}/login/oauth/access_token", endpoints.github))
            .header("accept", "application/json")
            .form(&[
                ("client_id", client_id),
                ("device_code", &device.device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .context("asking GitHub whether the login completed")?
            .json()
            .await
            .context("GitHub's token answer was not what we expected")?;

        if let Some(token) = answer.access_token {
            let login = whoami(endpoints, &token).await?;
            return Ok(Credentials {
                access_token: token,
                login,
            });
        }

        match answer.error.as_deref() {
            // The human has not finished yet. This is the normal case and is
            // not an error worth printing.
            Some("authorization_pending") | None => {}
            // GitHub asks for a slower poll and says how much slower.
            Some("slow_down") => {
                interval = Duration::from_secs(answer.interval.unwrap_or(interval.as_secs() + 5));
            }
            Some("expired_token") => {
                bail!("the code expired before it was entered — run `hickory login` again")
            }
            Some("access_denied") => bail!("the login was declined in the browser"),
            Some(other) => bail!("GitHub refused the login: {other}"),
        }
    }
}

/// Which account a token belongs to.
pub async fn whoami(endpoints: &Endpoints, token: &str) -> Result<String> {
    let body: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/user", endpoints.api))
        .header("authorization", format!("Bearer {token}"))
        .header("accept", "application/vnd.github+json")
        .header("user-agent", "hickory-cli")
        .send()
        .await
        .context("asking GitHub who this token belongs to")?
        .json()
        .await
        .context("GitHub's answer was not what we expected")?;
    body.get("login")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .context("GitHub's answer carried no login")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Credentials must never land in a repository, and must not be readable
    /// by other users on a shared machine.
    #[test]
    fn credentials_are_stored_outside_any_repo_and_locked_down() {
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("HICKORY_CONFIG_DIR", dir.path()) };

        let path = credentials_path().unwrap();
        assert!(path.starts_with(dir.path()));
        assert!(
            load().is_none(),
            "signed out is a normal state, not an error"
        );

        let written = save(&Credentials {
            access_token: "gho_secret".into(),
            login: "nate".into(),
        })
        .unwrap();
        assert_eq!(written, path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "credentials must be user-only");
        }

        let back = load().unwrap();
        assert_eq!(back.access_token, "gho_secret");
        assert_eq!(back.login, "nate");

        assert!(forget().unwrap(), "there were credentials to forget");
        assert!(!forget().unwrap(), "forgetting twice is not an error");
        assert!(load().is_none());

        unsafe { std::env::remove_var("HICKORY_CONFIG_DIR") };
    }

    #[test]
    fn a_missing_oauth_app_says_what_to_register() {
        unsafe { std::env::remove_var("HICKORY_GH_CLIENT_ID") };
        let err = client_id().unwrap_err().to_string();
        assert!(err.contains("HICKORY_GH_CLIENT_ID"), "{err}");
        assert!(err.contains("Device Flow"), "{err}");
        // Nobody should think this breaks the tool itself.
        assert!(err.contains("works signed out"), "{err}");
    }
}
