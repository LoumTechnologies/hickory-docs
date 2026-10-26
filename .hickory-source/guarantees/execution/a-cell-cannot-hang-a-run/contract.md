<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Cell Cannot Hang A Run

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document executed through the local or sandbox executor, when a cell
runs longer than its time limit — the cell's own `timeout="<seconds>"`
attribute, else `HICKORY_CELL_TIMEOUT` (seconds), else 120 seconds — then the
cell's process is really killed (on Unix, its whole process group, so
background children die with it) and the cell fails with an error naming the
container, the command, the limit that was hit, and both next steps (raise
`timeout=` on the cell, or set `HICKORY_CELL_TIMEOUT`). `timeout="0"`
declares one cell unbounded — allowed, but only explicitly — and a malformed
`timeout=` or `HICKORY_CELL_TIMEOUT` value fails loudly instead of silently
removing the limit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-cell-cannot-hang-a-run.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `timeout_attribute_parses_seconds_and_zero`

This is the maintained test fragment from `crates/hick-exec/src/dag/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-cc538e30a76b-timeout_attribute_parses_seconds_and_zero"></a>
### `timeout_attribute_parses_seconds_and_zero`

The three tests below protect
attribute-parsing half of it.

<storycode:copy id="f-cc538e30a76b-timeout_attribute_parses_seconds_and_zero">
// The three tests below protect
// docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
// attribute-parsing half of it.

#[test]
fn timeout_attribute_parses_seconds_and_zero() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="slow" image="alpine" timeout="300">
run the long benchmark
</hick:exec>
<hick:exec container="endless" image="alpine" timeout="0">
serve until killed by hand
</hick:exec>
</hick:doc>"#;
    let dag = parse_and_build(src).unwrap();
    assert_eq!(dag.execs[0].timeout_secs, Some(300));
    assert_eq!(
        dag.execs[1].timeout_secs,
        Some(0),
        "timeout=\"0\" is an explicit 'unbounded', distinct from the attribute being absent"
    );
}

</storycode:copy>

<a id="output-c44f2255b447"></a>
## `crates/hick-literate/src/cell_timeout.rs`

Related contracts: [A Cell Cannot Hang A Run](contract.md).

<a id="f-c44f2255b447-CELL_TIMEOUT_VAR"></a>
### `CELL_TIMEOUT_VAR`

The run-wide default for per-cell execution timeouts.
A cell that never finishes — reading stdin nobody will feed, serving a
socket, an infinite loop — used to hang `hick run`, `hick test`, the
pre-commit hook, and CI forever. Every executed cell now runs under a
wall-clock limit resolved in three layers, most specific first:
1. the cell's own `timeout="<seconds>"` attribute (`timeout="0"` means
explicitly unbounded),
2. `HICKORY_CELL_TIMEOUT` (seconds, `0` = unbounded) for the machine,
3. the built-in default of 120 seconds.

<storycode:copy id="f-c44f2255b447-CELL_TIMEOUT_VAR">
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

</storycode:copy>
<a id="f-c44f2255b447-BUILT_IN_DEFAULT"></a>
### `BUILT_IN_DEFAULT`

<storycode:copy id="f-c44f2255b447-BUILT_IN_DEFAULT">
/// The built-in default limit for one cell.
pub const BUILT_IN_DEFAULT: Duration = Duration::from_secs(120);

</storycode:copy>
<a id="f-c44f2255b447-CellTimeoutDefault"></a>
### `CellTimeoutDefault`

The run-wide default timeout for a cell: what a cell gets when it
declares no `timeout=` of its own.
`Default` is the built-in 120 seconds — NOT unbounded — so a caller that
constructs a `PipelineConfig` without thinking about timeouts still
cannot hang.

<storycode:copy id="f-c44f2255b447-CellTimeoutDefault">
/// The run-wide default timeout for a cell: what a cell gets when it
/// declares no `timeout=` of its own.
///
/// `Default` is the built-in 120 seconds — NOT unbounded — so a caller that
/// constructs a `PipelineConfig` without thinking about timeouts still
/// cannot hang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellTimeoutDefault(pub Option<Duration>);

</storycode:copy>
<a id="f-c44f2255b447-impl-Default-for-CellTimeoutDefault"></a>
### `impl-Default for CellTimeoutDefault`

<storycode:copy id="f-c44f2255b447-impl-Default-for-CellTimeoutDefault">
impl Default for CellTimeoutDefault {
    fn default() -> Self {
        Self(Some(BUILT_IN_DEFAULT))
    }
}

</storycode:copy>
<a id="f-c44f2255b447-impl-CellTimeoutDefault"></a>
### `impl-CellTimeoutDefault`

<storycode:copy id="f-c44f2255b447-impl-CellTimeoutDefault">
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

</storycode:copy>
<a id="f-c44f2255b447-tests"></a>
### `tests`

Protects docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
environment-resolution layer.

<storycode:copy id="f-c44f2255b447-tests">
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
</storycode:copy>

### Assemble `crates/hick-literate/src/cell_timeout.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/cell_timeout.rs"><storycode:paste select="#f-c44f2255b447-CELL_TIMEOUT_VAR"/><storycode:paste select="#f-c44f2255b447-BUILT_IN_DEFAULT"/><storycode:paste select="#f-c44f2255b447-CellTimeoutDefault"/><storycode:paste select="#f-c44f2255b447-impl-Default-for-CellTimeoutDefault"/><storycode:paste select="#f-c44f2255b447-impl-CellTimeoutDefault"/><storycode:paste select="#f-c44f2255b447-tests"/></storycode:file>

