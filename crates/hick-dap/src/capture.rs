//! `<hick:capture>`: the same expressions, with nobody stepping.
//!
//! A capture is a breakpoint the document owns. On a run the adapter sets it,
//! and each time it is hit the runner issues **the same `evaluate` request the
//! interactive pane issues**, records what came back, and continues. Nothing
//! waits for a human, so a document can assert something about a value *inside
//! a function* — which no amount of checking stdout reaches.
//!
//! Three rules from the spec are enforced here rather than advised:
//!
//! * **Evaluation uses DAP's `watch` context**, which adapters treat as
//!   repeatable. A capture that mutates the program is a mistake the document
//!   would keep making, and `watch` is the strongest hint the protocol has.
//! * **Hits are bounded.** A breakpoint in a hot loop is a slow run and an
//!   unreadable table, so a capture stops at `max` and *says* it stopped
//!   rather than silently truncating.
//! * **The run is a reader.** The program is woven into a scratch directory
//!   and launched there, so a captured cell cannot write to the repository
//!   even though the cell it captures does.
//!
//! See `docs/specs/freeform/literate-debugging.md`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::build::{BuildOutput, build};
use crate::program::{adapter_for, entry_point, weave_into};
use crate::{Breakpoint, Launch, Mapping, Session, Step};

/// How long the whole capture run may take before it is abandoned.
///
/// Generous, because it covers starting an adapter and running the program,
/// and bounded, because a run that never ends is a build that never ends.
const RUN_BUDGET: Duration = Duration::from_secs(120);

/// Default hit bound. Small on purpose: a table longer than this is not read.
pub const DEFAULT_MAX_HITS: usize = 20;

/// One `<hick:capture>`, resolved to coordinates the adapter understands.
#[derive(Debug, Clone)]
pub struct CaptureSpec {
    /// Generated file the location names, as written in `at` (`pricing.py`).
    pub file: String,
    /// 1-based line within that file, as written in `at` (`pricing.py:14`).
    pub line: u32,
    /// The expressions to evaluate, in the order they were written.
    pub expressions: Vec<String>,
    /// A breakpoint condition, evaluated by the adapter in the debuggee.
    pub condition: Option<String>,
    /// Hit bound.
    pub max: usize,
}

/// What one hit recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// 1-based hit index — the run's own ordering, not wall-clock, so a table
    /// is stable between runs.
    pub index: usize,
    /// One entry per expression, in the order they were written. The value is
    /// the adapter's own rendering, or the error it gave for that expression.
    pub values: Vec<Recorded>,
}

/// The result of evaluating one expression at one hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    pub expression: String,
    /// What the adapter returned, or the message it failed with. A failure is
    /// recorded rather than fatal: a name that is not in scope at that line is
    /// a fact about the document worth weaving, not a reason to stop the run.
    pub value: String,
    pub ok: bool,
}

/// Everything one capture recorded across a run.
#[derive(Debug, Clone)]
pub struct Captured {
    pub spec: CaptureSpec,
    pub hits: Vec<Hit>,
    /// Whether the bound stopped it early — woven, never hidden.
    pub truncated: bool,
    /// Set when the breakpoint could not be placed at all, in words that say
    /// what to do about it.
    pub problem: Option<String>,
    /// The document line the adapter slid the breakpoint down to, when it was
    /// not the line the capture named. Reported, because a table of values
    /// from a line the author did not write is a table they cannot trust.
    pub moved: Option<u32>,
}

