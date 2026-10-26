<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-observe/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-022e74f036b0"></a>
## `crates/hick-observe/src/lib.rs`

Related contracts: [Runtime observations retain their producing revision and occurrence](../../../../guarantees/execution/revision-bound-runtime-observations/contract.md), [Browser-local prediction practice uses the shared gate](../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-022e74f036b0-prediction"></a>
### `prediction`

Revision-bound runtime evidence. No execution, Markdown writes, or policy selection.
Receive order is an observation order, never a cross-thread happens-before claim.

<storycode:copy id="f-022e74f036b0-prediction">
//! Revision-bound runtime evidence. No execution, Markdown writes, or policy selection.
//! Receive order is an observation order, never a cross-thread happens-before claim.
#[cfg(not(target_arch = "wasm32"))]
use anyhow::bail;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, ops::Range};

pub mod prediction;
</storycode:copy>
<a id="f-022e74f036b0-reading"></a>
### `reading`

<storycode:copy id="f-022e74f036b0-reading">
pub mod reading;
</storycode:copy>
<a id="f-022e74f036b0-store"></a>
### `store`

<storycode:copy id="f-022e74f036b0-store">
#[cfg(not(target_arch = "wasm32"))]
mod store;
</storycode:copy>
<a id="f-022e74f036b0-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-022e74f036b0-use_declaration">
#[cfg(not(target_arch = "wasm32"))]
pub use store::*;

</storycode:copy>
<a id="f-022e74f036b0-Source"></a>
### `Source`

<storycode:copy id="f-022e74f036b0-Source">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub id: String,
    pub uri: String,
    /// SHA-256 of the exact captured bytes; not a cache key or moving Git ref.
    pub revision: String,
    pub bytes: Vec<u8>,
}
</storycode:copy>
<a id="f-022e74f036b0-impl-Source"></a>
### `impl-Source`

<storycode:copy id="f-022e74f036b0-impl-Source">
impl Source {
    pub fn capture(id: String, uri: String, bytes: Vec<u8>) -> Self {
        Self {
            id,
            uri,
            revision: format!("{:x}", Sha256::digest(&bytes)),
            bytes,
        }
    }
}
</storycode:copy>
<a id="f-022e74f036b0-Precision"></a>
### `Precision`

<storycode:copy id="f-022e74f036b0-Precision">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    Exact,
    AdapterReported,
    BlockOnly,
    Unknown,
}
</storycode:copy>
<a id="f-022e74f036b0-Location"></a>
### `Location`

<storycode:copy id="f-022e74f036b0-Location">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub source: String,
    pub block: Option<String>,
    /// Zero-based byte range in captured source. Never inferred from output text.
    pub bytes: Option<Range<usize>>,
    pub precision: Precision,
}
</storycode:copy>
<a id="f-022e74f036b0-Clock"></a>
### `Clock`

<storycode:copy id="f-022e74f036b0-Clock">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Clock {
    pub domain: String,
    pub unit: String,
    pub value: f64,
}
</storycode:copy>
<a id="f-022e74f036b0-Interval"></a>
### `Interval`

<storycode:copy id="f-022e74f036b0-Interval">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Interval {
    pub start: Clock,
    pub end: Clock,
    pub scope: String,
}
</storycode:copy>
<a id="f-022e74f036b0-Handling"></a>
### `Handling`

<storycode:copy id="f-022e74f036b0-Handling">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Handling {
    Throwing,
    Caught,
    Unhandled,
    Unknown,
}
</storycode:copy>
<a id="f-022e74f036b0-FunctionAggregate"></a>
### `FunctionAggregate`

<storycode:copy id="f-022e74f036b0-FunctionAggregate">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionAggregate {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub primitive_calls: u64,
    pub total_calls: u64,
}
</storycode:copy>
<a id="f-022e74f036b0-Observation"></a>
### `Observation`

<storycode:copy id="f-022e74f036b0-Observation">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Observation {
    Output {
        stream: String,
        category: String,
        bytes: Vec<u8>,
    },
    Exception {
        type_name: String,
        message: String,
        handling: Handling,
        throwing: Option<Location>,
        catching: Option<Location>,
    },
    Log {
        level: String,
        message: String,
        fields: BTreeMap<String, serde_json::Value>,
    },
    Debugger {
        reason: String,
        details: serde_json::Value,
    },
    Profile {
        metric: String,
        value: f64,
        unit: String,
        interval: Interval,
        aggregation: String,
        samples: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        function: Option<FunctionAggregate>,
    },
}
</storycode:copy>
<a id="f-022e74f036b0-Event"></a>
### `Event`

