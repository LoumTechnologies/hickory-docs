<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/src/harness.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `db7f85dd8b72ddfec86f3f2b7c0e09d5a8d178c97cb7c49bedfd0a6f793c3445`.

<book:entry path="crates/hickory-agent/src/harness.rs">
//! Offline reporting for historical token-economics measurements.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::usage::Usage;

/// One JSONL row. Per-turn rows carry `turn`; the task-summary row carries
/// `turns`/`completed`/`check_pass` instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub experiment: String,
    pub arm: String,
    #[serde(default)]
    pub baseline: bool,
    pub task_id: String,
    pub model: String,
    /// Per-turn rows: which LLM turn this usage belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<usize>,
    /// The four-way token split for this row.
    pub usage: Usage,
    /// USD from real usage (`None` = model not in the price table).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    /// Summary rows: total LLM turns used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turns: Option<usize>,
    /// Summary rows: did the agent produce a final answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    /// Summary rows: did `check_cmd` exit 0 (None = no check configured).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_pass: Option<bool>,
    /// Whether the arm asked for tools and whether the loop ran them.
    #[serde(default)]
    pub tools_requested: bool,
    #[serde(default)]
    pub tools_active: bool,
    /// RFC3339 wall-clock of the row (in the JSONL only — never in a
    /// prompt, where it would invalidate the cache prefix).
    pub at: String,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Builds the LLM client for an arm. Live runs construct an
/// `AnthropicClient` with the arm's model/effort; offline tests hand back
/// Per-arm aggregate for the report.
#[derive(Debug, Default, Clone)]
struct ArmStats {
    baseline: bool,
    costs: Vec<f64>,
    turns: Vec<f64>,
    checks_pass: usize,
    checks_total: usize,
    completed: usize,
    tasks: usize,
    usage: Usage,
    tools_requested_inactive: bool,
}

fn median(sorted: &[f64]) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let mid = sorted.len() / 2;
    Some(if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    })
}

fn p90(sorted: &[f64]) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let idx = ((sorted.len() as f64) * 0.9).ceil() as usize;
    Some(sorted[idx.saturating_sub(1).min(sorted.len() - 1)])
}

fn fmt_opt(v: Option<f64>, precision: usize) -> String {
    v.map(|x| format!("{x:.precision$}"))
        .unwrap_or_else(|| "—".into())
}

