//! Development overrides, and the reason there are exactly two of them.
//!
//! In a shipped app the window loads the engine's own address and the engine
//! serves the UI from `apps/web/dist`, embedded in the binary (`server.rs`
//! explains why one origin is worth having). That arrangement has no hot
//! reload in it anywhere: the UI is baked in at build time, so seeing a
//! frontend change means rebuilding the bundle and reloading the window, and
//! a `just dev` that did not rebuild showed the UI from whenever somebody last
//! ran `npm run build` — silently, with a Vite running beside it looking for
//! all the world like it was the thing being served.
//!
//! The fix is to let the window load Vite in development, because Vite already
//! knows how to be the other half of this: `apps/web/vite.config.ts` proxies
//! `/api` — including the WebSocket, which is the whole live-sync layer — to
//! the engine. So the browser still sees one origin, relative `fetch` and
//! `location.host` still work, and modules arrive with hot reload attached.
//!
//! That needs two things the shipped app has no use for, which is why they are
//! here and not in `ServeOptions`:
//!
//!  - **A known engine port.** Vite's proxy target is configured before Vite
//!    starts, so the engine cannot pick an ephemeral one. A real app has no
//!    reason to want a particular port and every reason not to — asking for
//!    one means failing to start because something unrelated holds it — so
//!    this is unset by default and stays that way in anything anybody
//!    downloads.
//!  - **A UI origin for the window.** Otherwise the window loads the engine,
//!    which serves the embedded bundle, which is the stale thing.
//!
//! Neither is read anywhere but at startup, and neither has a default that
//! changes what a downloaded app does: with nothing set, this is the app it
//! was before any of it existed.

use std::fmt;

/// The environment variables this reads. Named here so an error message can
/// quote the same spelling the shell needs.
const SERVE_PORT: &str = "HICKORY_SERVE_PORT";
const UI_ORIGIN: &str = "HICKORY_UI_ORIGIN";

/// What development asked for, validated.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DevConfig {
    /// The port the engine must bind. `None` — the default, and what every
    /// downloaded copy does — means an ephemeral one.
    pub serve_port: Option<u16>,
    /// Where the window loads the UI from, without a trailing slash. `None`
    /// means the engine's own address and the bundle inside this binary.
    pub ui_origin: Option<String>,
}

/// A variable that was set to something unusable.
///
/// A hard failure rather than a fallback: both of these exist so that two
/// processes can agree about an address, and quietly ignoring one produces an
/// app that starts, opens, and cannot reach its own engine — which is a far
/// worse half hour than a message at boot.
#[derive(Debug, PartialEq, Eq)]
pub struct DevConfigError {
    pub variable: &'static str,
    pub value: String,
    pub problem: String,
    pub example: &'static str,
}

impl fmt::Display for DevConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} is set to `{}`, which {}.\n\nSet it to something like `{}`, or unset it to \
             run the app the way a downloaded copy runs: the engine on a port it picks, \
             serving the UI built into it.\n\nIf you did not set this yourself, it came from \
             `just dev` (scripts/dev.sh) — stop that run and start it again.",
            self.variable, self.value, self.problem, self.example
        )
    }
}

impl std::error::Error for DevConfigError {}

/// Read the overrides from the environment.
///
/// Total in the ordinary case: nothing set is not an error, it is the shipped
/// app. Only a variable that is *present and unusable* fails.
pub fn from_env() -> Result<DevConfig, DevConfigError> {
    let read = |name| present(name, std::env::var(name).ok());
    Ok(DevConfig {
        serve_port: port(read(SERVE_PORT))?,
        ui_origin: origin(read(UI_ORIGIN))?,
    })
}

/// Whether a variable was really set, treating whitespace-only as absent — a
/// shell that exports an empty string meant "no", not "yes, the empty one".
///
/// Takes the raw value rather than reading it, so the rule can be tested
/// without a process-wide environment two parallel tests would fight over.
fn present(name: &'static str, raw: Option<String>) -> Option<(&'static str, String)> {
    let raw = raw?;
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| (name, trimmed.to_owned()))
}

