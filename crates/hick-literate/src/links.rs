//! Cross-document links, as they must read in the woven markdown.
//!
//! A note that links to another note is written the way anyone would write
//! it: `[the plan](plan.hick)`. That is correct in the SOURCE — `plan.hick`
//! is the file beside it — and wrong in the WEAVE, because every document
//! weaves a `.md` of its own name (`docs/specs/freeform/bare-documents.md`)
//! and the reader of `notes.md` has `plan.md`, not `plan.hick`. A link that
//! points at the source from the weave is a link that resolves to a file the
//! reader may not be looking at, or on a static host, to nothing at all.
//!
//! So the weaver rewrites exactly that one thing: the destination of a
//! markdown link, when it is a relative path inside the folder that ends in
//! `.hick`. Everything else — the label, the title, a URL, an image, a
//! fragment, an absolute path, a link to a `.csv` or a `.png` — is left byte
//! for byte as written.
//!
//! ## Why this is a segmenter and not a string replace
//!
//! Because of the ribbons. Prose reaches the weave as one node carrying the
//! span it came from, and `hick_flow::ProvenanceTransformNode` splits that
//! node into segments, narrowing each passthrough segment's span to the bytes
//! it actually covers. Rewriting the text in place would produce one node
//! whose length no longer matches its span, and the lineage layer would
//! (correctly) call the whole paragraph synthetic — no ribbon, and an edit
//! from the woven side that could only be refused.
//!
//! Segmenting keeps the label, the brackets, and every word around the link
//! attached to the source text they came from. Only the four bytes that
//! changed (`hick` → `md`) are marked substituted, which is exactly true:
//! those bytes are the weaver's, not the author's.
//!
//! The rule this implements is the same one the editor uses to decide where a
//! link goes — see `apps/web/src/lib/mdLinks.ts` (`wovenTarget`). Two
//! implementations of one rule is a real cost; the alternative is the editor
//! asking the server where every link in the buffer points, on every
//! keystroke, which is worse. The tests on both sides use the same cases.

use hick_exec::node::{TransformSegment, TransformSegmentOrigin};

/// The extension a document is written with, and the one it weaves.
const SOURCE_EXT: &str = ".hick";
const WOVEN_EXT: &str = ".md";

/// The destination a link should carry in woven markdown, or `None` when it
/// should be left exactly as written.
///
/// `None` for a URL (it has a scheme), for a protocol-relative address, for a
/// bare `#fragment` (already correct — it points inside the woven file), and
/// for anything that is not a `.hick` path.
pub(crate) fn woven_target(target: &str) -> Option<String> {
    if target.is_empty() || target.starts_with('#') || target.starts_with("//") {
        return None;
    }
    if has_scheme(target) {
        return None;
    }
    let (path, fragment) = match target.find('#') {
        Some(at) => (&target[..at], &target[at..]),
        None => (target, ""),
    };
    if !path.to_ascii_lowercase().ends_with(SOURCE_EXT) {
        return None;
    }
    Some(format!(
        "{}{WOVEN_EXT}{fragment}",
        &path[..path.len() - SOURCE_EXT.len()]
    ))
}

/// `scheme:` at the start — `https:`, `mailto:`, and also `C:` on Windows,
/// which is deliberate: an absolute Windows path is not a link this weave can
/// make portable, so it is left alone rather than half-rewritten.
fn has_scheme(target: &str) -> bool {
    let Some(colon) = target.find(':') else {
        return false;
    };
    if colon == 0 {
        return false;
    }
    let mut chars = target[..colon].chars();
    let first = chars.next().unwrap_or(' ');
    first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
}

