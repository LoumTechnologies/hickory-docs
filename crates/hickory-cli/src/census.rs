//! `hick census`: how many lines of the code on disk a person actually typed.
//!
//! The question this answers is not "how big is the project" but **how much
//! of what landed on disk was written by hand**. A literate codebase produces
//! its files, and the produced bytes come from three very different places:
//! text a person typed into a document, output something the document ran
//! produced (or the weaver drew), and whole files a cell wrote into an output
//! volume. Lineage already knows which is which, byte by byte
//! (`hickory_lineage::Provenance`); this counts it.
//!
//! # What the census counts, exactly
//!
//! Every document under PATH is **woven, never run** — recorded transcripts
//! where present, no cell ever executes — the same way `hick weave` and
//! `hick lineage` do. From each weave it takes:
//!
//! - the **woven markdown** (`weave=` on the document, or its own name), and
//! - every **tangled `hick:file` product**,
//!
//! both of which have byte-precise lineage, and classifies each **line**:
//!
//! - `authored` — every byte of the line came from a `Literal` or `Paste`
//!   span of a document: prose, a `hick:copy`/`hick:cut` fragment pasted in
//!   verbatim, the literal text inside a `hick:file`.
//! - `generated` — anything else on the line: an exec transcript, a
//!   `hick:sample`, a `hick:transform` passage, a `hick:val`, ingested
//!   bytes, an agent cell's writing, or something the weaver synthesised
//!   (headings, code fences, attribution lines, an SVG it drew).
//!
//! **A line with mixed bytes is generated.** That is deliberate and it is
//! the one rule worth stating out loud: the fence line the weaver draws
//! around a `hick:file` block is the *weaver's* line even though every line
//! of code inside it is yours, so the fence counts generated and the code
//! inside counts authored. Counting the fence as authored would credit a
//! person for punctuation they never typed; counting the block as generated
//! because of its fence would erase the code they did.
//!
//! # What a weave cannot attribute
//!
//! A file a cell wrote into an **output volume** has no byte lineage and
//! never will — its bytes are a program's output, and no span of any
//! document produced them. Worse, a weave does not even know its *contents*:
//! `volume_outputs` is filled by the live pipeline only, so weaving a
//! document produces an empty map (`hick_literate::PipelineResult::
//! volume_outputs`, and the two `// Weave-only: nothing executed` comments
//! beside it). What the weave *does* know, from the document alone, is which
//! volumes are declared `output=` and which cells mount them.
//!
//! So the census attributes those files **coarsely and from disk**: every
//! file under a declared output volume's prefix that no document tangles is
//! counted as `cell-output`, blamed on the cell ids (or the `hick:exec`
//! source lines, when a cell has no `container=`) that mount that volume.
//! Its lines are neither authored nor generated-with-lineage; they are their
//! own column, because "this whole file is that cell's output" is a weaker
//! claim than a byte range and must not be printed as if it were the same
//! one. If the cell has never run on this machine, the files are not there
//! and the volume contributes nothing — which the report says rather than
//! guessing.
//!
//! # What is not counted at all
//!
//! Files under `.hick-cache/`, gitignored files, and files no document
//! produces. The last of those is reported as a footer count rather than
//! silently dropped, so a repository that is only half literate cannot look
//! fully literate.
//!
//! Guarantee: `docs/guarantees/lineage/a-census-counts-what-was-not-typed.md`

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use hick_lang::{HickNode, HickTag};
use hickory_lineage::{Origin, Provenance};
use serde::Serialize;

use crate::{DocRun, ExecutorChoice, RunMode, expand_docs, output_lineage, run_doc};

// ---------------------------------------------------------------------------
// The report
// ---------------------------------------------------------------------------