/// Run a document's captures.
///
/// `document` is the `.hick` file (for naming and for the mapping), `source`
/// its text, `project` the directory adapters are discovered from.
pub async fn run(
    document: &Path,
    source: &str,
    project: &Path,
    specs: &[CaptureSpec],
) -> Result<Vec<Captured>> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let scratch = tempfile::tempdir().context("making a scratch directory for the capture run")?;
    let files = weave_into(source, scratch.path())?;
    // `entry_point` names the SOURCE file and `adapter_for` reads the
    // language from it — both unchanged, because the source is what has a
    // language. What that source *produces* is what gets launched, and for
    // Python, Node and Go the two are the same path.
    let entry = entry_point(&files)?;
    let adapter = adapter_for(&entry, project)?;
    // A capture has no terminal to watch a build in, so the build's own words
    // go to the log. That is weaker than the interactive path on purpose
    // rather than by oversight: `hick test` is not a person watching, and the
    // failure message points at the log the way the session's points at the
    // terminal.
    let program = build(&entry, scratch.path(), project, &mut |line| match line {
        BuildOutput::Cmd(text) | BuildOutput::Note(text) | BuildOutput::Out(text) => {
            tracing::info!(target: "hick_dap::build", "{text}")
        }
        BuildOutput::Err(text) => tracing::warn!(target: "hick_dap::build", "{text}"),
        BuildOutput::Exit(code) => tracing::info!(target: "hick_dap::build", "exit {code}"),
    })
    .await?;
    let mapping = Arc::new(Mapping::for_document(document, source, scratch.path())?);

    // A capture names a location in a generated file; a breakpoint is set in
    // document lines. Resolving here means a bad `at` is reported against the
    // capture that wrote it rather than as a breakpoint that never verifies.
    let mut resolved: Vec<(usize, Option<u32>)> = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let path = scratch.path().join(&spec.file);
        // `at` is 1-based the way an editor shows it; the mapping is 0-based.
        let doc_line = mapping.to_document(&path, spec.line.saturating_sub(1));
        resolved.push((index, doc_line));
    }

    let breakpoints: Vec<Breakpoint> = resolved
        .iter()
        .filter_map(|(index, line)| {
            line.map(|line| Breakpoint {
                line,
                condition: specs[*index].condition.clone(),
                hit_condition: None,
                log_message: None,
            })
        })
        .collect();

    let mut out: Vec<Captured> = specs
        .iter()
        .zip(&resolved)
        .map(|(spec, (_, line))| Captured {
            spec: spec.clone(),
            hits: Vec::new(),
            truncated: false,
            moved: None,
            problem: match line {
                Some(_) => None,
                None => Some(format!(
                    "{}:{} is not a line this document generates. `at` names a line in a \
                     `hick:file` block's *generated* file, counted from 1.",
                    spec.file, spec.line
                )),
            },
        })
        .collect();

    if breakpoints.is_empty() {
        return Ok(out);
    }

    let (session, statuses) = Session::start(
        Launch {
            adapter: adapter.command,
            program,
            cwd: scratch.path().to_path_buf(),
            extra: serde_json::json!({}),
        },
        mapping.clone(),
        &breakpoints,
    )
    .await?;

    for (slot, (index, line)) in resolved.iter_mut().enumerate() {
        let Some(asked) = *line else { continue };
        let Some(status) = statuses.iter().find(|s| s.line == asked) else {
            continue;
        };
        if !status.verified {
            out[slot].problem = Some(format!(
                "the debugger would not place a breakpoint at {}:{}{}. A blank line, a comment, \
                 or a line the program never loads cannot hold one.",
                specs[*index].file,
                specs[*index].line,
                status
                    .message
                    .as_deref()
                    .map(|m| format!(" ({m})"))
                    .unwrap_or_default(),
            ));
        }
        if let Some(moved) = status.moved_to {
            // Follow it. A capture written on a docstring or a blank line
            // binds to the statement below, and recording nothing there
            // would report "never hit" about a breakpoint that fired every
            // time. The document is told where it ended up.
            *line = Some(moved);
            out[slot].moved = Some(moved);
        }
    }

    let result = drive(&session, &mapping, &resolved, specs, &mut out).await;
    session.shutdown().await;
    result?;
    Ok(out)
}

/// Run the program to the end, recording every hit on the way.
async fn drive(
    session: &Session,
    mapping: &Mapping,
    resolved: &[(usize, Option<u32>)],
    specs: &[CaptureSpec],
    out: &mut [Captured],
) -> Result<()> {
    let deadline = tokio::time::Instant::now() + RUN_BUDGET;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            anyhow::bail!(
                "the capture run did not finish within {} seconds. A capture runs the program \
                 under a debugger, so a cell that waits for input or for the network will not \
                 finish here even when it finishes on its own.",
                RUN_BUDGET.as_secs()
            );
        }
        // Every stop is one of ours: the only breakpoints set are the
        // captures'. A stop for any other reason (an uncaught exception) ends
        // the run the way the program ending does.
        let Some(stopped) = session.wait_for_stop(remaining).await? else {
            return Ok(());
        };
        let frames = session.stack(stopped.thread_id).await.unwrap_or_default();
        let Some(frame) = frames.first() else {
            return Ok(());
        };

        if let Some(slot) = slot_for(frame.line, mapping, resolved) {
            record(session, frame.id, &specs[slot], &mut out[slot]).await;
            if out[slot].hits.len() >= specs[slot].max.max(1) {
                out[slot].truncated = true;
                // Removing the breakpoint would be tidier, but changing the
                // set mid-run costs a round trip per capture and can move the
                // others; stepping past it costs nothing.
            }
        }
        // Stopped somewhere no capture asked about — an exception, most
        // likely. Nothing to record, and continuing lets the adapter report it
        // the way it would have anyway.

        if session
            .step(Step::Continue, stopped.thread_id, None)
            .await
            .is_err()
        {
            return Ok(());
        }
    }
}

/// Which capture is this stop for?
fn slot_for(
    frame_line: Option<u32>,
    _mapping: &Mapping,
    resolved: &[(usize, Option<u32>)],
) -> Option<usize> {
    let line = frame_line?;
    resolved
        .iter()
        .position(|(_, doc_line)| *doc_line == Some(line))
}

