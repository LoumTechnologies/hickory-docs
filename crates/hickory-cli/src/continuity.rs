//! Continuity: the correspondence journal, and the switch the whole of it
//! rides on.
//!
//! See `docs/specs/freeform/provenance-across-versions.md`. Continuity is the
//! **fourth provenance family** and it answers a different question from the
//! other three: they say *why is this here* about the present, and it says
//! **what was this before**.
//!
//! Addressing is solved — `(commit, path, line)` points at published bytes
//! exactly. **Correlating** is the gap: `abc123:main.rs:42` and
//! `def456:main.rs:87` are two addresses, and neither says they are one span
//! at two times. A **recorded correspondence** is what says it, written at a
//! moment when both sides were in hand.
//!
//! Three things about this module are decisions rather than mechanics:
//!
//! 1. **It is off by default, and the whole of it rides one switch.** No
//!    continuity, no journal, no check — see
//!    [`hickory_workspace::WorkspaceStore::continuity`]. Making the overlay
//!    opt-in while the bookkeeping stayed mandatory would tax every commit to
//!    feed a feature most people never turn on.
//! 2. **The journal is a RECORD, not a cache.** It exists precisely because
//!    what it holds cannot be recomputed once both sides are gone. Records
//!    belong in git if you want them, so whether CI can check anything here is
//!    a line in `.gitignore` rather than a property of the design. `hick init`
//!    writes that line, so the default is private and a project deletes it to
//!    opt in.
//! 3. **Every link is a recorded fact or an explicit guess, and says which**
//!    — along with the precision it was captured at. There are deliberately no
//!    element ids: an id is bytes anyone can edit or paste onto an unrelated
//!    element, so a link that looked derived would be forgeable.
//!
//! ## Precision is a property of the recording site
//!
//! Each correspondence carries the precision it was recorded at, and a query
//! that composes several reports **the weakest link in the chain**. That is
//! why there is no anchor scheme to layer: each hop already knows how well it
//! was captured.
//!
//! A correspondence can be coarse **because nothing was watching**, or
//! **because nothing exact exists to record** — two runs of a scaffolder share
//! no history, so there is no byte-precise thread even with the tool watching
//! the whole time. The first is a gap in coverage; the second is a property of
//! the artifact. [`Precision::Diff`] is the second, and it is not a lesser
//! [`Precision::Byte`].
//!
//! ## Keys above the publication floor
//!
//! `(commit, path, span)` is stable below the floor and **not** above it,
//! because re-emitting the frontier rebuilds commits rather than patching them
//! (`expression-and-log.md`) — and above the floor is where the active work is.
//! The choice made here (2026-08-23) is: **entries above the floor are
//! provisional**, and are rewritten alongside the commits they name when
//! re-emission replaces them. That concedes that part of the journal is
//! derived after all, which is the honest cost of recording at the moment both
//! sides are in hand — which is, by definition, in the working tree.
//!
//! A correspondence recorded in the working tree has `commit: None` on both
//! sides and `head` naming the commit the work sat on.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

/// The journal directory, relative to the project root.
pub const JOURNAL_DIR: &str = ".hick-journal";

/// The one file inside it. Append-only, one JSON object per line: a record
/// that is only ever added to is the one shape that merges without a merge
/// algorithm, and the one shape a crash cannot half-write into corruption.
pub const JOURNAL_FILE: &str = "correspondences.jsonl";

/// How well a correspondence was captured, by the site that captured it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Precision {
    /// Byte-exact, joined through outputs proven identical. Refactor mode.
    Byte,
    /// Line-precise, reconstructed by matching. The pre-commit repair.
    Line,
    /// Diff-precise, and no better exists: two runs of a foreign tool share
    /// no history, so there is no byte-precise thread to record even with the
    /// tool watching the whole time. A re-scaffold.
    Diff,
    /// Guessed by an agent and confirmed by a person. A declared claim, drawn
    /// in the declared family's language because that is what it is.
    Asserted,
}

impl Precision {
    /// The weaker of two — what a composed chain reports.
    pub fn weaker(self, other: Self) -> Self {
        let rank = |p: Precision| match p {
            Precision::Byte => 0,
            Precision::Line => 1,
            Precision::Diff => 2,
            Precision::Asserted => 3,
        };
        if rank(self) >= rank(other) {
            self
        } else {
            other
        }
    }

