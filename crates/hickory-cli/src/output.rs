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
            let open = exec.source_span.context("an exec without a source span")?;
            // Recording whitespace is display metadata, never generated file
            // content. Place it outside every enclosing file, not between an
            // inline exec and its file's closing tag.
            let position = doc
                .all_tags()
                .into_iter()
                .filter(|tag| tag.name == "file")
                .filter_map(|tag| Some((tag.source_span?, tag.close_span?)))
                .filter(|(file_open, file_close)| {
                    file_open.start < open.start && file_close.end >= close.end
                })
                .min_by_key(|(file_open, _)| file_open.start)
                .map(|(_, file_close)| file_close.end)
                .unwrap_or(close.end);
            let text = format!("\n\n{replacement}");
            // Several cells in one file share this boundary; append in source
            // order rather than reversing equal-position insertion edits.
            if let Some(edit) = edits
                .iter_mut()
                .find(|edit| edit.span == (position, position))
            {
                edit.text.push_str(&text);
            } else {
                edits.push(Edit {
                    span: (position, position),
                    text,
                });
            }
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
        .map(|line| {
            // The parser accepts closing fences indented by up to three spaces.
            let indent = line.bytes().take_while(|byte| *byte == b' ').count();
            if indent <= 3 {
                line[indent..]
                    .bytes()
                    .take_while(|byte| *byte == b'`')
                    .count()
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
        .max(2)
        + 1;
    let fence = "`".repeat(width);
    // A fence is line-delimited even when stdout has no final newline. Hash
    // the body actually stored, and keep following source out of its closer.
    let body = if output.ends_with('\n') {
        output.to_owned()
    } else {
        format!("{output}\n")
    };
    let hash = hick_literate::cache::sha256_hex(&body);
    format!(
        "{fence}output-for=\"{selector}\" hash=\"{hash}\" input-hash=\"{input_hash}\" exit=\"0\"\n{body}{fence}\n"
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
        assert!(out.ends_with("````\n"));
    }

    #[test]
    fn output_fence_keeps_indented_markers_and_hick_tags_inside_transcript() {
        for indent in 0..=3 {
            let output = format!(
                "{}```\n<hick:file path=\"escaped.txt\">literal output</hick:file>\n",
                " ".repeat(indent)
            );
            let fence = output_fence("#cell", "input", &output);
            let source = format!(
                "<hick:doc>\n{fence}<hick:file path=\"real.txt\">source</hick:file>\n</hick:doc>"
            );
            let doc = hick_lang::parse(&source).unwrap();
            let files: Vec<_> = doc
                .all_tags()
                .into_iter()
                .filter(|tag| tag.name == "file")
                .collect();
            assert_eq!(files.len(), 1);
            assert_eq!(files[0].get_attribute("path"), Some("real.txt"));
            let saved = doc
                .all_tags()
                .into_iter()
                .find(|tag| tag.name == "output")
                .unwrap();
            let body = &source[saved.source_span.unwrap().end..saved.close_span.unwrap().start];
            assert_eq!(body, output);
            assert_eq!(
                saved.get_attribute("hash"),
                Some(hick_literate::cache::sha256_hex(body).as_str())
            );
        }
    }

    #[tokio::test]
    async fn an_inline_exec_output_keeps_its_enclosing_file_and_document_parseable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        std::fs::write(&path, r#"<hick:doc><hick:file path="value.txt"><hick:exec container="gen" show="output">printf hello</hick:exec></hick:file></hick:doc>"#).unwrap();
        let mut run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        // Existing transcript rendering ends each shown entry with a newline;
        // raw execution bytes remain distinct from this generated-file view.
        assert_eq!(run.result.files["value.txt"].as_text(), Some("hello\n"));
        assert_eq!(run.result.transcripts["gen"][0].output, "hello");
        assert_eq!(refresh_outputs(&run).unwrap(), 2);
        let once = std::fs::read_to_string(&path).unwrap();
        hick_lang::parse(&once).unwrap();
        run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        assert!(verify_outputs(&run).is_empty());
        assert_eq!(run.result.files["value.txt"].as_text(), Some("hello\n"));
        assert_eq!(run.result.transcripts["gen"][0].output, "hello");
        refresh_outputs(&run).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), once);
    }

    #[tokio::test]
    async fn several_recordings_in_one_file_preserve_artifact_bytes_and_source_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        std::fs::write(&path, r#"<hick:doc><hick:file path="value.txt"><hick:exec container="one" show="output">printf hello</hick:exec><hick:exec container="two" show="output">printf world</hick:exec></hick:file></hick:doc>"#).unwrap();
        let run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        assert_eq!(
            run.result.files["value.txt"].as_text(),
            Some("hello\nworld\n")
        );
        refresh_outputs(&run).unwrap();
        let once = std::fs::read_to_string(&path).unwrap();
        assert!(
            once.find("output-for=\"#one-1\"").unwrap()
                < once.find("output-for=\"#two-1\"").unwrap()
        );
        let run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        assert_eq!(
            run.result.files["value.txt"].as_text(),
            Some("hello\nworld\n")
        );
        assert!(verify_outputs(&run).is_empty());
        refresh_outputs(&run).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), once);
    }

    #[tokio::test]
    async fn an_existing_recording_inside_a_file_keeps_authored_whitespace_stable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        // The old writer had no newline after its closing fence: the source's
        // existing newline made this a valid recording inside the file.
        let recording = output_fence("#legacy", "old-input", "hello\n");
        let recording = recording.strip_suffix('\n').unwrap();
        let source = format!(
            r#"<hick:doc><hick:file path="value.txt"><hick:exec container="gen" id="legacy" show="output">printf 'hello\n'</hick:exec>

{recording}
</hick:file></hick:doc>"#
        );
        std::fs::write(&path, source).unwrap();
        let run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        let before = run.result.files["value.txt"].as_text().unwrap().to_owned();
        assert_eq!(before, "hello\n\n\n");
        assert_eq!(run.result.transcripts["gen"][0].output, "hello\n");
        refresh_outputs(&run).unwrap();
        let once = std::fs::read_to_string(&path).unwrap();
        assert!(once.contains("</hick:exec>\n\n```output-for="));
        let run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
            .await
            .unwrap();
        assert_eq!(
            run.result.files["value.txt"].as_text(),
            Some(before.as_str())
        );
        assert!(verify_outputs(&run).is_empty());
        refresh_outputs(&run).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), once);
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
