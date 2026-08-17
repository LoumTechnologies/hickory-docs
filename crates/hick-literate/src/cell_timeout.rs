//! The run-wide default for per-cell execution timeouts.
//!
//! A cell that never finishes — reading stdin nobody will feed, serving a
//! socket, an infinite loop — used to hang `hick run`, `hick test`, the
//! pre-commit hook, and CI forever. Every executed cell now runs under a
//! wall-clock limit resolved in three layers, most specific first:
//!
//! 1. the cell's own `timeout="<seconds>"` attribute (`timeout="0"` means
//!    explicitly unbounded),
//! 2. `HICKORY_CELL_TIMEOUT` (seconds, `0` = unbounded) for the machine,
//! 3. the built-in default of 120 seconds.
//!
//! Guarantee: `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`.

use std::time::Duration;

use anyhow::{Result, bail};

/// The environment variable that overrides the machine-wide default.
pub const CELL_TIMEOUT_VAR: &str = "HICKORY_CELL_TIMEOUT";

/// The built-in default limit for one cell.
pub const BUILT_IN_DEFAULT: Duration = Duration::from_secs(120);

/// The run-wide default timeout for a cell: what a cell gets when it
/// declares no `timeout=` of its own.
///
/// `Default` is the built-in 120 seconds — NOT unbounded — so a caller that
/// constructs a `PipelineConfig` without thinking about timeouts still
/// cannot hang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellTimeoutDefault(pub Option<Duration>);

impl Default for CellTimeoutDefault {
    fn default() -> Self {
        Self(Some(BUILT_IN_DEFAULT))
    }
}

impl CellTimeoutDefault {
    /// No limit for cells that do not declare one. Only reachable by an
    /// explicit `HICKORY_CELL_TIMEOUT=0` or an explicit caller decision.
    pub fn unbounded() -> Self {
        Self(None)
    }

    /// A specific limit for cells that do not declare one.
    pub fn limit(limit: Duration) -> Self {
        Self(Some(limit))
    }

    /// The effective timeout for one cell: the cell's own `timeout=`
    /// attribute (as parsed into `ExecInfo::timeout_secs`) wins; absent
    /// that, this run-wide default. `Some(0)` — the attribute spelled
    /// `timeout="0"` — means the cell is explicitly unbounded.
    pub fn for_cell(self, timeout_secs: Option<u64>) -> Option<Duration> {
        match timeout_secs {
            Some(0) => None,
            Some(secs) => Some(Duration::from_secs(secs)),
            None => self.0,
        }
    }

    /// Resolve the machine-wide default from the process environment.
    ///
    /// Call this once at the start of a run; a malformed value fails the
    /// run loudly rather than silently running unbounded (or silently
    /// clamping). See `.instructions/config-and-environments.md`.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// [`CellTimeoutDefault::from_env`] with the environment injected, so
    /// parsing is testable without touching the real (process-global,
    /// race-prone) environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        match lookup(CELL_TIMEOUT_VAR) {
            None => Ok(Self::default()),
            Some(raw) if raw.trim().is_empty() => Ok(Self::default()),
            Some(raw) => match raw.trim().parse::<u64>() {
                Ok(0) => Ok(Self::unbounded()),
                Ok(secs) => Ok(Self::limit(Duration::from_secs(secs))),
                Err(_) => bail!(
                    "{CELL_TIMEOUT_VAR}=\"{raw}\" is not a whole number of seconds, so no \
                     cell can run under it.\n  \
                     Next steps: set {CELL_TIMEOUT_VAR}=300 (any whole number of seconds) \
                     to change the default time limit for every cell, {CELL_TIMEOUT_VAR}=0 \
                     to remove the default limit entirely, or unset it to use the built-in \
                     default of {} seconds. Fractions and units ('1.5', '2m') are not \
                     accepted. A single cell can still override the default with a \
                     timeout=\"<seconds>\" attribute on its <hick:exec> tag.",
                    BUILT_IN_DEFAULT.as_secs()
                ),
            },
        }
    }
}

// Protects docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
// environment-resolution layer.
#[cfg(test)]
mod tests {
    use super::*;

    fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.to_string())
        }
    }

    #[test]
    fn unset_means_the_built_in_default() {
        let d = CellTimeoutDefault::from_lookup(env_of(&[])).unwrap();
        assert_eq!(d, CellTimeoutDefault(Some(BUILT_IN_DEFAULT)));
        assert_eq!(BUILT_IN_DEFAULT, Duration::from_secs(120));
    }

    #[test]
    fn a_number_of_seconds_is_honored() {
        let d = CellTimeoutDefault::from_lookup(env_of(&[("HICKORY_CELL_TIMEOUT", "7")])).unwrap();
        assert_eq!(d, CellTimeoutDefault(Some(Duration::from_secs(7))));
    }

    #[test]
    fn zero_means_unbounded_and_empty_means_unset() {
        let d = CellTimeoutDefault::from_lookup(env_of(&[("HICKORY_CELL_TIMEOUT", "0")])).unwrap();
        assert_eq!(d, CellTimeoutDefault(None));
        let d = CellTimeoutDefault::from_lookup(env_of(&[("HICKORY_CELL_TIMEOUT", "  ")])).unwrap();
        assert_eq!(d, CellTimeoutDefault::default());
    }

    #[test]
    fn a_malformed_value_fails_loudly_with_the_valid_shape() {
        for bad in ["1.5", "2m", "-3", "forever"] {
            let err = CellTimeoutDefault::from_lookup(env_of(&[("HICKORY_CELL_TIMEOUT", bad)]))
                .unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("HICKORY_CELL_TIMEOUT"), "{bad}: {msg}");
            assert!(msg.contains(bad), "must echo the rejected value: {msg}");
            assert!(
                msg.contains("whole number of seconds"),
                "must say what a valid value looks like: {msg}"
            );
        }
    }

    #[test]
    fn the_cell_attribute_wins_over_the_default() {
        let default = CellTimeoutDefault::limit(Duration::from_secs(120));
        assert_eq!(default.for_cell(None), Some(Duration::from_secs(120)));
        assert_eq!(default.for_cell(Some(5)), Some(Duration::from_secs(5)));
        assert_eq!(
            default.for_cell(Some(0)),
            None,
            "timeout=\"0\" is unbounded"
        );
        assert_eq!(CellTimeoutDefault::unbounded().for_cell(None), None);
    }
}
