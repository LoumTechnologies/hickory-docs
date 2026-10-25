<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-coverage/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Coverage retains its measured scope](../../../../contract.md)

Captured SHA-256: `fd91f24352863798c1e536f76623a73963d6ec41359ea5a8a801d3ef41ff9102`.

<book:entry path="crates/hick-coverage/src/lib.rs">
//! Collector evidence, independent of a debugger and of test success.
mod compare;
mod import;
pub use compare::{Comparison, compare};
pub use import::parse;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Branch {
    pub id: String,
    pub line: u32,
    /// None means the collector did not supply a count.
    pub count: Option<u64>,
    pub covered: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileCoverage {
    /// Only executable lines. Missing entries are unknown/non-executable, not zero.
    pub lines: BTreeMap<u32, u64>,
    /// False means line values are reachability flags (0/1), not execution counts.
    pub line_counts: bool,
    /// Provider aggregate fractions, without inferred branch identities.
    pub branch_totals: BTreeMap<u32, (u64, u64)>,
    /// None is an unsupported metric; Some([]) is a measured empty denominator.
    pub branches: Option<Vec<Branch>>,
    pub excluded: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub format: String,
    pub files: BTreeMap<String, FileCoverage>,
    /// The original report retains regions, contexts and other provider detail.
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub collector: String,
    pub version: String,
    pub configuration: String,
    pub scope: String,
    pub source_hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub identity: Identity,
    pub report: Report,
    /// Snapshot of metadata used at run time; never re-read for historical runs.
    pub labels: Vec<String>,
    pub tests: Vec<String>,
    /// group, individual, or unavailable. Aggregate imports default to unavailable.
    pub attribution: String,
}
</book:entry>

</book:doc>
