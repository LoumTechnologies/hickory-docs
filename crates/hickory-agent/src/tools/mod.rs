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
}

impl ToolOutcome {
    fn ok(name: &str, text: String) -> Self {
        Self {
            name: name.to_string(),
            ok: true,
            text,
        }
    }

    fn err(name: &str, text: String) -> Self {
        Self {
            name: name.to_string(),
            ok: false,
            text,
        }
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
}

impl EditSession {
    /// Open a session on `doc_path`: read, parse, and weave (no execution).
    pub async fn open(doc_path: &Path, params: &[(String, String)]) -> Result<Self> {
        let source = std::fs::read_to_string(doc_path)
            .with_context(|| format!("failed to read {}", doc_path.display()))?;
        let doc_name = doc_path.display().to_string();
        let weave = weave_source(doc_path, &doc_name, &source, params).await?;
        let upstream = upstream_closure(doc_path);
        Ok(Self {
            doc_path: doc_path.to_path_buf(),
            upstream,
            doc_name,
            params: params.to_vec(),
            source,
            weave,
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
        let index = LineIndex::new(&source);
        let mut text = format!("doc: {name}\n{}", index.render());
        if self.upstream.is_empty() {
            return ToolOutcome::ok("read_doc", text);
        }
        // Name the rest of the chain, or the agent has no way to learn that a
        // decision it needs to change lives one document up.
        text.push_str(&format!(
            "\nupstream of this session (readable and editable by passing \
             <hick:arg name=\"doc\">NAME</hick:arg>): {}\n",
            self.upstream_names()
        ));
        ToolOutcome::ok("read_doc", text)
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
    /// `billing.hick` without reconstructing the relative path.
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
        ToolOutcome::ok("read_output", text)
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
        ToolOutcome::ok(
            NAME,
            format!(
                "edited {path}; the document was updated through lineage and re-woven.\n\
                 edited region (fresh hashes):\n{region}"
            ),
        )
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
                if let Some((doc, s, e)) = parse_conflict_location(detail) {
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
        ToolOutcome::ok(
            NAME,
            format!(
                "edited {} and re-wove.\nedited region (fresh hashes):\n{region}",
                self.doc_name
            ),
        )
    }

    // -- verify ------------------------------------------------------------

    /// Execute the document for real through the session's executor,
    /// evaluate every expectation, and write the produced output files next
    /// to the document (so a following `hickory check` sees no drift).
    async fn verify(&mut self, executor: Arc<dyn Executor>) -> ToolOutcome {
        const NAME: &str = "verify";
        if let Err(e) = self.sync_with_disk().await {
            return ToolOutcome::err(NAME, e);
        }
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
                    outcome.doc, outcome.line, outcome.container, outcome.detail
                ));
            }
        }

        // Write outputs to disk (like `hickory run`), so document and
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
        if failures.is_empty() {
            ToolOutcome::ok(
                NAME,
                format!(
                    "PASS — executed the document: {checked} expectation(s) met; wrote {} output \
                     file(s): {}",
                    written.len(),
                    if written.is_empty() {
                        "(none)".to_string()
                    } else {
                        written.join(", ")
                    }
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
        "edit_output" => session.edit_output(inv).await,
        "edit_doc" => session.edit_doc(inv).await,
        "verify" => session.verify(executor).await,
        other => ToolOutcome::err(
            other,
            format!(
                "unknown tool '{other}' — available: read_doc, read_output, edit_output, \
                 edit_doc, verify"
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
fn origin_kind(p: &Provenance) -> &'static str {
    use hickory_lineage::Origin;
    match p.origin {
        Origin::Literal { .. } => "literal",
        Origin::Paste { .. } => "paste",
        Origin::Exec { .. } => "exec",
        Origin::Variable { .. } => "variable",
        Origin::Substitution { .. } => "substitution",
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

/// Weave (no execution) one document source, mirroring `hickory weave`:
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
    let cc = hick_literate::cache::CacheConfig::new(project_dir, true, false);
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
/// a broken document elsewhere in the chain — that is what `hickory check`
/// is for, and the agent may well have been called to fix it.
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
