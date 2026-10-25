<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-coverage/src/import.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Coverage retains its measured scope](../../../../contract.md)

Captured SHA-256: `5f6e35180b9bff595db44f5df955c73c29673152dc3b96fd74f61cd8eb8f6d6b`.

<book:entry path="crates/hick-coverage/src/import.rs">
use crate::{Branch, FileCoverage, Report};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn parse(format: &str, raw: &str) -> Result<Report> {
    ensure!(
        raw.len() <= 16 * 1024 * 1024,
        "Coverage report exceeds 16 MiB"
    );
    let files = match format {
        "lcov" => lcov(raw)?,
        "coverage-py" => python(raw)?,
        "cobertura" => cobertura(raw)?,
        "istanbul" => istanbul(raw)?,
        _ => bail!("Unsupported coverage format: {format}"),
    };
    ensure!(!files.is_empty(), "Report contains no measured files");
    Ok(Report {
        format: format.into(),
        files,
        raw: raw.into(),
    })
}

fn line(value: u64) -> Result<u32> {
    ensure!(
        value > 0 && value <= u32::MAX as u64,
        "Invalid source line {value}"
    );
    Ok(value as u32)
}

fn lcov(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let mut files = BTreeMap::<String, FileCoverage>::new();
    let mut current = None;
    for row in raw.lines() {
        if let Some(path) = row.strip_prefix("SF:") {
            ensure!(current.is_none(), "Unterminated LCOV record");
            current = Some(path.to_string());
            files.entry(path.into()).or_default();
        } else if row == "end_of_record" {
            current = None;
        } else if let Some(path) = &current {
            let file = files.get_mut(path).unwrap();
            file.line_counts = true;
            if let Some(data) = row.strip_prefix("DA:") {
                let parts: Vec<_> = data.split(',').collect();
                ensure!(parts.len() >= 2, "Invalid LCOV line");
                let number = line(parts[0].parse()?)?;
                let count: u64 = parts[1].parse()?;
                let value = file.lines.entry(number).or_default();
                *value = value.saturating_add(count);
            } else if let Some(data) = row.strip_prefix("BRDA:") {
                let p: Vec<_> = data.split(',').collect();
                ensure!(p.len() == 4, "Invalid LCOV branch");
                let count = if p[3] == "-" {
                    None
                } else {
                    Some(p[3].parse::<u64>()?)
                };
                let branches = file.branches.get_or_insert_default();
                let id = format!("{}:{}:{}", p[0], p[1], p[2]);
                if let Some(previous) = branches.iter_mut().find(|b| b.id == id) {
                    previous.covered |= count.is_some_and(|n| n > 0);
                    previous.count = match (previous.count, count) {
                        (Some(a), Some(b)) => Some(a.saturating_add(b)),
                        _ => None,
                    };
                } else {
                    branches.push(Branch {
                        id,
                        line: line(p[0].parse()?)?,
                        covered: count.is_some_and(|n| n > 0),
                        count,
                    });
                }
            } else if row == "BRF:0" {
                file.branches.get_or_insert_default();
            }
        }
    }
    ensure!(current.is_none(), "Unterminated LCOV record");
    Ok(files)
}

fn python(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let value: Value = serde_json::from_str(raw)?;
    let files = value["files"]
        .as_object()
        .context("Missing coverage.py files")?;
    let mut output = BTreeMap::new();
    for (path, data) in files {
        let mut file = FileCoverage::default();
        for (key, count) in [("executed_lines", 1), ("missing_lines", 0)] {
            for number in data[key]
                .as_array()
                .context("Missing coverage.py executable lines")?
            {
                file.lines.insert(
                    line(number.as_u64().context("Invalid executable line")?)?,
                    count,
                );
            }
        }
        if let Some(excluded) = data["excluded_lines"].as_array() {
            for n in excluded {
                let n = line(n.as_u64().context("Invalid excluded line")?)?;
                file.lines.remove(&n);
                file.excluded.push(n);
            }
        }
        if value["meta"]["branch_coverage"].as_bool() == Some(true) {
            let mut branches = Vec::new();
            // coverage.py arcs can end at a negative synthetic function-exit line.
            for (key, covered) in [("executed_branches", true), ("missing_branches", false)] {
                for arc in data[key].as_array().context("Missing branch arcs")? {
                    let from = arc[0].as_i64().context("Invalid arc")?;
                    let to = arc[1].as_i64().context("Invalid arc")?;
                    if from > 0 {
                        branches.push(Branch {
                            id: format!("{from}:{to}"),
                            line: line(from as u64)?,
                            count: None,
                            covered,
                        });
                    }
                }
            }
            file.branches = Some(branches);
        }
        output.insert(path.clone(), file);
    }
    Ok(output)
}

