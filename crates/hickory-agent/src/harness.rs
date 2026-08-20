//! Token-economics measurement harness (docs/specs/freeform/token-economics.md).
//!
//! Executes a committed task list against configurable ARMS (tools vs
//! script-only, doc-first vs outputs-first context, effort levels),
//! recording per-task per-turn [`Usage`] as JSONL under
//! `experiments/token-economics/runs/`, and regenerates the report
//! (`just tokens-report`).
//!
//! Ground rules enforced here:
//! - cost comes from real `usage`, split four ways — never a single total;
//! - `count_tokens` is the only static measuring instrument;
//! - medians + p90, never a single mean;
//! - the honest-result rule: when an arm shows no benefit against its
//!   baseline, the report says so in plain words.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::events::AgentEvent;
use crate::llm::LlmClient;
use crate::react_loop::{AgentConfig, run_agent};
use crate::usage::Usage;
use hickory_executor::Executor;

// ---------------------------------------------------------------------------
// Specs (committed under experiments/token-economics/tasks/)
// ---------------------------------------------------------------------------

/// One experiment: a fixed task list run identically under every arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSpec {
    /// Experiment id (e.g. "E1-edit-surface").
    pub experiment: String,
    /// One-line description for the report.
    #[serde(default)]
    pub description: String,
    /// Directory (relative to the spec file) holding the fixed seed corpus.
    /// It is copied fresh into the work dir for EVERY task, so tasks never
    /// see each other's edits; `{corpus}` in prompts and `check_cmd`
    /// expands to the staged copy's absolute path.
    #[serde(default)]
    pub corpus: Option<PathBuf>,
    /// The arms to compare. Exactly one may set `baseline: true`.
    pub arms: Vec<ArmSpec>,
    /// The fixed task list (same list, same order, for every arm).
    pub tasks: Vec<TaskSpec>,
}

/// One arm: the single variable under test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmSpec {
    /// Arm name (e.g. "tools", "script-only", "effort-low").
    pub name: String,
    /// Whether this arm is the baseline other arms are compared against.
    #[serde(default)]
    pub baseline: bool,
    /// Enable the document tool set (read_doc/read_output/edit_output/
    /// edit_doc/verify): the runner opens an `EditSession` on the task's
    /// staged doc (`AgentConfig::doc_path`). Tasks without a `doc` fall
    /// back to script-only and record `tools_active: false` so the row
    /// stays honest.
    #[serde(default)]
    pub tools: bool,
    /// Which context the agent starts with: "doc" (the .hick source),
    /// "outputs" (the woven files), or "none".
    #[serde(default)]
    pub context: Option<String>,
    /// `output_config.effort` for this arm (low|medium|high|xhigh|max);
    /// omitted = API default.
    #[serde(default)]
    pub effort: Option<String>,
    /// Model override; omitted = the client's default.
    #[serde(default)]
    pub model: Option<String>,
}

/// One task, fixed across arms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Stable task id (rows join on it across runs and dates).
    pub id: String,
    /// The user prompt.
    pub prompt: String,
    /// Path (relative to the spec file) of the primary .hick document.
    #[serde(default)]
    pub doc: Option<PathBuf>,
    /// Paths (relative to the spec file) of woven outputs, for
    /// outputs-first arms.
    #[serde(default)]
    pub outputs: Vec<PathBuf>,
    /// Shell command run after the task; exit 0 = check pass (e.g.
    /// `cargo run -p hickory-cli -- check doc.hick`). Run from the spec
    /// file's directory.
    #[serde(default)]
    pub check_cmd: Option<String>,
    /// Max LLM turns (default 20).
    #[serde(default)]
    pub max_turns: Option<usize>,
}

impl ExperimentSpec {
    /// Load a spec from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("bad spec {}", path.display()))
    }
}

// ---------------------------------------------------------------------------
// Run records (JSONL under experiments/token-economics/runs/)
// ---------------------------------------------------------------------------

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
/// a `ScriptedLlmClient`.
pub type LlmFactory<'a> = dyn Fn(&ArmSpec) -> Result<Arc<dyn LlmClient>> + 'a;