<storycode:copy id="f-022e74f036b0-Event">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub provider: String,
    pub block: Option<String>,
    pub invocation: Option<String>,
    pub parent_invocation: Option<String>,
    pub thread: Option<String>,
    pub task: Option<String>,
    pub time: Option<Clock>,
    pub emission: Option<Location>,
    pub callers: Vec<Location>,
    pub observation: Observation,
}
</storycode:copy>
<a id="f-022e74f036b0-ObservationProvider"></a>
### `ObservationProvider`

Providers supply actual mapping evidence, typically via hick-dap::mapping::Mapping.
`None` from poll means currently drained, not successful run completion.

<storycode:copy id="f-022e74f036b0-ObservationProvider">
/// Providers supply actual mapping evidence, typically via hick-dap::mapping::Mapping.
/// `None` from poll means currently drained, not successful run completion.
pub trait ObservationProvider {
    fn id(&self) -> &str;
    fn poll(&mut self) -> Result<Option<Event>>;
}
</storycode:copy>
<a id="f-022e74f036b0-Status"></a>
### `Status`

<storycode:copy id="f-022e74f036b0-Status">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Running,
    Completed,
    Partial,
    Failed,
    Cancelled,
}
</storycode:copy>
<a id="f-022e74f036b0-Run"></a>
### `Run`

<storycode:copy id="f-022e74f036b0-Run">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Run {
    pub schema_version: u32,
    pub id: String,
    pub started_unix_ms: u64,
    pub status: Status,
    pub pinned: bool,
    pub sources: Vec<Source>,
    /// Array position is receive order. Provider timestamps need not be comparable.
    pub events: Vec<Event>,
    pub truncated: bool,
    pub missing_artifacts: Vec<String>,
    pub detail: Option<String>,
}
</storycode:copy>
<a id="f-022e74f036b0-impl-Run"></a>
### `impl-Run`

<storycode:copy id="f-022e74f036b0-impl-Run">
impl Run {
    pub fn new(sources: Vec<Source>, started_unix_ms: u64) -> Self {
        Self {
            schema_version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            started_unix_ms,
            status: Status::Running,
            pinned: false,
            sources,
            events: vec![],
            truncated: false,
            missing_artifacts: vec![],
            detail: None,
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 1, "unsupported observation schema");
        let mut sources = BTreeMap::new();
        for s in &self.sources {
            ensure!(sources.insert(&s.id, s).is_none(), "duplicate source id");
            ensure!(
                s.revision == format!("{:x}", Sha256::digest(&s.bytes)),
                "source revision mismatch"
            );
        }
        let location = |l: &Location| -> Result<()> {
            let s = sources
                .get(&l.source)
                .ok_or_else(|| anyhow::anyhow!("missing captured source"))?;
            match l.precision {
                Precision::Exact => ensure!(l.bytes.is_some(), "exact location requires range"),
                Precision::BlockOnly => ensure!(
                    l.block.is_some() && l.bytes.is_none(),
                    "block precision requires only block"
                ),
                Precision::Unknown => ensure!(l.bytes.is_none(), "unknown location has range"),
                _ => (),
            }
            if let Some(r) = &l.bytes {
                ensure!(
                    r.start <= r.end && r.end <= s.bytes.len(),
                    "location outside captured source"
                );
            }
            Ok(())
        };
        let mut ids = std::collections::HashSet::new();
        for e in &self.events {
            ensure!(
                !e.id.is_empty() && ids.insert(&e.id),
                "duplicate/empty event identity"
            );
            ensure!(!e.provider.is_empty(), "provider required");
            for l in e.emission.iter().chain(e.callers.iter()) {
                location(l)?;
            }
            match &e.observation {
                Observation::Exception {
                    throwing, catching, ..
                } => {
                    for l in throwing.iter().chain(catching.iter()) {
                        location(l)?;
                    }
                }
                Observation::Profile {
                    value,
                    interval,
                    unit,
                    ..
                } => {
                    ensure!(
                        value.is_finite() && !unit.is_empty(),
                        "invalid profile measurement"
                    );
                    ensure!(
                        interval.start.domain == interval.end.domain
                            && interval.start.unit == interval.end.unit
                            && interval.start.value.is_finite()
                            && interval.end.value.is_finite()
                            && interval.start.value <= interval.end.value,
                        "incomparable profile interval"
                    );
                }
                _ => (),
            }
        }
        Ok(())
    }
    pub fn append(&mut self, event: Event, max_event_bytes: usize) -> Result<bool> {
        ensure!(self.status == Status::Running, "run already finished");
        if serde_json::to_vec(&event)?.len() > max_event_bytes {
            self.truncated = true;
            return Ok(false);
        }
        self.events.push(event);
        if let Err(e) = self.validate() {
            self.events.pop();
            return Err(e);
        }
        Ok(true)
    }
    /// Drain currently available provider events with a total serialized-event budget.
    /// Save after each drain; finish is explicit because a drained provider may be live.
    pub fn collect(
        &mut self,
        provider: &mut dyn ObservationProvider,
        max_bytes: usize,
    ) -> Result<usize> {
        ensure!(self.status == Status::Running, "run already finished");
        let mut used = self
            .events
            .iter()
            .try_fold(0usize, |sum, e| -> Result<usize> {
                Ok(sum.saturating_add(serde_json::to_vec(e)?.len()))
            })?;
        let mut added = 0;
        while let Some(event) = provider.poll()? {
            ensure!(
                event.provider == provider.id(),
                "provider identity mismatch"
            );
            let size = serde_json::to_vec(&event)?.len();
            if size > max_bytes.saturating_sub(used) {
                self.truncated = true;
                break;
            }
            self.append(event, size)?;
            used += size;
            added += 1;
        }
        Ok(added)
    }
    pub fn finish(&mut self, status: Status) -> Result<()> {
        ensure!(
            self.status == Status::Running && status != Status::Running,
            "invalid lifecycle transition"
        );
        self.status = status;
        Ok(())
    }
}
</storycode:copy>
<a id="f-022e74f036b0-Contributor"></a>
### `Contributor`

