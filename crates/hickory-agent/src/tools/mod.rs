//! The agent's first-class document edit tool set.
//!
//! Doctrine (see [`crate::protocol::TOOLS_SYSTEM_PROMPT`]): read both
//! surfaces; write CODE through the OUTPUT (byte-exact via lineage);
//! escalate to the DOCUMENT for structural/prose work and whenever lineage
//! refuses; verify via full execution before finishing.
//!
//! Everything runs locally through the `hick-literate` library — no HTTP.
//! The [`EditSession`] is single-writer over the document: after every
//! successful edit it re-weaves immediately and refreshes hashes and
//! provenance, so stale provenance (a 409-class failure) is impossible
//! inside a session. External mutation of the document file is detected by
//! content comparison before every edit and absorbed by one automatic
//! re-weave; anchors that no longer resolve after that produce a structured
//! error, never a silently misapplied edit.

pub mod hashline;

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use hick_exec::node::FileContent;
use hickory_executor::Executor;
use hickory_lineage::{LineageError, OutputEdit, Provenance, apply_source_edits, map_edits};

use crate::protocol::ToolInvocation;
use hashline::{Anchor, LineIndex, ResolvedAnchor, parse_anchor, resolve_anchor};

/// The result of one tool invocation, returned to the model as a
/// `<hick:tool-result>` observation and logged in the session file.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    /// Tool name (echoed on the result element).
    pub name: String,
    /// Whether the tool did what was asked. Lineage refusals are `false`
    /// but carry routing, not failure.
    pub ok: bool,
    /// The observation text.
    pub text: String,
    /// What this call put IN FRONT OF the model: every file (and line range)
    /// a read tool returned. Recorded into the session as `<hick:read>`, so
    /// "what was in context when these lines were written" is derivable from
    /// the conversation record alone — the model never asserts it.
    pub reads: Vec<ContextRead>,
    /// The lines an edit tool wrote, in the file it wrote them to, as they
    /// stand after the edit. Recorded as `<hick:wrote>`; everything read
    /// earlier in the same session is that write's context.
    pub wrote: Option<Wrote>,
}

/// One thing a tool showed the model: a file, at a content hash and (when
/// the file is in a git repository) a commit, over a 1-based inclusive line
/// range. Derived by the tool, never declared by the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRead {
    /// The path as the session knows it — the document's name for the
    /// primary, the upstream's path, an output's path, or the path given to
    /// `read_file` resolved relative to the primary document.
    pub path: String,
    /// `git rev-parse HEAD` of the repository containing the file, if any.
    /// The working tree may differ from HEAD; `sha256` is the truth about
    /// the bytes, `commit` is where to look for them later.
    pub commit: Option<String>,
    /// SHA-256 of the whole file's bytes as read.
    pub sha256: String,
    /// First line shown, 1-based.
    pub first_line: usize,
    /// Last line shown, 1-based, inclusive.
    pub last_line: usize,
}

/// The lines an edit left in a file, 1-based inclusive, with their content
/// hashes — the anchors by which a later reader finds them again after the
/// file has moved on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrote {
    pub file: String,
    pub first_line: usize,
    pub last_line: usize,
    /// 4-hex hashline hashes of those lines, in order.
    pub hashes: Vec<String>,
}

impl ToolOutcome {
    fn ok(name: &str, text: String) -> Self {
        Self {
            name: name.to_string(),
            ok: true,
            text,
            reads: Vec::new(),
            wrote: None,
        }
    }

    fn err(name: &str, text: String) -> Self {
        Self {
            name: name.to_string(),
            ok: false,
            text,
            reads: Vec::new(),
            wrote: None,
        }
    }

    fn with_read(mut self, read: ContextRead) -> Self {
        self.reads.push(read);
        self
    }

    fn with_wrote(mut self, wrote: Option<Wrote>) -> Self {
        self.wrote = wrote;
        self
    }

    /// A bare outcome for callers outside this module (the loop's "no
    /// document in this session" refusal).
    pub fn refused(name: &str, text: String) -> Self {
        Self::err(name, text)
    }
}

/// SHA-256 of `bytes`, lowercase hex.
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// `git rev-parse HEAD` for the repository containing `path`, if there is
/// one and git is installed. Anything else is `None`: outside a repository
/// there is no commit to name, and the read is still recorded by its hash.
fn head_commit_for(path: &Path) -> Option<String> {
    let dir = if path.is_dir() {
        path
    } else {
        match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        }
    };
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// The lines of `new` that differ from `old`: the first and last (1-based,
/// inclusive) line indices into `new` outside the common prefix and suffix,
/// with their hashes. `None` when nothing changed or the change was a pure
/// deletion (no lines of `new` were written).
fn changed_lines(old: &str, new: &str, file: &str) -> Option<Wrote> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let mut prefix = 0;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < a.len() - prefix
        && suffix < b.len() - prefix
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let last = b.len().checked_sub(suffix)?;
    if last <= prefix {
        return None;
    }
    Some(Wrote {
        file: file.to_string(),
        first_line: prefix + 1,
        last_line: last,
        hashes: b[prefix..last]
            .iter()
            .map(|l| hashline::line_hash(l))
            .collect(),
    })
}

/// Weave state of one pipeline pass: text outputs plus their provenance.
struct WeaveState {
    files: HashMap<String, String>,
    provenance: HashMap<String, Vec<Provenance>>,
}

/// Single-writer edit session over one primary hick document.
///
/// Owns the current document source, the latest weave (output files +
/// byte-precise provenance), and the content-hash line index both surfaces
/// are rendered from. Every successful edit re-weaves before returning, so
/// hashes and provenance handed to the model always describe the current
/// state.
pub struct EditSession {
    doc_path: PathBuf,
    /// Documents the primary reaches through `hick:upstream`, nearest first.
    ///
    /// A session scoped to ONE document cannot do lifecycle work: the fix for
    /// "the requirements are wrong" is usually a decision recorded in a
    /// meeting note two hops up, and an agent that can only edit the document
    /// in front of it will instead paper over the disagreement where it found
    /// it — writing the contradiction into the very chain built to prevent
    /// contradictions. The editable set is the pipeline closure, which the
    /// documents declare themselves; it is not a flag anyone has to remember
    /// to pass.
    upstream: std::collections::BTreeMap<PathBuf, String>,
    /// The name the document is known by in pipeline sources and
    /// provenance `doc_path` fields.
    doc_name: String,
    params: Vec<(String, String)>,
    source: String,
    weave: WeaveState,
    /// Where `read_file` may look: the git repository containing the
    /// document, or its directory when there is none. Nothing above it is
    /// readable — the tool surface is the agent's only view of the project,
    /// and that view is the project.
    root: PathBuf,
}

