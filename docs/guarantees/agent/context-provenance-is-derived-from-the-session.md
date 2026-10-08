# Context Provenance Is Derived From The Session, Never From The Model

Given an agent session, when a tool shows the model a file (`read_doc`,
`read_output`, `read_file`), then the session records `<hick:read>` — the
path, the content's SHA-256, the repository commit when there is one, and the
lines shown; when an edit tool writes, then the session records
`<hick:wrote>` — the file, the lines as they stand after the edit, and their
hashline hashes; and when anyone asks (`hick context`, `GET /api/docs/:id/context`,
the app's Context layer), native writes use the recorded message envelope supplied to the provider client and
reads from this turn. Adapter writes use recorded observations from this turn,
with opaque adapter-managed context explicitly outside that scope. Abandoned
branches are retained historically but are never counted as supplied inputs.
Availability is not evidence that a claim follows from a source.

Writes carry SHA-256 line fingerprints. Matching current text refuses weak
legacy fingerprints and ambiguous duplicate matches. It establishes byte
correspondence, not an author's identity or a causal explanation.

## Exact returned ranges (2026-10-07)

ACP file reads and workspace `read_text` derive the receipt from the same range
of bytes returned by the read. Reads from an unsaved editor buffer honor the
requested line and limit, record the supplied buffer and request, and emit a
completed read receipt with the whole snapshot's SHA-256 and only the returned
lines. They do not save that buffer to disk. Empty, out-of-range and zero-limit
responses contribute no invented line range. Native relative reads record the
resolved workspace-relative path rather than the caller's relative spelling.

The read's explanation identifies the snapshot hash and commit when recorded.
A read invocation's location link says **Read request**; a completed receipt
says **Recorded read**. Neither establishes that an assistant's conclusion is
supported. Current file contents may differ from that recorded snapshot.

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

Verification update (2026-10-07): the native relative-path regression in
`tools/read_file.rs`, range/hash regression in `serve/acp/read_receipt.rs`, and
real HTTP `serve_acp::a_live_buffer_read_records_the_returned_range_and_links_to_its_snapshot`
exercise the resolved path, Unicode/CRLF, clipped and empty ranges, an unsaved
buffer, its recorded hash and the resulting line-specific session link.

`just test-agent-pane` passes (92 native agent tests and 48 frontend tests).
`just test-conversation-lineage` passes (10 vocabulary/receipt tests, two HTTP
tests, 67 frontend tests and TypeScript checking), including the distinction
between a failed read request and a separately completed read receipt. Real
model authentication is not exercised by these fixture-based checks.

The ACP backend checks pass (five unit tests, 35 integration tests and the
desktop ACP proxy test; two authenticated model tests remain ignored). Its full
frontend step initially exposed a startup-test race: folder hydration remounts
the introduction editor when it adds the Files split. `App.test.tsx` now waits
for that arrangement before the close-window gesture. The final `just test-web`
passes all 1,945 tests and TypeScript checking; the file-length check passes.

Verification update (2026-10-08): `context::derive_from_session`, `locate`,
`session::SessionEvent::Read`, native request recording and ACP supplied prompts
now establish the narrower scopes above. `context::tests` challenge branch
contamination, weak hashes and duplicates; `model_failure_audit` covers the
previous 16-bit collision pair and forged nested evidence. Sessions remain
editable user records; this is not a signed or authenticated audit trail.

Provider wire encoding, context editing and model-internal processing are outside
the recorded envelope. It is evidence of what the client was supplied, not a
claim about what the model attended to or how the provider transformed it.