<storycode:copy id="f-022e74f036b0-Contributor">
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contributor {
    pub event: String,
    pub chunk_bytes: Range<usize>,
    pub line_bytes: Range<usize>,
}
</storycode:copy>
<a id="f-022e74f036b0-OutputLine"></a>
### `OutputLine`

<storycode:copy id="f-022e74f036b0-OutputLine">
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputLine {
    pub bytes: Vec<u8>,
    pub contributors: Vec<Contributor>,
    pub terminated: bool,
}
</storycode:copy>
<a id="f-022e74f036b0-output_lines"></a>
### `output_lines`

Lossless LF framing per stream/category. CR and terminal controls stay literal;
UTF-8 decoding belongs after assembly so split codepoints are preserved.

<storycode:copy id="f-022e74f036b0-output_lines">
/// Lossless LF framing per stream/category. CR and terminal controls stay literal;
/// UTF-8 decoding belongs after assembly so split codepoints are preserved.
pub fn output_lines(run: &Run, stream: &str, category: &str) -> Vec<OutputLine> {
    let mut lines = Vec::new();
    let mut line = OutputLine {
        bytes: vec![],
        contributors: vec![],
        terminated: false,
    };
    for e in &run.events {
        let Observation::Output {
            stream: s,
            category: c,
            bytes,
        } = &e.observation
        else {
            continue;
        };
        if s != stream || c != category {
            continue;
        }
        let mut offset = 0;
        for part in bytes.split_inclusive(|b| *b == b'\n') {
            let start = line.bytes.len();
            line.bytes.extend(part);
            line.contributors.push(Contributor {
                event: e.id.clone(),
                chunk_bytes: offset..offset + part.len(),
                line_bytes: start..start + part.len(),
            });
            offset += part.len();
            if part.last() == Some(&b'\n') {
                line.terminated = true;
                lines.push(line);
                line = OutputLine {
                    bytes: vec![],
                    contributors: vec![],
                    terminated: false,
                };
            }
        }
    }
    if !line.bytes.is_empty() {
        lines.push(line);
    }
    lines
}

</storycode:copy>
<a id="f-022e74f036b0-tests"></a>
### `tests`

<storycode:copy id="f-022e74f036b0-tests">
#[cfg(test)]
mod tests;
</storycode:copy>

### Assemble `crates/hick-observe/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/src/lib.rs"><storycode:paste select="#f-022e74f036b0-prediction"/><storycode:paste select="#f-022e74f036b0-reading"/><storycode:paste select="#f-022e74f036b0-store"/><storycode:paste select="#f-022e74f036b0-use_declaration"/><storycode:paste select="#f-022e74f036b0-Source"/><storycode:paste select="#f-022e74f036b0-impl-Source"/><storycode:paste select="#f-022e74f036b0-Precision"/><storycode:paste select="#f-022e74f036b0-Location"/><storycode:paste select="#f-022e74f036b0-Clock"/><storycode:paste select="#f-022e74f036b0-Interval"/><storycode:paste select="#f-022e74f036b0-Handling"/><storycode:paste select="#f-022e74f036b0-FunctionAggregate"/><storycode:paste select="#f-022e74f036b0-Observation"/><storycode:paste select="#f-022e74f036b0-Event"/><storycode:paste select="#f-022e74f036b0-ObservationProvider"/><storycode:paste select="#f-022e74f036b0-Status"/><storycode:paste select="#f-022e74f036b0-Run"/><storycode:paste select="#f-022e74f036b0-impl-Run"/><storycode:paste select="#f-022e74f036b0-Contributor"/><storycode:paste select="#f-022e74f036b0-OutputLine"/><storycode:paste select="#f-022e74f036b0-output_lines"/><storycode:paste select="#f-022e74f036b0-tests"/></storycode:file>

</storycode:doc>
