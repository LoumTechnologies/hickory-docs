//! The attributes a cell declares about *how* it is run, and their parsing.
//!
//! Three of them share one shape: absent means "inherit the run-wide
//! default", and a value that is not one of the accepted spellings is an
//! error rather than a silent fallback. A typo in a switch that governs
//! verification — `freeze="yes"`, `tier="pre-push"`, `timeout="2m"` — must
//! never quietly turn the thing off, because the run then goes green for a
//! reason nobody can see.
//!
//! They live here rather than in `dag.rs` because parsing them is not DAG
//! construction: `dag.rs` decides what depends on what, and these decide what
//! happens to one vertex once the graph says it is that vertex's turn.

use hick_lang::HickTag;

use crate::dag::DagValidationError;

/// When a cell is meant to run: `tier="commit"` or `tier="push"`.
///
/// A document's cells are not all worth the same wait. A cell that runs the
/// full test suite is worth thirty seconds before a push and is not worth
/// thirty seconds before every commit; the tangle and the weave of the same
/// document are worth checking on both. `tier` is where the document says
/// which of the two a cell is, so that the gate — not the author, and not a
/// `.gitignore`-shaped list kept somewhere else — can act on it.
///
/// The ordering is deliberate and load-bearing: `Commit < Push`, so a run
/// admitting tier *g* executes exactly the cells whose tier is `<= g`. See
/// [`CellTier::runs_at`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CellTier {
    /// The default: this cell runs everywhere, including under
    /// `hick test --tier commit`.
    #[default]
    Commit,
    /// This cell runs on `hick test` and `hick run`, and is skipped under
    /// `--tier commit` unless a recording can answer it.
    Push,
}

impl CellTier {
    /// The attribute spelling, which is also what a report prints.
    pub fn as_str(self) -> &'static str {
        match self {
            CellTier::Commit => "commit",
            CellTier::Push => "push",
        }
    }

    /// Whether a run admitting cells up to `gate` executes this cell.
    pub fn runs_at(self, gate: CellTier) -> bool {
        self <= gate
    }
}

impl std::fmt::Display for CellTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parse a `tier="…"` attribute into the cell's tier.
///
/// Absent means [`CellTier::Commit`] — a cell says nothing about tiers until
/// it is expensive enough to be worth saying something, and the safe default
/// for everything else is "checked at every gate". Anything other than the
/// two names is rejected, for the same reason a bad `freeze` is: a tier
/// nobody recognises would otherwise silently become the tier that runs
/// least.
pub fn parse_tier(tag: &HickTag) -> Result<CellTier, DagValidationError> {
    match tag.get_attribute("tier") {
        None => Ok(CellTier::Commit),
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "commit" => Ok(CellTier::Commit),
            "push" => Ok(CellTier::Push),
            _ => Err(DagValidationError::InvalidTier {
                line: tag.source_line,
                value: raw.to_string(),
            }),
        },
    }
}

/// Parse a `freeze="…"` attribute into a per-cell freeze declaration.
///
/// Absent means "inherit the run-wide default"; anything other than an exact
/// `true`/`false` is rejected rather than silently treated as false, because a
/// typo in a verification switch that quietly disables verification is the
/// worst possible failure mode for this attribute.
pub fn parse_freeze(tag: &HickTag) -> Result<Option<bool>, DagValidationError> {
    match tag.get_attribute("freeze") {
        None => Ok(None),
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err(DagValidationError::InvalidFreeze {
                line: tag.source_line,
                value: raw.to_string(),
            }),
        },
    }
}