<a id="output-8696e0c20596"></a>
## `crates/hick-literate/tests/cell_timeout_tests.rs`

Related contracts: [A Cell Cannot Hang A Run](contract.md).

<a id="f-8696e0c20596-common"></a>
### `common`

Per-cell execution timeouts through the live pipeline.
These protect `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`:
the run-wide default bounds every cell, a cell's own `timeout=` attribute
overrides it in both directions, and `timeout="0"` is an explicit
unbounded declaration — never a silent one.

<storycode:copy id="f-8696e0c20596-common">
//! Per-cell execution timeouts through the live pipeline.
//!
//! These protect `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`:
//! the run-wide default bounds every cell, a cell's own `timeout=` attribute
//! overrides it in both directions, and `timeout="0"` is an explicit
//! unbounded declaration — never a silent one.

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_literate::cell_timeout::CellTimeoutDefault;
use hick_literate::{LocalExecutor, PipelineConfig, run_pipeline_live};

mod common;
</storycode:copy>
<a id="f-8696e0c20596-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-8696e0c20596-use_declaration">
use common::{runs_until_killed, sleeps_then_echoes};

</storycode:copy>
<a id="f-8696e0c20596-hick_doc"></a>
### `hick_doc`

<storycode:copy id="f-8696e0c20596-hick_doc">
fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="c" image="alpine" />
{body}
</hick:doc>"#
    )
}

</storycode:copy>
<a id="f-8696e0c20596-run_with_default"></a>
### `run_with_default`

<storycode:copy id="f-8696e0c20596-run_with_default">
async fn run_with_default(src: &str, cell_timeout: CellTimeoutDefault) -> anyhow::Result<()> {
    let config = PipelineConfig {
        cell_timeout,
        ..Default::default()
    };
    run_pipeline_live(
        &[("test.md", src)],
        &config,
        &[],
        None,
        Arc::new(LocalExecutor::new()?),
    )
    .await
    .map(|_| ())
}

</storycode:copy>
<a id="f-8696e0c20596-the_run_wide_default_bounds_a_cell_without_an_attribute"></a>
### `the_run_wide_default_bounds_a_cell_without_an_attribute`

<storycode:copy id="f-8696e0c20596-the_run_wide_default_bounds_a_cell_without_an_attribute">
#[tokio::test(flavor = "multi_thread")]
async fn the_run_wide_default_bounds_a_cell_without_an_attribute() {
    let sleeper = runs_until_killed();
    let src = hick_doc(&format!(
        r#"<hick:exec container="c">{sleeper}</hick:exec>"#
    ));
    let started = Instant::now();
    let err = run_with_default(&src, CellTimeoutDefault::limit(Duration::from_millis(300)))
        .await
        .unwrap_err();
    assert!(started.elapsed() < Duration::from_secs(10));
    let msg = err.to_string();
    assert!(msg.contains("timed out"), "{msg}");
    assert!(
        msg.contains(sleeper.as_str()),
        "must name the command: {msg}"
    );
    assert!(msg.contains("HICKORY_CELL_TIMEOUT"), "{msg}");
}

