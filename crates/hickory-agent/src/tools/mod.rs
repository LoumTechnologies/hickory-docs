//! Shared document tools: read both
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
mod dispatch;
pub mod hashline;
mod prediction;
pub use dispatch::execute_tool;
mod preview;
mod verify;
mod verify_publish;
mod weave;
use weave::weave_source;
mod receipts;
use anyhow::{Context as _, Result};
use hick_exec::node::FileContent;
use hickory_executor::Executor;
use hickory_lineage::{LineageError, OutputEdit, Provenance, apply_source_edits, map_edits};
pub use receipts::{ContextRead, Wrote};
use receipts::{changed_lines, head_commit_for, sha256_hex};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::invocation::ToolInvocation;
mod read_file;
use hashline::LineIndex;
mod edit_resolve;
use edit_resolve::resolve_edit;
pub(crate) use read_file::read_file;

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
    verified_publication: bool,
    prediction: Option<prediction::PredictionVerification>,
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
            verified_publication: false,
            prediction: None,
            root,
        })
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
        if self.prediction.is_some() {
            if disk != self.source {
                return Err("Private source changed outside the captured edit session; host review and a new source-bound session are required".into());
            }
            return Ok(false);
        }
        let upstream = upstream_closure(&self.doc_path);
        if disk == self.source && upstream == self.upstream {
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
        self.upstream = upstream;
        self.weave = weave;
        Ok(true)
    }

    /// Re-weave from the in-memory source, refreshing files and provenance.
    async fn reweave(&mut self) -> Result<()> {
        if self.prediction.is_some() {
            prediction::private_source(&self.source)?;
            self.weave = WeaveState {
                files: HashMap::new(),
                provenance: HashMap::new(),
            };
            return Ok(());
        }
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
        let mut text = format!(
            "doc: {name}\nsnapshot: {}\n{}",
            sha256_hex(source.as_bytes()),
            index.render()
        );
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
        let mut text = format!(
            "output: {path}\nsnapshot: {}\n{}",
            sha256_hex(content.as_bytes()),
            index.render()
        );
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
            path: self
                .doc_path
                .parent()
                .unwrap_or(Path::new("."))
                .join(path)
                .display()
                .to_string(),
            source_path: Some(self.doc_path.parent().unwrap_or(Path::new(".")).join(path)),
            commit: None, // an output is derived; its document's commit is the record
            sha256: sha256_hex(content.as_bytes()),
            first_line: 1,
            last_line: lines,
            content: content.clone(),
            byte_span: (0, content.len()),
            unique: !content.is_empty(),
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
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

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
    let mut out = weave::declared_sources(doc_path, None).unwrap_or_default();
    out.remove(doc_path);
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