/// Byte ranges of the DESTINATIONS of markdown links in `text`, in order.
///
/// A deliberately small scanner rather than a markdown parser: it finds a
/// `](`, balances the parentheses that close it (Wikipedia URLs contain
/// them), stops at a newline because an unclosed one is a typo rather than a
/// link running to the end of the document, and excludes a `"title"` after
/// the destination.
pub(crate) fn destination_spans(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] != b']' || bytes[i + 1] != b'(' {
            i += 1;
            continue;
        }
        // An escaped bracket closes nothing.
        if i > 0 && bytes[i - 1] == b'\\' {
            i += 1;
            continue;
        }
        let start = i + 2;
        let Some(close) = closing_paren(bytes, start) else {
            i += 1;
            continue;
        };
        let end = title_start(&text[start..close])
            .map(|at| start + at)
            .unwrap_or(close);
        if end > start {
            found.push((start, end));
        }
        i = close + 1;
    }
    found
}

/// Index of the `)` closing the `(` before `from`, or `None`.
fn closing_paren(bytes: &[u8], from: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => return None,
            b'\\' => i += 1,
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Where a `"…"` / `'…'` title begins inside a destination, if one does.
fn title_start(inner: &str) -> Option<usize> {
    let bytes = inner.as_bytes();
    for i in 0..bytes.len() {
        if !bytes[i].is_ascii_whitespace() {
            continue;
        }
        let rest = &bytes[i + 1..];
        let quoted = rest
            .iter()
            .find(|b| !b.is_ascii_whitespace())
            .is_some_and(|b| *b == b'"' || *b == b'\'');
        if quoted {
            return Some(i);
        }
    }
    None
}

/// Split one run of text so every `.hick` destination in it becomes its own
/// substituted segment, and everything around it stays passthrough.
///
/// The `pattern` on a substituted segment is the text it REPLACED, not the
/// text it produced: `ProvenanceTransformNode` advances its position in the
/// input by that length, which is what keeps the spans of the passthrough
/// segments after it correct.
fn split_one(text: &str) -> Vec<TransformSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    for (start, end) in destination_spans(text) {
        let target = &text[start..end];
        let Some(woven) = woven_target(target.trim()) else {
            continue;
        };
        if start > cursor {
            segments.push(TransformSegment {
                text: text[cursor..start].to_string(),
                origin: TransformSegmentOrigin::Passthrough,
            });
        }
        // The trim is preserved around the rewritten path: a destination
        // written with padding keeps it, because this pass is about the
        // extension and nothing else.
        let lead = target.len() - target.trim_start().len();
        let trail = target.len() - target.trim_end().len();
        segments.push(TransformSegment {
            text: format!(
                "{}{woven}{}",
                &target[..lead],
                &target[target.len() - trail..]
            ),
            origin: TransformSegmentOrigin::Substituted {
                pattern: target.to_string(),
            },
        });
        cursor = end;
    }
    if segments.is_empty() {
        return vec![TransformSegment {
            text: text.to_string(),
            origin: TransformSegmentOrigin::Passthrough,
        }];
    }
    if cursor < text.len() {
        segments.push(TransformSegment {
            text: text[cursor..].to_string(),
            origin: TransformSegmentOrigin::Passthrough,
        });
    }
    segments
}

