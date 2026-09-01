//! `hick ingest --from` — a scaffolder's output becomes bytes this document
//! owns.
//!
//! See `docs/specs/freeform/owning-what-a-scaffolder-wrote.md`. `dotnet new
//! webapi` writes forty files nobody typed and the interesting work is
//! changing four lines across three of them. Pasting the scaffold in makes
//! the document a silent snapshot; leaving it out makes the scaffold a step
//! in a README. Ingest is the third answer: the run's bytes enter the
//! document as ordinary `hick:file` blocks under an `<hick:ingested>` element
//! that records the run, so **the base survives a clone** — which is exactly
//! what a `from=` pointing into the gitignored transcript cache could not do.
//!
//! This is the same verb as the inbox's `hick ingest` through a different
//! door, and it keeps that verb's discipline (`crate::ingest`):
//!
//! 1. **It never deletes the user's bytes.** Nothing outside the document is
//!    touched at all.
//! 2. **It never calls a model.** Reading a volume into document elements is
//!    offline, first-party, deterministic.
//! 3. **Identity is a content hash**, recorded so the same run ingested twice
//!    is one result rather than two.
//!
//! What changes with the door is the unit of identity: the inbox's rule is
//! one note per source file, and here it is **one fingerprint per run, N
//! files under it** — a scaffold is a single event that happens to write
//! forty things, and forty unrelated hashes would lose that.
//!
//! Two refusals are load-bearing rather than incidental:
//!
//! - **Anything the project would gitignore is filtered out**, using the
//!   repository's own `.gitignore` through `git check-ignore`. A scaffolder
//!   writes build output beside source, and ingesting `obj/` would put
//!   derived bytes into the document that owns the source. Filtered files are
//!   counted in `skipped=` and named on the way past.
//! - **Non-UTF-8 output cannot be ingested and is refused by name.** A
//!   `hick:file` body is raw bytes under the no-escaping invariant, so a
//!   binary would be silently mangled into the document. A side-car for
//!   binaries is a later question and is deliberately not designed here.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, Result, bail};
use hick_lang::{HickDocument, HickNode, HickTag};
use sha2::{Digest, Sha256};

use crate::{ExecutorChoice, RunMode, run_doc};

/// What one ingest produced.
pub struct IngestExecOutcome {
    /// The document that now owns the bytes.
    pub doc_path: PathBuf,
    /// `from=` as written into the element.
    pub from: String,
    /// The recorded run fingerprint (`sha256=`).
    pub fingerprint: String,
    /// Paths ingested, document-relative, in the order they were written.
    pub ingested: Vec<String>,
    /// Paths the repository's `.gitignore` filtered out, with the volume
    /// path they had.
    pub skipped: Vec<String>,
    /// What a re-ingest's three-way merge did. Empty for a first ingest.
    pub merge: MergeReport,
    /// The commit the base was recovered from, for a re-ingest.
    pub base_commit: Option<String>,
    /// Correspondences written to the journal — zero unless continuity is on.
    pub recorded: usize,
}

// ---------------------------------------------------------------------------
// Locating the cell
// ---------------------------------------------------------------------------