/// Evaluate one capture's expressions in the frame it stopped in.
async fn record(session: &Session, frame_id: i64, spec: &CaptureSpec, into: &mut Captured) {
    if into.hits.len() >= spec.max.max(1) {
        return;
    }
    let mut values = Vec::new();
    for expression in &spec.expressions {
        // `watch`, not `repl`: the context adapters treat as repeatable and
        // side-effect-free, which is what a document's expression must be.
        match session.evaluate(expression, Some(frame_id), "watch").await {
            Ok(variable) => values.push(Recorded {
                expression: expression.clone(),
                value: variable.value,
                ok: true,
            }),
            Err(error) => values.push(Recorded {
                expression: expression.clone(),
                value: format!("{error}")
                    .lines()
                    .next()
                    .unwrap_or("failed")
                    .to_string(),
                ok: false,
            }),
        }
    }
    into.hits.push(Hit {
        index: into.hits.len() + 1,
        values,
    });
}

/// Parse `pricing.py:14` into its parts.
pub fn parse_at(at: &str) -> Result<(String, u32), String> {
    let (file, line) = at.rsplit_once(':').ok_or_else(|| {
        format!(
            "capture location `{at}` has no line number. Write it as \
             `at=\"file.py:14\"` — the file a `hick:file` block generates, and the line in it."
        )
    })?;
    let line: u32 = line
        .trim()
        .parse()
        .map_err(|_| format!("capture location `{at}` does not end in a line number"))?;
    if file.trim().is_empty() {
        return Err(format!("capture location `{at}` names no file"));
    }
    if line == 0 {
        return Err(format!(
            "capture location `{at}` starts counting at 0; lines are counted from 1"
        ));
    }
    Ok((file.trim().to_string(), line))
}

/// Split an `of` attribute into expressions.
///
/// Commas separate, but not inside brackets or a string — `of="f(a, b), c"`
/// is two expressions, and getting that wrong would silently evaluate
/// something the author did not write.
pub fn split_expressions(of: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut current = String::new();
    let mut chars = of.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), '\\') => {
                // Keep the escape and whatever it escapes together, so a
                // quote inside a string does not end it.
                current.push(c);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                let _ = q;
                continue;
            }
            (Some(q), c) if c == q => {
                quote = None;
                current.push(c);
            }
            (Some(_), c) => current.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                current.push(c);
            }
            (None, '(' | '[' | '{') => {
                depth += 1;
                current.push(c);
            }
            (None, ')' | ']' | '}') => {
                depth -= 1;
                current.push(c);
            }
            (None, ',') if depth <= 0 => {
                out.push(current.trim().to_string());
                current = String::new();
            }
            (None, c) => current.push(c),
        }
    }
    out.push(current.trim().to_string());
    out.retain(|e| !e.is_empty());
    out
}

/// Render one capture as the table the weaver puts under the cell.
pub fn render(captured: &Captured) -> String {
    let mut out = String::new();
    let location = format!("{}:{}", captured.spec.file, captured.spec.line);
    if let Some(problem) = &captured.problem {
        return format!("_capture at {location}: {problem}_\n");
    }
    if captured.moved.is_some() && !captured.hits.is_empty() {
        out.push_str(&format!(
            "_the debugger moved this capture off {location} to the next line that can hold a \
             breakpoint_\n\n"
        ));
    }
    if captured.hits.is_empty() {
        return format!(
            "_capture at {location}: never hit{}_\n",
            captured
                .spec
                .condition
                .as_deref()
                .map(|c| format!(" (condition `{c}`)"))
                .unwrap_or_default()
        );
    }

    out.push_str("| hit |");
    for expression in &captured.spec.expressions {
        out.push_str(&format!(" `{}` |", escape_cell(expression)));
    }
    out.push_str("\n|-----|");
    for _ in &captured.spec.expressions {
        out.push_str("------|");
    }
    out.push('\n');
    for hit in &captured.hits {
        out.push_str(&format!("| {} |", hit.index));
        for value in &hit.values {
            let rendered = if value.ok {
                format!("`{}`", escape_cell(&value.value))
            } else {
                escape_cell(&value.value)
            };
            out.push_str(&format!(" {rendered} |"));
        }
        out.push('\n');
    }
    if captured.truncated {
        out.push_str(&format!(
            "\n_stopped after {} hits ({location} has a `max` of {})_\n",
            captured.hits.len(),
            captured.spec.max
        ));
    }
    out
}

/// A value with a newline or a pipe in it would break the table it sits in.
fn escape_cell(text: &str) -> String {
    let flattened = text.replace('\n', " ").replace('|', "\\|");
    if flattened.chars().count() > 60 {
        let head: String = flattened.chars().take(59).collect();
        format!("{head}…")
    } else {
        flattened
    }
}

