# Context Provenance Is Derived From The Session, Never From The Model

Given an agent session, when a tool shows the model a file (`read_doc`,
`read_output`, `read_file`), then the session records `<hick:read>` — the
path, the content's SHA-256, the repository commit when there is one, and the
lines shown; when an edit tool writes, then the session records
`<hick:wrote>` — the file, the lines as they stand after the edit, and their
hashline hashes; and when anyone asks (`hick context`, `GET /api/docs/:id/context`,
the app's Context layer), then every write's context is **every input before
it in the same session** — those reads, plus the prompt, tool results and
observations, each summarized and pointing back at its session element by
id and line — located in the document as it stands now by the written lines'
hashes, or reported as no longer present.

The reason: the question "what did the model have when it wrote this?" has an
answer that does not depend on the model's honesty, because the tool surface
is the model's only view of anything (`agent-cells.md`: no write primitive;
and now no read outside the recorded tools either). Recording it at the
tools, and deriving from the record, keeps it that way. The model's own view
of what it used is a different thing — declared provenance — and is kept
apart on purpose (`docs/specs/freeform/three-provenances.md`).

Four properties hold it up:

1. **Every input element has an id.** `hick:user`, `hick:observation`,
   `hick:tool-result` carry `id="in<n>"`, seeded from the file when a session
   is appended to across processes, so the derivation can name them and a
   reader can find them.
2. **Reads record the bytes, not the name.** `sha256` is the truth about what
   was shown; `commit` is where to look later; `lines` is how much.
3. **Writes record what stands, as hashes.** `changed_lines` diffs the file
   before and after the edit (common prefix/suffix), so an `edit_output` that
   lands in the document through lineage records the DOCUMENT lines it wrote.
4. **`read_file` reads the project and nothing else.** Paths resolve against
   the document's directory and must stay inside the project root (the git
   top level, or the document's directory); a directory lists; a non-UTF-8 or
   over-sized file is refused with the reason.

## Boundary

Present, not used: nothing here claims a line was derived from an input.
Context is line-granular; a later human edit to a line changes its hash and
the write stops resolving, which is correct. Sessions are the user's record —
`hick init` ignores `sessions/` — so context provenance is as available as
the session is.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified (implemented and reviewed in the same change; driven
  end-to-end: a real `hick agent` session on
  `examples/receipts/hick-agent/analysis.hick` used `read_file` on the CSV and
  `hick context` reports the CSV at commit `ab7a2d9`, lines 1–8, by hash, in
  the context of the line it then wrote; the app drew it —
  `examples/receipts/context-ribbon.png`)
- Evidence: `ContextRead`, `Wrote`, `ToolOutcome::{reads,wrote}`,
  `read_file`, `changed_lines`, `head_commit_for`, `project_root` in
  `crates/hickory-agent/src/tools/mod.rs`; `SessionEvent::{Read,Wrote}`,
  `record_outcome`, input ids in `crates/hickory-agent/src/session.rs`;
  `crates/hickory-agent/src/context.rs`; `cmd_context` in
  `crates/hickory-cli/src/main.rs`; `get_context` in
  `crates/hickory-cli/src/serve/api.rs`; `hick doc read-file` and the MCP
  `read_file` tool.
- Tests: `crates/hickory-agent/tests/tools_edit_session.rs`
  (`read_file_shows_a_project_file_and_records_the_read`,
  `the_session_records_reads_and_writes_as_elements`),
  `crates/hickory-agent/src/context.rs::tests`,
  `crates/hickory-agent/src/session.rs::tests`.