    /// Whether this is a recorded fact rather than an explicit guess.
    pub fn is_recorded(self) -> bool {
        !matches!(self, Precision::Asserted)
    }
}

/// Where a correspondence was recorded. Named, because "how coarse is this"
/// is answered by *where it came from* and not by an anchor scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Site {
    /// Refactor mode: the baseline document and its outputs, proven identical,
    /// so the outputs are a join key.
    Refactor,
    /// A re-ingest: the recorded bytes, the fresh run, and the document's own
    /// edits — three sides at one moment.
    Reingest,
    /// A merge: base, ours and theirs. The richest site there is.
    Merge,
    /// The pre-commit repair: HEAD's document and the working tree's.
    Repair,
}

/// One end of a correspondence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    /// `None` means the working tree at the moment of recording — which is
    /// above the floor by definition, and therefore provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Repository-relative path of the document.
    pub path: String,
    /// Byte span in that document.
    pub span: (usize, usize),
}

/// One recorded correspondence: this span and that one are the same thing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Correspondence {
    pub from: Endpoint,
    pub to: Endpoint,
    pub precision: Precision,
    pub site: Site,
    /// True when either side is at or above the publication floor, so the
    /// address it is keyed on can still be replaced by a re-emission.
    pub provisional: bool,
    /// The commit the work sat on when this was recorded. Context for an
    /// entry whose endpoints are the working tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// The date, from the caller. There is no clock in here, so a test and a
    /// replay produce the same bytes.
    pub recorded_at: String,
}

/// The journal for one project.
pub struct Journal {
    path: PathBuf,
}