impl EditSession {
    /// Open a session on `doc_path`: read, parse, and weave (no execution).
    pub async fn open(doc_path: &Path, params: &[(String, String)]) -> Result<Self> {
        let source = std::fs::read_to_string(doc_path)
            .with_context(|| format!("failed to read {}", doc_path.display()))?;
        let doc_name = doc_path.display().to_string();
        let weave = weave_source(doc_path, &doc_name, &source, params).await?;
        let upstream = upstream_closure(doc_path);
        let root = project_root(doc_path);
        Ok(Self {
            doc_path: doc_path.to_path_buf(),
            upstream,
            doc_name,
            params: params.to_vec(),
            source,
            weave,
            root,
        })
    }

    /// A context record for showing the whole of `source`, known as `path`.
    fn read_of(&self, path: &Path, name: &str, source: &str) -> ContextRead {
        let lines = source.lines().count().max(1);
        ContextRead {
            path: name.to_string(),
            commit: head_commit_for(path),
            sha256: sha256_hex(source.as_bytes()),
            first_line: 1,
            last_line: lines,
        }
    }

    /// The session's primary document path.
    pub fn doc_path(&self) -> &Path {
        &self.doc_path
    }

    /// Detect external mutation of the document file (content mismatch) and
    /// absorb it with one automatic re-weave. Returns whether a re-sync
    /// happened. A broken on-disk document is an error.
    async fn sync_with_disk(&mut self) -> Result<bool, String> {
        let disk = std::fs::read_to_string(&self.doc_path)
            .map_err(|e| format!("cannot read {}: {e}", self.doc_path.display()))?;
        if disk == self.source {
            return Ok(false);
        }
        let weave = weave_source(&self.doc_path, &self.doc_name, &disk, &self.params)
            .await
            .map_err(|e| {
                format!(
                    "the document changed on disk and the new content does not weave: {e}. \
                     Fix {} before editing.",
                    self.doc_path.display()
                )
            })?;
        self.source = disk;
        self.weave = weave;
        Ok(true)
    }

    /// Re-weave from the in-memory source, refreshing files and provenance.
    async fn reweave(&mut self) -> Result<()> {
        self.weave =
            weave_source(&self.doc_path, &self.doc_name, &self.source, &self.params).await?;
        Ok(())
    }

    fn output_names(&self) -> String {
        let mut names: Vec<&str> = self.weave.files.keys().map(String::as_str).collect();
        names.sort_unstable();
        if names.is_empty() {
            "(none)".to_string()
        } else {
            names.join(", ")
        }
    }

    // -- read_doc ----------------------------------------------------------

    fn read_doc(&self, inv: &ToolInvocation) -> ToolOutcome {
        let Some((name, source)) = self.resolve_target(inv.arg("doc")) else {
            return ToolOutcome::err("read_doc", self.unknown_doc(inv.arg("doc").unwrap_or("")));
        };
        let path = self.path_of_target(&name);
        let read = self.read_of(&path, &name, &source);
        let index = LineIndex::new(&source);
        let mut text = format!("doc: {name}\n{}", index.render());
        if self.upstream.is_empty() {
            return ToolOutcome::ok("read_doc", text).with_read(read);
        }
        // Name the rest of the chain, or the agent has no way to learn that a
        // decision it needs to change lives one document up.
        text.push_str(&format!(
            "\nupstream of this session (readable and editable by passing \
             <hick:arg name=\"doc\">NAME</hick:arg>): {}\n",
            self.upstream_names()
        ));
        ToolOutcome::ok("read_doc", text).with_read(read)
    }

    /// The on-disk path of a resolved target name (the primary or an
    /// upstream), for hashing and for finding its repository.
    fn path_of_target(&self, name: &str) -> PathBuf {
        if name == self.doc_name {
            return self.doc_path.clone();
        }
        self.upstream
            .keys()
            .find(|p| p.display().to_string() == name)
            .cloned()
            .unwrap_or_else(|| PathBuf::from(name))
    }

    // -- read_file ---------------------------------------------------------

