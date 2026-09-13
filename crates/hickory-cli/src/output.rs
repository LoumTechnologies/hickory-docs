//! Derived `hick:output` regions kept beside the cells that produced them.

use anyhow::{Context, Result, bail};
use std::collections::HashSet;

use crate::DocRun;

struct Edit {
    span: (usize, usize),
    text: String,
}

#[derive(Debug)]
pub enum OutputIssue {
    Modified { selector: String },
    Stale { selector: String },
}

pub fn verify_outputs(run: &DocRun) -> Vec<OutputIssue> {
    let mut issues = Vec::new();
    for output in run
        .doc
        .all_tags()
        .into_iter()
        .filter(|tag| tag.name == "output")
    {
        let Some(selector) = output.get_attribute("output-for") else {
            continue;
        };
        let (Some(open), Some(close), Some(expected)) = (
            output.source_span,
            output.close_span,
            output.get_attribute("hash"),
        ) else {
            issues.push(OutputIssue::Modified {
                selector: selector.to_owned(),
            });
            continue;
        };
        if hick_literate::cache::sha256_hex(&run.source[open.end..close.start]) != expected {
            issues.push(OutputIssue::Modified {
                selector: selector.to_owned(),
            });
            continue;
        }
        let Some(id) = selector.strip_prefix('#') else {
            issues.push(OutputIssue::Stale {
                selector: selector.to_owned(),
            });
            continue;
        };
        let current = run.doc.all_tags().into_iter().find_map(|exec| {
            if exec.name == "exec" && exec.get_attribute("id") == Some(id) {
                exec.get_attribute("container")
                    .map(|container| hick_literate::CellId::exec(container, exec.source_line))
            } else {
                None
            }
        });
        let Some(current) = current.and_then(|cell| run.result.keys.get(&cell)) else {
            issues.push(OutputIssue::Stale {
                selector: selector.to_owned(),
            });
            continue;
        };
        if output.get_attribute("input-hash") != Some(current.as_str()) {
            issues.push(OutputIssue::Stale {
                selector: selector.to_owned(),
            });
        }
    }
    issues
}

/// Refresh every `show="output"` cell's same-document output block.
///
/// An output is derived, not an editor-owned passage. Before changing one we
/// verify its `hash`: an outside edit is a fact worth showing, not something a
/// run gets to erase silently.
pub fn refresh_outputs(run: &DocRun) -> Result<usize> {
    let source = std::fs::read_to_string(&run.doc_path)
        .with_context(|| format!("reading {}", run.doc_path.display()))?;
    let doc =
        hick_lang::parse(&source).with_context(|| format!("parsing {}", run.doc_path.display()))?;
    let mut used: HashSet<String> = doc
        .all_tags()
        .into_iter()
        .filter_map(|tag| tag.get_attribute("id").map(str::to_owned))
        .collect();
    let mut edits = Vec::new();

    for exec in doc.all_tags().into_iter().filter(|tag| tag.name == "exec") {
        if exec.get_attribute("show") != Some("output") {
            continue;
        }
        let container = exec.get_attribute("container").unwrap_or("cell");
        let id = match exec.get_attribute("id") {
            Some(id) => id.to_owned(),
            None => {
                let id = fresh_id(container, &used);
                used.insert(id.clone());
                let span = exec.source_span.context("an exec without a source span")?;
                edits.push(Edit {
                    span: (
                        opening_attr_at(&source, span)?,
                        opening_attr_at(&source, span)?,
                    ),
                    text: format!(" id=\"{id}\""),
                });
                id
            }
        };
        let cell = hick_literate::CellId::exec(container, exec.source_line);
        let Some(input_hash) = run.result.keys.get(&cell) else {
            continue;
        };
        let Some(output) = run
            .result
            .transcripts
            .get(container)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|entry| entry.source_line == Some(exec.source_line))
            })
            .map(|entry| entry.output.as_str())
        else {
            continue;
        };
        let selector = format!("#{id}");
        let outputs: Vec<_> = doc
            .all_tags()
            .into_iter()
            .filter(|tag| {
                tag.name == "output" && tag.get_attribute("output-for") == Some(selector.as_str())
            })
            .collect();
        if outputs.len() > 1 {
            bail!(
                "{selector} names {} output blocks; an exec has exactly one output",
                outputs.len()
            );
        }
        let replacement = output_fence(&selector, input_hash, output);
        if let Some(existing) = outputs.first() {
            let open = existing
                .source_span
                .context("an output without a source span")?;
            let close = existing
                .close_span
                .context("an output without a closing span")?;
            let body = &source[open.end..close.start];
            let expected = existing
                .get_attribute("hash")
                .context("output is missing hash")?;
            let actual = hick_literate::cache::sha256_hex(body);
            if actual != expected {
                bail!(
                    "{selector}'s output was modified outside Hickory (its hash does not match); remove it or use an explicit replace action"
                )
            }
            edits.push(Edit {
                span: (open.start, close.end),
                text: replacement,
            });
        } else {
            let close = exec.close_span.context("an exec without a closing span")?;
            edits.push(Edit {
                span: (close.end, close.end),
                text: format!("\n\n{replacement}"),
            });
        }
    }
    if edits.is_empty() {
        return Ok(0);
    }
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.span.0));
    let mut next = source;
    for edit in &edits {
        next.replace_range(edit.span.0..edit.span.1, &edit.text);
    }
    hick_lang::parse(&next).context("validating output update")?;
    std::fs::write(&run.doc_path, next)
        .with_context(|| format!("writing {}", run.doc_path.display()))?;
    Ok(edits.len())
}

