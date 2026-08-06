//! Hashline rendering and anchor resolution.
//!
//! Both editable surfaces (document source and woven outputs) are shown to
//! the model with every line prefixed `hhhh|`, where `hhhh` is a 4-hex
//! **content** hash of the line text. Content hashes make staleness
//! impossible to act on: an anchor only resolves against the text that is
//! actually there right now. Duplicate-content lines share a hash by design;
//! edits disambiguate by contiguous runs and, when the run anchor pair is
//! still ambiguous, an explicit 1-based occurrence index.

use std::fmt::Write as _;

/// 4-hex content hash of one line of text (FNV-1a 64, xor-folded to 16
/// bits). Stable across runs and processes — it is pure content.
pub fn line_hash(line: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in line.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    let folded = (h ^ (h >> 16) ^ (h >> 32) ^ (h >> 48)) & 0xffff;
    format!("{folded:04x}")
}

/// A text buffer split into lines, with byte offsets and content hashes.
pub struct LineIndex {
    /// Line texts, without their trailing newlines.
    pub lines: Vec<String>,
    /// Byte offset where each line starts.
    pub starts: Vec<usize>,
    /// 4-hex content hash per line.
    pub hashes: Vec<String>,
    /// Whether the buffer ends with a newline.
    pub trailing_newline: bool,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut lines = Vec::new();
        let mut starts = Vec::new();
        let mut pos = 0;
        for line in text.split_inclusive('\n') {
            starts.push(pos);
            pos += line.len();
            let content = line.strip_suffix('\n').unwrap_or(line);
            let content = content.strip_suffix('\r').unwrap_or(content);
            lines.push(content.to_string());
        }
        if text.is_empty() {
            starts.push(0);
            lines.push(String::new());
        }
        let hashes = lines.iter().map(|l| line_hash(l)).collect();
        Self {
            lines,
            starts,
            hashes,
            trailing_newline: text.ends_with('\n'),
        }
    }

    /// Byte offset just past the content of line `i` (excluding its newline).
    pub fn content_end(&self, i: usize) -> usize {
        self.starts[i] + self.lines[i].len()
    }

    /// Byte offset just past line `i` including its newline (if any).
    pub fn line_end(&self, i: usize) -> usize {
        let end = self.content_end(i);
        if i + 1 < self.starts.len() || self.trailing_newline {
            end + 1
        } else {
            end
        }
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

/// Anchor resolution failure, phrased for the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorError(pub String);

impl std::fmt::Display for AnchorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn valid_hash(s: &str) -> bool {
    s.len() == 4 && s.chars().all(|c| c.is_ascii_hexdigit())
}

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
                    "run anchor '{run}' is not a 4-hex hash or first..last hash pair"
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
                    "after anchor '{after}' is not a 4-hex line hash (or \"^\")"
                )));
            }
            Ok(Anchor::After(after.to_ascii_lowercase()))
        }
    }
}

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

fn describe_anchor(anchor: &Anchor) -> String {
    match anchor {
        Anchor::Run { first, last } if first == last => format!("run=\"{first}\""),
        Anchor::Run { first, last } => format!("run=\"{first}..{last}\""),
        Anchor::After(h) => format!("after=\"{h}\""),
        Anchor::AtStart => "after=\"^\"".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
