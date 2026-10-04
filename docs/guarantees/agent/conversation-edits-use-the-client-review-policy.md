# Conversation edits use the client's review policy

Given an ACP conversation, when the user chooses **Review** or **Auto-accept**
under **Document edits**, then that choice belongs to this conversation and is
restored when its saved conversation is reopened. New conversations start with
Review. Changing it during a turn is refused.

The agent uses the same editing tools in either mode. `read_buffer` and
`edit_buffer` operate on the identified open editor, including unsaved and
untitled notes. `read_doc` and `edit_doc` also recognize open editor context.
The prompt directs requested buffer changes through tools rather than asking
for a replacement document in the answer. Sending a prompt or editing an
untitled note never manufactures a filename or saves that note implicitly.

In Review, edits submitted through these tools, the existing `edit_output`
and `create_doc` tools, or ACP text-file writes wait for Accept or Reject.
In Auto-accept they proceed without a review click. Proposals open in a document review tab beside the current panes. Accept and
Reject live beside that editor. Proposed bytes stay in a read-only lens until
accepted; the target buffer is never used as a temporary preview. Pending and
completed changes use `DocumentEditor` and its `comparisonField`, the same
mechanism as literate Git comparisons and commit readings. Adapter-reported
raw diff content remains available in tool details.

Accepting a live-buffer edit uses the ordinary editor transaction path and
remains undoable. A changed or closed target buffer refuses application; a
changed disk/tool surface is rechecked after review. A stale buffer rejection
returns its newer text to the tool bridge so the agent can read again. Rejection reaches the
agent as a failed tool call, and stopping cancels pending edits. Decisions are
consumed once. Generated buffers direct writes through `edit_output`, which
maps changes to source through lineage.

This policy governs Hickory-handled document edits, not the adapter's native
shell commands or the separate literate-view arrangement tools. Those retain
their existing execution and permission boundaries.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/acp/{edits,mcp,client,mod,record}.rs`, `mcp_preview.rs`,
  `hickory-agent/src/tools/preview.rs`, `WorkspaceChat`, `AgentChanges`,
  `agentEdit`, `DocumentReading`, `comparisonField`, and `AcpControls.ToolDetails`.
- Tests: `serve_acp` real HTTP/process regressions cover buffered tool calls,
  review acceptance/rejection, output-tool review, cancellation, single-use
  decisions, independent conversations, and policy recovery after restart.
  `AgentChanges.test.tsx` covers the document comparison editor, review, automatic
  application, and stale-edit reporting. `agentEdit.test.ts` uses real
  CodeMirror editors to check exact text, Unicode, undo, stale typing, and
  closed targets. Existing ACP and UI regression checks remain in place.
- Live evidence: the installed authenticated Codex ACP adapter received the
  ordinary prompt “Change the heading in the current document to Meeting notes.”
  It submitted a live-buffer tool edit, left disk unchanged during review, and
  completed after the HTTP client applied and accepted the exact expected text.
- Validation: full web suite (1,883 tests), targeted final-newline and editor
  checks, ACP HTTP/process integration, built-in agent and external MCP
  regressions, workspace filesystem tests, and all-target CLI Clippy.
- Caveats: native desktop visual inspection has not been established for this
  change. Change cards live with the active
  connection; the setting and edit evidence are recorded in the conversation.

Verification update (2026-10-04): `AgentChanges` publishes immutable readings;
`representationTabs` opens a review alongside existing panes, and
`DocumentReading` places the decision beside `DocumentComparison`.
`serve_acp` also checks that reconnecting during pending review returns the
active connection promptly. `ConversationEditor` now renders the Agent pane.