/// Line counts over one file, one folder, or everything.
///
/// Field order is the printed order and the JSON order, and it is stable so
/// a `hick:exec` cell can pin `hick census --json .` with a `hick:expect`.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct LineTotals {
    /// Every line counted, `authored + generated + cell_output`.
    pub lines: usize,
    /// Lines every byte of which came from a document a person typed.
    pub authored: usize,
    /// Lines carrying at least one byte nobody typed.
    pub generated: usize,
    /// Lines of a file a cell wrote into an output volume: no byte lineage,
    /// attributed to the cell.
    pub cell_output: usize,
}

impl LineTotals {
    fn add(&mut self, other: &LineTotals) {
        self.lines += other.lines;
        self.authored += other.authored;
        self.generated += other.generated;
        self.cell_output += other.cell_output;
    }

    /// Authored lines as a percentage of lines counted, to one decimal.
    pub fn authored_pct(&self) -> f64 {
        pct(self.authored, self.lines)
    }

    /// Generated lines as a percentage of lines counted, to one decimal.
    pub fn generated_pct(&self) -> f64 {
        pct(self.generated, self.lines)
    }

    /// Cell-output lines as a percentage of lines counted, to one decimal.
    pub fn cell_output_pct(&self) -> f64 {
        pct(self.cell_output, self.lines)
    }
}

fn pct(part: usize, whole: usize) -> f64 {
    match whole {
        0 => 0.0,
        _ => (part as f64) * 100.0 / (whole as f64),
    }
}

/// How a produced file got its bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileKind {
    /// The document's own markdown weave.
    Woven,
    /// A `hick:file` product tangled out of the document.
    Tangled,
    /// A file a cell left in an output volume.
    CellOutput,
}

impl FileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FileKind::Woven => "woven",
            FileKind::Tangled => "tangled",
            FileKind::CellOutput => "cell-output",
        }
    }
}

/// One produced file.
#[derive(Debug, Clone, Serialize)]
pub struct FileCensus {
    /// Path relative to the census root, with `/` separators.
    pub path: String,
    /// The document that produces it, relative to the census root.
    pub document: String,
    pub kind: FileKind,
    #[serde(flatten)]
    pub totals: LineTotals,
    /// The output volume, for a `cell-output` file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// The cells that mount that volume, by `container=` id.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cells: Vec<String>,
    /// The `hick:exec` source lines that mount it — the answer when a cell
    /// has no id, and always worth printing beside one that has.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cell_lines: Vec<usize>,
}

impl FileCensus {
    /// "document + cell id" — who to go and change.
    pub fn generator(&self) -> String {
        match (&self.volume, self.cells.is_empty()) {
            (None, _) => self.document.clone(),
            (Some(vol), true) => format!(
                "{} · volume {vol} (lines {})",
                self.document,
                join_lines(&self.cell_lines)
            ),
            (Some(vol), false) => format!(
                "{} · #{} → volume {vol}",
                self.document,
                self.cells.join(", #")
            ),
        }
    }
}

fn join_lines(lines: &[usize]) -> String {
    lines
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// What a document is, in its own lines: prose, code, and markup.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct DocumentLines {
    /// Lines of the document, its own source only.
    pub lines: usize,
    /// Prose: text outside any element that holds bytes.
    pub prose: usize,
    /// Fragment: lines inside an element whose body is bytes rather than
    /// prose — `copy`, `cut`, `file`, `exec`, `expect`, `diagram`, `table`,
    /// `math`. This is "how much of what I wrote is code".
    pub fragment: usize,
    /// Tag: lines that are nothing but hick markup.
    pub tag: usize,
}

impl DocumentLines {
    fn add(&mut self, other: &DocumentLines) {
        self.lines += other.lines;
        self.prose += other.prose;
        self.fragment += other.fragment;
        self.tag += other.tag;
    }
}

/// One document: what it is, and what it produced.
#[derive(Debug, Clone, Serialize)]
pub struct DocumentCensus {
    /// Path relative to the census root.
    pub path: String,
    #[serde(flatten)]
    pub source: DocumentLines,
    /// Totals over every file this document produces.
    pub produced: LineTotals,
    /// Those files, in path order.
    pub files: Vec<String>,
}

