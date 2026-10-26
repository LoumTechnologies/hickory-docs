<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Agent Edit Session Never Applies A Stale Edit

For: Authors using an agent to change documents while retaining review control.

Given an agent edit session on a primary document, when a document or upstream
changes between read and edit, then the next edit refuses and directs another
read. Successful edits re-weave before returning fresh SHA-256 line anchors.
Ambiguous anchors require an occurrence; old short anchors do not resolve.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/edit-session-staleness-impossible.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-55ecf32bef2e"></a>
## `crates/hickory-agent/src/tools/hashline.rs`

Related contracts: [An Agent Edit Session Never Applies A Stale Edit](contract.md).

<a id="f-55ecf32bef2e-line_hash"></a>
### `line_hash`

Hashline rendering and anchor resolution.
Both editable surfaces (document source and woven outputs) are shown to
the model with every line prefixed by its SHA-256 content hash. Whole-surface
snapshots additionally detect changes inside a range. A short legacy hash
can be parsed for a useful refusal, but cannot match new surfaces. Duplicate-content lines share a hash by design;
edits disambiguate by contiguous runs and, when the run anchor pair is
still ambiguous, an explicit 1-based occurrence index.

<storycode:copy id="f-55ecf32bef2e-line_hash">
//! Hashline rendering and anchor resolution.
//!
//! Both editable surfaces (document source and woven outputs) are shown to
//! the model with every line prefixed by its SHA-256 content hash. Whole-surface
//! snapshots additionally detect changes inside a range. A short legacy hash
//! can be parsed for a useful refusal, but cannot match new surfaces. Duplicate-content lines share a hash by design;
//! edits disambiguate by contiguous runs and, when the run anchor pair is
//! still ambiguous, an explicit 1-based occurrence index.

use std::fmt::Write as _;

/// SHA-256 content identity for one line. Old short anchors no longer resolve.
pub fn line_hash(line: &str) -> String {
    line_fingerprint(line)
}

</storycode:copy>
<a id="f-55ecf32bef2e-line_fingerprint"></a>
### `line_fingerprint`

Strong identity used for attribution, independently of display anchors.

<storycode:copy id="f-55ecf32bef2e-line_fingerprint">
/// Strong identity used for attribution, independently of display anchors.
pub fn line_fingerprint(line: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(line.as_bytes()))
}

</storycode:copy>
<a id="f-55ecf32bef2e-LineIndex"></a>
### `LineIndex`

A text buffer split into lines, with byte offsets and content hashes.

<storycode:copy id="f-55ecf32bef2e-LineIndex">
/// A text buffer split into lines, with byte offsets and content hashes.
pub struct LineIndex {
    pub snapshot: String,
    /// Line texts, without their trailing newlines.
    pub lines: Vec<String>,
    /// Byte offset where each line starts.
    pub starts: Vec<usize>,
    /// SHA-256 content hash per line.
    pub hashes: Vec<String>,
    /// Whether the buffer ends with a newline.
    pub trailing_newline: bool,
    ends: Vec<usize>,
}

</storycode:copy>
<a id="f-55ecf32bef2e-impl-LineIndex"></a>
### `impl-LineIndex`

<storycode:copy id="f-55ecf32bef2e-impl-LineIndex">
impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut lines = Vec::new();
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        let mut pos = 0;
        for line in text.split_inclusive('\n') {
            starts.push(pos);
            pos += line.len();
            ends.push(pos);
            let content = line.strip_suffix('\n').unwrap_or(line);
            let content = content.strip_suffix('\r').unwrap_or(content);
            lines.push(content.to_string());
        }
        if text.is_empty() {
            starts.push(0);
            lines.push(String::new());
            ends.push(0);
        }
        let hashes = lines.iter().map(|l| line_hash(l)).collect();
        Self {
            snapshot: line_fingerprint(text),
            lines,
            starts,
            hashes,
            trailing_newline: text.ends_with('\n'),
            ends,
        }
    }

    /// Byte offset just past the content of line `i` (excluding its newline).
    pub fn content_end(&self, i: usize) -> usize {
        self.starts[i] + self.lines[i].len()
    }

    /// Byte offset just past line `i` including its newline (if any).
    pub fn line_end(&self, i: usize) -> usize {
        self.ends[i]
    }

    /// Render the whole buffer as hashlines.
    pub fn render(&self) -> String {
        self.render_range(0, self.lines.len().saturating_sub(1))
    }

    /// Render lines `first..=last` as hashlines.
    pub fn render_range(&self, first: usize, last: usize) -> String {
        let mut out = String::new();
        for i in first..=last.min(self.lines.len().saturating_sub(1)) {
            let _ = writeln!(out, "{}|{}", self.hashes[i], self.lines[i]);
        }
        out
    }
}