/// Run every (arm x task) of `spec`, appending JSONL rows to `out_path`.
/// `spec_dir` anchors relative `doc`/`outputs`/`check_cmd` paths;
/// `work_dir` hosts agent session files and script workspaces.
pub async fn run_experiment(
    spec: &ExperimentSpec,
    spec_dir: &Path,
    work_dir: &Path,
    llm_factory: &LlmFactory<'_>,
    executor: Arc<dyn Executor>,
    out_path: &Path,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_path)?;
    let file = Mutex::new(file);
    let write_record = |r: &RunRecord| -> Result<()> {
        let mut f = file.lock().unwrap();
        serde_json::to_writer(&mut *f, r)?;
        writeln!(f)?;
        Ok(())
    };

    for arm in &spec.arms {
        let llm = llm_factory(arm)?;
        for task in &spec.tasks {
            // Stage a fresh corpus copy per task: no cross-task pollution.
            let (context_dir, corpus_token) = match &spec.corpus {
                Some(rel) => {
                    let staged = work_dir.join(format!("{}-{}", arm.name, task.id));
                    copy_dir(&spec_dir.join(rel), &staged)?;
                    let token = staged.display().to_string();
                    (staged, token)
                }
                None => (spec_dir.to_path_buf(), spec_dir.display().to_string()),
            };
            let prompt = task.prompt.replace("{corpus}", &corpus_token);

            let mut config = AgentConfig::new(prompt, work_dir);
            if let Some(max) = task.max_turns {
                config.max_turns = max;
            }
            config.doc_context = build_context(arm, task, &context_dir)?;

            // Tools arms open an EditSession on the task's staged document
            // (read_doc/read_output/edit_output/edit_doc/verify enabled).
            let doc_abs = task.doc.as_ref().map(|d| context_dir.join(d));
            let tools_active = arm.tools && doc_abs.is_some();
            if arm.tools {
                if let Some(doc) = &doc_abs {
                    config.doc_path = Some(doc.clone());
                } else {
                    log::warn!(
                        "arm '{}' requests tools but task '{}' has no doc — running \
                         script-only and recording tools_active=false",
                        arm.name,
                        task.id
                    );
                }
            }

            let turn_rows: Mutex<Vec<(usize, Usage, Option<f64>)>> = Mutex::new(Vec::new());
            let mut on_event = |event: AgentEvent| {
                if let AgentEvent::TurnUsage {
                    turn,
                    usage,
                    cost_usd,
                    ..
                } = event
                {
                    turn_rows.lock().unwrap().push((turn, usage, cost_usd));
                }
            };

            let outcome = run_agent(llm.as_ref(), executor.clone(), &config, &mut on_event).await;
            let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let base = RunRecord {
                experiment: spec.experiment.clone(),
                arm: arm.name.clone(),
                baseline: arm.baseline,
                task_id: task.id.clone(),
                model: llm.model_name().to_string(),
                turn: None,
                usage: Usage::default(),
                cost_usd: None,
                turns: None,
                completed: None,
                check_pass: None,
                tools_requested: arm.tools,
                tools_active,
                at: now,
            };

            for (turn, usage, cost) in turn_rows.lock().unwrap().iter() {
                write_record(&RunRecord {
                    turn: Some(*turn),
                    usage: *usage,
                    cost_usd: *cost,
                    ..base.clone()
                })?;
            }

            let (completed, turns, total_usage, total_cost) = match &outcome {
                Ok(o) => (true, o.turns, o.total_usage, o.total_cost_usd),
                Err(_) => {
                    let rows = turn_rows.lock().unwrap();
                    let mut u = Usage::default();
                    let mut c = 0.0;
                    let mut any_cost = false;
                    for (_, usage, cost) in rows.iter() {
                        u.add(usage);
                        if let Some(x) = cost {
                            c += x;
                            any_cost = true;
                        }
                    }
                    (false, rows.len(), u, any_cost.then_some(c))
                }
            };
            let check_pass = match (&outcome, &task.check_cmd) {
                (Ok(_), Some(cmd)) => {
                    let cmd = cmd.replace("{corpus}", &corpus_token);
                    Some(run_check(&cmd, &context_dir)?)
                }
                (Err(_), Some(_)) => Some(false),
                (_, None) => None,
            };
            write_record(&RunRecord {
                usage: total_usage,
                cost_usd: total_cost,
                turns: Some(turns),
                completed: Some(completed),
                check_pass,
                ..base
            })?;
        }
    }
    Ok(())
}

/// Assemble the arm's starting context from the task's files.
fn build_context(arm: &ArmSpec, task: &TaskSpec, spec_dir: &Path) -> Result<Option<String>> {
    match arm.context.as_deref() {
        Some("doc") => {
            let Some(doc) = &task.doc else {
                return Ok(None);
            };
            let path = spec_dir.join(doc);
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("task {}: cannot read doc {}", task.id, path.display()))?;
            Ok(Some(text))
        }
        Some("outputs") => {
            if task.outputs.is_empty() {
                return Ok(None);
            }
            let mut ctx = String::new();
            for rel in &task.outputs {
                let path = spec_dir.join(rel);
                let text = std::fs::read_to_string(&path).with_context(|| {
                    format!("task {}: cannot read output {}", task.id, path.display())
                })?;
                let _ = writeln!(ctx, "### {}\n\n{text}\n", rel.display());
            }
            Ok(Some(ctx))
        }
        _ => Ok(None),
    }
}

/// Recursively copy `from` into `to` (files + directories only).
fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)
        .with_context(|| format!("cannot read corpus dir {}", from.display()))?
    {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Run a task's check command from `dir`; exit 0 = pass.
///
/// The HOST's shell, through the one place that knows which that is. This was
/// a hardcoded `sh`, which does not exist on a Windows machine — the check
/// could not spawn, so a task could never be judged to pass there.
fn run_check(cmd: &str, dir: &Path) -> Result<bool> {
    let status = hickory_executor::host_shell_command(cmd)
        .current_dir(dir)
        .status()
        .with_context(|| format!("check command failed to spawn: {cmd}"))?;
    Ok(status.success())
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

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

    #[test]
    fn spec_round_trips() {
        let json = r#"{
            "experiment": "E4-effort",
            "arms": [
                {"name": "effort-low", "effort": "low"},
                {"name": "effort-high", "baseline": true}
            ],
            "tasks": [
                {"id": "t1", "prompt": "do the thing", "check_cmd": "true"}
            ]
        }"#;
        let spec: ExperimentSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.arms.len(), 2);
        assert!(spec.arms[1].baseline);
        assert_eq!(spec.tasks[0].check_cmd.as_deref(), Some("true"));
    }
}
