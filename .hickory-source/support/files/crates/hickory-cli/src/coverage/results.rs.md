<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/coverage/results.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `277033e4739b7e2c904746359fe81d54b16d8ea4a3fede743c55478cdb60ab8d`.

<book:entry path="crates/hickory-cli/src/coverage/results.rs">
use super::*;

fn test(
    id: String,
    status: &str,
    output: String,
    path: Option<String>,
    line: Option<u32>,
    metadata: &Config,
) -> TestResult {
    let labels = metadata.tests.get(&id).cloned().unwrap_or_default();
    TestResult {
        id,
        status: status.into(),
        output,
        path,
        line,
        labels,
    }
}
pub(super) fn discover(language: &str, text: &str, metadata: &Config) -> Vec<TestResult> {
    if language == "rust" {
        return text
            .lines()
            .filter_map(|row| row.strip_suffix(": test"))
            .map(|id| test(id.into(), "discovered", String::new(), None, None, metadata))
            .collect();
    }
    if language == "python" {
        return text
            .lines()
            .filter(|row| row.contains(".py::") && !row.starts_with(' '))
            .map(|id| {
                test(
                    id.into(),
                    "discovered",
                    String::new(),
                    id.split("::").next().map(str::to_string),
                    None,
                    metadata,
                )
            })
            .collect();
    }
    if language == "csharp" {
        return text
            .split("The following Tests are available:")
            .nth(1)
            .unwrap_or("")
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|id| test(id.into(), "discovered", String::new(), None, None, metadata))
            .collect();
    }
    let value = json_from_output(text);
    if value.get("testResults").is_some()
        && let Ok(mut tests) = parse(
            "typescript",
            &value.to_string(),
            "",
            metadata,
            Path::new("."),
        )
    {
        for test in &mut tests {
            test.status = "discovered".into();
        }
        return tests;
    }
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| {
            if let Some(file) = v.as_str() {
                Some(test(
                    file.into(),
                    "discovered-file",
                    "Jest discovery enumerates files; individual results appear after running"
                        .into(),
                    Some(file.into()),
                    None,
                    metadata,
                ))
            } else {
                Some(test(
                    v["fullName"]
                        .as_str()
                        .or_else(|| v["name"].as_str())?
                        .into(),
                    "discovered",
                    String::new(),
                    v["file"].as_str().map(str::to_string),
                    v["location"]["line"].as_u64().map(|n| n as u32),
                    metadata,
                ))
            }
        })
        .collect()
}
fn json_from_output(text: &str) -> Value {
    if let Ok(value) = serde_json::from_str(text) {
        return value;
    }
    for (open, close) in [('{', '}'), ('[', ']')] {
        if let (Some(start), Some(end)) = (text.find(open), text.rfind(close))
            && start <= end
            && let Ok(value) = serde_json::from_str(&text[start..=end])
        {
            return value;
        }
    }
    Value::Null
}

pub(super) fn parse(
    language: &str,
    raw: &str,
    output: &str,
    metadata: &Config,
    project: &Path,
) -> Result<Vec<TestResult>> {
    if language == "rust" {
        return Ok(output
            .lines()
            .filter_map(|row| {
                let row = row.strip_prefix("test ")?;
                let (id, status) = row.rsplit_once(" ... ")?;
                let status = match status {
                    "ok" => "passed",
                    "FAILED" => "failed",
                    s if s.starts_with("ignored") => "skipped",
                    _ => return None,
                };
                Some(test(
                    id.into(),
                    status,
                    if status == "failed" {
                        output.into()
                    } else {
                        String::new()
                    },
                    None,
                    None,
                    metadata,
                ))
            })
            .collect());
    }
    if language == "python" || language == "csharp" {
        let doc = roxmltree::Document::parse(raw)?;
        if language == "csharp" {
            let definitions: BTreeMap<_, _> = doc
                .descendants()
                .filter(|n| n.has_tag_name("UnitTest"))
                .filter_map(|n| {
                    Some((
                        n.attribute("id")?,
                        n.descendants().find(|n| n.has_tag_name("TestMethod"))?,
                    ))
                })
                .collect();
            return Ok(doc
                .descendants()
                .filter(|n| n.has_tag_name("UnitTestResult"))
                .map(|n| {
                    let method = n.attribute("testId").and_then(|id| definitions.get(id));
                    let id = method
                        .map(|m| {
                            format!(
                                "{}.{}",
                                m.attribute("className")
                                    .unwrap_or("")
                                    .split(',')
                                    .next()
                                    .unwrap_or(""),
                                m.attribute("name").unwrap_or("")
                            )
                        })
                        .unwrap_or_else(|| n.attribute("testName").unwrap_or("unknown").into());
                    let status = match n.attribute("outcome") {
                        Some("Passed") => "passed",
                        Some("Failed") => "failed",
                        Some("NotExecuted") => "skipped",
                        _ => "unknown",
                    };
                    test(
                        id,
                        status,
                        n.descendants()
                            .filter_map(|c| c.text())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        None,
                        None,
                        metadata,
                    )
                })
                .collect());
        }
        return Ok(doc
            .descendants()
            .filter(|n| n.has_tag_name("testcase"))
            .map(|n| {
                let class = n.attribute("classname").unwrap_or("");
                let name = n.attribute("name").unwrap_or("");
                let file = n.attribute("file").map(str::to_string).or_else(|| {
                    let mut parts: Vec<_> = class.split('.').collect();
                    while !parts.is_empty() {
                        let path = format!("{}.py", parts.join("/"));
                        if project.join(&path).is_file() {
                            return Some(path);
                        }
                        parts.pop();
                    }
                    None
                });
                let id = file
                    .as_ref()
                    .map(|f| {
                        let module = f.trim_end_matches(".py").replace('/', ".");
                        let suffix = class
                            .strip_prefix(&module)
                            .unwrap_or("")
                            .trim_start_matches('.')
                            .replace('.', "::");
                        if suffix.is_empty() {
                            format!("{f}::{name}")
                        } else {
                            format!("{f}::{suffix}::{name}")
                        }
                    })
                    .unwrap_or_else(|| format!("{class}::{name}"));
                let status = if n
                    .children()
                    .any(|c| c.has_tag_name("failure") || c.has_tag_name("error"))
                {
                    "failed"
                } else if n.children().any(|c| c.has_tag_name("skipped")) {
                    "skipped"
                } else {
                    "passed"
                };
                test(
                    id,
                    status,
                    n.children()
                        .filter_map(|c| c.text())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    file,
                    n.attribute("line")
                        .and_then(|l| l.parse::<u32>().ok())
                        .map(|l| l + 1),
                    metadata,
                )
            })
            .collect());
    }
    let v: Value = serde_json::from_str(raw)?;
    Ok(v["testResults"]
        .as_array()
        .context("Missing test results")?
        .iter()
        .flat_map(|suite| {
            suite["assertionResults"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|a| {
                    let status = match a["status"].as_str() {
                        Some("passed") => "passed",
                        Some("failed") => "failed",
                        Some("pending" | "todo" | "skipped" | "disabled") => "skipped",
                        _ => "unknown",
                    };
                    test(
                        a["fullName"]
                            .as_str()
                            .or_else(|| a["title"].as_str())
                            .unwrap_or("unknown")
                            .into(),
                        status,
                        a["failureMessages"]
                            .as_array()
                            .map(|m| {
                                m.iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                            .unwrap_or_default(),
                        suite["name"].as_str().map(str::to_string),
                        a["location"]["line"].as_u64().map(|l| l as u32),
                        metadata,
                    )
                })
        })
        .collect())
}
</book:entry>

</book:doc>