/// A top-level folder of the census root, or `.` for files beside it.
#[derive(Debug, Clone, Serialize)]
pub struct FolderCensus {
    pub folder: String,
    /// Produced files under it.
    pub files: usize,
    #[serde(flatten)]
    pub totals: LineTotals,
}

/// The whole report.
#[derive(Debug, Clone, Serialize)]
pub struct Census {
    /// The path the census was taken over, as given.
    pub root: String,
    /// Documents, in path order.
    pub documents: Vec<DocumentCensus>,
    /// Produced files, in path order.
    pub files: Vec<FileCensus>,
    /// Totals per top-level folder, in name order.
    pub folders: Vec<FolderCensus>,
    /// Totals over every produced file.
    pub totals: LineTotals,
    /// Totals over every document's own source.
    pub document_totals: DocumentLines,
    /// Tracked, non-ignored files under the root that no document produces,
    /// not counting the `.hick` documents themselves. Reported, never
    /// counted: a repository that is only half literate must not be able to
    /// look fully literate.
    pub unproduced: usize,
    /// Documents that declare an output volume the weave could not see the
    /// contents of, because nothing under its prefix is on disk. Named so
    /// "no cell-output lines" is never mistaken for "no cell wrote".
    pub empty_volumes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Taking the census
// ---------------------------------------------------------------------------

/// Weave every document under `path` and count what it produced.
///
/// Weave-only: recorded transcripts where present, and no cell ever runs.
pub async fn census(path: &Path) -> Result<Census> {
    let docs = expand_docs(path)?;
    let root = census_root(path);
    let mut files: Vec<FileCensus> = Vec::new();
    let mut documents: Vec<DocumentCensus> = Vec::new();
    let mut claimed: BTreeSet<String> = BTreeSet::new();
    // Volume claims run in a second pass over every document, because a
    // volume declared `output="."` covers its whole folder — including files
    // a SIBLING document tangles, which are that sibling's and not the
    // cell's.
    let mut pending_volumes: Vec<(String, PathBuf, VolumeDecl)> = Vec::new();

    for doc_path in &docs {
        let run = run_doc(doc_path, &[], RunMode::Weave, ExecutorChoice::Local)
            .await
            .with_context(|| format!("weaving {}", doc_path.display()))?;
        let doc_rel = relative_to(&root, doc_path);
        let doc_dir = doc_path.parent().unwrap_or(Path::new("."));
        let mut produced = Vec::new();
        for (rel_path, _) in sorted_files(&run) {
            let Some(text) = content_text(&run, rel_path) else {
                // A picture the document tangles has no lines to count, but
                // it is still produced, so it must not fall into the
                // "nobody produces this" footer.
                claimed.insert(relative_to(&root, &doc_dir.join(rel_path)));
                continue;
            };
            let provenance = output_lineage(&run, rel_path).unwrap_or_default();
            let totals = classify_output(&text, &provenance);
            let path = relative_to(&root, &doc_dir.join(rel_path));
            claimed.insert(path.clone());
            produced.push(path.clone());
            files.push(FileCensus {
                path,
                document: doc_rel.clone(),
                kind: match run.doc.weave_path.as_deref() == Some(rel_path) {
                    true => FileKind::Woven,
                    false => FileKind::Tangled,
                },
                totals,
                volume: None,
                cells: Vec::new(),
                cell_lines: Vec::new(),
            });
        }
        for volume in output_volumes(&run) {
            pending_volumes.push((doc_rel.clone(), doc_dir.to_path_buf(), volume));
        }
        documents.push(DocumentCensus {
            path: doc_rel,
            source: document_lines(&run.source),
            produced: LineTotals::default(),
            files: produced,
        });
    }

    let mut empty_volumes = Vec::new();
    for (doc_rel, doc_dir, volume) in &pending_volumes {
        let found = claim_volume(&root, doc_dir, doc_rel, volume, &mut claimed, &mut files);
        if found == 0 {
            empty_volumes.push(format!("{doc_rel} · volume {}", volume.name));
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    finish(root, path, files, documents, claimed, empty_volumes)
}

/// Fold per-file counts into documents, folders and the grand total, and
/// count what nothing produced.
fn finish(
    root: PathBuf,
    given: &Path,
    files: Vec<FileCensus>,
    mut documents: Vec<DocumentCensus>,
    claimed: BTreeSet<String>,
    empty_volumes: Vec<String>,
) -> Result<Census> {
    let mut by_document: BTreeMap<&str, LineTotals> = BTreeMap::new();
    let mut by_folder: BTreeMap<String, (usize, LineTotals)> = BTreeMap::new();
    let mut totals = LineTotals::default();
    for file in &files {
        totals.add(&file.totals);
        by_document
            .entry(file.document.as_str())
            .or_default()
            .add(&file.totals);
        let entry = by_folder.entry(top_folder(&file.path)).or_default();
        entry.0 += 1;
        entry.1.add(&file.totals);
    }
    let mut document_totals = DocumentLines::default();
    for doc in &mut documents {
        document_totals.add(&doc.source);
        doc.produced = by_document
            .get(doc.path.as_str())
            .copied()
            .unwrap_or_default();
        // Recomputed from the finished list rather than kept as the first
        // pass left it, so a volume claimed in the second pass appears
        // against the document that declared it.
        doc.files = files
            .iter()
            .filter(|f| f.document == doc.path)
            .map(|f| f.path.clone())
            .collect();
    }
    documents.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Census {
        root: given.display().to_string(),
        documents,
        unproduced: unproduced_count(&root, &claimed),
        files,
        folders: by_folder
            .into_iter()
            .map(|(folder, (files, totals))| FolderCensus {
                folder,
                files,
                totals,
            })
            .collect(),
        totals,
        document_totals,
        empty_volumes,
    })
}

/// The directory the census is rooted at: PATH, or the folder of a single
/// document named directly.
fn census_root(path: &Path) -> PathBuf {
    match path.is_file() {
        true => path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        false => path.to_path_buf(),
    }
}

/// `path` relative to the census root, `/`-separated, never absolute and
/// never `..`-prefixed when it can be avoided.
fn relative_to(root: &Path, path: &Path) -> String {
    let normal = normalize(path);
    let base = normalize(root);
    let rel = normal.strip_prefix(&base).unwrap_or(&normal);
    let text = rel.display().to_string().replace('\\', "/");
    match text.is_empty() {
        true => ".".to_string(),
        false => text,
    }
}

/// Resolve `.` and `..` textually — `canonicalize` refuses a path that is
/// not on disk yet, and a document may declare an output nothing has written.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The top-level folder of the census root a path sits under, `.` for a file
/// beside it.
fn top_folder(path: &str) -> String {
    match path.split_once('/') {
        Some((head, _)) => head.to_string(),
        None => ".".to_string(),
    }
}

/// This run's outputs in a stable order, so two censuses of one repository
/// agree line for line.
fn sorted_files(run: &DocRun) -> Vec<(&String, &hick_exec::node::FileContent)> {
    let mut out: Vec<_> = run.result.files.iter().collect();
    out.sort_by(|a, b| a.0.cmp(b.0));
    out
}

/// The text of one output, or `None` for a picture and anything else binary.
fn content_text(run: &DocRun, rel_path: &str) -> Option<String> {
    run.result
        .files
        .get(rel_path)
        .and_then(|c| c.as_text().map(str::to_string))
}

// ---------------------------------------------------------------------------
// Classifying an output's lines
// ---------------------------------------------------------------------------

/// Count one output's lines by the origin of their bytes.
///
/// A line is `authored` when EVERY byte of it (its newline included) came
/// from a `Literal` or `Paste` span of some document, and `generated`
/// otherwise — see the module header for why the fence line around a
/// literal block is the weaver's.
pub fn classify_output(content: &str, provenance: &[Provenance]) -> LineTotals {
    let mut typed = vec![false; content.len()];
    for span in provenance {
        if !matches!(span.origin, Origin::Literal { .. } | Origin::Paste { .. }) {
            continue;
        }
        let start = span.start.min(typed.len());
        let end = span.end.min(typed.len());
        for byte in &mut typed[start..end] {
            *byte = true;
        }
    }
    let mut totals = LineTotals::default();
    for (start, end) in line_spans(content) {
        totals.lines += 1;
        match typed[start..end].iter().all(|t| *t) {
            true => totals.authored += 1,
            false => totals.generated += 1,
        }
    }
    totals
}

/// Byte ranges of each line, the terminating newline included.
///
/// The newline belongs to the line it ends, which is what lets a blank line
/// be attributed at all: it has no other bytes. A file that does not end in
/// a newline still has a last line; one that does has no phantom line after
/// it.
fn line_spans(content: &str) -> Vec<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            out.push((start, i + 1));
            start = i + 1;
        }
    }
    if start < bytes.len() {
        out.push((start, bytes.len()));
    }
    out
}