    /// Show the model any file of the project, read-only, hashline-rendered,
    /// optionally one line range — and record that it was shown.
    ///
    /// This is the agent's only window onto files that are not documents:
    /// a data export, a config, a source file the document tangles from. Its
    /// scripts run in a scratch workspace that cannot see the project, on
    /// purpose; without this tool the agent, told a file exists, looks,
    /// finds nothing, and makes one up. With it the read is real AND on the
    /// record, which is what lets context provenance say "this export, at
    /// this hash, was in front of the model when it wrote those findings".
    fn read_file(&self, inv: &ToolInvocation) -> ToolOutcome {
        const NAME: &str = "read_file";
        const MAX_BYTES: usize = 1 << 20;
        let Some(rel) = inv.arg("path") else {
            return ToolOutcome::err(
                NAME,
                "read_file needs a path argument, relative to the document's directory".into(),
            );
        };
        let base = self.doc_path.parent().unwrap_or(Path::new("."));
        let candidate = base.join(rel);
        let canonical = match candidate.canonicalize() {
            Ok(c) => c,
            Err(e) => {
                return ToolOutcome::err(
                    NAME,
                    format!(
                        "cannot read '{rel}' (resolved against {}): {e}",
                        base.display()
                    ),
                );
            }
        };
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        if !canonical.starts_with(&root) {
            return ToolOutcome::err(
                NAME,
                format!(
                    "'{rel}' is outside the project ({}); read_file reads project files only",
                    root.display()
                ),
            );
        }
        if canonical.is_dir() {
            let mut names: Vec<String> = std::fs::read_dir(&canonical)
                .map(|rd| {
                    rd.filter_map(Result::ok)
                        .map(|e| {
                            let n = e.file_name().to_string_lossy().to_string();
                            if e.path().is_dir() {
                                format!("{n}/")
                            } else {
                                n
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            names.sort();
            return ToolOutcome::ok(
                NAME,
                format!("'{rel}' is a directory; it holds:\n{}", names.join("\n")),
            );
        }
        let bytes = match std::fs::read(&canonical) {
            Ok(b) => b,
            Err(e) => return ToolOutcome::err(NAME, format!("cannot read '{rel}': {e}")),
        };
        if bytes.len() > MAX_BYTES {
            return ToolOutcome::err(
                NAME,
                format!(
                    "'{rel}' is {} bytes; read_file shows at most {MAX_BYTES}. Read a range \
                     with from/to, or let a cell process the file",
                    bytes.len()
                ),
            );
        }
        let Ok(text) = String::from_utf8(bytes.clone()) else {
            return ToolOutcome::err(
                NAME,
                format!(
                    "'{rel}' is not UTF-8 text; a cell can process it, read_file cannot show it"
                ),
            );
        };
        let index = LineIndex::new(&text);
        let total = index.lines.len();
        let parse_line = |key: &str, default: usize| -> Result<usize, String> {
            match inv.arg(key) {
                None => Ok(default),
                Some(v) => v
                    .parse::<usize>()
                    .ok()
                    .filter(|n| *n >= 1)
                    .ok_or_else(|| format!("{key}='{v}' is not a positive line number")),
            }
        };
        let (first, last) = match (parse_line("from", 1), parse_line("to", total)) {
            (Ok(a), Ok(b)) => (a.min(total.max(1)), b.min(total.max(1))),
            (Err(e), _) | (_, Err(e)) => return ToolOutcome::err(NAME, e),
        };
        if last < first {
            return ToolOutcome::err(NAME, format!("to={last} is before from={first}"));
        }
        let shown = index.render_range(first - 1, last - 1);
        let read = ContextRead {
            path: rel.to_string(),
            commit: head_commit_for(&canonical),
            sha256: sha256_hex(&bytes),
            first_line: first,
            last_line: last,
        };
        let header = if first == 1 && last == total {
            format!("file: {rel} ({total} lines)\n")
        } else {
            format!("file: {rel} lines {first}-{last} of {total}\n")
        };
        ToolOutcome::ok(NAME, format!("{header}{shown}")).with_read(read)
    }

    /// Apply an edit to an upstream document and re-weave the primary.
    ///
    /// The order matters: parse, then write, then re-weave. A re-weave that
    /// fails after the write would leave the chain edited and the session
    /// describing a weave that no longer holds, so the write is rolled back
    /// and the outcome says nothing changed.
    async fn edit_upstream(&mut self, inv: &ToolInvocation, doc: &str) -> ToolOutcome {
        const NAME: &str = "edit_doc";
        let Some((path, source)) = self
            .upstream
            .iter()
            .find(|(p, _)| Self::path_matches(p, doc))
            .map(|(p, s)| (p.clone(), s.clone()))
        else {
            return ToolOutcome::err(NAME, self.unknown_doc(doc));
        };

        let index = LineIndex::new(&source);
        let edit = match resolve_edit(&index, inv, false) {
            Ok(e) => e,
            Err(text) => return ToolOutcome::err(NAME, text),
        };
        let mut new_source = source.clone();
        new_source.replace_range(edit.start..edit.end, &edit.text);

        if let Err(e) = hick_lang::parse(&new_source) {
            return ToolOutcome::err(
                NAME,
                format!(
                    "that edit would break {}: parse error: {e}; nothing was changed",
                    path.display()
                ),
            );
        }
        if let Err(e) = std::fs::write(&path, &new_source) {
            return ToolOutcome::err(NAME, format!("could not write {}: {e}", path.display()));
        }
        if let Err(e) = self.reweave().await {
            let _ = std::fs::write(&path, &source);
            let _ = self.reweave().await;
            return ToolOutcome::err(
                NAME,
                format!(
                    "that edit parses but breaks the weave of {} ({e}); {} was restored and \
                     nothing changed",
                    self.doc_name,
                    path.display()
                ),
            );
        }
        self.upstream.insert(path.clone(), new_source.clone());

        let new_index = LineIndex::new(&new_source);
        let wrote = changed_lines(&source, &new_source, &path.display().to_string());
        ToolOutcome::ok(
            NAME,
            format!(
                "edited {} and re-wove {}.\n{}\noutputs now: {}",
                path.display(),
                self.doc_name,
                new_index.render(),
                self.output_names()
            ),
        )
        .with_wrote(wrote)
    }

    /// The upstream documents, nearest first, as a display list.
    fn upstream_names(&self) -> String {
        self.upstream
            .keys()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Resolve a `doc` argument to (display name, source). `None` selects the
    /// primary document, which keeps every existing single-document call site
    /// working unchanged.
    fn resolve_target(&self, doc: Option<&str>) -> Option<(String, String)> {
        let Some(doc) = doc else {
            return Some((self.doc_name.clone(), self.source.clone()));
        };
        if doc == self.doc_name || Path::new(doc) == self.doc_path {
            return Some((self.doc_name.clone(), self.source.clone()));
        }
        self.upstream
            .iter()
            .find(|(p, _)| Self::path_matches(p, doc))
            .map(|(p, s)| (p.display().to_string(), s.clone()))
    }

    /// Match on the full path or the file name, so an agent can say
    /// `billing.md` without reconstructing the relative path.
    fn path_matches(path: &Path, needle: &str) -> bool {
        path == Path::new(needle)
            || path.ends_with(needle)
            || path.file_name().is_some_and(|f| f == needle)
    }

    fn unknown_doc(&self, asked: &str) -> String {
        format!(
            "no document '{asked}' in this session. The primary document is {}{}",
            self.doc_name,
            if self.upstream.is_empty() {
                String::new()
            } else {
                format!("; upstream: {}", self.upstream_names())
            }
        )
    }

    // -- read_output -------------------------------------------------------

    fn read_output(&self, inv: &ToolInvocation) -> ToolOutcome {
        let Some(path) = inv.arg("path") else {
            return ToolOutcome::err(
                "read_output",
                format!(
                    "read_output needs a path argument — available outputs: {}",
                    self.output_names()
                ),
            );
        };
        let Some(content) = self.weave.files.get(path) else {
            return ToolOutcome::err(
                "read_output",
                format!(
                    "no output named '{path}' — available outputs: {}",
                    self.output_names()
                ),
            );
        };
        let index = LineIndex::new(content);
        let mut text = format!("output: {path}\n{}", index.render());
        let with_lineage = inv
            .arg("with_lineage")
            .is_some_and(|v| v == "true" || v == "1");
        if with_lineage {
            let prov = self.weave.provenance.get(path).cloned().unwrap_or_default();
            text.push_str("\nlineage:\n");
            text.push_str(&self.lineage_summary(&prov, &index));
        }
        let lines = content.lines().count().max(1);
        ToolOutcome::ok("read_output", text).with_read(ContextRead {
            path: path.to_string(),
            commit: None, // an output is derived; its document's commit is the record
            sha256: sha256_hex(content.as_bytes()),
            first_line: 1,
            last_line: lines,
        })
    }

    /// Compact per-range lineage annotations: kind, editability, and the
    /// source document span (as doc line numbers).
    fn lineage_summary(&self, prov: &[Provenance], out_index: &LineIndex) -> String {
        let doc_index = LineIndex::new(&self.source);
        let mut text = String::new();
        for p in prov {
            if p.start == p.end {
                continue;
            }
            let (first, last) = byte_range_to_lines(out_index, p.start, p.end);
            let kind = origin_kind(p);
            let line_part = if first == last {
                format!("line {}", first + 1)
            } else {
                format!("lines {}-{}", first + 1, last + 1)
            };
            match p.origin.source() {
                Some((doc, s, e)) => {
                    let (df, dl) = byte_range_to_lines(&doc_index, s, e);
                    let _ = writeln!(
                        text,
                        "  {line_part}: {kind} — editable via edit_output (from {doc} lines {}-{})",
                        df + 1,
                        dl + 1
                    );
                }
                None => {
                    let loc = p
                        .origin
                        .location()
                        .map(|(doc, s, e)| {
                            let (df, dl) = byte_range_to_lines(&doc_index, s, e);
                            format!(" (produced by {doc} lines {}-{})", df + 1, dl + 1)
                        })
                        .unwrap_or_default();
                    let _ = writeln!(
                        text,
                        "  {line_part}: {kind} — NOT editable via edit_output; use edit_doc{loc}"
                    );
                }
            }
        }
        text
    }

    // -- edit_output -------------------------------------------------------

    async fn edit_output(&mut self, inv: &ToolInvocation) -> ToolOutcome {
        const NAME: &str = "edit_output";
        let resynced = match self.sync_with_disk().await {
            Ok(r) => r,
            Err(e) => return ToolOutcome::err(NAME, e),
        };
        let Some(path) = inv.arg("path").map(str::to_string) else {
            return ToolOutcome::err(
                NAME,
                format!(
                    "edit_output needs a path argument — available outputs: {}",
                    self.output_names()
                ),
            );
        };
        let Some(content) = self.weave.files.get(&path).cloned() else {
            return ToolOutcome::err(
                NAME,
                format!(
                    "no output named '{path}' — available outputs: {}",
                    self.output_names()
                ),
            );
        };

        let index = LineIndex::new(&content);
        let edit = match resolve_edit(&index, inv, resynced) {
            Ok(e) => e,
            Err(text) => return ToolOutcome::err(NAME, text),
        };

        let prov = self
            .weave
            .provenance
            .get(&path)
            .cloned()
            .unwrap_or_default();
        let output_edit = OutputEdit {
            start: edit.start,
            end: edit.end,
            text: edit.text.clone(),
        };
        let source_edits = match map_edits(&content, std::slice::from_ref(&output_edit), &prov) {
            Ok(edits) => edits,
            Err(err) => return ToolOutcome::err(NAME, self.route_refusal(&err, &prov)),
        };

        // A refusal with routing, not corruption: lineage can name an
        // included/upstream document as the edit's destination, but this
        // session holds (and re-weaves) only the primary document. The
        // owning file is reachable through the same tool set.
        if let Some(foreign) = source_edits.iter().find(|e| e.doc_path != self.doc_name) {
            // Show the owning lines, so the next call needs no search: the
            // agent obeying this message should land directly on the
            // fragment (e.g. the `<hick:copy id=…>` block) it must edit.
            let excerpt = foreign_excerpt(&foreign.doc_path, foreign.span)
                .map(|text| format!("\nThe owning text is:\n{text}\n"))
                .unwrap_or_default();
            return ToolOutcome::err(
                NAME,
                format!(
                    "this range comes from {}, a document included by {} — the edit \
                     belongs there.{excerpt}\
                     Read it with read_doc (doc=\"{}\"), make the change with edit_doc \
                     against that document, then verify.",
                    foreign.doc_path, self.doc_name, foreign.doc_path
                ),
            );
        }

        let mut sources = HashMap::new();
        sources.insert(self.doc_name.clone(), self.source.clone());
        let updated = match apply_source_edits(&sources, &source_edits) {
            Ok(u) => u,
            Err(err) => return ToolOutcome::err(NAME, self.route_refusal(&err, &prov)),
        };
        let Some(new_source) = updated.get(&self.doc_name).cloned() else {
            return ToolOutcome::err(NAME, "edit mapped to no change in the document".into());
        };

        if let Err(e) = hick_lang::parse(&new_source) {
            return ToolOutcome::err(
                NAME,
                format!(
                    "the mapped document edit would break the document (parse error: {e}); \
                     nothing was changed"
                ),
            );
        }

        let old_source = std::mem::replace(&mut self.source, new_source);
        if let Err(e) = self.reweave().await {
            self.source = old_source;
            // Restore the previous weave for the rolled-back source.
            let _ = self.reweave().await;
            return ToolOutcome::err(
                NAME,
                format!("the edit broke the weave ({e}); nothing was changed"),
            );
        }

        // The lineage Ok ⇒ byte-exact guarantee makes this a tautology, but
        // verify it anyway: the re-woven output must contain the edit.
        let new_content = self.weave.files.get(&path).cloned().unwrap_or_default();
        if !edit.text.is_empty() && !new_content.contains(&edit.text) {
            self.source = old_source;
            let _ = self.reweave().await;
            return ToolOutcome::err(
                NAME,
                "re-weave did not reproduce the edit byte-for-byte; nothing was changed \
                 (this is a bug — please report it)"
                    .into(),
            );
        }

        if let Err(e) = std::fs::write(&self.doc_path, &self.source) {
            return ToolOutcome::err(
                NAME,
                format!(
                    "edited in memory but could not write {}: {e}",
                    self.doc_path.display()
                ),
            );
        }

        let new_index = LineIndex::new(&new_content);
        let region = edited_region(&new_index, edit.first_line, &edit.text);
        // The write that matters for context is the DOCUMENT's: the output
        // is re-derived from it, and provenance already covers that hop.
        let wrote = changed_lines(&old_source, &self.source, &self.doc_name);
        ToolOutcome::ok(
            NAME,
            format!(
                "edited {path}; the document was updated through lineage and re-woven.\n\
                 edited region (fresh hashes):\n{region}"
            ),
        )
        .with_wrote(wrote)
    }

    /// Turn a lineage refusal into the routing signal: explain why, name
    /// the document location to edit, and include fresh doc hashlines there.
    fn route_refusal(&self, err: &LineageError, prov: &[Provenance]) -> String {
        let doc_index = LineIndex::new(&self.source);
        match err {
            LineageError::SyntheticOverlap { start, end } => {
                // Name the producing document location if any overlapping or
                // neighboring entry knows one.
                let nearby = prov
                    .iter()
                    .filter(|p| p.start < *end && p.end > *start)
                    .find_map(|p| p.origin.location())
                    .or_else(|| {
                        prov.iter()
                            .rev()
                            .find(|p| p.end <= *start)
                            .and_then(|p| p.origin.location())
                    });
                let mut msg = format!(
                    "REFUSED — routing, not failure: output bytes {start}..{end} are synthetic \
                     (produced by the pipeline: exec output, separators, or variable values) and \
                     do not map byte-for-byte onto the document."
                );
                match nearby {
                    Some((doc, s, e)) => {
                        let (a, b) = byte_range_to_lines(&doc_index, s, e);
                        let _ = write!(
                            msg,
                            " Edit the DOCUMENT instead: use edit_doc on {doc} lines {}-{}:\n{}",
                            a + 1,
                            b + 1,
                            doc_index.render_range(a, b)
                        );
                    }
                    None => {
                        let _ = write!(
                            msg,
                            " Edit the DOCUMENT instead: use read_doc + edit_doc on {} to change \
                             what produces these bytes.",
                            self.doc_name
                        );
                    }
                }
                msg
            }
            LineageError::Conflict(detail) => {
                let mut msg = format!(
                    "REFUSED — routing, not failure: {detail}.\nEditing this occurrence cannot \
                     be reproduced byte-exactly on the next weave."
                );
                // Extract "<doc> bytes S..E" from map_edits' duplicate-paste
                // detail to point at the shared source block.
                //
                // That location is the PASTE SITE, which is only the source
                // block when the fragment lives in the same document. Follow
                // the selector to where the fragment is actually declared —
                // otherwise a chain sends the reader to the `<hick:paste>` tag
                // in the wrong file, and an agent that obeys edits the tag
                // that performs the paste.
                if let Some((doc, s, e)) = parse_conflict_location(detail) {
                    if let Some(home) = self.fragment_home(&doc, s, e) {
                        let _ = write!(
                            msg,
                            "\nThose bytes are pasted in from another document. Edit the \
                             fragment where it is DECLARED: use edit_doc on {}, from line {}:\n{}",
                            home.doc,
                            home.line + 1,
                            home.excerpt
                        );
                        return msg;
                    }
                    let (a, b) = byte_range_to_lines(&doc_index, s, e);
                    let _ = write!(
                        msg,
                        "\nEdit the shared source block instead: use edit_doc on {doc} lines \
                         {}-{}:\n{}",
                        a + 1,
                        b + 1,
                        doc_index.render_range(a, b)
                    );
                } else {
                    let _ = write!(
                        msg,
                        "\nUse read_doc + edit_doc on {} to change the shared source block.",
                        self.doc_name
                    );
                }
                msg
            }
            LineageError::InvalidEdit(detail) => format!("invalid edit: {detail}"),
        }
    }

    // -- edit_doc ----------------------------------------------------------

    async fn edit_doc(&mut self, inv: &ToolInvocation) -> ToolOutcome {
        const NAME: &str = "edit_doc";
        // An upstream edit takes a separate path: it is not the woven
        // document, so it has no provenance of its own here — but the primary
        // must be re-woven afterwards, because it pastes fragments from it.
        if let Some(doc) = inv.arg("doc")
            && !(doc == self.doc_name || Path::new(doc) == self.doc_path)
        {
            return self.edit_upstream(inv, doc).await;
        }
        let resynced = match self.sync_with_disk().await {
            Ok(r) => r,
            Err(e) => return ToolOutcome::err(NAME, e),
        };

        let index = LineIndex::new(&self.source);
        let edit = match resolve_edit(&index, inv, resynced) {
            Ok(e) => e,
            Err(text) => return ToolOutcome::err(NAME, text),
        };

        let mut new_source = self.source.clone();
        new_source.replace_range(edit.start..edit.end, &edit.text);

        if let Err(e) = hick_lang::parse(&new_source) {
            return ToolOutcome::err(
                NAME,
                format!(
                    "that edit would break the document (parse error: {e}); nothing was changed"
                ),
            );
        }

        let old_source = std::mem::replace(&mut self.source, new_source);
        if let Err(e) = self.reweave().await {
            self.source = old_source;
            let _ = self.reweave().await;
            return ToolOutcome::err(
                NAME,
                format!("that edit parses but breaks the weave ({e}); nothing was changed"),
            );
        }

        if let Err(e) = std::fs::write(&self.doc_path, &self.source) {
            return ToolOutcome::err(
                NAME,
                format!(
                    "edited in memory but could not write {}: {e}",
                    self.doc_path.display()
                ),
            );
        }

        let new_index = LineIndex::new(&self.source);
        let region = edited_region(&new_index, edit.first_line, &edit.text);
        let wrote = changed_lines(&old_source, &self.source, &self.doc_name);
        ToolOutcome::ok(
            NAME,
            format!(
                "edited {} and re-wove.\nedited region (fresh hashes):\n{region}",
                self.doc_name
            ),
        )
        .with_wrote(wrote)
    }

    // -- verify ------------------------------------------------------------

    /// Execute the document for real through the session's executor,
    /// evaluate every expectation, and write the produced output files next
    /// to the document (so a following `hick test` sees no drift).
    async fn verify(&mut self, executor: Arc<dyn Executor>) -> ToolOutcome {
        const NAME: &str = "verify";
        if let Err(e) = self.sync_with_disk().await {
            return ToolOutcome::err(NAME, e);
        }
        // Verify the WHOLE editable set, not just the primary.
        //
        // The agent can edit any document in the closure, so a verify scoped
        // to the primary is a feedback loop that lies: edit a decision two
        // hops up, run verify, get PASS — while that document's own outputs
        // were never re-woven and `hick test` fails on every one of them.
        // Observed exactly once in a live run, where the agent then reported
        // success in good faith. Verification scope must equal edit scope.
        let upstream_report = self.verify_upstream(executor.clone()).await;
        let project_dir = self
            .doc_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();
        let sources = vec![(self.doc_name.as_str(), self.source.as_str())];
        let config = hick_literate::PipelineConfig {
            working_dir: Some(project_dir.clone()),
            max_rounds: 1,
            on_exec: None,
            ..Default::default()
        };
        let result =
            match hick_literate::run_pipeline_live(&sources, &config, &self.params, None, executor)
                .await
            {
                Ok(r) => r,
                Err(e) => return ToolOutcome::err(NAME, format!("execution failed: {e}")),
            };

        let mut failures = Vec::new();
        for outcome in &result.expectations {
            if !outcome.passed {
                failures.push(format!(
                    "expectation at {}:{} (container {}) failed: {}",
                    outcome.doc,
                    outcome.line,
                    outcome.container.as_deref().unwrap_or("agent cell"),
                    outcome.detail
                ));
            }
        }

        // Write outputs to disk (like `hick run`), so document and
        // committed outputs stay in sync.
        let mut written = Vec::new();
        for (rel_path, content) in &result.files {
            let full = project_dir.join(rel_path);
            if let Some(parent) = full.parent()
                && !parent.as_os_str().is_empty()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                failures.push(format!("could not create {}: {e}", parent.display()));
                continue;
            }
            let write_result = match content {
                FileContent::Text(s) => std::fs::write(&full, s),
                FileContent::Binary(data) => match data.to_bytes() {
                    Ok(bytes) => std::fs::write(&full, bytes),
                    Err(e) => {
                        failures.push(format!("could not encode {rel_path}: {e}"));
                        continue;
                    }
                },
            };
            match write_result {
                Ok(()) => written.push(rel_path.clone()),
                Err(e) => failures.push(format!("could not write {}: {e}", full.display())),
            }
        }
        written.sort();

        // Refresh the session's weave state from the executed result: real
        // transcripts give the freshest provenance.
        let mut files = HashMap::new();
        for (path, content) in &result.files {
            if let FileContent::Text(s) = content {
                files.insert(path.clone(), s.clone());
            }
        }
        let provenance = result
            .provenance_maps
            .iter()
            .map(|(p, m)| (p.clone(), hickory_lineage::from_provenance_map(m)))
            .collect();
        self.weave = WeaveState { files, provenance };

        let checked = result.expectations.len();
        failures.extend(upstream_report.failures);
        if failures.is_empty() {
            ToolOutcome::ok(
                NAME,
                format!(
                    "PASS — executed the document: {checked} expectation(s) met; wrote {} output \
                     file(s): {}{}",
                    written.len(),
                    if written.is_empty() {
                        "(none)".to_string()
                    } else {
                        written.join(", ")
                    },
                    upstream_report.note
                ),
            )
        } else {
            ToolOutcome::err(
                NAME,
                format!(
                    "FAIL — {} problem(s):\n{}",
                    failures.len(),
                    failures.join("\n")
                ),
            )
        }
    }

    /// Re-run every upstream document and write its outputs.
    ///
    /// Each is executed on its own, in its own directory, because that is
    /// where its outputs are committed — an upstream document woven into the
    /// primary's directory would create files nobody checks and leave the
    /// real ones stale.
    async fn verify_upstream(&self, executor: Arc<dyn Executor>) -> UpstreamReport {
        let mut failures = Vec::new();
        let mut verified = 0usize;
        for (path, _) in self.upstream.iter() {
            let Ok(source) = std::fs::read_to_string(path) else {
                failures.push(format!("upstream {}: could not be read", path.display()));
                continue;
            };
            let name = path.display().to_string();
            let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let config = hick_literate::PipelineConfig {
                working_dir: Some(dir.clone()),
                max_rounds: 1,
                on_exec: None,
                ..Default::default()
            };
            let sources = vec![(name.as_str(), source.as_str())];
            let result = match hick_literate::run_pipeline_live(
                &sources,
                &config,
                &self.params,
                None,
                executor.clone(),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    failures.push(format!("upstream {name}: execution failed: {e}"));
                    continue;
                }
            };
            for outcome in &result.expectations {
                if !outcome.passed {
                    failures.push(format!(
                        "upstream {name}: expectation at line {} failed: {}",
                        outcome.line, outcome.detail
                    ));
                }
            }
            for (rel_path, content) in &result.files {
                let full = dir.join(rel_path);
                if let Some(parent) = full.parent()
                    && !parent.as_os_str().is_empty()
                    && let Err(e) = std::fs::create_dir_all(parent)
                {
                    failures.push(format!("upstream {name}: could not create {parent:?}: {e}"));
                    continue;
                }
                let write = match content {
                    FileContent::Text(t) => std::fs::write(&full, t),
                    FileContent::Binary(d) => match d.to_bytes() {
                        Ok(b) => std::fs::write(&full, b),
                        Err(e) => {
                            failures
                                .push(format!("upstream {name}: could not encode {rel_path}: {e}"));
                            continue;
                        }
                    },
                };
                if let Err(e) = write {
                    failures.push(format!(
                        "upstream {name}: could not write {}: {e}",
                        full.display()
                    ));
                }
            }
            verified += 1;
        }
        let note = if verified == 0 {
            String::new()
        } else {
            format!("; also re-wove {verified} upstream document(s)")
        };
        UpstreamReport { failures, note }
    }
}

/// Result of re-running the upstream documents during `verify`.
struct UpstreamReport {
    failures: Vec<String>,
    note: String,
}

/// Execute one tool invocation against the session. Failures are tool
/// results (observations), never process errors.
pub async fn execute_tool(
    session: &mut EditSession,
    executor: Arc<dyn Executor>,
    inv: &ToolInvocation,
) -> ToolOutcome {
    match inv.name.as_str() {
        "read_doc" => session.read_doc(inv),
        "read_output" => session.read_output(inv),
        "read_file" => session.read_file(inv),
        "edit_output" => session.edit_output(inv).await,
        "edit_doc" => session.edit_doc(inv).await,
        "verify" => session.verify(executor).await,
        other => ToolOutcome::err(
            other,
            format!(
                "unknown tool '{other}' — available: read_doc, read_output, read_file, \
                 edit_output, edit_doc, verify"
            ),
        ),
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// A resolved hashline edit: byte range, replacement text, and the line the
/// edited region starts on (for rendering the result).
struct ResolvedEdit {
    start: usize,
    end: usize,
    text: String,
    first_line: usize,
}

/// Resolve run/after/occurrence + `<hick:input>` into a byte edit.
fn resolve_edit(
    index: &LineIndex,
    inv: &ToolInvocation,
    resynced: bool,
) -> Result<ResolvedEdit, String> {
    let anchor: Anchor = parse_anchor(inv.arg("run"), inv.arg("after")).map_err(|e| e.0)?;
    let occurrence = match inv.arg("occurrence") {
        Some(v) => Some(
            v.parse::<usize>()
                .map_err(|_| format!("occurrence '{v}' is not a positive integer"))?,
        ),
        None => None,
    };
    let resolved = resolve_anchor(index, &anchor, occurrence).map_err(|e| {
        if resynced {
            format!(
                "the document changed on disk since you last read it; it was re-woven \
                 automatically, but this edit's anchors no longer resolve: {}",
                e.0
            )
        } else {
            e.0
        }
    })?;

    let payload = inv.input.clone();
    match resolved {
        ResolvedAnchor::Run { first, last } => match payload {
            Some(text) => Ok(ResolvedEdit {
                start: index.starts[first],
                end: index.content_end(last),
                text,
                first_line: first,
            }),
            // No payload: delete the whole run including its newlines.
            None => Ok(ResolvedEdit {
                start: index.starts[first],
                end: index.line_end(last),
                text: String::new(),
                first_line: first,
            }),
        },
        ResolvedAnchor::After(at) => {
            let text = payload
                .ok_or_else(|| "insertion (after=...) needs an <hick:input> payload".to_string())?;
            match at {
                None => Ok(ResolvedEdit {
                    start: 0,
                    end: 0,
                    text: format!("{text}\n"),
                    first_line: 0,
                }),
                Some(i) => {
                    let last_line = i + 1 == index.lines.len();
                    if last_line && !index.trailing_newline {
                        Ok(ResolvedEdit {
                            start: index.content_end(i),
                            end: index.content_end(i),
                            text: format!("\n{text}"),
                            first_line: i + 1,
                        })
                    } else {
                        Ok(ResolvedEdit {
                            start: index.line_end(i),
                            end: index.line_end(i),
                            text: format!("{text}\n"),
                            first_line: i + 1,
                        })
                    }
                }
            }
        }
    }
}

/// Every tag in a document, nested ones included.
///
/// `HickDocument::tags()` is top-level only, and the tags this needs are not:
/// a `<hick:paste>` lives inside the `<hick:file>` it fills, and a fragment
/// can sit inside a `<hick:when>`. Searching only the top level finds neither.
fn all_tags(nodes: &[hick_lang::HickNode]) -> Vec<&hick_lang::HickTag> {
    let mut out = Vec::new();
    let mut stack: Vec<&hick_lang::HickNode> = nodes.iter().collect();
    while let Some(node) = stack.pop() {
        if let hick_lang::HickNode::Tag(tag) = node {
            out.push(tag);
            stack.extend(tag.children.iter());
        }
    }
    out
}

/// Where a pasted fragment is actually declared.
struct FragmentHome {
    doc: String,
    /// 0-based line the fragment is DECLARED on. Only the opening tag's span
    /// is known here, so this is stated as a point rather than a range — a
    /// range would have to guess where the block ends.
    line: usize,
    /// A few lines from the declaration, as context.
    excerpt: String,
}

impl EditSession {
    /// Follow a paste site to the document that declares what it pastes.
    ///
    /// `Origin::Paste` records the location of the `<hick:paste>` tag, not of
    /// the fragment supplying the bytes. Inside one document those are close
    /// enough to be useful; across a `hick:upstream` edge they are different
    /// FILES, and routing an edit to the paste tag sends it to the one place
    /// changing it cannot possibly help.
    ///
    /// So: find the paste tag at that location, read its selector, and look
    /// for the matching `hick:copy`/`hick:cut` across the pipeline closure.
    /// Returns `None` when the fragment is declared in the same document (the
    /// existing message is already right) or when nothing matches — a
    /// best-effort improvement to a message must never become a way to fail.
    fn fragment_home(&self, doc: &str, start: usize, end: usize) -> Option<FragmentHome> {
        let paste_source = if doc == self.doc_name {
            self.source.clone()
        } else {
            self.upstream
                .iter()
                .find(|(path, _)| path.display().to_string() == doc)
                .map(|(_, text)| text.clone())?
        };
        let parsed = hick_lang::parse(&paste_source).ok()?;
        let selector = all_tags(&parsed.nodes)
            .into_iter()
            .filter(|t| t.name == "paste")
            .find(|t| {
                t.source_span
                    .as_ref()
                    .is_some_and(|span| span.start < end && span.end > start)
            })
            .and_then(|t| t.get_attribute("select"))?
            .to_string();

        // The closure, primary first: a fragment declared in more than one
        // document is already an error the parser reports, so first hit wins.
        let mut candidates: Vec<(String, String)> =
            vec![(self.doc_name.clone(), self.source.clone())];
        for (path, text) in &self.upstream {
            candidates.push((path.display().to_string(), text.clone()));
        }

        for (name, text) in candidates {
            let Ok(parsed) = hick_lang::parse(&text) else {
                continue;
            };
            let found = all_tags(&parsed.nodes)
                .into_iter()
                .filter(|t| t.name == "copy" || t.name == "cut")
                .find(|t| {
                    selector.split(',').map(str::trim).any(|sel| {
                        sel.strip_prefix('#')
                            .is_some_and(|id| t.get_attribute("id") == Some(id))
                            || sel.strip_prefix('.').is_some_and(|class| {
                                t.get_attribute("class")
                                    .is_some_and(|c| c.split_whitespace().any(|x| x == class))
                            })
                    })
                });
            let Some(tag) = found else { continue };
            // Same document as the paste: the existing wording already points
            // at the right file, so do not add a second, noisier sentence.
            if name == doc {
                return None;
            }
            let Some(span) = tag.source_span.as_ref() else {
                continue;
            };
            let index = LineIndex::new(&text);
            let (a, _) = byte_range_to_lines(&index, span.start, span.end);
            let total = text.lines().count().saturating_sub(1);
            return Some(FragmentHome {
                doc: name,
                line: a,
                excerpt: index.render_range(a, (a + 3).min(total)),
            });
        }
        None
    }
}

/// Map a byte range to inclusive (first, last) line indices.
fn byte_range_to_lines(index: &LineIndex, start: usize, end: usize) -> (usize, usize) {
    let line_of = |pos: usize| match index.starts.binary_search(&pos) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let first = line_of(start);
    let last = line_of(end.saturating_sub(1).max(start));
    (first, last.max(first))
}

/// Render the edited region with fresh hashes: the replacement lines plus
/// two lines of context on each side.
fn edited_region(index: &LineIndex, first_line: usize, replacement: &str) -> String {
    let replacement_lines = if replacement.is_empty() {
        0
    } else {
        replacement.lines().count()
    };
    let from = first_line.saturating_sub(2);
    let to = (first_line + replacement_lines.saturating_sub(1) + 2)
        .min(index.lines.len().saturating_sub(1));
    index.render_range(from, to)
}

/// Kind label for a provenance entry.
/// The whole lines covering `span` in `doc_path`, plus the line above —
/// which, for a fragment body that opens mid-line, is the line carrying the
/// enclosing tag (`<hick:copy id=…>`). Capped so a large span cannot flood
/// the refusal.
fn foreign_excerpt(doc_path: &str, span: (usize, usize)) -> Option<String> {
    let src = std::fs::read_to_string(doc_path).ok()?;
    let (s, e) = span;
    let s = s.min(src.len());
    let e = e.clamp(s, src.len());
    let line_start = src.get(..s)?.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let start = src
        .get(..line_start.saturating_sub(1))?
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = src.get(e..)?.find('\n').map(|i| e + i).unwrap_or(src.len());
    let lines: Vec<&str> = src.get(start..end)?.lines().collect();
    let shown: Vec<&str> = lines.iter().copied().take(8).collect();
    let mut out = shown.join("\n");
    if lines.len() > shown.len() {
        out.push_str("\n  …");
    }
    Some(out)
}

fn origin_kind(p: &Provenance) -> &'static str {
    use hickory_lineage::Origin;
    match p.origin {
        Origin::Literal { .. } => "literal",
        Origin::Paste { .. } => "paste",
        Origin::Exec { .. } => "exec",
        Origin::Variable { .. } => "variable",
        Origin::Substitution { .. } => "substitution",
        Origin::Agent { .. } => "agent",
        // Not "literal": these bytes are in the document and editable, but a
        // tool outside it wrote them. An agent that reported them as literal
        // would tell itself somebody here typed a scaffolder's forty files.
        Origin::Ingested { .. } => "ingested",
        Origin::Synthetic => "synthetic",
    }
}

/// Extract `"<doc> bytes S..E"` from a lineage duplicate-paste conflict
/// message.
fn parse_conflict_location(detail: &str) -> Option<(String, usize, usize)> {
    // Shape: "output bytes A..B come from <doc> bytes S..E, which is woven ..."
    let idx = detail.find(" come from ")?;
    let rest = &detail[idx + " come from ".len()..];
    let bytes_idx = rest.find(" bytes ")?;
    let doc = rest[..bytes_idx].trim().to_string();
    let span_str = &rest[bytes_idx + " bytes ".len()..];
    let span_end = span_str.find(',').unwrap_or(span_str.len());
    let (s, e) = span_str[..span_end].trim().split_once("..")?;
    Some((doc, s.parse().ok()?, e.parse().ok()?))
}

/// Weave (no execution) one document source, mirroring `hick weave`:
/// cached transcripts are used when the project has a transcript cache.
async fn weave_source(
    doc_path: &Path,
    doc_name: &str,
    source: &str,
    params: &[(String, String)],
) -> Result<WeaveState> {
    // Fail early with the parse error rather than a pipeline error.
    hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {doc_name}: {e}"))?;

    let project_dir = doc_path.parent().unwrap_or(Path::new("."));
    let cc =
        hick_literate::cache::CacheConfig::new(project_dir, hick_literate::cache::CacheMode::Reuse);
    let cache_config = cc.cache_dir.is_dir().then_some(&cc);

    let sources = vec![(doc_name, source)];
    let result = hick_literate::run_pipeline_weave(&sources, params, cache_config).await?;

    let mut files = HashMap::new();
    for (path, content) in &result.files {
        if let FileContent::Text(s) = content {
            files.insert(path.clone(), s.clone());
        }
    }
    let provenance = result
        .provenance_maps
        .iter()
        .map(|(p, m)| (p.clone(), hickory_lineage::from_provenance_map(m)))
        .collect();
    Ok(WeaveState { files, provenance })
}

/// Every document reachable from `doc_path` through `hick:upstream`, nearest
/// first, mapped to its current source.
///
/// Breadth-first with a visited set, so a diamond is loaded once and a cycle
/// terminates. Unreadable or unparseable edges are skipped rather than
/// failing the session: an agent opening a document should not be blocked by
/// a broken document elsewhere in the chain — that is what `hick test`
/// is for, and the agent may well have been called to fix it.
/// The project a document belongs to: its git repository's top level when it
/// is in one, otherwise its own directory. `read_file` reads nothing above it.
fn project_root(doc_path: &Path) -> PathBuf {
    let dir = doc_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let top = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    top.unwrap_or(dir)
}

fn upstream_closure(doc_path: &Path) -> std::collections::BTreeMap<PathBuf, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut seen: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut queue = vec![doc_path.to_path_buf()];
    seen.insert(normalize(doc_path));

    while let Some(current) = queue.pop() {
        let Ok(source) = std::fs::read_to_string(&current) else {
            continue;
        };
        let Ok(doc) = hick_lang::parse(&source) else {
            continue;
        };
        let dir = current.parent().unwrap_or(Path::new("."));
        for tag in doc.tags().filter(|t| t.name == "upstream") {
            let Some(file) = tag.get_attribute("file") else {
                continue;
            };
            let path = dir.join(file);
            if !seen.insert(normalize(&path)) {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.insert(path.clone(), text);
                queue.push(path);
            }
        }
    }
    out
}

/// Canonicalize for identity comparison, falling back to the path itself so a
/// file that does not exist still de-duplicates against itself.
fn normalize(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
