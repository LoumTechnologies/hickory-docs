# Conversation edits use the client's review policy

Given a built-in or ACP conversation, when the user chooses **Review** or **Auto-accept**
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
In Auto-accept they proceed without a review click. Proposals appear directly in the Agent pane, with an optional separate review
tab. Accept and Reject live beside the comparison editor. Proposed New bytes
can be revised during review; the target buffer is never used as a temporary preview. Pending and
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

Built-in scope update (2026-10-08): `serve/agent_review.rs` and
`react_loop.rs` submit built-in `edit_doc` and `edit_output` proposals to the same
queue. Review is the app default, rejects writes before application, rechecks
the complete preview after approval, and cancellation releases pending review.
Standalone `hick agent` has no interactive review queue unless a caller supplies
`ToolReview`; scripts continue under the selected executor. Native disk tools
operate on saved source, and do not gain ACP's live-buffer editing capability.
`model_failure_audit::rejected_native_edits_record_the_decision_without_changing_the_document`
is a regression witness. Restoring a saved policy occurs when its turn starts;
pending proposals are connection state, not reconstructed on restart.

Verification update (2026-10-17): `AgentChanges` embeds `DocumentReading` for
each proposal and no longer automatically opens another pane. The comparison
uses the same `ComparisonEditor` as `.md` patches. `AgentChanges.test.tsx`
drives Accept/Reject from the Agent pane itself. This is shared comparison
rendering, not full literate-project patch parity; see
`../audits/2026-10-17-agent-document-parity.md` for outstanding differences.

## October 21 reachable-review correction

An actual attached-provider correction proposed the exact expected patch, but the
completed response panel painted over the pending review button and intercepted
pointer clicks. The run was interrupted after the obstruction was diagnosed;
no forced click or artificial approval was counted as a successful workflow.
`AgentChanges.css` now prevents the review section from shrinking beneath its
content and lets inline document previews use automatic content height.

The focused scripted-provider Chrome flow now performs pointer hit testing on a
second pending proposal's Reject control after a completed response, then cancels
that proposal through Stop. This challenges the observed overlap while leaving
the proposal undecided. The test passed with the real UI/engine/confined worker;
its model responses remain explicitly scripted. Nine focused AgentChanges cases
also passed. The separate actual-provider result is recorded in the handoff.
Human correctness judgment remains not supplied; layout accessibility does not
verify a patch or establish application strict-mode enforcement.