fn opening_attr_at(source: &str, span: hick_lang::SourceSpan) -> Result<usize> {
    let opening = &source[span.start..span.end];
    if opening.starts_with('<') {
        return Ok(span.end - 1);
    }
    Ok(span.start + opening.trim_end_matches(['\r', '\n']).len())
}

fn fresh_id(container: &str, used: &HashSet<String>) -> String {
    let base: String = container
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    for n in 1usize.. {
        let id = format!("{base}-{n}");
        if !used.contains(&id) {
            return id;
        }
    }
    unreachable!()
}

fn output_fence(selector: &str, input_hash: &str, output: &str) -> String {
    let width = output
        .lines()
        .map(|line| line.bytes().take_while(|byte| *byte == b'`').count())
        .max()
        .unwrap_or(0)
        .max(2)
        + 1;
    let fence = "`".repeat(width);
    let hash = hick_literate::cache::sha256_hex(output);
    format!(
        "{fence}output-for=\"{selector}\" hash=\"{hash}\" input-hash=\"{input_hash}\" exit=\"0\"\n{output}{fence}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutorChoice, RunMode, run_doc};

    #[test]
    fn output_fence_carries_both_hashes_and_escapes_its_body() {
        let out = output_fence("#test-1", "input", "```\nhello\n");
        assert!(out.starts_with("````output-for=\"#test-1\" hash=\""));
        assert!(out.contains("input-hash=\"input\""));
        assert!(out.ends_with("````"));
    }

    #[tokio::test]
    async fn a_fenced_exec_gains_a_hashed_read_only_output_block() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        std::fs::write(
            &path,
            "```shell container=\"reporter\" show=\"output\"\necho hello\n```\n",
        )
        .unwrap();
        let run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        assert_eq!(refresh_outputs(&run).unwrap(), 2);
        let source = std::fs::read_to_string(path).unwrap();
        assert!(source.contains("id=\"reporter-1\""), "{source}");
        assert!(
            source.contains("```output-for=\"#reporter-1\" hash=\""),
            "{source}"
        );
        assert!(source.contains("input-hash=\""), "{source}");
        assert!(source.contains("\nhello\n```"), "{source}");
    }
}