// ---------------------------------------------------------------------------
// A document's own lines
// ---------------------------------------------------------------------------

/// Elements whose body is BYTES rather than prose — the same closed list
/// `hick_lang`'s parser keeps as `holds_bytes`, which is `pub(crate)` there.
/// A line inside one of these is code the author wrote.
fn holds_bytes(name: &str) -> bool {
    matches!(
        name,
        "copy" | "cut" | "file" | "exec" | "expect" | "diagram" | "table" | "math"
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ByteKind {
    Prose,
    Fragment,
    Markup,
}

/// Split a document's OWN source into prose, fragment and tag lines.
///
/// Parsed here rather than taken from the run, because the run's document
/// has every sibling in the folder spliced into it (ambient contribution)
/// and the question is what *this* author wrote in *this* file. A document
/// that does not parse is all prose, which is the honest answer for text
/// nothing could read as structure.
pub fn document_lines(source: &str) -> DocumentLines {
    let mut kind = vec![ByteKind::Prose; source.len()];
    if let Ok(doc) = hick_lang::parse(source) {
        if let Some(root) = &doc.root_tag {
            mark_tag_spans(root, &mut kind);
        }
        mark_nodes(&doc.nodes, false, &mut kind);
    }
    let mut out = DocumentLines::default();
    for (start, end) in line_spans(source) {
        out.lines += 1;
        let bytes = &source.as_bytes()[start..end];
        let kinds = &kind[start..end];
        let non_ws = || {
            bytes
                .iter()
                .zip(kinds)
                .filter(|(b, _)| !b.is_ascii_whitespace())
        };
        // Fragment first, so a blank line inside a `hick:file` is a line of
        // that file rather than a blank line of prose. Then markup: a line
        // that is nothing but a tag, and a whitespace-only line that falls
        // inside one, are both the document's structure and not its prose —
        // but a prose line with an inline `<hick:val/>` in it is prose, which
        // is why this asks whether EVERY non-blank byte is markup.
        let all_markup = match non_ws().next() {
            Some(_) => non_ws().all(|(_, k)| *k == ByteKind::Markup),
            None => kinds.contains(&ByteKind::Markup),
        };
        if kinds.contains(&ByteKind::Fragment) {
            out.fragment += 1;
        } else if all_markup {
            out.tag += 1;
        } else {
            out.prose += 1;
        }
    }
    out
}

fn mark_nodes(nodes: &[HickNode], in_bytes: bool, kind: &mut [ByteKind]) {
    for node in nodes {
        match node {
            HickNode::Text(_, Some(span)) if span.file_id.is_none() => {
                let what = match in_bytes {
                    true => ByteKind::Fragment,
                    false => ByteKind::Prose,
                };
                fill(kind, span.start, span.end, what);
            }
            HickNode::Text(..) => {}
            HickNode::Tag(tag) => {
                mark_tag_spans(tag, kind);
                mark_nodes(&tag.children, holds_bytes(&tag.name), kind);
            }
        }
    }
}

fn mark_tag_spans(tag: &HickTag, kind: &mut [ByteKind]) {
    for span in [tag.source_span, tag.close_span].into_iter().flatten() {
        if span.file_id.is_none() {
            fill(kind, span.start, span.end, ByteKind::Markup);
        }
    }
}

fn fill(kind: &mut [ByteKind], start: usize, end: usize, what: ByteKind) {
    let start = start.min(kind.len());
    let end = end.min(kind.len());
    for byte in &mut kind[start..end] {
        *byte = what;
    }
}

// ---------------------------------------------------------------------------
// Output volumes: the files a cell wrote
// ---------------------------------------------------------------------------

/// One `<hick:volume output="…">` and the cells that mount it.
#[derive(Debug, Clone)]
pub struct VolumeDecl {
    pub name: String,
    /// The `output=` prefix, relative to the document's folder.
    pub output: String,
    /// `container=` ids of the `hick:exec` cells that mount it.
    pub cells: Vec<String>,
    /// Source lines of those cells.
    pub lines: Vec<usize>,
}

/// Every output volume this document declares, with its writers.
///
/// The same reading `hickory_cli::generated_file` does, taken from the
/// document rather than from a path, because the census asks the question the
/// other way round: not "who wrote this file" but "what did this document's
/// cells write".
pub fn output_volumes(run: &DocRun) -> Vec<VolumeDecl> {
    let tags = run.doc.all_tags();
    let mut out = Vec::new();
    for tag in &tags {
        if tag.name != "volume" {
            continue;
        }
        let (Some(name), Some(output)) = (tag.get_attribute("name"), tag.get_attribute("output"))
        else {
            continue;
        };
        let mut cells = Vec::new();
        let mut lines = Vec::new();
        for exec in &tags {
            if exec.name != "exec" {
                continue;
            }
            let mounts = exec.get_attribute("mount").unwrap_or_default();
            let mounted = mounts
                .split(',')
                .filter_map(|entry| entry.trim().split(':').next())
                .any(|v| v == name);
            if !mounted {
                continue;
            }
            if let Some(container) = exec.get_attribute("container") {
                cells.push(container.to_string());
            }
            lines.push(exec.source_line);
        }
        cells.dedup();
        out.push(VolumeDecl {
            name: name.to_string(),
            output: output.trim_end_matches('/').to_string(),
            cells,
            lines,
        });
    }
    out
}

/// Count every file under a volume's `output=` prefix that no document
/// tangles, as that volume's cells' output. Returns how many it claimed.
fn claim_volume(
    root: &Path,
    doc_dir: &Path,
    doc_rel: &str,
    volume: &VolumeDecl,
    claimed: &mut BTreeSet<String>,
    files: &mut Vec<FileCensus>,
) -> usize {
    let dir = match volume.output.is_empty() || volume.output == "." {
        true => doc_dir.to_path_buf(),
        false => doc_dir.join(&volume.output),
    };
    if !dir.is_dir() {
        return 0;
    }
    let mut found = 0usize;
    for entry in ignore::WalkBuilder::new(&dir)
        .hidden(false)
        .build()
        .flatten()
    {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        if skip_path(path) {
            continue;
        }
        let rel = relative_to(root, path);
        if !claimed.insert(rel.clone()) {
            continue;
        }
        found += 1;
        let lines = count_lines(path);
        files.push(FileCensus {
            path: rel,
            document: doc_rel.to_string(),
            kind: FileKind::CellOutput,
            totals: LineTotals {
                lines,
                authored: 0,
                generated: 0,
                cell_output: lines,
            },
            volume: Some(volume.name.clone()),
            cells: volume.cells.clone(),
            cell_lines: volume.lines.clone(),
        });
    }
    found
}

/// Lines of a file on disk, counted the way [`line_spans`] counts them and
/// without assuming the bytes are UTF-8.
fn count_lines(path: &Path) -> usize {
    let Ok(bytes) = std::fs::read(path) else {
        return 0;
    };
    let newlines = bytes.iter().filter(|b| **b == b'\n').count();
    match bytes.last() {
        Some(b'\n') | None => newlines,
        Some(_) => newlines + 1,
    }
}

/// `.hick-cache/` bookkeeping and the documents themselves are never
/// produced files.
fn skip_path(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == ".hick-cache" || c.as_os_str() == ".git")
        || path.extension().and_then(|e| e.to_str()) == Some("hick")
}

/// Tracked, non-ignored files under the root that nothing produced.
fn unproduced_count(root: &Path, claimed: &BTreeSet<String>) -> usize {
    let mut count = 0usize;
    for entry in ignore::WalkBuilder::new(root)
        .hidden(false)
        .build()
        .flatten()
    {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if skip_path(entry.path()) {
            continue;
        }
        if !claimed.contains(&relative_to(root, entry.path())) {
            count += 1;
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Who wrote one generated file (`hick lineage`'s answer for a volume)
// ---------------------------------------------------------------------------

/// A file a CELL wrote into an output volume, rather than one the document
/// weaves.
///
/// These have no byte-precise lineage and never will: their bytes are a
/// program's output, and no span of any document produced them. But "no
/// output named that" — which is what lineage said before this existed — is
/// the wrong answer to a real question. A generated file has a provenance,
/// just a coarser one: **this cell, in this document, wrote all of it.**
///
/// That distinction is the point. A reader looking at a source file needs to
/// know which of three things it is: text a person typed (`literal`), bytes a
/// foreign tool wrote and this document owns (`ingested`), or output a cell
/// produced on the way past (this). The first two are byte-precise and
/// editable; this one is neither, and saying so plainly is what stops someone
/// editing a file that regenerates over them.
pub struct GeneratedFile {
    /// The output volume the file landed in.
    pub volume: String,
    /// Containers whose cells wrote into that volume, in document order.
    pub cells: Vec<String>,
    /// Source lines of the `hick:exec` tags that mounted it.
    pub lines: Vec<usize>,
}

/// Whether `path` is written by one of this document's output volumes.
///
/// Matched on the volume's `output=` prefix, which is where the volume's
/// contents land — so `apiout` declared `output="src/Api"` claims
/// `src/Api/Endpoints.g.cs`.
pub fn generated_file(run: &DocRun, path: &str) -> Option<GeneratedFile> {
    let normalized = path.replace('\\', "/");
    let mut best: Option<(usize, String, String)> = None; // (prefix len, volume, output)
    for tag in run.doc.tags() {
        if tag.name != "volume" {
            continue;
        }
        let (Some(name), Some(output)) = (tag.get_attribute("name"), tag.get_attribute("output"))
        else {
            continue;
        };
        let prefix = output.trim_end_matches('/');
        // The longest matching declaration wins, so a narrow volume nested
        // inside a wide one is reported as the one that actually wrote it.
        if (normalized == prefix || normalized.starts_with(&format!("{prefix}/")))
            && best.as_ref().is_none_or(|(len, _, _)| prefix.len() > *len)
        {
            best = Some((prefix.len(), name.to_string(), output.to_string()));
        }
    }
    let (_, volume, _) = best?;

    let mut cells = Vec::new();
    let mut lines = Vec::new();
    for tag in run.doc.tags() {
        if tag.name != "exec" {
            continue;
        }
        let mounts = tag.get_attribute("mount").unwrap_or_default();
        let mounts_it = mounts
            .split(',')
            .filter_map(|entry| entry.trim().split(':').next())
            .any(|v| v == volume);
        if mounts_it {
            if let Some(container) = tag.get_attribute("container") {
                cells.push(container.to_string());
            }
            lines.push(tag.source_line);
        }
    }
    Some(GeneratedFile {
        volume,
        cells,
        lines,
    })
}
