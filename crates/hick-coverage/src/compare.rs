use crate::Evidence;
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Serialize)]
pub struct Comparison {
    pub shared: BTreeSet<String>,
    pub left_only: BTreeSet<String>,
    pub right_only: BTreeSet<String>,
    pub missed: BTreeSet<String>,
    pub denominator: usize,
}

/// Compare only evidence with identical provenance and executable denominators.
/// Missing reports or unsupported metrics are an error, never an empty set.
pub fn compare(left: &Evidence, right: &Evidence, metric: &str) -> Result<Comparison> {
    ensure!(
        left.identity == right.identity,
        "Source, scope, collector/version or instrumentation differ"
    );
    ensure!(
        !left.identity.source_hashes.is_empty(),
        "Source revision is unknown"
    );
    ensure!(
        left.report.format == right.report.format,
        "Report semantics differ"
    );
    let locations =
        |e: &Evidence| -> Result<BTreeMap<String, bool>> {
            let mut out = BTreeMap::new();
            for (path, file) in &e.report.files {
                match metric {
                    "line" => {
                        for (n, count) in &file.lines {
                            out.insert(format!("{path}:{n}"), *count > 0);
                        }
                    }
                    "branch" => {
                        for b in file.branches.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("Branch metric unavailable for {path}")
                        })? {
                            out.insert(format!("{path}:{}", b.id), b.covered);
                        }
                    }
                    _ => anyhow::bail!("Unsupported metric"),
                }
            }
            Ok(out)
        };
    let a = locations(left)?;
    let b = locations(right)?;
    ensure!(
        a.keys().eq(b.keys()),
        "Measured executable denominators differ"
    );
    let mut result = Comparison {
        denominator: a.len(),
        ..Default::default()
    };
    for (key, covered) in a {
        match (covered, b[&key]) {
            (true, true) => &mut result.shared,
            (true, false) => &mut result.left_only,
            (false, true) => &mut result.right_only,
            (false, false) => &mut result.missed,
        }
        .insert(key);
    }
    Ok(result)
}