/// Read every `*.jsonl` under `runs_dir` and render the Markdown report.
/// Deterministic given the same rows (BTreeMap ordering throughout).
pub fn generate_report(runs_dir: &Path) -> Result<String> {
    let mut records: Vec<RunRecord> = Vec::new();
    if runs_dir.is_dir() {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(runs_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
            .collect();
        paths.sort();
        for path in paths {
            for line in std::fs::read_to_string(&path)?.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let rec: RunRecord = serde_json::from_str(line)
                    .with_context(|| format!("bad record in {}", path.display()))?;
                records.push(rec);
            }
        }
    }

    // experiment -> arm -> stats, from task-summary rows only.
    let mut experiments: BTreeMap<String, BTreeMap<String, ArmStats>> = BTreeMap::new();
    for rec in records.iter().filter(|r| r.turn.is_none()) {
        let stats = experiments
            .entry(rec.experiment.clone())
            .or_default()
            .entry(rec.arm.clone())
            .or_default();
        stats.baseline |= rec.baseline;
        stats.tasks += 1;
        if let Some(c) = rec.cost_usd {
            stats.costs.push(c);
        }
        if let Some(t) = rec.turns {
            stats.turns.push(t as f64);
        }
        if rec.completed == Some(true) {
            stats.completed += 1;
        }
        if let Some(pass) = rec.check_pass {
            stats.checks_total += 1;
            if pass {
                stats.checks_pass += 1;
            }
        }
        stats.usage.add(&rec.usage);
        stats.tools_requested_inactive |= rec.tools_requested && !rec.tools_active;
    }

    let mut out = String::new();
    let _ = writeln!(out, "# Token economics — results\n");
    let _ = writeln!(
        out,
        "Generated by `just tokens-report` from `experiments/token-economics/runs/*.jsonl`.\n\
         Do not edit by hand. Design: `docs/specs/freeform/token-economics.md`.\n"
    );
    let _ = writeln!(
        out,
        "Ground rules: costs come from real `usage` split four ways; medians and p90, \
         never a single mean; **honest-result rule** — if an arm shows no benefit, this \
         report says so plainly.\n"
    );

    if experiments.is_empty() {
        let _ = writeln!(
            out,
            "_No runs recorded yet._ Live runs need `ANTHROPIC_API_KEY`; see the \
             Implementation section of the design doc for the exact commands."
        );
        return Ok(out);
    }

    for (experiment, arms) in &experiments {
        let _ = writeln!(out, "## {experiment}\n");
        let _ = writeln!(
            out,
            "| arm | tasks | completed | median cost (USD) | p90 cost (USD) | median turns | check-pass rate |"
        );
        let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|---:|");
        for (arm, stats) in arms {
            let mut costs = stats.costs.clone();
            costs.sort_by(|a, b| a.total_cmp(b));
            let mut turns = stats.turns.clone();
            turns.sort_by(|a, b| a.total_cmp(b));
            let pass_rate = if stats.checks_total > 0 {
                format!(
                    "{}/{} ({:.0}%)",
                    stats.checks_pass,
                    stats.checks_total,
                    100.0 * stats.checks_pass as f64 / stats.checks_total as f64
                )
            } else {
                "—".into()
            };
            let marker = if stats.baseline { " (baseline)" } else { "" };
            let _ = writeln!(
                out,
                "| {arm}{marker} | {} | {} | {} | {} | {} | {pass_rate} |",
                stats.tasks,
                stats.completed,
                fmt_opt(median(&costs), 4),
                fmt_opt(p90(&costs), 4),
                fmt_opt(median(&turns), 1),
            );
        }

        let _ = writeln!(out, "\n### Four-way token split\n");
        let _ = writeln!(
            out,
            "| arm | input | cache write (1.25x) | cache read (~0.1x) | output |"
        );
        let _ = writeln!(out, "|---|---:|---:|---:|---:|");
        for (arm, stats) in arms {
            let u = &stats.usage;
            let _ = writeln!(
                out,
                "| {arm} | {} | {} | {} | {} |",
                u.input_tokens,
                u.cache_creation_input_tokens,
                u.cache_read_input_tokens,
                u.output_tokens
            );
        }

        // Verdicts vs the baseline arm (honest-result rule).
        if let Some((base_name, base)) = arms.iter().find(|(_, s)| s.baseline) {
            let mut base_costs = base.costs.clone();
            base_costs.sort_by(|a, b| a.total_cmp(b));
            if let Some(base_median) = median(&base_costs) {
                let _ = writeln!(out, "\n### Verdicts (vs `{base_name}`)\n");
                for (arm, stats) in arms.iter().filter(|(n, _)| *n != base_name) {
                    let mut costs = stats.costs.clone();
                    costs.sort_by(|a, b| a.total_cmp(b));
                    match median(&costs) {
                        Some(m) if base_median > 0.0 => {
                            let reduction = 100.0 * (base_median - m) / base_median;
                            let base_pass = pass_fraction(base);
                            let arm_pass = pass_fraction(stats);
                            let quality_ok = match (arm_pass, base_pass) {
                                (Some(a), Some(b)) => a >= b,
                                _ => true,
                            };
                            if reduction >= 25.0 && quality_ok {
                                let _ = writeln!(
                                    out,
                                    "- **{arm}**: {reduction:.0}% median cost reduction with \
                                     equal-or-better check-pass rate — the threshold \
                                     (≥25%) is met."
                                );
                            } else {
                                let _ = writeln!(
                                    out,
                                    "- **{arm}**: NO significant benefit over the baseline \
                                     ({reduction:.0}% median cost change{}). Reported \
                                     honestly per the design's honest-result rule.",
                                    if quality_ok {
                                        ""
                                    } else {
                                        "; check-pass rate is WORSE than baseline"
                                    }
                                );
                            }
                        }
                        _ => {
                            let _ =
                                writeln!(out, "- **{arm}**: not enough cost data for a verdict.");
                        }
                    }
                }
            }
        } else {
            let _ = writeln!(
                out,
                "\n_No baseline arm in this experiment; per-arm numbers only._"
            );
        }

        if arms.values().any(|s| s.tools_requested_inactive) {
            let _ = writeln!(
                out,
                "\n> Caveat: at least one arm requested the document tool set but ran \
                 script-only (the tool-enabled loop was not wired into the harness for \
                 these rows) — treat tools-vs-script comparisons above as PENDING, not \
                 as a result."
            );
        }
        let _ = writeln!(out);
    }
    Ok(out)
}

fn pass_fraction(stats: &ArmStats) -> Option<f64> {
    (stats.checks_total > 0).then(|| stats.checks_pass as f64 / stats.checks_total as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_and_p90() {
        assert_eq!(median(&[1.0, 2.0, 3.0]), Some(2.0));
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
        assert_eq!(median(&[]), None);
        assert_eq!(
            p90(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]),
            Some(9.0)
        );
    }

    #[test]
    fn empty_runs_dir_reports_no_runs() {
        let dir = tempfile::tempdir().unwrap();
        let report = generate_report(dir.path()).unwrap();
        assert!(report.contains("No runs recorded yet"));
    }

    #[test]
    fn report_states_no_benefit_plainly() {
        let dir = tempfile::tempdir().unwrap();
        let mut lines = String::new();
        // Baseline cheaper than the challenger: the verdict must say NO benefit.
        for (arm, baseline, cost) in [
            ("script-only", true, 0.010),
            ("script-only", true, 0.012),
            ("tools", false, 0.011),
            ("tools", false, 0.013),
        ] {
            let rec = RunRecord {
                experiment: "E1".into(),
                arm: arm.into(),
                baseline,
                task_id: format!("t-{cost}"),
                model: "claude-sonnet-5".into(),
                turn: None,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 10,
                    ..Default::default()
                },
                cost_usd: Some(cost),
                turns: Some(3),
                completed: Some(true),
                check_pass: Some(true),
                tools_requested: arm == "tools",
                tools_active: false,
                at: "2026-08-06T00:00:00Z".into(),
            };
            lines.push_str(&serde_json::to_string(&rec).unwrap());
            lines.push('\n');
        }
        std::fs::write(dir.path().join("e1.jsonl"), lines).unwrap();
        let report = generate_report(dir.path()).unwrap();
        assert!(report.contains("NO significant benefit"), "{report}");
        assert!(report.contains("(baseline)"));
        assert!(report.contains("cache read"));
        assert!(
            report.contains("PENDING"),
            "tools-inactive caveat missing:\n{report}"
        );
    }
}
</book:entry>

</book:doc>
