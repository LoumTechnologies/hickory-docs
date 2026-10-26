<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-profile/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-412a741fe5c5"></a>
## `crates/hick-profile/src/lib.rs`

<a id="f-412a741fe5c5-chrome"></a>
### `chrome`

Profiler evidence. Sampling weights are never call counts or per-line costs.

<storycode:copy id="f-412a741fe5c5-chrome">
//! Profiler evidence. Sampling weights are never call counts or per-line costs.
mod chrome;
</storycode:copy>
<a id="f-412a741fe5c5-firefox"></a>
### `firefox`

<storycode:copy id="f-412a741fe5c5-firefox">
mod firefox;
</storycode:copy>
<a id="f-412a741fe5c5-speedscope"></a>
### `speedscope`

<storycode:copy id="f-412a741fe5c5-speedscope">
mod speedscope;
</storycode:copy>
<a id="f-412a741fe5c5-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-412a741fe5c5-use_declaration">
use anyhow::{Context, Result, bail, ensure};
</storycode:copy>
<a id="f-412a741fe5c5-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-412a741fe5c5-use_declaration-2">
use serde::{Deserialize, Serialize};
</storycode:copy>
<a id="f-412a741fe5c5-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-412a741fe5c5-use_declaration-3">
use serde_json::Value;
</storycode:copy>
<a id="f-412a741fe5c5-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-412a741fe5c5-use_declaration-4">
use std::collections::{BTreeMap, BTreeSet};

</storycode:copy>
<a id="f-412a741fe5c5-Frame"></a>
### `Frame`

<storycode:copy id="f-412a741fe5c5-Frame">
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
</storycode:copy>
<a id="f-412a741fe5c5-Row"></a>
### `Row`

<storycode:copy id="f-412a741fe5c5-Row">
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
</storycode:copy>
<a id="f-412a741fe5c5-Thread"></a>
### `Thread`

<storycode:copy id="f-412a741fe5c5-Thread">
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
</storycode:copy>
<a id="f-412a741fe5c5-Report"></a>
### `Report`

<storycode:copy id="f-412a741fe5c5-Report">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub format: String,
    pub threads: Vec<Thread>,
    pub raw: String,
    pub limitations: Vec<String>,
}
</storycode:copy>
<a id="f-412a741fe5c5-parse"></a>
### `parse`

<storycode:copy id="f-412a741fe5c5-parse">
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
</storycode:copy>
<a id="f-412a741fe5c5-finite"></a>
### `finite`

<storycode:copy id="f-412a741fe5c5-finite">
pub(crate) fn finite(value: &Value) -> Result<f64> {
    let n = value.as_f64().context("Missing numeric measurement")?;
    ensure!(
        n.is_finite() && n >= 0.,
        "Invalid negative/nonfinite measurement"
    );
    Ok(n)
}
</storycode:copy>
<a id="f-412a741fe5c5-number"></a>
### `number`

<storycode:copy id="f-412a741fe5c5-number">
pub(crate) fn number(value: &Value) -> Result<usize> {
    usize::try_from(value.as_u64().context("Invalid frame/sample index")?)
        .context("Frame index exceeds platform limits")
}
</storycode:copy>
<a id="f-412a741fe5c5-frame"></a>
### `frame`

<storycode:copy id="f-412a741fe5c5-frame">
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
</storycode:copy>
<a id="f-412a741fe5c5-add"></a>
### `add`

<storycode:copy id="f-412a741fe5c5-add">
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
</storycode:copy>
<a id="f-412a741fe5c5-unit"></a>
### `unit`

<storycode:copy id="f-412a741fe5c5-unit">
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
</storycode:copy>
<a id="f-412a741fe5c5-cprofile"></a>
### `cprofile`

<storycode:copy id="f-412a741fe5c5-cprofile">
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
</storycode:copy>

### Assemble `crates/hick-profile/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-profile/src/lib.rs"><storycode:paste select="#f-412a741fe5c5-chrome"/><storycode:paste select="#f-412a741fe5c5-firefox"/><storycode:paste select="#f-412a741fe5c5-speedscope"/><storycode:paste select="#f-412a741fe5c5-use_declaration"/><storycode:paste select="#f-412a741fe5c5-use_declaration-2"/><storycode:paste select="#f-412a741fe5c5-use_declaration-3"/><storycode:paste select="#f-412a741fe5c5-use_declaration-4"/><storycode:paste select="#f-412a741fe5c5-Frame"/><storycode:paste select="#f-412a741fe5c5-Row"/><storycode:paste select="#f-412a741fe5c5-Thread"/><storycode:paste select="#f-412a741fe5c5-Report"/><storycode:paste select="#f-412a741fe5c5-parse"/><storycode:paste select="#f-412a741fe5c5-finite"/><storycode:paste select="#f-412a741fe5c5-number"/><storycode:paste select="#f-412a741fe5c5-frame"/><storycode:paste select="#f-412a741fe5c5-add"/><storycode:paste select="#f-412a741fe5c5-unit"/><storycode:paste select="#f-412a741fe5c5-cprofile"/></storycode:file>

</storycode:doc>