/// Rewrite document links in a segmented weave.
///
/// Runs over the output of the substitution segmenter, so a link inside a
/// substituted value is left alone: those bytes are already the weaver's and
/// have no source span to keep intact.
pub(crate) fn rewrite_document_links(segments: Vec<TransformSegment>) -> Vec<TransformSegment> {
    let mut out = Vec::with_capacity(segments.len());
    for segment in segments {
        match segment.origin {
            TransformSegmentOrigin::Passthrough => out.extend(split_one(&segment.text)),
            TransformSegmentOrigin::Substituted { .. } => out.push(segment),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(segments: &[TransformSegment]) -> String {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    fn passthrough(text: &str) -> Vec<TransformSegment> {
        vec![TransformSegment {
            text: text.to_string(),
            origin: TransformSegmentOrigin::Passthrough,
        }]
    }

    #[test]
    fn a_document_link_points_at_the_markdown_that_document_weaves() {
        assert_eq!(woven_target("plan.hick").as_deref(), Some("plan.md"));
        assert_eq!(
            woven_target("notes/weekly/mon.hick").as_deref(),
            Some("notes/weekly/mon.md")
        );
        assert_eq!(
            woven_target("plan.hick#risks").as_deref(),
            Some("plan.md#risks")
        );
        assert_eq!(woven_target("../plan.HICK").as_deref(), Some("../plan.md"));
    }

    #[test]
    fn everything_that_is_not_a_document_link_is_left_exactly_as_written() {
        for target in [
            "https://example.com/a.hick",
            "mailto:nate@example.com",
            "//cdn.example.com/a.hick",
            "#section",
            "assets/chart.png",
            "data.csv",
            "",
        ] {
            assert_eq!(woven_target(target), None, "{target}");
        }
    }

    #[test]
    fn the_rewrite_touches_the_destination_and_nothing_around_it() {
        let out = rewrite_document_links(passthrough(
            "See [the plan](plan.hick) and [a chart](assets/c.png).",
        ));
        assert_eq!(
            text_of(&out),
            "See [the plan](plan.md) and [a chart](assets/c.png)."
        );
    }

    #[test]
    fn only_the_changed_bytes_are_marked_substituted_so_the_prose_keeps_its_ribbon() {
        let out = rewrite_document_links(passthrough("See [the plan](plan.hick) today."));
        let kinds: Vec<&str> = out
            .iter()
            .map(|s| match s.origin {
                TransformSegmentOrigin::Passthrough => "source",
                TransformSegmentOrigin::Substituted { .. } => "woven",
            })
            .collect();
        assert_eq!(kinds, ["source", "woven", "source"]);
        assert_eq!(out[0].text, "See [the plan](");
        assert_eq!(out[1].text, "plan.md");
        assert_eq!(out[2].text, ") today.");
    }

    #[test]
    fn the_pattern_is_what_was_replaced_so_later_spans_stay_aligned() {
        // ProvenanceTransformNode advances through the INPUT by the pattern's
        // length. A pattern holding the output instead would slide every
        // span after the first link by the difference in length.
        let out = rewrite_document_links(passthrough("[a](one.hick) [b](two.hick)"));
        let patterns: Vec<String> = out
            .iter()
            .filter_map(|s| match &s.origin {
                TransformSegmentOrigin::Substituted { pattern } => Some(pattern.clone()),
                TransformSegmentOrigin::Passthrough => None,
            })
            .collect();
        assert_eq!(patterns, ["one.hick", "two.hick"]);
        assert_eq!(text_of(&out), "[a](one.md) [b](two.md)");
    }

    #[test]
    fn a_title_after_the_destination_survives() {
        let out = rewrite_document_links(passthrough("[a](plan.hick \"why it matters\")"));
        assert_eq!(text_of(&out), "[a](plan.md \"why it matters\")");
    }

    #[test]
    fn parentheses_inside_a_url_do_not_end_it_early() {
        let out = rewrite_document_links(passthrough(
            "[x](https://en.wikipedia.org/wiki/Foo_(bar)) and [y](p.hick)",
        ));
        assert_eq!(
            text_of(&out),
            "[x](https://en.wikipedia.org/wiki/Foo_(bar)) and [y](p.md)"
        );
    }

    #[test]
    fn an_image_of_a_document_is_still_rewritten_and_a_broken_link_is_not_touched() {
        assert_eq!(
            text_of(&rewrite_document_links(passthrough("![a](p.hick)"))),
            "![a](p.md)"
        );
        // Unclosed, and spanning a newline: a typo, left alone.
        assert_eq!(
            text_of(&rewrite_document_links(passthrough("[a](p.hick\n)"))),
            "[a](p.hick\n)"
        );
    }

    #[test]
    fn text_with_no_links_comes_back_as_exactly_one_untouched_segment() {
        let out = rewrite_document_links(passthrough("just prose"));
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].origin, TransformSegmentOrigin::Passthrough));
    }

    #[test]
    fn a_link_inside_a_substituted_value_is_left_to_the_substitution() {
        let out = rewrite_document_links(vec![TransformSegment {
            text: "[a](p.hick)".to_string(),
            origin: TransformSegmentOrigin::Substituted {
                pattern: "{{link}}".to_string(),
            },
        }]);
        assert_eq!(text_of(&out), "[a](p.hick)");
    }
}