fn cobertura(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let doc = roxmltree::Document::parse(raw)?;
    let mut files = BTreeMap::<String, FileCoverage>::new();
    let sources: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("source"))
        .filter_map(|n| n.text())
        .filter(|s| !s.is_empty())
        .collect();
    for class in doc.descendants().filter(|n| n.has_tag_name("class")) {
        let path = class
            .attribute("filename")
            .context("Class has no source filename")?;
        let path = if !std::path::Path::new(path).is_absolute() && sources.len() == 1 {
            std::path::Path::new(sources[0])
                .join(path)
                .to_string_lossy()
                .into_owned()
        } else {
            path.to_string()
        };
        let file = files.entry(path).or_default();
        file.line_counts = true;
        // Method line tables repeat class line tables. Read only the class's direct lines.
        for node in class
            .children()
            .filter(|n| n.has_tag_name("lines"))
            .flat_map(|n| n.children())
            .filter(|n| n.has_tag_name("line"))
        {
            let n = line(
                node.attribute("number")
                    .context("Missing line number")?
                    .parse()?,
            )?;
            let count: u64 = node
                .attribute("hits")
                .context("Missing hit count")?
                .parse()?;
            file.lines
                .entry(n)
                .and_modify(|c| *c = (*c).max(count))
                .or_insert(count);
            if let Some(fraction) = node
                .attribute("condition-coverage")
                .and_then(|s| s.split('(').nth(1))
                .and_then(|s| s.strip_suffix(')'))
            {
                let (covered, total) = fraction
                    .split_once('/')
                    .context("Invalid branch fraction")?;
                let covered: u64 = covered.parse()?;
                let total: u64 = total.parse()?;
                ensure!(covered <= total, "Invalid branch denominator");
                file.branch_totals
                    .entry(n)
                    .and_modify(|old| {
                        old.0 = old.0.max(covered);
                        old.1 = old.1.max(total);
                    })
                    .or_insert((covered, total));
            }
            // Cobertura gives aggregate branch fractions, not branch identities. Raw data
            // retains those fractions; never manufacture identifiable branches from them.
        }
    }
    Ok(files)
}

fn istanbul(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let value: Value = serde_json::from_str(raw)?;
    let mut files = BTreeMap::new();
    for (path, data) in value
        .as_object()
        .context("Expected Istanbul coverage map")?
    {
        let mut file = FileCoverage {
            line_counts: true,
            ..Default::default()
        };
        let statements = data["statementMap"]
            .as_object()
            .context("Missing statement map")?;
        for (id, location) in statements {
            let count = data["s"][id].as_u64().context("Missing statement count")?;
            let n = line(
                location["start"]["line"]
                    .as_u64()
                    .context("Invalid statement line")?,
            )?;
            // Istanbul line coverage uses the maximum count of statements starting there.
            file.lines
                .entry(n)
                .and_modify(|c| *c = (*c).max(count))
                .or_insert(count);
        }
        if let Some(map) = data["branchMap"].as_object() {
            let mut branches = Vec::new();
            for (id, location) in map {
                for (index, count) in data["b"][id]
                    .as_array()
                    .context("Missing branch counters")?
                    .iter()
                    .enumerate()
                {
                    let count = count.as_u64().context("Invalid branch count")?;
                    let loc = &location["locations"][index];
                    branches.push(Branch {
                        id: format!("{id}:{index}"),
                        line: line(
                            loc["start"]["line"]
                                .as_u64()
                                .or_else(|| location["line"].as_u64())
                                .context("Missing branch location")?,
                        )?,
                        count: Some(count),
                        covered: count > 0,
                    });
                }
            }
            file.branches = Some(branches);
        }
        files.insert(data["path"].as_str().unwrap_or(path).into(), file);
    }
    Ok(files)
}
</book:entry>

</book:doc>