/// The `<hick:exec>` that owns the element `selector` names, and the id the
/// element carries.
///
/// The selector may name the exec itself or — the shape the design pins — the
/// `<hick:copy>` child holding the command, because wrapping the command in a
/// child is what stops it being ambient text once file bodies sit beside it.
fn locate_exec<'a>(doc: &'a HickDocument, selector: &str) -> Result<(&'a HickTag, String)> {
    let id = selector.trim().trim_start_matches('#').to_string();
    if id.is_empty() {
        bail!("--from needs an element id, like `--from '#scaffold'`");
    }

    fn walk<'a>(
        nodes: &'a [HickNode],
        exec: Option<&'a HickTag>,
        id: &str,
        found: &mut Option<(Option<&'a HickTag>, &'a HickTag)>,
        ids: &mut Vec<String>,
    ) {
        for node in nodes {
            let HickNode::Tag(tag) = node else { continue };
            let here = if tag.name == "exec" { Some(tag) } else { exec };
            if let Some(this_id) = tag.get_attribute("id") {
                ids.push(this_id.to_string());
                if this_id == id && found.is_none() {
                    *found = Some((here, tag));
                }
            }
            walk(&tag.children, here, id, found, ids);
        }
    }

    let mut found = None;
    let mut ids = Vec::new();
    walk(&doc.nodes, None, &id, &mut found, &mut ids);

    match found {
        Some((Some(exec), _)) => Ok((exec, id)),
        Some((None, tag)) => bail!(
            "`#{id}` is a <{p}:{name}>, but it is not inside a <{p}:exec> — \
             ingest reads the output volume of a cell that RAN.\n  \
             Next step: put the id on the cell's command block, like \
             `<{p}:copy id=\"{id}\">…</{p}:copy>` inside the \
             `<{p}:exec>` that runs it.",
            p = doc.prefix,
            name = tag.name,
        ),
        None => bail!(
            "no element with id `{id}` in this document.\n  \
             Ids present: {}.\n  \
             Next step: give the cell's command block an id — \
             `<{p}:copy id=\"scaffold\">dotnet new webapi -o .</{p}:copy>` — \
             and pass `--from '#scaffold'`.",
            if ids.is_empty() {
                "(none)".to_string()
            } else {
                ids.join(", ")
            },
            p = doc.prefix,
        ),
    }
}