/// Parse a `timeout="…"` attribute into a per-cell limit in whole seconds.
///
/// Absent means "inherit the run-wide default"; `0` means "this cell runs
/// unbounded" — allowed, but only as an explicit declaration. Anything that
/// is not a whole number of seconds is rejected rather than silently
/// ignored, because a typo in a time limit that quietly removes the limit
/// is precisely the hang this attribute exists to prevent.
pub fn parse_timeout(tag: &HickTag) -> Result<Option<u64>, DagValidationError> {
    match tag.get_attribute("timeout") {
        None => Ok(None),
        Some(raw) => match raw.trim().parse::<u64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => Err(DagValidationError::InvalidTimeout {
                line: tag.source_line,
                value: raw.to_string(),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::dag::{DagValidationError, FlowDag, build_dag};

    use super::CellTier;

    fn parse_and_build(src: &str) -> Result<FlowDag, DagValidationError> {
        let doc = hick_lang::parse(src).expect("parse failed");
        build_dag(&doc)
    }

    // The three tests below protect
    // docs/guarantees/verification/freeze-is-declared-per-cell.md — the
    // attribute-parsing half of it.

    #[test]
    fn freeze_attribute_parses_both_values() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="frozen" image="alpine" freeze="true">
cat lockfile
</hick:exec>
<hick:exec container="live" image="alpine" freeze="FALSE">
run the integration test
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs[0].freeze, Some(true));
        assert_eq!(dag.execs[1].freeze, Some(false));
    }

    #[test]
    fn freeze_attribute_defaults_to_inherit() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(
            dag.execs[0].freeze, None,
            "an absent attribute must mean 'inherit the run-wide default', \
             not 'live' — otherwise --freeze would stop working"
        );
    }

    #[test]
    fn freeze_attribute_rejects_non_boolean() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine" freeze="yes">
echo hello
</hick:exec>
</hick:doc>"#;
        let err = parse_and_build(src).expect_err("freeze=\"yes\" must not be accepted");
        assert!(
            matches!(err, DagValidationError::InvalidFreeze { ref value, .. } if value == "yes"),
            "unexpected error: {err}"
        );
        let msg = err.to_string();
        assert!(msg.contains("freeze=\"true\"") && msg.contains("freeze=\"false\""));
    }

    // The four tests below protect
    // docs/guarantees/verification/a-push-tier-cell-is-skipped-at-commit.md —
    // the attribute-parsing half of it.

    #[test]
    fn tier_attribute_parses_both_values() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="slow" image="alpine" tier="push">
cargo test --workspace
</hick:exec>
<hick:exec container="quick" image="alpine" tier="COMMIT">
cargo fmt --check
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs[0].tier, CellTier::Push);
        assert_eq!(
            dag.execs[1].tier,
            CellTier::Commit,
            "the value is read case-insensitively, exactly as freeze= is"
        );
    }

    #[test]
    fn tier_attribute_defaults_to_commit() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(
            dag.execs[0].tier,
            CellTier::Commit,
            "a cell that says nothing runs at every gate; the cheap default \
             must be the verifying one, never the skipping one"
        );
    }

    #[test]
    fn tier_attribute_rejects_any_other_value() {
        for bad in ["pre-push", "ci", "true", "", "PUSH!"] {
            let src = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine" tier="{bad}">
echo hello
</hick:exec>
</hick:doc>"#
            );
            let err = parse_and_build(&src).expect_err("an unknown tier must not be accepted");
            assert!(
                matches!(err, DagValidationError::InvalidTier { ref value, .. } if value == bad),
                "tier=\"{bad}\": unexpected error: {err}"
            );
            let msg = err.to_string();
            assert!(
                msg.contains("tier=\"commit\"") && msg.contains("tier=\"push\""),
                "the error must name both accepted values: {msg}"
            );
        }
    }

    #[test]
    fn a_commit_cell_runs_at_every_gate_and_a_push_cell_does_not() {
        assert!(CellTier::Commit.runs_at(CellTier::Commit));
        assert!(CellTier::Commit.runs_at(CellTier::Push));
        assert!(CellTier::Push.runs_at(CellTier::Push));
        assert!(
            !CellTier::Push.runs_at(CellTier::Commit),
            "the whole point: the commit gate does not execute a push cell"
        );
    }

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

    #[test]
    fn timeout_attribute_defaults_to_inherit() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(
            dag.execs[0].timeout_secs, None,
            "an absent attribute must mean 'inherit the run-wide default', \
             not 'unbounded' — otherwise the default timeout would stop working"
        );
    }

    #[test]
    fn timeout_attribute_rejects_non_integer_values() {
        for bad in ["1.5", "2m", "-1", "", "unbounded"] {
            let src = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine" timeout="{bad}">
echo hello
</hick:exec>
</hick:doc>"#
            );
            let err = parse_and_build(&src).expect_err("a malformed timeout must not be accepted");
            assert!(
                matches!(err, DagValidationError::InvalidTimeout { ref value, .. } if value == bad),
                "timeout=\"{bad}\": unexpected error: {err}"
            );
            let msg = err.to_string();
            assert!(
                msg.contains("timeout=\"0\"") && msg.contains("HICKORY_CELL_TIMEOUT"),
                "the error must name the valid shapes and the env default: {msg}"
            );
        }
    }
}
