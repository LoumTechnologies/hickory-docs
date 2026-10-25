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