/// The output volume the cell writes, and the prefix its files appear under
/// in the pipeline result.
fn output_volume(doc: &HickDocument, exec: &HickTag) -> Result<(String, String)> {
    let mounts: Vec<String> = exec
        .get_attribute("mount")
        .unwrap_or_default()
        .split(',')
        .filter_map(|entry| entry.trim().split_once(':').map(|(v, _)| v.to_string()))
        .collect();
    if mounts.is_empty() {
        bail!(
            "that cell mounts no volume, so it has no output to ingest.\n  \
             A scaffolder's files reach the document through a volume the \
             cell writes into.\n  \
             Next step: declare one — `<{p}:volume name=\"project\" \
             output=\".\" />` — and mount it on the cell with \
             `mount=\"project:/out\"`.",
            p = doc.prefix
        );
    }

    let mut outputs: Vec<(String, String)> = Vec::new();
    for node in &doc.nodes {
        let HickNode::Tag(tag) = node else { continue };
        if tag.name != "volume" {
            continue;
        }
        let Some(name) = tag.get_attribute("name") else {
            continue;
        };
        if !mounts.iter().any(|m| m == name) {
            continue;
        }
        if let Some(out) = tag.get_attribute("output") {
            outputs.push((name.to_string(), out.to_string()));
        }
    }

    match outputs.len() {
        1 => Ok(outputs.remove(0)),
        0 => bail!(
            "the volume(s) that cell mounts ({}) declare no `output=`, so \
             nothing they contain reaches the pipeline as files — and ingest \
             reads the run's output.\n  \
             Next step: add `output=\".\"` to the volume declaration.",
            mounts.join(", ")
        ),
        _ => bail!(
            "that cell mounts more than one output volume ({}), so which run \
             the ingest records is ambiguous.\n  \
             Next step: mount one output volume on the cell you ingest from.",
            outputs
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

// ---------------------------------------------------------------------------
// The element
// ---------------------------------------------------------------------------

/// The fingerprint recorded as `sha256=`: one run, N files.
///
/// Over the bytes the DOCUMENT now holds, path by path in sorted order, each
/// length-prefixed so no rearrangement of names and contents collides. That
/// is the base a re-ingest merges against, and it has to be the set the
/// document contains for the three-way merge to have a real base.
pub fn run_fingerprint(files: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    for (path, content) in files {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// The `<hick:ingested>` block, indented to sit inside its cell.
///
/// Bodies go in raw — the no-escaping invariant means there is no other way
/// to put them there, and no escaping to undo on the way out.
fn ingested_block(
    prefix: &str,
    from: &str,
    fingerprint: &str,
    at: &str,
    files: &BTreeMap<String, String>,
    skipped: usize,
) -> String {
    let mut out = format!(
        "<{prefix}:ingested from=\"{from}\" sha256=\"{fingerprint}\" at=\"{at}\" files=\"{}\" skipped=\"{skipped}\">\n",
        files.len()
    );
    for (path, content) in files {
        // The body starts on its OWN line, which costs the file nothing: the
        // break that ends the open tag's line belongs to the tag and is not
        // part of the file (`strip_opening_break`, guaranteed by
        // docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md).
        // Before that rule existed this had to be written inline, and the
        // byte-exactness check below would have refused the whole ingest over
        // the newline — which is why the old convention was what it was.
        //
        // Legibility is the whole reason: a scaffolded `Program.cs` opening
        // with a BOM and a comment, jammed onto the end of the tag line, is
        // the first thing a reader of this document meets.
        //
        // The CLOSING tag still follows the last byte with nothing added. A
        // file that ends with a newline gets `</hick:file>` on its own line
        // for free; one that does not must not be given a trailing byte it
        // never had.
        out.push_str(&format!("<{prefix}:file path=\"{path}\">\n"));
        out.push_str(content);
        out.push_str(&format!("</{prefix}:file>\n"));
    }
    out.push_str(&format!("</{prefix}:ingested>\n"));
    out
}

/// Byte offset just after `<prefix:name …>`'s matching close tag content —
/// that is, where the closing tag begins.
///
/// The parser records only a tag's OPENING span, and an ingest has to write
/// inside the element. Scanning for the matching close with a depth counter
/// is what the parser itself does; a self-closing opening tag has no inside
/// at all and returns `None`.
pub fn element_content_end(
    source: &str,
    open_span_end: usize,
    prefix: &str,
    name: &str,
) -> Option<usize> {
    let open = format!("<{prefix}:{name}");
    let close = format!("</{prefix}:{name}>");
    if source[..open_span_end].trim_end().ends_with("/>") {
        return None;
    }
    let mut depth = 1usize;
    let mut at = open_span_end;
    loop {
        let next_open = source[at..].find(&open).map(|i| at + i);
        let next_close = source[at..].find(&close).map(|i| at + i);
        match (next_open, next_close) {
            (_, None) => return None,
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                at = o + open.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(c);
                }
                at = c + close.len();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Re-ingest: a three-way merge with a real base
// ---------------------------------------------------------------------------

/// The `<hick:ingested>` block already in the document.
struct RecordedIngest {
    /// The fingerprint of the run it recorded. This is the key that finds the
    /// BASE in git — see [`base_from_git`].
    sha256: String,
    at: String,
    /// The files as the document holds them NOW: the last run's bytes plus
    /// whatever you changed. This is **ours**.
    ours: BTreeMap<String, String>,
    /// Byte span of the whole element in the document source, so a re-ingest
    /// replaces it rather than appending a second one.
    span: (usize, usize),
}

/// What one re-ingest did, per file.
#[derive(Default)]
pub struct MergeReport {
    /// Merged cleanly — either side changed it, or both did compatibly.
    pub merged: Vec<String>,
    /// The fresh run introduced it; the document did not have it.
    pub added: Vec<String>,
    /// The fresh run no longer produces it and you had not changed it, so it
    /// is gone from the document too.
    pub removed: Vec<String>,
    /// The fresh run no longer produces it but you HAD changed it, so it is
    /// kept and named. Deleting somebody's edit because a scaffolder stopped
    /// emitting the file is not a decision a tool gets to make.
    pub kept: Vec<String>,
    /// Both sides changed the same region differently. The markers are in the
    /// document.
    pub conflicted: Vec<String>,
}

impl MergeReport {
    pub fn is_empty_of_change(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.conflicted.is_empty()
    }
}

/// The bytes as they were when this run was ingested — recovered from git.
///
/// This is the piece the design did not have: the document holds `ours` and
/// records the run's `sha256`, but the ORIGINAL bytes are gone, because your
/// four lines overwrote them. A hash verifies; it does not reconstruct.
///
/// Git is where they are. `expression-and-log.md`'s division of labour is
/// exactly this — **the document describes the present, git holds the past** —
/// so the base is the version of this document at the commit that introduced
/// this fingerprint. The ingest is one commit and your four lines are the
/// next, which is the shape that document argues for on its own merits.
///
/// `None` when the ingest was never committed: then there IS no base, and a
/// re-ingest has to say so rather than invent one.
fn base_from_git(
    doc_path: &Path,
    prefix: &str,
    sha256: &str,
) -> Result<Option<(String, BTreeMap<String, String>)>> {
    let Some(root) = crate::replay::git_root(doc_path.parent().unwrap_or(Path::new("."))) else {
        return Ok(None);
    };
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let abs = std::fs::canonicalize(doc_path).unwrap_or_else(|_| doc_path.to_path_buf());
    let Ok(rel) = abs.strip_prefix(&root) else {
        return Ok(None);
    };
    let rel = rel.to_string_lossy().replace('\\', "/");

    // `-S` finds the commits where the count of this string CHANGED, so the
    // oldest of them is the one that introduced it. Reversed, so the first is
    // the oldest — a fingerprint that was introduced, removed and reintroduced
    // would otherwise give the wrong base.
    let out = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args([
            "log",
            "--reverse",
            "--format=%H",
            &format!("-S{sha256}"),
            "--",
            &rel,
        ])
        .output()
        .context("failed to run git log")?;
    if !out.status.success() {
        return Ok(None);
    }
    let Some(commit) = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
    else {
        return Ok(None);
    };

    let show = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["show", &format!("{commit}:{rel}")])
        .output()
        .context("failed to run git show")?;
    if !show.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&show.stdout).to_string();
    let Ok(old) = hick_lang::parse(&text) else {
        return Ok(None);
    };

    let mut files = BTreeMap::new();
    let mut stack: Vec<&hick_lang::HickTag> = old.tags().collect();
    while let Some(tag) = stack.pop() {
        if tag.name == "ingested" && tag.get_attribute("sha256") == Some(sha256) {
            for child in tag.child_tags() {
                if child.name == "file"
                    && let Some(path) = child.get_attribute("path")
                {
                    files.insert(path.to_string(), file_body(child));
                }
            }
        }
        for child in &tag.children {
            if let hick_lang::HickNode::Tag(t) = child {
                stack.push(t);
            }
        }
    }
    let _ = prefix;
    if files.is_empty() {
        return Ok(None);
    }
    Ok(Some((commit, files)))
}

/// The bytes an ingested `<hick:file>` block stands for.
///
/// The line break that ends the block's open tag belongs to the TAG, not to
/// the file — the same rule the pipeline applies when it writes the file out
/// (`docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md`).
/// Reading it back with `tag_text` alone would hand the merge a leading
/// newline the scaffolder never produced, and since the run's side has no
/// such byte, EVERY file would look changed on our side: a re-ingest that
/// should merge cleanly reports a conflict on the first line of everything.
fn file_body(tag: &hick_lang::HickTag) -> String {
    let text = hick_lang::tag_text(tag);
    text.strip_prefix("\r\n")
        .or_else(|| text.strip_prefix('\n'))
        .map(str::to_string)
        .unwrap_or(text)
}

/// Three-way merge one file's text. Returns `(merged, conflicted)`.
fn merge_one(base: &str, ours: &str, theirs: &str) -> Result<(String, bool)> {
    if ours == theirs {
        return Ok((ours.to_string(), false));
    }
    if base == ours {
        // Only the scaffolder changed it: take the new SDK's version.
        return Ok((theirs.to_string(), false));
    }
    if base == theirs {
        // Only you changed it: the new run produces what the old one did.
        return Ok((ours.to_string(), false));
    }
    let dir = tempfile::tempdir().context("could not create a scratch directory")?;
    let (b, o, t) = (
        dir.path().join("b"),
        dir.path().join("o"),
        dir.path().join("t"),
    );
    std::fs::write(&b, base)?;
    std::fs::write(&o, ours)?;
    std::fs::write(&t, theirs)?;
    let out = Command::new("git")
        .args([
            "merge-file",
            "-L",
            "yours",
            "-L",
            "the previous run",
            "-L",
            "this run",
        ])
        .arg(&o)
        .arg(&b)
        .arg(&t)
        .output()
        .context("failed to run `git merge-file`")?;
    let conflicts = out.status.code().unwrap_or(-1);
    if conflicts < 0 {
        bail!(
            "the three-way merge failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let merged = std::fs::read_to_string(&o)?;
    Ok((merged, conflicts > 0))
}

/// Merge the three sides of a re-ingest into one file set.
///
/// Set membership is merged as well as content, because a new SDK adds and
/// drops files: a path only in `theirs` is new, and a path the run stopped
/// producing is removed **only if you had not touched it** — deleting
/// somebody's edit because a scaffolder changed its mind is not a decision a
/// tool gets to make.
fn merge_ingests(
    base: &BTreeMap<String, String>,
    ours: &BTreeMap<String, String>,
    theirs: &BTreeMap<String, String>,
) -> Result<(BTreeMap<String, String>, MergeReport)> {
    let mut out = BTreeMap::new();
    let mut report = MergeReport::default();

    let mut paths: Vec<&String> = base
        .keys()
        .chain(ours.keys())
        .chain(theirs.keys())
        .collect();
    paths.sort();
    paths.dedup();

    for path in paths {
        match (base.get(path), ours.get(path), theirs.get(path)) {
            (Some(b), Some(o), Some(t)) => {
                let (merged, conflicted) = merge_one(b, o, t)?;
                out.insert(path.clone(), merged);
                if conflicted {
                    report.conflicted.push(path.clone());
                } else if o != t {
                    report.merged.push(path.clone());
                }
            }
            // The run introduced it.
            (None, None, Some(t)) => {
                out.insert(path.clone(), t.clone());
                report.added.push(path.clone());
            }
            // You added it inside the block; the run knows nothing about it.
            (None, Some(o), None) => {
                out.insert(path.clone(), o.clone());
            }
            // Both added the same path independently: no base, so merge is
            // two-way and there is nothing to reconcile against.
            (None, Some(o), Some(t)) => {
                let (merged, conflicted) = merge_one("", o, t)?;
                out.insert(path.clone(), merged);
                if conflicted {
                    report.conflicted.push(path.clone());
                } else if o != t {
                    report.merged.push(path.clone());
                }
            }
            // The run stopped producing it.
            (Some(b), Some(o), None) => {
                if b == o {
                    report.removed.push(path.clone());
                } else {
                    out.insert(path.clone(), o.clone());
                    report.kept.push(path.clone());
                }
            }
            // It was in the base and is back in the run, but you deleted it.
            (Some(_), None, Some(_)) => {
                report.removed.push(path.clone());
            }
            (Some(_), None, None) => {
                report.removed.push(path.clone());
            }
            (None, None, None) => {}
        }
    }
    Ok((out, report))
}

// ---------------------------------------------------------------------------
// The command
// ---------------------------------------------------------------------------

/// Ingest the output volume of the cell `selector` names into `doc`.
pub async fn ingest_from_exec(
    doc_path: &Path,
    selector: &str,
    executor: ExecutorChoice,
    today: &str,
) -> Result<IngestExecOutcome> {
    let source = std::fs::read_to_string(doc_path)
        .with_context(|| format!("could not read {}", doc_path.display()))?;
    let doc = hick_lang::parse(&source)
        .map_err(|e| anyhow::anyhow!("{} does not parse: {e}", doc_path.display()))?;
    let prefix = doc.prefix.clone();

    let (exec, id) = locate_exec(&doc, selector)?;
    let from = format!("#{id}");

    // A second ingest into the same cell is a THREE-WAY MERGE against the
    // recorded base — the old ingested bytes are the base, the fresh run is
    // theirs, and the document (with your four lines) is ours.
    let existing = exec
        .child_tags()
        .find(|c| c.name == "ingested")
        .map(|tag| RecordedIngest {
            sha256: tag.get_attribute("sha256").unwrap_or_default().to_string(),
            at: tag.get_attribute("at").unwrap_or_default().to_string(),
            ours: tag
                .child_tags()
                .filter(|c| c.name == "file")
                .filter_map(|c| Some((c.get_attribute("path")?.to_string(), file_body(c))))
                .collect::<BTreeMap<String, String>>(),
            span: (
                tag.source_span.map(|s| s.start).unwrap_or(0),
                element_content_end(
                    &source,
                    tag.source_span.map(|s| s.end).unwrap_or(0),
                    &prefix,
                    "ingested",
                )
                .map(|end| end + format!("</{prefix}:ingested>").len())
                .unwrap_or(0),
            ),
        });

    let (volume, out_prefix) = output_volume(&doc, exec)?;
    let open_end = exec
        .source_span
        .map(|s| s.end)
        .context("that cell has no recorded source span, so there is nowhere to write inside it")?;
    let insert_at = element_content_end(&source, open_end, &prefix, "exec").with_context(|| {
        format!(
            "could not find the closing </{prefix}:exec> for the cell at line {}",
            exec.source_line
        )
    })?;

    // Run the document. The volume is extracted, unpacked and merged into the
    // pipeline result by the pipeline itself; ingest reads that, and nothing
    // watches a directory and nothing is moved.
    let run = run_doc(doc_path, &[], RunMode::Execute, executor)
        .await
        .with_context(|| format!("running {} failed", doc_path.display()))?;

    let mut text: BTreeMap<String, String> = BTreeMap::new();
    let mut binary: Vec<String> = Vec::new();
    // Read the volume BY NAME rather than by sifting the merged file set for a
    // path prefix. Two things go wrong with the prefix approach, and both are
    // silent: a volume declared `output="."` shares its prefix with every
    // other output the document produces, so the ingest swallows files
    // belonging to the document's own `hick:file` blocks; and a volume this
    // document has already ingested is deliberately kept out of the merged set
    // altogether, so a re-ingest would find nothing.
    let produced = run.result.volume_outputs.get(&volume);
    for (path, content) in produced.into_iter().flatten() {
        // The path is kept AS THE PIPELINE NAMES IT — prefixed by the volume's
        // `output=`. Stripping the prefix would move every file the moment it
        // was ingested: `hick run` wrote `service/main.rs` before, and would
        // write `main.rs` after, which is an ingest silently rearranging the
        // tree it was asked to preserve.
        match content.as_text() {
            Some(t) => {
                text.insert(path.clone(), t.to_string());
            }
            None => binary.push(path.clone()),
        }
    }
    if text.is_empty() && binary.is_empty() {
        bail!(
            "the run produced no files under volume `{volume}` (output \
             `{out_prefix}`), so there is nothing to ingest.\n  \
             Next step: check the cell's command actually writes into its \
             mount, and that the mount path and the volume name match."
        );
    }

    // Filter first, refuse second. `obj/` full of binaries is the normal
    // case and must disappear quietly; a binary the project would KEEP is
    // the one that cannot be represented and has to be named.
    let doc_dir = doc_path.parent().unwrap_or(Path::new("."));
    let candidates: Vec<String> = text.keys().chain(binary.iter()).cloned().collect();
    let ignored = hick_literate::volume_state::gitignored(doc_dir, &candidates)?;
    let mut skipped: Vec<String> = Vec::new();
    match &ignored {
        Some(ignored) => {
            for path in ignored {
                text.remove(path);
                binary.retain(|b| b != path);
                skipped.push(path.clone());
            }
        }
        // `gitignored` distinguishes "checked, nothing to skip" (`Some([])`)
        // from "could not check at all" (`None`) for exactly this reason —
        // silently treating them alike would report `skipped="0"` on an
        // ingest that never consulted `.gitignore`, which reads as "every
        // produced file was reviewed" when in fact the filter never ran.
        None => {
            log::warn!(
                "not a git repository at {}: .gitignore filtering was skipped, \
                 all {} produced file(s) were ingested unfiltered. Run `hick \
                 init` for a real repository, or `git init` directly.",
                doc_dir.display(),
                candidates.len(),
            );
        }
    }
    skipped.sort();

    if !binary.is_empty() {
        binary.sort();
        let shown: Vec<&str> = binary.iter().take(10).map(String::as_str).collect();
        bail!(
            "{} file(s) that run produced are not UTF-8 text, so they cannot \
             be document bytes: {}{}.\n  \
             A <{prefix}:file> body is raw bytes under the no-escaping \
             invariant — there is no encoding to hide a binary in, and \
             writing one would silently mangle it. Nothing was changed.\n  \
             Next steps: if these are build output, add them to the \
             project's .gitignore and run this again — the gitignore is the \
             filter. If they are real artifacts the scaffold needs, they have \
             no home in a document yet.",
            binary.len(),
            shown.join(", "),
            if binary.len() > shown.len() {
                format!(", … ({} more)", binary.len() - shown.len())
            } else {
                String::new()
            },
        );
    }

    // The no-escaping invariant, checked before anything is written rather
    // than discovered as a mangled weave: a scaffolded file containing a
    // `prefix:`-tagged string would be read as structure.
    let opener = format!("<{prefix}:");
    if let Some((path, _)) = text.iter().find(|(_, c)| c.contains(&opener)) {
        bail!(
            "{path} contains the text `{opener}`, which the parser reads as \
             structure rather than as content. The no-escaping invariant is \
             what makes every other byte in a document literal, and it has no \
             way to quote this. Nothing was changed.\n  \
             Next step: exclude that file from the run's output volume, or \
             ingest a cell whose output does not contain hick markup."
        );
    }

    // A re-ingest is a three-way merge; a first ingest is just the run.
    let (text, report, base_commit) = match &existing {
        None => (text, MergeReport::default(), None),
        Some(existing) => {
            let Some((commit, base)) = base_from_git(doc_path, &prefix, &existing.sha256)? else {
                bail!(
                    "that cell already has an <{prefix}:ingested> block recorded \
                     {} with sha256 {}, but the commit that introduced it is not \
                     in this repository's history — so there is no BASE to merge \
                     against.\n  \
                     A re-ingest is a three-way merge: the bytes as they were \
                     ingested are the base, this run is theirs, and the document \
                     is ours. The hash records WHICH run it was; git is what \
                     holds the bytes, because your own edits overwrote them in \
                     the document.\n  \
                     Next steps: commit the existing ingest first and run this \
                     again — or, to start over and lose your edits inside the \
                     block, delete the block by hand.",
                    if existing.at.is_empty() {
                        "(no date)"
                    } else {
                        &existing.at
                    },
                    if existing.sha256.is_empty() {
                        "(none)"
                    } else {
                        &existing.sha256
                    },
                );
            };
            let (merged, report) = merge_ingests(&base, &existing.ours, &text)?;
            (merged, report, Some(commit))
        }
    };

    let fingerprint = run_fingerprint(&text);
    let block = ingested_block(&prefix, &from, &fingerprint, today, &text, skipped.len());

    let mut next = source.clone();
    match &existing {
        // Replace the old element in place: a second block beside the first
        // would make the cell claim two runs produced it.
        Some(existing) => next.replace_range(existing.span.0..existing.span.1, &block),
        None => next.insert_str(insert_at, &block),
    }
    // A machine writing your source, and one of the writers that had no way
    // back at all. Recorded before the write, so the document immediately
    // before an ingest is one command away — which is also what makes the
    // re-ingest merge's false conflicts cheap to study rather than
    // frightening to trigger.
    crate::history::record(
        doc_path.parent().unwrap_or(std::path::Path::new(".")),
        hickory_workspace::history::ActKind::Ingest,
        Some(from.clone()),
        &[(doc_path.to_path_buf(), next.clone().into_bytes())],
    );
    std::fs::write(doc_path, &next)
        .with_context(|| format!("could not write {}", doc_path.display()))?;

    // The same gate adoption passes or dies on: weave the document as it now
    // stands and prove every ingested file comes back byte-for-byte. A
    // failure restores the document, so a refused ingest leaves no trace.
    let restore = |reason: anyhow::Error| -> anyhow::Error {
        match std::fs::write(doc_path, &source) {
            Ok(()) => reason,
            Err(e) => anyhow::anyhow!(
                "{reason:#}; AND restoring {} failed ({e}) — check it by hand",
                doc_path.display()
            ),
        }
    };
    let woven = run_doc(doc_path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .map_err(|e| {
            restore(anyhow::anyhow!(
                "the document does not weave once the run's files are in it \
                 ({e:#}). Nothing was changed."
            ))
        })?;
    for (path, content) in &text {
        match woven.result.files.get(path).and_then(|c| c.as_text()) {
            Some(back) if back == content => {}
            Some(back) => {
                return Err(restore(anyhow::anyhow!(
                    "ingest would not be byte-exact: {path} weaves as {} bytes \
                     but the run produced {}. Nothing was changed.",
                    back.len(),
                    content.len(),
                )));
            }
            None => {
                return Err(restore(anyhow::anyhow!(
                    "the document does not produce {path} after ingesting it. \
                     Nothing was changed."
                )));
            }
        }
    }

    // A re-ingest is the richest recording site the scaffolder path has —
    // base, ours and theirs in hand at one moment. Its precision is DIFF and
    // that is not a lesser byte-precision: two runs of a scaffolder share no
    // history, so no byte-precise thread exists to record even with the tool
    // watching the whole time. This is the case that forces the distinction.
    let recorded = record_reingest(doc_path, &existing, &text, base_commit.as_deref(), today);

    Ok(IngestExecOutcome {
        doc_path: doc_path.to_path_buf(),
        from,
        fingerprint,
        ingested: text.keys().cloned().collect(),
        skipped,
        merge: report,
        base_commit,
        recorded,
    })
}

/// Record what a re-ingest moved, if continuity is on. Zero otherwise — the
/// switch is the whole feature, not just its drawing.
fn record_reingest(
    doc_path: &Path,
    existing: &Option<RecordedIngest>,
    merged: &BTreeMap<String, String>,
    base_commit: Option<&str>,
    today: &str,
) -> usize {
    let Some(existing) = existing else { return 0 };
    let Some(root) = crate::replay::git_root(doc_path.parent().unwrap_or(Path::new("."))) else {
        return 0;
    };
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    if !crate::continuity::enabled(&root) {
        return 0;
    }
    let abs = std::fs::canonicalize(doc_path).unwrap_or_else(|_| doc_path.to_path_buf());
    let Ok(rel) = abs.strip_prefix(&root) else {
        return 0;
    };
    let rel = rel.to_string_lossy().replace('\\', "/");

    // One entry per file that survived the merge: the block it was in before,
    // and the block it is in now. Coarse on purpose — the span is the whole
    // element on each side, because there is no finer thread between two runs
    // of a foreign tool.
    let entries: Vec<crate::continuity::Correspondence> = merged
        .keys()
        .filter(|path| existing.ours.contains_key(*path))
        .map(|_| crate::continuity::Correspondence {
            from: crate::continuity::Endpoint {
                commit: base_commit.map(str::to_string),
                path: rel.clone(),
                span: existing.span,
            },
            to: crate::continuity::Endpoint {
                commit: None,
                path: rel.clone(),
                span: existing.span,
            },
            precision: crate::continuity::Precision::Diff,
            site: crate::continuity::Site::Reingest,
            provisional: true,
            head: None,
            recorded_at: today.to_string(),
        })
        .collect();
    crate::continuity::Journal::at(&root)
        .append(&entries)
        .unwrap_or(0)
}
