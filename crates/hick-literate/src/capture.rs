//! `<hick:capture>` declarations, read off the document.
//!
//! A capture is the non-interactive half of the debugger: a breakpoint the
//! document owns, evaluated by the same session API the app's debugger drives
//! (`hick_dap::capture`). This module is the *parsing* half — what the tag
//! means and how a malformed one is reported. The running is in `hick-dap`,
//! and the two are deliberately separate: a document can be checked for a
//! nonsense `at=` with no adapter installed and no program run.
//!
//! ```xml
//! <hick:exec container="lab">
//! python3 pricing.py
//!   <hick:capture at="pricing.py:14" of="subtotal, len(lines)" when="subtotal < 0" />
//! </hick:exec>
//! ```
//!
//! `at` is a location in a generated file, which is a location in the
//! document. `of` is a list of expressions. `when` is a breakpoint condition —
//! and a condition wants `<` and `>`, which the no-escaping invariant makes
//! ordinary rather than awkward: a comparison reads as a comparison.
//!
//! See `docs/specs/freeform/literate-debugging.md`.

use hick_dap::capture::{CaptureSpec, DEFAULT_MAX_HITS, parse_at, split_expressions};
use hick_lang::{HickNode, HickTag};

use crate::{CellId, tag_attr};

/// Every capture one cell declared, in document order.
#[derive(Debug, Clone)]
pub struct CellCaptures {
    pub cell: CellId,
    /// 1-based source line of the owning exec tag.
    pub exec_line: usize,
    pub specs: Vec<CaptureSpec>,
}

/// Collect the captures declared in a document.
///
/// Returns an error for a capture that cannot mean anything — a missing or
/// malformed `at`, an empty `of`, a `max` of zero — rather than running the
/// program and weaving an empty table, which would read as "this never
/// happened" when the truth is "this was never asked".
pub fn collect(nodes: &[HickNode]) -> Result<Vec<CellCaptures>, String> {
    let mut out = Vec::new();
    collect_from(nodes, &mut out)?;
    Ok(out)
}

fn collect_from(nodes: &[HickNode], out: &mut Vec<CellCaptures>) -> Result<(), String> {
    for node in nodes {
        let HickNode::Tag(tag) = node else { continue };
        if tag.name == "capture" {
            // Reached without passing through an exec, so it belongs to
            // nothing. Silently ignoring it is how a document ends up
            // claiming to verify something it never looked at.
            return Err(format!(
                "line {}: <hick:capture> must be a child of <hick:exec> — it is a breakpoint \
                 in the program that cell runs, so on its own there is nothing to stop.",
                tag.source_line
            ));
        }
        if tag.name == "exec" {
            let captures: Vec<&HickTag> =
                tag.child_tags().filter(|t| t.name == "capture").collect();
            if !captures.is_empty() {
                let mut specs = Vec::new();
                for capture in captures {
                    specs.push(parse_capture(capture)?);
                }
                out.push(CellCaptures {
                    cell: CellId::exec(
                        tag_attr(tag, "container").unwrap_or_default(),
                        tag.source_line,
                    ),
                    exec_line: tag.source_line,
                    specs,
                });
            }
            // Its own captures are accounted for; anything nested deeper is
            // not an exec's child and is caught by the branch above.
            for child in tag.child_tags().filter(|t| t.name != "capture") {
                collect_from(&child.children, out)?;
            }
            continue;
        }
        collect_from(&tag.children, out)?;
    }
    Ok(())
}

fn parse_capture(tag: &HickTag) -> Result<CaptureSpec, String> {
    let line_no = tag.source_line;
    let at = tag_attr(tag, "at").ok_or_else(|| {
        format!(
            "line {line_no}: <hick:capture> has no `at`. Write the location to stop at as \
             `at=\"pricing.py:14\"` — the file a `hick:file` block generates, and the line in it."
        )
    })?;
    let (file, line) = parse_at(&at).map_err(|e| format!("line {line_no}: {e}"))?;

    let of = tag_attr(tag, "of").ok_or_else(|| {
        format!(
            "line {line_no}: <hick:capture> has no `of`. List what to record as \
             `of=\"subtotal, len(lines)\"` — expressions in the debugged program's own language."
        )
    })?;
    let expressions = split_expressions(&of);
    if expressions.is_empty() {
        return Err(format!(
            "line {line_no}: <hick:capture of=\"{of}\"> lists no expression to record."
        ));
    }

    let max = match tag_attr(tag, "max") {
        None => DEFAULT_MAX_HITS,
        Some(raw) => {
            let parsed: usize = raw.trim().parse().map_err(|_| {
                format!(
                    "line {line_no}: <hick:capture max=\"{raw}\"> is not a number. `max` bounds \
                     how many hits are recorded (default {DEFAULT_MAX_HITS})."
                )
            })?;
            if parsed == 0 {
                return Err(format!(
                    "line {line_no}: <hick:capture max=\"0\"> would record nothing. Remove the \
                     capture instead, or raise `max`."
                ));
            }
            parsed
        }
    };

    Ok(CaptureSpec {
        file,
        line,
        expressions,
        condition: tag_attr(tag, "when").filter(|c| !c.trim().is_empty()),
        max,
    })
}

