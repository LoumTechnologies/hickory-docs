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

use anyhow::{Context as _, Result, bail};
use hick_lang::{HickDocument, HickNode, HickTag};
use sha2::{Digest, Sha256};

use crate::{CacheMode, ExecutorChoice, RunMode, run_doc, run_doc_subset};

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

/// `target`, and every exec cell it transitively depends on — the subgraph
/// `hick ingest --from` needs to run to read `target`'s output, and no more.
///
/// `FlowDag::predecessors` returns only DIRECT predecessors, so this walks
/// them to a fixpoint. The result always includes `target` itself.
fn transitive_predecessors(
    dag: &hick_exec::dag::FlowDag,
    target: hick_exec::dag::ExecId,
) -> std::collections::HashSet<hick_exec::dag::ExecId> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![target];
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        stack.extend(dag.predecessors(id));
    }
    seen
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

/// Whether the cell already carries an ingest: its `<hick:ingested>` child,
/// with the run it recorded.
///
/// A second ingest into a cell is **refused**, not merged. The three-way
/// re-ingest merge this used to do treated a scaffold as a living
/// expression to re-evaluate over your edits; `lenses.md` retired it, since
/// a scaffold is an act and is upgraded through its commit's recipe in the
/// history lens. `hick ingest --from '#cell'` stays for what it was really
/// for — bringing an exec's output into a document you are writing, once.
fn existing_ingest(exec: &HickTag) -> Option<(String, String)> {
    exec.child_tags().find(|c| c.name == "ingested").map(|tag| {
        (
            tag.get_attribute("sha256").unwrap_or_default().to_string(),
            tag.get_attribute("at").unwrap_or_default().to_string(),
        )
    })
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

    if let Some((sha256, at)) = existing_ingest(exec) {
        bail!(
            "that cell already has an <{prefix}:ingested> block, recorded {} with sha256 {}. \
             An ingest brings a run's output into a document once; it is not re-run over \
             your edits.\n  \
             To take a newer scaffold, scaffold it as a commit (File → New Project) and \
             read it in the history lens, where a recipe commit can be replayed. To start \
             this cell over and lose the edits inside its block, delete the block by hand \
             and ingest again. Nothing was changed.",
            if at.is_empty() { "(no date)" } else { &at },
            if sha256.is_empty() { "(none)" } else { &sha256 },
        );
    }

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

    // Run only the target cell and its real dependencies, not the whole
    // document — a cell being authored BEFORE the rest of the document that
    // will depend on it is the ordinary order to write one in, and a
    // document with other, unrelated cells that do not yet pass must not
    // block ingesting this one. The subset is the target's transitive
    // predecessors in the DAG, plus the target itself; anything outside it
    // is never visited, so it is neither run nor required to succeed.
    let flow_dag = hick_exec::dag::build_dag(&doc)
        .map_err(|e| anyhow::anyhow!("DAG validation failed in {}: {e}", doc_path.display()))?;
    let target_container = exec.get_attribute("container").unwrap_or_default();
    let target_id = flow_dag
        .execs
        .iter()
        .find(|e| e.container == target_container && e.source_line == exec.source_line)
        .map(|e| e.id)
        .with_context(|| {
            format!(
                "could not find the cell at line {} in the document's DAG",
                exec.source_line
            )
        })?;
    let subset = transitive_predecessors(&flow_dag, target_id);

    // The volume is extracted, unpacked and merged into the pipeline result
    // by the pipeline itself; ingest reads that, and nothing watches a
    // directory and nothing is moved.
    let run = run_doc_subset(
        doc_path,
        &[],
        RunMode::Execute,
        executor,
        CacheMode::Off,
        Some(subset),
    )
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

    let fingerprint = run_fingerprint(&text);
    let block = ingested_block(&prefix, &from, &fingerprint, today, &text, skipped.len());

    let mut next = source.clone();
    next.insert_str(insert_at, &block);
    // A machine writing your source, and one of the writers that had no way
    // back at all. Recorded before the write, so the document immediately
    // before an ingest is one command away.
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

    Ok(IngestExecOutcome {
        doc_path: doc_path.to_path_buf(),
        from,
        fingerprint,
        ingested: text.keys().cloned().collect(),
        skipped,
    })
}