</storycode:copy>
<a id="f-8696e0c20596-the_timeout_attribute_overrides_the_default_upward"></a>
### `the_timeout_attribute_overrides_the_default_upward`

<storycode:copy id="f-8696e0c20596-the_timeout_attribute_overrides_the_default_upward">
#[tokio::test(flavor = "multi_thread")]
async fn the_timeout_attribute_overrides_the_default_upward() {
    // Default far too small for the cell; its own timeout="5" rescues it.
    let src = hick_doc(&format!(
        r#"<hick:exec container="c" timeout="5">{}</hick:exec>"#,
        sleeps_then_echoes("ok")
    ));
    run_with_default(&src, CellTimeoutDefault::limit(Duration::from_millis(50)))
        .await
        .expect("the cell's own timeout must win over the run-wide default");
}

</storycode:copy>
<a id="f-8696e0c20596-timeout_zero_is_explicitly_unbounded"></a>
### `timeout_zero_is_explicitly_unbounded`

<storycode:copy id="f-8696e0c20596-timeout_zero_is_explicitly_unbounded">
#[tokio::test(flavor = "multi_thread")]
async fn timeout_zero_is_explicitly_unbounded() {
    // Same tiny default, but the cell declares itself unbounded.
    let src = hick_doc(&format!(
        r#"<hick:exec container="c" timeout="0">{}</hick:exec>"#,
        sleeps_then_echoes("ok")
    ));
    run_with_default(&src, CellTimeoutDefault::limit(Duration::from_millis(50)))
        .await
        .expect("timeout=\"0\" must remove the limit for this one cell");
}

</storycode:copy>
<a id="f-8696e0c20596-a_default_constructed_config_still_has_a_limit"></a>
### `a_default_constructed_config_still_has_a_limit`

<storycode:copy id="f-8696e0c20596-a_default_constructed_config_still_has_a_limit">
#[tokio::test(flavor = "multi_thread")]
async fn a_default_constructed_config_still_has_a_limit() {
    // No caller decision at all must still mean "bounded": the built-in
    // 120-second default, not unbounded.
    assert_eq!(
        CellTimeoutDefault::default(),
        CellTimeoutDefault::limit(Duration::from_secs(120))
    );
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/cell_timeout_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/cell_timeout_tests.rs"><storycode:paste select="#f-8696e0c20596-common"/><storycode:paste select="#f-8696e0c20596-use_declaration"/><storycode:paste select="#f-8696e0c20596-hick_doc"/><storycode:paste select="#f-8696e0c20596-run_with_default"/><storycode:paste select="#f-8696e0c20596-the_run_wide_default_bounds_a_cell_without_an_attribute"/><storycode:paste select="#f-8696e0c20596-the_timeout_attribute_overrides_the_default_upward"/><storycode:paste select="#f-8696e0c20596-timeout_zero_is_explicitly_unbounded"/><storycode:paste select="#f-8696e0c20596-a_default_constructed_config_still_has_a_limit"/></storycode:file>

<a id="output-295eca818e03"></a>
## `crates/hickory-cli/tests/cell_timeout_env.rs`

Related contracts: [A Cell Cannot Hang A Run](contract.md).

<a id="f-295eca818e03-common"></a>
### `common`

`HICKORY_CELL_TIMEOUT` through the shipped binary.
These protect `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`:
the machine-wide default is really read from the environment of the
process the user runs, a timed-out cell fails the run with the
user-facing message, and a malformed value fails loudly at the start of
the run instead of silently running unbounded.
Environment variables are process-global, so these tests set them only on
the SPAWNED `hick` process, never in this test process — that is what
keeps them from racing every other test.

<storycode:copy id="f-295eca818e03-common">
//! `HICKORY_CELL_TIMEOUT` through the shipped binary.
//!
//! These protect `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`:
//! the machine-wide default is really read from the environment of the
//! process the user runs, a timed-out cell fails the run with the
//! user-facing message, and a malformed value fails loudly at the start of
//! the run instead of silently running unbounded.
//!
//! Environment variables are process-global, so these tests set them only on
//! the SPAWNED `hick` process, never in this test process — that is what
//! keeps them from racing every other test.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

mod common;
</storycode:copy>
<a id="f-295eca818e03-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-295eca818e03-use_declaration">
use common::{runs_until_killed, sleeps_then_echoes};

</storycode:copy>
<a id="f-295eca818e03-hick"></a>
### `hick`

<storycode:copy id="f-295eca818e03-hick">
fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-295eca818e03-write_doc"></a>
### `write_doc`

<storycode:copy id="f-295eca818e03-write_doc">
fn write_doc(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("doc.md");
    let source = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="doc.md">
<hick:container name="c" image="alpine" />
{body}
</hick:doc>
"#
    );
    std::fs::write(&path, source).unwrap();
    path
}