impl Journal {
    /// The journal for the project rooted at `root`. Nothing is created until
    /// something is written.
    pub fn at(root: &Path) -> Self {
        Self {
            path: root.join(JOURNAL_DIR).join(JOURNAL_FILE),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append entries. Nothing is written for an empty batch — a refactor
    /// that moved nothing must not leave a file behind announcing that it
    /// happened.
    pub fn append(&self, entries: &[Correspondence]) -> Result<usize> {
        if entries.is_empty() {
            return Ok(0);
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let mut body = String::new();
        for entry in entries {
            body.push_str(&serde_json::to_string(entry).context("encoding a correspondence")?);
            body.push('\n');
        }
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("opening {}", self.path.display()))?;
        file.write_all(body.as_bytes())
            .with_context(|| format!("writing {}", self.path.display()))?;
        Ok(entries.len())
    }

    /// Every entry, in the order they were recorded. A line that does not
    /// parse is skipped rather than fatal: a journal is appended to by several
    /// processes, and one bad line must not cost the reader the rest.
    pub fn read(&self) -> Vec<Correspondence> {
        let Ok(raw) = std::fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        raw.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }
}

/// Whether continuity is on for the project at `root`, for this user.
///
/// A store that cannot be opened reads as OFF, for the same reason a missing
/// file does: this decides whether to start writing records into somebody's
/// repository, and the answer to "I could not tell" is "no".
pub fn enabled(root: &Path) -> bool {
    hickory_workspace::WorkspaceStore::for_project(root)
        .map(|store| store.continuity())
        .unwrap_or(false)
}

/// Join two provenance maps over the SAME output bytes into correspondences.
///
/// This is the mechanism, and the case that motivated the whole design hands
/// it over for free: **refactor mode proves the outputs are byte-identical,
/// which makes the outputs a join key.** An output byte maps through lineage
/// to a span in the baseline document *and* to a span in the restructured one.
/// Compose, and the old-span → new-span correspondence is exact, derived at
/// that moment, with no identity scheme involved.
///
/// Only ranges where BOTH sides are byte-precise and editable produce an
/// entry. A synthetic range on either side means there is nothing to
/// correspond, and inventing one would be the forgeable middle this design
/// deleted.
///
/// Identical spans on both sides are dropped: "this span is itself" is not a
/// fact worth a line in a record.
pub fn join_through_outputs(
    before: &[hickory_lineage::Provenance],
    after: &[hickory_lineage::Provenance],
) -> Vec<(Endpoint, Endpoint)> {
    let mut out = Vec::new();
    for b in before {
        let Some((b_doc, b_start, b_end)) = b.origin.source() else {
            continue;
        };
        for a in after {
            // The join is on the OUTPUT range: these bytes, at both times.
            let start = b.start.max(a.start);
            let end = b.end.min(a.end);
            if start >= end {
                continue;
            }
            let Some((a_doc, a_start, a_end)) = a.origin.source() else {
                continue;
            };
            // Narrow both sides to the overlapping output range, so a span
            // that was split or merged by the restructure corresponds only
            // where it actually corresponds.
            let b_from = b_start + (start - b.start);
            let b_to = b_from + (end - start);
            let a_from = a_start + (start - a.start);
            let a_to = a_from + (end - start);
            if b_to > b_end || a_to > a_end {
                continue;
            }
            if b_doc == a_doc && (b_from, b_to) == (a_from, a_to) {
                continue;
            }
            out.push((
                Endpoint {
                    commit: None,
                    path: b_doc.to_string(),
                    span: (b_from, b_to),
                },
                Endpoint {
                    commit: None,
                    path: a_doc.to_string(),
                    span: (a_from, a_to),
                },
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_lineage::{Origin, Provenance};

    fn lit(start: usize, end: usize, doc: &str, s: usize, e: usize) -> Provenance {
        Provenance {
            start,
            end,
            origin: Origin::Literal {
                doc_path: doc.to_string(),
                span: (s, e),
            },
        }
    }

    #[test]
    fn a_composed_chain_reports_its_weakest_link() {
        assert_eq!(Precision::Byte.weaker(Precision::Byte), Precision::Byte);
        assert_eq!(Precision::Byte.weaker(Precision::Line), Precision::Line);
        assert_eq!(Precision::Line.weaker(Precision::Diff), Precision::Diff);
        assert_eq!(
            Precision::Diff.weaker(Precision::Asserted),
            Precision::Asserted
        );
    }

    #[test]
    fn a_guess_is_not_a_recorded_fact() {
        assert!(Precision::Byte.is_recorded());
        assert!(Precision::Diff.is_recorded());
        assert!(!Precision::Asserted.is_recorded());
    }

    #[test]
    fn a_restructure_that_moved_a_span_is_joined_through_the_output() {
        // The same eight output bytes come from byte 100 of the baseline and
        // byte 500 of the restructured document. That is the correspondence,
        // and nothing but the outputs was needed to derive it.
        let before = vec![lit(0, 8, "a.hick", 100, 108)];
        let after = vec![lit(0, 8, "a.hick", 500, 508)];
        let joined = join_through_outputs(&before, &after);
        assert_eq!(joined.len(), 1);
        assert_eq!(joined[0].0.span, (100, 108));
        assert_eq!(joined[0].1.span, (500, 508));
    }

    #[test]
    fn a_span_that_did_not_move_records_nothing() {
        // "This span is itself" is not a fact worth a line in a record.
        let before = vec![lit(0, 8, "a.hick", 100, 108)];
        let after = vec![lit(0, 8, "a.hick", 100, 108)];
        assert!(join_through_outputs(&before, &after).is_empty());
    }

    #[test]
    fn a_split_span_corresponds_only_where_it_corresponds() {
        // One baseline block became two blocks. Each half corresponds to its
        // own half — not the whole to the whole, which would be a claim the
        // outputs do not support.
        let before = vec![lit(0, 8, "a.hick", 100, 108)];
        let after = vec![lit(0, 4, "a.hick", 500, 504), lit(4, 8, "a.hick", 700, 704)];
        let mut joined = join_through_outputs(&before, &after);
        joined.sort_by_key(|(f, _)| f.span);
        assert_eq!(joined.len(), 2);
        assert_eq!(
            (joined[0].0.span, joined[0].1.span),
            ((100, 104), (500, 504))
        );
        assert_eq!(
            (joined[1].0.span, joined[1].1.span),
            ((104, 108), (700, 704))
        );
    }

    #[test]
    fn a_synthetic_side_records_nothing_rather_than_a_forgeable_guess() {
        let before = vec![lit(0, 8, "a.hick", 100, 108)];
        let after = vec![Provenance {
            start: 0,
            end: 8,
            origin: Origin::Synthetic,
        }];
        assert!(join_through_outputs(&before, &after).is_empty());
    }

    #[test]
    fn an_empty_batch_leaves_no_file_behind() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::at(dir.path());
        assert_eq!(journal.append(&[]).unwrap(), 0);
        assert!(!journal.path().exists());
        assert!(journal.read().is_empty());
    }

    #[test]
    fn entries_round_trip_and_append() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::at(dir.path());
        let one = Correspondence {
            from: Endpoint {
                commit: None,
                path: "a.hick".into(),
                span: (1, 2),
            },
            to: Endpoint {
                commit: Some("abc".into()),
                path: "a.hick".into(),
                span: (3, 4),
            },
            precision: Precision::Byte,
            site: Site::Refactor,
            provisional: true,
            head: Some("def".into()),
            recorded_at: "2026-08-23".into(),
        };
        journal.append(std::slice::from_ref(&one)).unwrap();
        journal.append(std::slice::from_ref(&one)).unwrap();
        let read = journal.read();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0], one);
    }

    #[test]
    fn one_bad_line_does_not_cost_the_reader_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::at(dir.path());
        let good = Correspondence {
            from: Endpoint {
                commit: None,
                path: "a.hick".into(),
                span: (1, 2),
            },
            to: Endpoint {
                commit: None,
                path: "a.hick".into(),
                span: (3, 4),
            },
            precision: Precision::Diff,
            site: Site::Reingest,
            provisional: true,
            head: None,
            recorded_at: "2026-08-23".into(),
        };
        journal.append(std::slice::from_ref(&good)).unwrap();
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(journal.path())
            .unwrap();
        f.write_all(b"{not json\n").unwrap();
        journal.append(std::slice::from_ref(&good)).unwrap();
        assert_eq!(journal.read().len(), 2);
    }
}

// ---------------------------------------------------------------------------
// The repair
// ---------------------------------------------------------------------------

/// Run the pre-commit repair over every staged `.hick` document.
///
/// Returns the number of correspondences recorded. **Never fails a commit.**
/// The check IS the repair: there is no enforcement mode, because a rule with
/// no escape hatch gets the hook disabled entirely, taking the `hick test`
/// gate down with it.
///
/// Off unless continuity is on for the project — no continuity, no journal,
/// no check.
pub fn repair_staged(root: &Path, today: &str) -> Result<usize> {
    if !enabled(root) {
        return Ok(0);
    }
    let head = git_stdout(root, &["rev-parse", "HEAD"]);
    let staged =
        git_stdout(root, &["diff", "--cached", "--name-only", "--", "*.hick"]).unwrap_or_default();

    let mut entries = Vec::new();
    for rel in staged.lines().map(str::trim).filter(|l| !l.is_empty()) {
        // HEAD's document, and the one about to be committed. A file with no
        // version at HEAD is new: there is no "before", so nothing moved.
        let Some(before) = git_stdout(root, &["show", &format!("HEAD:{rel}")]) else {
            continue;
        };
        let Some(after) = git_stdout(root, &["show", &format!(":{rel}")]) else {
            continue;
        };
        entries.extend(repair(&before, &after, rel, head.as_deref(), today));
    }
    Journal::at(root).append(&entries)
}

fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Reconstruct the correspondence for a document edited outside every
/// recording site, and record it — the pre-commit **repair**.
///
/// At commit time both sides are in hand: HEAD's document and the working
/// tree's. That is the same position `refactor/end` is in, so the hook does
/// not REJECT an unwatched edit — it reconstructs what happened and writes it
/// down:
///
/// > **An unwatched edit leaves its correspondence behind before it is
/// > committed.**
///
/// Two rungs, honestly labelled: recorded at edit time
/// ([`Precision::Byte`]) or reconstructed at commit time
/// ([`Precision::Line`]) — but recorded **once**, so it does not decay across
/// later hops the way matching done fresh at query time would.
///
/// **That the check IS the repair is what makes this safe.** There is no
/// enforcement mode to reason about, and a rule with no escape hatch gets the
/// hook disabled entirely, taking the `hick test` gate down with it. The
/// message says what it recorded, never what it refused.
///
/// Line-precise by construction: this matches lines, because that is all two
/// snapshots of a text file support. A moved run of lines yields one entry per
/// contiguous run, not one per line, so a block that moved reads as a block.
pub fn repair(
    before: &str,
    after: &str,
    path: &str,
    head: Option<&str>,
    today: &str,
) -> Vec<Correspondence> {
    use similar::{Algorithm, DiffOp, capture_diff_slices};

    // Byte offsets of each line's start, so a line index becomes a span.
    fn line_spans(text: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut start = 0;
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                out.push((start, i + ch.len_utf8()));
                start = i + ch.len_utf8();
            }
        }
        if start < text.len() {
            out.push((start, text.len()));
        }
        out
    }

    let before_lines: Vec<&str> = before.split_inclusive('\n').collect();
    let after_lines: Vec<&str> = after.split_inclusive('\n').collect();
    let before_spans = line_spans(before);
    let after_spans = line_spans(after);

    let mut out = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &before_lines, &after_lines) {
        let DiffOp::Equal {
            old_index,
            new_index,
            len,
        } = op
        else {
            continue;
        };
        if len == 0 {
            continue;
        }
        // A run that did not move is not a fact worth recording: the address
        // already answers it.
        if old_index == new_index {
            continue;
        }
        let Some(from_start) = before_spans.get(old_index).map(|s| s.0) else {
            continue;
        };
        let Some(from_end) = before_spans.get(old_index + len - 1).map(|s| s.1) else {
            continue;
        };
        let Some(to_start) = after_spans.get(new_index).map(|s| s.0) else {
            continue;
        };
        let Some(to_end) = after_spans.get(new_index + len - 1).map(|s| s.1) else {
            continue;
        };
        out.push(Correspondence {
            from: Endpoint {
                commit: head.map(str::to_string),
                path: path.to_string(),
                span: (from_start, from_end),
            },
            to: Endpoint {
                commit: None,
                path: path.to_string(),
                span: (to_start, to_end),
            },
            precision: Precision::Line,
            site: Site::Repair,
            provisional: true,
            head: head.map(str::to_string),
            recorded_at: today.to_string(),
        });
    }
    out
}