</storycode:copy>
<a id="f-55ecf32bef2e-Anchor"></a>
### `Anchor`

<storycode:copy id="f-55ecf32bef2e-Anchor">
/// A parsed edit anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anchor {
    /// Replace the contiguous run of lines starting at a line hashing to
    /// `first` and ending at the following (or same) line hashing to `last`.
    Run { first: String, last: String },
    /// Insert new lines below the line hashing to this value.
    After(String),
    /// Insert new lines at the very top of the buffer.
    AtStart,
}

</storycode:copy>
<a id="f-55ecf32bef2e-ResolvedAnchor"></a>
### `ResolvedAnchor`

A resolved anchor: inclusive line indices, or an insertion point.

<storycode:copy id="f-55ecf32bef2e-ResolvedAnchor">
/// A resolved anchor: inclusive line indices, or an insertion point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedAnchor {
    Run {
        first: usize,
        last: usize,
    },
    /// Insert below this line index (`None` = at the very top).
    After(Option<usize>),
}

</storycode:copy>
<a id="f-55ecf32bef2e-AnchorError"></a>
### `AnchorError`

Anchor resolution failure, phrased for the model.

<storycode:copy id="f-55ecf32bef2e-AnchorError">
/// Anchor resolution failure, phrased for the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorError(pub String);

</storycode:copy>
<a id="f-55ecf32bef2e-impl-std-fmt-Display-for-AnchorError"></a>
### `impl-std::fmt::Display for AnchorError`

<storycode:copy id="f-55ecf32bef2e-impl-std-fmt-Display-for-AnchorError">
impl std::fmt::Display for AnchorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

</storycode:copy>
<a id="f-55ecf32bef2e-valid_hash"></a>
### `valid_hash`

<storycode:copy id="f-55ecf32bef2e-valid_hash">
fn valid_hash(s: &str) -> bool {
    matches!(s.len(), 4 | 64) && s.chars().all(|c| c.is_ascii_hexdigit())
}

</storycode:copy>
<a id="f-55ecf32bef2e-parse_anchor"></a>
### `parse_anchor`

Parse the `run` / `after` argument pair of an edit tool call.
Accepted forms: `run="aaaa..bbbb"`, `run="aaaa"` (single line),
`after="aaaa"`, `after="^"` (top of buffer).

<storycode:copy id="f-55ecf32bef2e-parse_anchor">
/// Parse the `run` / `after` argument pair of an edit tool call.
///
/// Accepted forms: `run="aaaa..bbbb"`, `run="aaaa"` (single line),
/// `after="aaaa"`, `after="^"` (top of buffer).
pub fn parse_anchor(run: Option<&str>, after: Option<&str>) -> Result<Anchor, AnchorError> {
    match (run, after) {
        (Some(_), Some(_)) => Err(AnchorError(
            "give either run (replace lines) or after (insert below a line), not both".into(),
        )),
        (None, None) => Err(AnchorError(
            "missing anchor: give run=\"first..last\" (or one hash) to replace lines, or \
             after=\"hash\" (or \"^\") to insert"
                .into(),
        )),
        (Some(run), None) => {
            let (first, last) = match run.split_once("..") {
                Some((f, l)) => (f.trim(), l.trim()),
                None => (run.trim(), run.trim()),
            };
            if !valid_hash(first) || !valid_hash(last) {
                return Err(AnchorError(format!(
                    "run anchor '{run}' is not a SHA-256 hash or first..last hash pair"
                )));
            }
            Ok(Anchor::Run {
                first: first.to_ascii_lowercase(),
                last: last.to_ascii_lowercase(),
            })
        }
        (None, Some(after)) => {
            let after = after.trim();
            if after == "^" {
                return Ok(Anchor::AtStart);
            }
            if !valid_hash(after) {
                return Err(AnchorError(format!(
                    "after anchor '{after}' is not a SHA-256 line hash (or \"^\")"
                )));
            }
            Ok(Anchor::After(after.to_ascii_lowercase()))
        }
    }
}

</storycode:copy>
<a id="f-55ecf32bef2e-resolve_anchor"></a>
### `resolve_anchor`

Resolve an anchor against a [`LineIndex`].
Candidates for a run are, for each line hashing to `first`, the shortest
contiguous run ending at a following line hashing to `last`. More than
one candidate without an `occurrence` index is a structured error listing
every candidate so the model can retry with `occurrence="N"`.