/// The generated files a set of captures name, for reporting.
pub fn files(specs: &[CaptureSpec]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = specs.iter().map(|s| PathBuf::from(&s.file)).collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_is_a_file_and_a_line() {
        assert_eq!(
            parse_at("pricing.py:14").unwrap(),
            ("pricing.py".into(), 14)
        );
        assert_eq!(
            parse_at("src/app/main.py:2").unwrap(),
            ("src/app/main.py".into(), 2)
        );
    }

    #[test]
    fn a_location_without_a_line_says_how_to_write_one() {
        let error = parse_at("pricing.py").unwrap_err();
        assert!(error.contains("at=\"file.py:14\""), "{error}");
    }

    #[test]
    fn lines_are_counted_from_one() {
        // Off-by-one here would capture the wrong line and look right.
        assert!(parse_at("a.py:0").unwrap_err().contains("counted from 1"));
    }

    #[test]
    fn expressions_split_on_commas() {
        assert_eq!(
            split_expressions("subtotal, len(lines)"),
            ["subtotal", "len(lines)"]
        );
    }

    #[test]
    fn a_comma_inside_a_call_is_not_a_separator() {
        // `of="max(a, b), c"` is two expressions. Splitting naively would
        // evaluate `max(a` and be told it is a syntax error.
        assert_eq!(split_expressions("max(a, b), c"), ["max(a, b)", "c"]);
        assert_eq!(split_expressions("d[\"x, y\"]"), ["d[\"x, y\"]"]);
    }

    #[test]
    fn a_comma_inside_a_string_is_not_a_separator() {
        assert_eq!(split_expressions("'a, b'"), ["'a, b'"]);
        assert_eq!(split_expressions("\"it's, fine\""), ["\"it's, fine\""]);
    }

    #[test]
    fn empty_pieces_are_dropped() {
        assert_eq!(split_expressions("a, , b,"), ["a", "b"]);
        assert!(split_expressions("  ").is_empty());
    }

    fn captured(hits: Vec<Hit>, truncated: bool) -> Captured {
        Captured {
            spec: CaptureSpec {
                file: "pricing.py".into(),
                line: 14,
                expressions: vec!["subtotal".into(), "len(lines)".into()],
                condition: None,
                max: 2,
            },
            hits,
            truncated,
            moved: None,
            problem: None,
        }
    }

    fn hit(index: usize, a: &str, b: &str) -> Hit {
        Hit {
            index,
            values: vec![
                Recorded {
                    expression: "subtotal".into(),
                    value: a.into(),
                    ok: true,
                },
                Recorded {
                    expression: "len(lines)".into(),
                    value: b.into(),
                    ok: true,
                },
            ],
        }
    }

    #[test]
    fn hits_weave_as_a_table_ordered_by_hit() {
        let table = render(&captured(
            vec![hit(1, "0", "3"), hit(2, "19.99", "3")],
            false,
        ));
        assert_eq!(
            table,
            "| hit | `subtotal` | `len(lines)` |\n\
             |-----|------|------|\n\
             | 1 | `0` | `3` |\n\
             | 2 | `19.99` | `3` |\n"
        );
    }

    #[test]
    fn a_bounded_capture_says_it_stopped_early() {
        // Silent truncation would read as "that is all there was".
        let table = render(&captured(vec![hit(1, "0", "3"), hit(2, "1", "3")], true));
        assert!(table.contains("stopped after 2 hits"), "{table}");
    }

    #[test]
    fn a_capture_that_never_fired_says_so() {
        let mut c = captured(Vec::new(), false);
        c.spec.condition = Some("subtotal < 0".into());
        let table = render(&c);
        assert!(table.contains("never hit"), "{table}");
        assert!(table.contains("subtotal < 0"), "{table}");
    }

    #[test]
    fn a_value_with_a_pipe_does_not_break_the_table() {
        let table = render(&captured(vec![hit(1, "a|b", "3")], false));
        assert!(table.contains("a\\|b"), "{table}");
        // Header, separator, one row — the pipe stayed inside its cell.
        assert_eq!(table.lines().count(), 3);
    }

    #[test]
    fn a_long_value_is_abbreviated_rather_than_wrapped() {
        let table = render(&captured(vec![hit(1, &"x".repeat(200), "3")], false));
        assert!(table.contains('…'), "{table}");
        assert!(table.lines().nth(2).unwrap().len() < 100);
    }

    #[test]
    fn an_unplaceable_location_explains_itself_instead_of_showing_a_table() {
        let mut c = captured(Vec::new(), false);
        c.problem = Some("nope".into());
        assert_eq!(render(&c), "_capture at pricing.py:14: nope_\n");
    }
}
