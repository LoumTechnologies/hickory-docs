//! Profiler evidence. Sampling weights are never call counts or per-line costs.
mod chrome;
mod firefox;
mod speedscope;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub name: String,
    pub file: Option<String>,
    /// One-based provider location; None means no source location was supplied.
    pub line: Option<u32>,
    pub column: Option<u32>,
    #[serde(default)]
    pub script_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub frame: Frame,
    pub self_weight: f64,
    /// Inclusive costs overlap across functions. Never sum them as a denominator.
    pub inclusive_weight: f64,
    pub samples: Option<u64>,
    pub total_calls: Option<u64>,
    pub primitive_calls: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub name: String,
    pub unit: String,
    pub aggregation: String,
    pub interval_unit: String,
    pub clock: String,
    pub interval_start: f64,
    pub interval_end: f64,
    pub measured_weight: f64,
    pub rows: Vec<Row>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub format: String,
    pub threads: Vec<Thread>,
    pub raw: String,
    pub limitations: Vec<String>,
}
pub fn parse(format: &str, raw: &str) -> Result<Report> {
    ensure!(raw.len() <= 32 * 1024 * 1024, "Profile exceeds 32 MiB");
    let value: Value = serde_json::from_str(raw)?;
    let (threads,limitations)=match format {
        "chrome-cpu" => (chrome::parse(value.get("profile").unwrap_or(&value))?,vec!["CPU sampling weights; call counts unavailable; native/unmapped functions retained".into()]),
        "speedscope" => (speedscope::parse(&value)?,vec!["Per-thread/profile weights; inclusive costs overlap; call counts unavailable".into()]),
        "firefox" => (firefox::parse(&value)?,vec!["Processed/symbolicated Firefox profiles; native addresses without source metadata stay unresolved".into()]),
        "cprofile-json" => (cprofile(&value)?,vec!["Deterministic function aggregates include instrumentation overhead; cumulative costs overlap".into()]),
        _ => bail!("Unsupported profile format: {format}"),
    };
    ensure!(!threads.is_empty(), "Profile contains no measured scopes");
    Ok(Report {
        format: format.into(),
        threads,
        raw: raw.into(),
        limitations,
    })
}
pub(crate) fn finite(value: &Value) -> Result<f64> {
    let n = value.as_f64().context("Missing numeric measurement")?;
    ensure!(
        n.is_finite() && n >= 0.,
        "Invalid negative/nonfinite measurement"
    );
    Ok(n)
}
pub(crate) fn number(value: &Value) -> Result<usize> {
    usize::try_from(value.as_u64().context("Invalid frame/sample index")?)
        .context("Frame index exceeds platform limits")
}
pub(crate) fn frame(v: &Value) -> Frame {
    Frame {
        name: v["name"].as_str().unwrap_or("(anonymous)").into(),
        file: v["file"].as_str().map(str::to_string),
        line: v["line"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n > 0),
        column: v["col"].as_u64().and_then(|n| u32::try_from(n).ok()),
        script_id: None,
    }
}
pub(crate) fn add(rows: &mut BTreeMap<Frame, Row>, stack: &[Frame], weight: f64, sampled: bool) {
    let leaf = stack.last();
    for frame in stack.iter().collect::<BTreeSet<_>>() {
        let row = rows.entry(frame.clone()).or_insert_with(|| Row {
            frame: frame.clone(),
            self_weight: 0.,
            inclusive_weight: 0.,
            samples: sampled.then_some(0),
            total_calls: None,
            primitive_calls: None,
        });
        row.inclusive_weight += weight;
        if Some(frame) == leaf {
            row.self_weight += weight;
        }
        if let Some(count) = &mut row.samples {
            *count += 1;
        }
    }
}
pub(crate) fn unit(raw: &str) -> Result<(&'static str, f64)> {
    Ok(match raw {
        "seconds" => ("seconds", 1.),
        "milliseconds" => ("seconds", 1e-3),
        "microseconds" => ("seconds", 1e-6),
        "nanoseconds" => ("seconds", 1e-9),
        "bytes" => ("bytes", 1.),
        "none" => ("samples", 1.),
        _ => bail!("Unsupported profile unit {raw}"),
    })
}
fn cprofile(value: &Value) -> Result<Vec<Thread>> {
    let elapsed = finite(&value["elapsed_seconds"])?;
    let mut rows = Vec::new();
    for f in value["functions"]
        .as_array()
        .context("Missing cProfile functions")?
    {
        let total = f["total_calls"]
            .as_u64()
            .context("Missing total call count")?;
        let primitive = f["primitive_calls"]
            .as_u64()
            .context("Missing primitive call count")?;
        ensure!(primitive <= total, "Primitive calls exceed total calls");
        let self_weight = finite(&f["self_seconds"])?;
        let inclusive_weight = finite(&f["cumulative_seconds"])?;
        ensure!(
            self_weight <= inclusive_weight + 1e-9,
            "Self cost exceeds inclusive cost"
        );
        rows.push(Row {
            frame: frame(f),
            self_weight,
            inclusive_weight,
            samples: None,
            total_calls: Some(total),
            primitive_calls: Some(primitive),
        });
    }
    Ok(vec![Thread {
        name: value["scope"]
            .as_str()
            .unwrap_or("main-thread cProfile")
            .into(),
        unit: "seconds".into(),
        aggregation: "deterministic function aggregates".into(),
        interval_unit: "seconds".into(),
        clock: "provider-monotonic elapsed".into(),
        interval_start: 0.,
        interval_end: elapsed,
        measured_weight: rows.iter().map(|r| r.self_weight).sum(),
        rows,
    }])
}