<storycode:copy id="f-55ecf32bef2e-resolve_anchor">
/// Resolve an anchor against a [`LineIndex`].
///
/// Candidates for a run are, for each line hashing to `first`, the shortest
/// contiguous run ending at a following line hashing to `last`. More than
/// one candidate without an `occurrence` index is a structured error listing
/// every candidate so the model can retry with `occurrence="N"`.
pub fn resolve_anchor(
    index: &LineIndex,
    anchor: &Anchor,
    occurrence: Option<usize>,
) -> Result<ResolvedAnchor, AnchorError> {
    let candidates: Vec<ResolvedAnchor> = match anchor {
        Anchor::AtStart => return Ok(ResolvedAnchor::After(None)),
        Anchor::After(hash) => index
            .hashes
            .iter()
            .enumerate()
            .filter(|(_, h)| *h == hash)
            .map(|(i, _)| ResolvedAnchor::After(Some(i)))
            .collect(),
        Anchor::Run { first, last } => {
            let mut found = Vec::new();
            for (i, h) in index.hashes.iter().enumerate() {
                if h == first
                    && let Some(j) = (i..index.hashes.len()).find(|&j| index.hashes[j] == *last)
                {
                    found.push(ResolvedAnchor::Run { first: i, last: j });
                }
            }
            found
        }
    };

    match candidates.len() {
        0 => Err(AnchorError(format!(
            "anchor {} matches no lines in the current content — the text may have changed; \
             re-read the surface and use fresh hashes",
            describe_anchor(anchor)
        ))),
        1 => Ok(candidates[0]),
        n => {
            if let Some(occ) = occurrence {
                candidates.get(occ.wrapping_sub(1)).copied().ok_or_else(|| {
                    AnchorError(format!(
                        "occurrence {occ} is out of range: anchor {} has {n} candidates",
                        describe_anchor(anchor)
                    ))
                })
            } else {
                let mut msg = format!(
                    "anchor {} is ambiguous — {n} candidate runs; retry with occurrence=\"N\":\n",
                    describe_anchor(anchor)
                );
                for (k, c) in candidates.iter().enumerate() {
                    let (a, b) = match c {
                        ResolvedAnchor::Run { first, last } => (*first, *last),
                        ResolvedAnchor::After(Some(i)) => (*i, *i),
                        ResolvedAnchor::After(None) => (0, 0),
                    };
                    let _ = writeln!(
                        msg,
                        "  occurrence {}: lines {}..{} ({})",
                        k + 1,
                        a + 1,
                        b + 1,
                        index.lines[a].trim()
                    );
                }
                Err(AnchorError(msg.trim_end().to_string()))
            }
        }
    }
}

</storycode:copy>
<a id="f-55ecf32bef2e-describe_anchor"></a>
### `describe_anchor`

<storycode:copy id="f-55ecf32bef2e-describe_anchor">
fn describe_anchor(anchor: &Anchor) -> String {
    match anchor {
        Anchor::Run { first, last } if first == last => format!("run=\"{first}\""),
        Anchor::Run { first, last } => format!("run=\"{first}..{last}\""),
        Anchor::After(h) => format!("after=\"{h}\""),
        Anchor::AtStart => "after=\"^\"".to_string(),
    }
}

</storycode:copy>
<a id="f-55ecf32bef2e-tests"></a>
### `tests`