</storycode:copy>
<a id="f-295eca818e03-the_env_default_cuts_a_sleeping_cell"></a>
### `the_env_default_cuts_a_sleeping_cell`

<storycode:copy id="f-295eca818e03-the_env_default_cuts_a_sleeping_cell">
#[test]
fn the_env_default_cuts_a_sleeping_cell() {
    let sleeper = runs_until_killed();
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        &format!(r#"<hick:exec container="c">{}</hick:exec>"#, sleeper),
    );
    let started = Instant::now();
    let out = hick()
        .arg("run")
        .arg(&doc)
        .env("HICKORY_CELL_TIMEOUT", "1")
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a cell sleeping past HICKORY_CELL_TIMEOUT must fail the run"
    );
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "the run must end near the 1s limit, not after the 30s sleep"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("timed out"), "stderr: {stderr}");
    assert!(
        stderr.contains(sleeper.as_str()),
        "must name the command: {stderr}"
    );
    assert!(
        stderr.contains("HICKORY_CELL_TIMEOUT") && stderr.contains("timeout=\"<seconds>\""),
        "must offer both next steps: {stderr}"
    );
}

</storycode:copy>
<a id="f-295eca818e03-a_malformed_env_value_fails_loudly_before_any_cell_runs"></a>
### `a_malformed_env_value_fails_loudly_before_any_cell_runs`

<storycode:copy id="f-295eca818e03-a_malformed_env_value_fails_loudly_before_any_cell_runs">
#[test]
fn a_malformed_env_value_fails_loudly_before_any_cell_runs() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        r#"<hick:exec container="c">echo should-not-run > ran.txt</hick:exec>"#,
    );
    let out = hick()
        .arg("run")
        .arg(&doc)
        .env("HICKORY_CELL_TIMEOUT", "2m")
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a malformed HICKORY_CELL_TIMEOUT must never silently run unbounded"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("HICKORY_CELL_TIMEOUT") && stderr.contains("2m"),
        "must name the variable and the rejected value: {stderr}"
    );
    assert!(
        stderr.contains("whole number of seconds"),
        "must say what a valid value looks like: {stderr}"
    );
}

</storycode:copy>
<a id="f-295eca818e03-a_cell_attribute_beats_the_env_default"></a>
### `a_cell_attribute_beats_the_env_default`