fn port(set: Option<(&'static str, String)>) -> Result<Option<u16>, DevConfigError> {
    let Some((variable, value)) = set else {
        return Ok(None);
    };
    let parsed: u16 = value.parse().map_err(|_| DevConfigError {
        variable,
        value: value.clone(),
        problem: "is not a port number between 1 and 65535".into(),
        example: "42371",
    })?;
    // Zero is spelled "unset" here. It means "pick one" to the operating
    // system, which is exactly the thing this variable exists to prevent, so
    // accepting it would be accepting an instruction that contradicts itself.
    if parsed == 0 {
        return Err(DevConfigError {
            variable,
            value,
            problem: "means \"pick any port\", which defeats the point of naming one — \
                      whatever is proxying to it cannot know what was picked"
                .into(),
            example: "42371",
        });
    }
    Ok(Some(parsed))
}

fn origin(set: Option<(&'static str, String)>) -> Result<Option<String>, DevConfigError> {
    let Some((variable, value)) = set else {
        return Ok(None);
    };
    // Parsed rather than pattern-matched: this string becomes the window's
    // URL, and "looks like it starts with http" is not the same as "a browser
    // can navigate to it".
    let url: tauri::Url = value.parse().map_err(|_| DevConfigError {
        variable,
        value: value.clone(),
        problem: "is not a URL".into(),
        example: "http://localhost:42370",
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(DevConfigError {
            variable,
            value,
            problem: "is not an http or https URL, and the window has nothing else to load".into(),
            example: "http://localhost:42370",
        });
    }
    Ok(Some(value.trim_end_matches('/').to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_set_is_the_shipped_app() {
        // The rule that matters most: a machine with no environment at all
        // runs the app that was downloaded.
        assert_eq!(
            DevConfig {
                serve_port: port(None).unwrap(),
                ui_origin: origin(None).unwrap(),
            },
            DevConfig::default()
        );
    }

    #[test]
    fn an_empty_variable_is_an_absent_one() {
        // Not merely "does not parse": absent, so the app runs its default
        // rather than refusing to start over a variable nobody meant to set.
        assert_eq!(present(SERVE_PORT, Some("   ".into())), None);
        assert_eq!(present(SERVE_PORT, None), None);
        assert_eq!(
            present(SERVE_PORT, Some(" 42371 ".into())),
            Some((SERVE_PORT, "42371".to_owned()))
        );
    }

    #[test]
    fn a_port_is_a_port() {
        assert_eq!(
            port(Some((SERVE_PORT, "42371".into()))).unwrap(),
            Some(42371)
        );
    }

    #[test]
    fn a_port_that_is_not_a_number_says_so_and_shows_one() {
        let error = port(Some((SERVE_PORT, "the usual".into()))).unwrap_err();
        let said = error.to_string();
        assert!(said.contains("HICKORY_SERVE_PORT"), "{said}");
        assert!(said.contains("the usual"), "{said}");
        assert!(said.contains("42371"), "{said}");
        // It has to name where the value came from: nobody sets this by hand.
        assert!(said.contains("just dev"), "{said}");
    }

    #[test]
    fn port_zero_is_refused_rather_than_obeyed() {
        // "Pick any port" is the exact thing this variable exists to stop.
        let said = port(Some((SERVE_PORT, "0".into())))
            .unwrap_err()
            .to_string();
        assert!(said.contains("cannot know what was picked"), "{said}");
    }

    #[test]
    fn an_origin_keeps_its_host_and_loses_its_slash() {
        assert_eq!(
            origin(Some((UI_ORIGIN, "http://localhost:42370/".into()))).unwrap(),
            Some("http://localhost:42370".to_owned())
        );
    }

    #[test]
    fn an_origin_the_window_could_not_load_is_refused() {
        let said = origin(Some((UI_ORIGIN, "file:///tmp/x".into())))
            .unwrap_err()
            .to_string();
        assert!(said.contains("http or https"), "{said}");
    }
}