<storycode:copy id="f-55ecf32bef2e-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deleting_a_crlf_line_consumes_both_newline_bytes() {
        let source = "first\r\nsecond\r\n";
        let index = LineIndex::new(source);
        let mut next = source.to_string();
        next.replace_range(index.starts[0]..index.line_end(0), "");
        assert_eq!(next, "second\r\n");
    }

    #[test]
    fn hashline_render_roundtrip() {
        let text = "alpha\nbeta\n\nalpha\n";
        let index = LineIndex::new(text);
        let rendered = index.render();
        // Every rendered line is hash|content; stripping prefixes restores
        // the original text.
        let mut restored = String::new();
        for line in rendered.lines() {
            let (hash, content) = line.split_once('|').unwrap();
            assert_eq!(hash, line_hash(content));
            restored.push_str(content);
            restored.push('\n');
        }
        assert_eq!(restored, text);
        // Duplicate-content lines share a hash.
        assert_eq!(index.hashes[0], index.hashes[3]);
        assert_ne!(index.hashes[0], index.hashes[1]);
    }

    #[test]
    fn line_offsets_track_bytes() {
        let index = LineIndex::new("ab\ncafé\nx");
        assert_eq!(index.starts, vec![0, 3, 9]);
        assert_eq!(index.content_end(1), 8);
        assert_eq!(index.line_end(1), 9);
        assert_eq!(index.line_end(2), 10); // no trailing newline
        assert!(!index.trailing_newline);
    }

    #[test]
    fn resolves_unique_run() {
        let index = LineIndex::new("one\ntwo\nthree\n");
        let anchor = parse_anchor(
            Some(&format!("{}..{}", line_hash("one"), line_hash("two"))),
            None,
        )
        .unwrap();
        assert_eq!(
            resolve_anchor(&index, &anchor, None).unwrap(),
            ResolvedAnchor::Run { first: 0, last: 1 }
        );
    }

    #[test]
    fn single_hash_run_is_one_line() {
        let index = LineIndex::new("one\ntwo\n");
        let anchor = parse_anchor(Some(&line_hash("two")), None).unwrap();
        assert_eq!(
            resolve_anchor(&index, &anchor, None).unwrap(),
            ResolvedAnchor::Run { first: 1, last: 1 }
        );
    }

    #[test]
    fn ambiguous_run_lists_candidates_and_occurrence_disambiguates() {
        let index = LineIndex::new("dup\nmid\ndup\n");
        let anchor = parse_anchor(Some(&line_hash("dup")), None).unwrap();
        let err = resolve_anchor(&index, &anchor, None).unwrap_err();
        assert!(err.0.contains("ambiguous"), "{}", err.0);
        assert!(err.0.contains("occurrence 1"), "{}", err.0);
        assert!(err.0.contains("occurrence 2"), "{}", err.0);
        assert_eq!(
            resolve_anchor(&index, &anchor, Some(2)).unwrap(),
            ResolvedAnchor::Run { first: 2, last: 2 }
        );
        let out_of_range = resolve_anchor(&index, &anchor, Some(9)).unwrap_err();
        assert!(
            out_of_range.0.contains("out of range"),
            "{}",
            out_of_range.0
        );
    }

    #[test]
    fn stale_hash_is_a_structured_miss() {
        let index = LineIndex::new("fresh content\n");
        let anchor = parse_anchor(Some(&line_hash("old content")), None).unwrap();
        let err = resolve_anchor(&index, &anchor, None).unwrap_err();
        assert!(err.0.contains("matches no lines"), "{}", err.0);
    }

    #[test]
    fn insertion_anchors() {
        let index = LineIndex::new("a\nb\n");
        assert_eq!(
            resolve_anchor(&index, &parse_anchor(None, Some("^")).unwrap(), None).unwrap(),
            ResolvedAnchor::After(None)
        );
        assert_eq!(
            resolve_anchor(
                &index,
                &parse_anchor(None, Some(&line_hash("b"))).unwrap(),
                None
            )
            .unwrap(),
            ResolvedAnchor::After(Some(1))
        );
    }

    #[test]
    fn anchor_parse_errors() {
        assert!(parse_anchor(None, None).is_err());
        assert!(parse_anchor(Some("zzzz"), None).is_err());
        assert!(parse_anchor(Some("abcd"), Some("abcd")).is_err());
        assert!(parse_anchor(None, Some("nope")).is_err());
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/hashline.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/hashline.rs"><storycode:paste select="#f-55ecf32bef2e-line_hash"/><storycode:paste select="#f-55ecf32bef2e-line_fingerprint"/><storycode:paste select="#f-55ecf32bef2e-LineIndex"/><storycode:paste select="#f-55ecf32bef2e-impl-LineIndex"/><storycode:paste select="#f-55ecf32bef2e-Anchor"/><storycode:paste select="#f-55ecf32bef2e-ResolvedAnchor"/><storycode:paste select="#f-55ecf32bef2e-AnchorError"/><storycode:paste select="#f-55ecf32bef2e-impl-std-fmt-Display-for-AnchorError"/><storycode:paste select="#f-55ecf32bef2e-valid_hash"/><storycode:paste select="#f-55ecf32bef2e-parse_anchor"/><storycode:paste select="#f-55ecf32bef2e-resolve_anchor"/><storycode:paste select="#f-55ecf32bef2e-describe_anchor"/><storycode:paste select="#f-55ecf32bef2e-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-agent/src/tools/hashline.rs](contract.md#output-55ecf32bef2e)
- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)

### Tests and fixtures

- [crates/hickory-agent/tests/tools_edit_session.rs](../../../mechanisms/crates/hickory-agent/tests/tools_edit_session.rs.md#output-35f0054c98d4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