<storycode:copy id="f-295eca818e03-a_cell_attribute_beats_the_env_default">
#[test]
fn a_cell_attribute_beats_the_env_default() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        &format!(
            r#"<hick:exec container="c" timeout="30">{}</hick:exec>"#,
            sleeps_then_echoes(2, "ok")
        ),
    );
    let status = hick()
        .arg("run")
        .arg(&doc)
        .env("HICKORY_CELL_TIMEOUT", "1")
        .status()
        .unwrap();
    assert!(
        status.success(),
        "the cell's own timeout=\"30\" must override the 1s machine default"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/cell_timeout_env.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/cell_timeout_env.rs"><storycode:paste select="#f-295eca818e03-common"/><storycode:paste select="#f-295eca818e03-use_declaration"/><storycode:paste select="#f-295eca818e03-hick"/><storycode:paste select="#f-295eca818e03-write_doc"/><storycode:paste select="#f-295eca818e03-the_env_default_cuts_a_sleeping_cell"/><storycode:paste select="#f-295eca818e03-a_malformed_env_value_fails_loudly_before_any_cell_runs"/><storycode:paste select="#f-295eca818e03-a_cell_attribute_beats_the_env_default"/></storycode:file>

## Test the behavior: `a_confined_cell_is_killed_at_its_timeout`

This is the maintained test fragment from `crates/hickory-executor-sandbox/tests/confinement.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c8a04313c63c-a_confined_cell_is_killed_at_its_timeout"></a>
### `a_confined_cell_is_killed_at_its_timeout`

Protects docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
timeout must survive the sandbox wrapper, group-kill included.

<storycode:copy id="f-c8a04313c63c-a_confined_cell_is_killed_at_its_timeout">
// Protects docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
// timeout must survive the sandbox wrapper, group-kill included.
#[tokio::test(flavor = "multi_thread")]
async fn a_confined_cell_is_killed_at_its_timeout() {
    if !available() {
        return;
    }
    let executor = started().await;
    let started_at = std::time::Instant::now();
    // `waitfor` rather than `ping -n`: waiting by pinging loopback needs the
    // network stack, and a confined cell is denied the network unless its
    // document asked for it — the cell would fail instantly for the wrong
    // reason and the test would pass without ever testing the timeout.
    // A busy loop, not `waitfor` and not `ping`. Measured inside a real
    // AppContainer: `waitfor /t 30 hickory` fails immediately with "Cannot
    // wait for the specified signal" — it needs an object in the global
    // namespace, the same thing that stops msys tools running here — and
    // pinging loopback needs the network a confined cell is denied. Either
    // would fail instantly for the wrong reason and the test would pass
    // without ever reaching the timeout it exists to check. A `for /L` burns
    // CPU, but it depends on nothing outside cmd itself.
    let sleeper = per_shell("sleep 30", "for /L %i in (1,1,2000000000) do @rem");
    let err = executor
        .execute_with_options(
            "c",
            &sleeper,
            None,
            hickory_executor::ExecOptions {
                timeout: Some(std::time::Duration::from_millis(300)),
            },
        )
        .await
        .expect_err("a confined cell sleeping past its limit must fail");
    assert!(
        started_at.elapsed() < std::time::Duration::from_secs(10),
        "the failure must arrive near the limit"
    );
    let msg = err.to_string();
    assert!(msg.contains("timed out"), "{msg}");
    assert!(
        msg.contains(sleeper.as_str()) && !msg.contains("--ro-bind"),
        "the error names the cell as written, not the sandbox wrapper: {msg}"
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-exec/src/dag.rs](../../../mechanisms/crates/hick-exec/src/dag.rs.md#output-e6f480388088)
- [crates/hick-literate/src/cell_timeout.rs](contract.md#output-c44f2255b447)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-executor-canopy/src/executor.rs](../../../mechanisms/crates/hickory-executor-canopy/src/executor.rs.md#output-47988291742d)
- [crates/hickory-executor-docker/src/lib.rs](../../../mechanisms/crates/hickory-executor-docker/src/lib.rs.md#output-772796e266eb)
- [crates/hickory-executor-sandbox/src/lib.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/lib.rs.md#output-3ed8f409da24)
- [crates/hickory-executor/src/lib.rs](../../../mechanisms/crates/hickory-executor/src/lib.rs.md#output-87ce480a6eab)

### Tests and fixtures

- [crates/hick-exec/src/dag/tests/tests.rs](../../../mechanisms/crates/hick-exec/src/dag/tests/tests.rs.md#output-cc538e30a76b)
- [crates/hick-literate/tests/cell_timeout_tests.rs](contract.md#output-8696e0c20596)
- [crates/hickory-cli/tests/cell_timeout_env.rs](contract.md#output-295eca818e03)
- [crates/hickory-executor-sandbox/tests/confinement.rs](../../../mechanisms/crates/hickory-executor-sandbox/tests/confinement.rs.md#output-c8a04313c63c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