/// Run one cell's captures and render what they recorded.
///
/// Returns the text to append to the cell's output, which is empty when there
/// was nothing to record.
///
/// **A capture degrades rather than failing the run.** A machine with no debug
/// adapter installed is the normal state of CI and of a reader who cloned the
/// repository to look at it, and a document that cannot be *read* without
/// `debugpy` would be a worse document. What the run weaves in that case says
/// what is missing and how to get it — the same treatment a missing model
/// credential gets for an agent cell.
pub async fn run_cell(
    document: &std::path::Path,
    source: &str,
    project: &std::path::Path,
    specs: &[CaptureSpec],
) -> String {
    match hick_dap::capture::run(document, source, project, specs).await {
        Ok(captured) => captured
            .iter()
            .map(hick_dap::capture::render)
            .collect::<Vec<_>>()
            .join("\n"),
        Err(error) => {
            log::warn!(
                "captures in {} could not run: {error:#}",
                document.display()
            );
            format!("_captures did not run: {error:#}_\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Vec<CellCaptures>, String> {
        let doc = hick_lang::parse(source).expect("the document parses");
        collect(&doc.nodes)
    }

    const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:exec container="lab">
python3 pricing.py
  <hick:capture at="pricing.py:14" of="subtotal, len(lines)" when="subtotal &lt; 0" max="5" />
</hick:exec>
</hick:doc>
"##;

    #[test]
    fn a_capture_is_read_off_its_cell() {
        let found = parse(&DOC.replace("&lt;", "<")).unwrap();
        assert_eq!(found.len(), 1);
        let spec = &found[0].specs[0];
        assert_eq!(spec.file, "pricing.py");
        assert_eq!(spec.line, 14);
        assert_eq!(spec.expressions, ["subtotal", "len(lines)"]);
        // A raw `<` in an attribute, which the no-escaping invariant makes
        // ordinary — a comparison reads as a comparison.
        assert_eq!(spec.condition.as_deref(), Some("subtotal < 0"));
        assert_eq!(spec.max, 5);
    }

    #[test]
    fn hits_are_bounded_by_default() {
        let source = DOC.replace("&lt;", "<").replace(" max=\"5\"", "");
        assert_eq!(parse(&source).unwrap()[0].specs[0].max, DEFAULT_MAX_HITS);
    }

    #[test]
    fn a_cell_may_declare_several() {
        let source = DOC.replace("&lt;", "<").replace(
            "</hick:exec>",
            "  <hick:capture at=\"pricing.py:2\" of=\"LINES\" />\n</hick:exec>",
        );
        assert_eq!(parse(&source).unwrap()[0].specs.len(), 2);
    }

    #[test]
    fn a_capture_with_no_location_says_how_to_write_one() {
        let source = DOC
            .replace("&lt;", "<")
            .replace("at=\"pricing.py:14\" ", "");
        let error = parse(&source).unwrap_err();
        assert!(error.contains("at=\"pricing.py:14\""), "{error}");
    }

    #[test]
    fn a_capture_with_nothing_to_record_is_an_error() {
        let source = DOC
            .replace("&lt;", "<")
            .replace("of=\"subtotal, len(lines)\" ", "");
        assert!(parse(&source).unwrap_err().contains("no `of`"));
    }

    #[test]
    fn a_max_that_is_not_a_number_is_reported_against_its_line() {
        let source = DOC
            .replace("&lt;", "<")
            .replace("max=\"5\"", "max=\"lots\"");
        let error = parse(&source).unwrap_err();
        assert!(error.contains("line 5"), "{error}");
        assert!(error.contains("is not a number"), "{error}");
    }

    #[test]
    fn a_max_of_zero_would_record_nothing() {
        let source = DOC.replace("&lt;", "<").replace("max=\"5\"", "max=\"0\"");
        assert!(parse(&source).unwrap_err().contains("Remove the capture"));
    }

    #[test]
    fn a_capture_outside_a_cell_is_refused() {
        let source = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:capture at="a.py:1" of="x" />
</hick:doc>
"##;
        let error = parse(source).unwrap_err();
        assert!(error.contains("must be a child of <hick:exec>"), "{error}");
    }

    #[test]
    fn a_document_with_no_captures_collects_nothing() {
        let source = DOC
            .replace("&lt;", "<")
            .replace("  <hick:capture at=\"pricing.py:14\" of=\"subtotal, len(lines)\" when=\"subtotal < 0\" max=\"5\" />\n", "");
        assert!(parse(&source).unwrap().is_empty());
    }
}