#[cfg(test)]
mod repair_tests {
    use super::*;

    #[test]
    fn a_block_that_moved_is_recorded_once_as_a_block() {
        // One entry per contiguous run, not one per line: a block that moved
        // should read as a block.
        let before = "a\nb\nc\nHEAD\n";
        let after = "HEAD\na\nb\nc\n";
        let entries = repair(before, after, "doc.hick", Some("abc"), "2026-08-23");
        assert_eq!(entries.len(), 1, "{entries:?}");
        let e = &entries[0];
        assert_eq!(e.precision, Precision::Line);
        assert_eq!(e.site, Site::Repair);
        // `a\nb\nc\n` was at 0..6 and is now at 5..11.
        assert_eq!(e.from.span, (0, 6));
        assert_eq!(e.to.span, (5, 11));
        // The old side names the commit; the new side is the working tree.
        assert_eq!(e.from.commit.as_deref(), Some("abc"));
        assert!(e.to.commit.is_none());
    }

    #[test]
    fn a_document_that_did_not_move_records_nothing() {
        let text = "a\nb\nc\n";
        assert!(repair(text, text, "doc.hick", None, "2026-08-23").is_empty());
    }

    #[test]
    fn an_edit_in_place_records_nothing_because_the_address_answers_it() {
        // Changing a line's CONTENT without moving anything leaves every
        // surviving line where it was, and `(commit, path, line)` already
        // says that.
        let entries = repair("a\nb\nc\n", "a\nB\nc\n", "doc.hick", None, "2026-08-23");
        assert!(entries.is_empty(), "{entries:?}");
    }

    #[test]
    fn an_insertion_moves_everything_after_it_and_records_that() {
        let entries = repair("a\nb\n", "a\nNEW\nb\n", "doc.hick", None, "2026-08-23");
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0].from.span, (2, 4));
        assert_eq!(entries[0].to.span, (6, 8));
    }
}
