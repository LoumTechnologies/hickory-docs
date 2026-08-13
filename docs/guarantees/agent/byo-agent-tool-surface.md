# An Outside Coding Agent Gets The Same Document Tools As The Built-In One

Given a coding agent that is not hick's own — Claude Code, Codex, Grok CLI,
or anything else that can run a process — when it works on a `.hick` document,
then it can reach the same five tools the built-in agent uses (`read_doc`,
`read_output`, `edit_output`, `edit_doc`, `verify`) through two surfaces, and
gets the same guarantees from both:

1. **Hashline anchors.** Every read returns lines prefixed with a 4-hex hash of
   their content, and every edit anchors on those hashes rather than on line
   numbers. An edit built against text that has since changed does not resolve
   and is **refused**, never misapplied.
2. **Byte-exact lineage.** An edit made through a generated output is mapped
   back into the document source byte-for-byte, or refused with a message
   naming the document location to edit instead. A refusal is routing, not
   failure.
3. **Real verification.** `verify` executes the document through the same
   `Executor` as `hick run` — same `HICKORY_EXECUTOR` selection, same
   expectation checking.
4. **A replayable session.** With `HICKORY_SESSION` set, every tool call and
   its result is appended to a `hick:session` document — across *separate
   processes* — and the file is a closed, parseable session after every call.

The two surfaces are `hick doc <tool>` (one process per call; works in any
harness and in CI) and `hick mcp` (stdio MCP; registered by `hick init`
in the project's `.mcp.json`). They are the same implementation: both build a
`ToolInvocation` and call `hickory_agent::execute_tool`. Neither reimplements
an edit, so they cannot drift apart in what an edit means.

## Why the MCP surface is the better one

The command surface opens a session, runs one tool, and exits, so freshness
comes only from the anchors. The MCP server is one long-lived process that
keeps an `EditSession` open per document and re-weaves after every edit — which
restores the in-process property that a stale edit is impossible *by
construction*: the second use of a spent anchor is refused without anything
having to re-read the file first.

Both are correct. One is cheaper and knows more.

## Boundary — what an outside agent's session does NOT contain

The recorded session holds tool calls and their results: what was asked of the
document and what came back. It does **not** hold the agent's reasoning, which
lives in that harness's own transcript in that vendor's own format. Importing
those was considered and rejected — three private log shapes to maintain, for
prose that nothing replays.

A second, related limit: `hick promote` extracts *script* writes from a
session. A tool-driven session edits the document in place, so promoting one
yields an empty pipeline — correctly, because there is nothing left to
reconstruct. Promote is for script-first sessions; for tool-driven work the
document itself is the product and the session is the record.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/doc_tools.rs` (command surface) and
  `crates/hickory-cli/src/mcp.rs` (MCP surface) both construct a
  `ToolInvocation` and call `hickory_agent::execute_tool` against an
  `EditSession`; no edit logic exists in either. `ToolInvocation::synthetic`
  (`crates/hickory-agent/src/protocol.rs`) renders the same `raw_xml` a parsed
  invocation carries, and rejects a payload containing `</hick:input>` rather
  than writing a session that would not parse.
  `HickSessionLog::append_or_create` (`crates/hickory-agent/src/session.rs`)
  reopens a closed session by dropping the root close, which is what makes
  per-process recording work; it refuses to append to a file that is not a
  session. `crates/hickory-cli/src/init.rs::ensure_mcp_registration` merges
  into `.mcp.json` without touching other servers.
- Test coverage: `crates/hickory-cli/tests/byo_agent_surface.rs` drives the
  shipped binary — read → edit through the output → the same anchor refused
  → verify; the cross-process session assertion; the ambiguous-document
  error; and an MCP exchange asserting the tool list, that a notification
  draws no reply, that the reused session knows a spent anchor is stale, and
  that an unknown method is a JSON-RPC error rather than a dropped
  connection. `crates/hickory-cli/tests/init_tests.rs` covers the `.mcp.json`
  merge (including not clobbering another server) and that the managed
  AGENTS.md section names the tools.
- Not covered by tests: registration in Codex's and Grok CLI's own global
  config files. `hick init` deliberately does not write those — it prints
  what to add instead of guessing another tool's config shape — so there is
  nothing to assert beyond the printed instruction.
