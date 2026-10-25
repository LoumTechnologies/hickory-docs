<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/review-investigations-link-observed-agent-work.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `bd619d76d89bf847a61c9aac0a09fea1ab37ae7a6173a722027f00364688ae22`.

<book:entry path="docs/guarantees/authoring/review-investigations-link-observed-agent-work.md">
# Review investigations link observed agent work without resolving human review

Given a saved review thread, posting a comment or preparing a prompt does not
start an agent. The user separately selects one open conversation and sends an
investigation. Dispatch uses only that conversation's current branch, provider,
context capture and existing edit approval flow. It preserves an unsent draft.
Closed, busy or unconfigured panes report failure rather than broadcasting.

Queued and accepted investigation links append to the workspace review ledger.
The prompt includes the original target, hash, relationship, objection history
and explicit saved-source comparison. Changed, unavailable and ambiguous targets
remain historical; no similar quotation is silently substituted. Source chips
with complete historical source evidence offer keyboard/context-menu entry.
Chips without sufficient snapshot evidence ask for an explicit source selection.

The server resolves a linked turn against the recorded agent conversation;
running, responded, failed, cancelled and unavailable states come from that
turn, independently of human open/resolved state. Responses and session paths
remain inspectable with the original review. Model prose never creates check
passes or auto-resolves a review. Existing edit previews and recorded session
verification receipts remain the evidence surface for proposed changes/checks.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: #62 and #47, ai-engineer-review-workflow.md.
- Evidence: ReviewPanel.investigate, reviewInvestigations registry, ChatDock.send,
  reviews::list/current_state and SourceChips.reviewSourceChip.
- Assumptions: the selected pane names one conversation subject and its visible
  branch. Local ledger links are user declarations; host turns establish actual
  status/response, not semantic truth or authenticated custody.
- Witnesses: focused ReviewPanel and source-chip tests; focused real confined
  Python/HTTP linkage test in serve_python_agent; existing review-store tests.
- Challenges: two conversations, ambiguous source edits, refused edit, stopped
  inference, explicit human resolve/reopen and preservation of original target.
- Coverage boundary: saved-source comparison does not inspect later unsaved
  editor buffers. A lost acceptance-to-link write is recovered when the queued
  investigation tag is present in the actual host prompt; absent host records
  remain unavailable/queued, never falsely delivered. Editing/verification uses
  existing agent pane and session receipts, not a second review executor.
- Dependencies: files above in this commit; changes invalidate this review.
- Executed checks: two review-store tests, one real confined Python/HTTP test,
  five UI tests and TypeScript/diff checks passed. Initial real-worker setup failed
  because an editable install was absent under Python -I; installed the locked
  non-editable worker as required by scripts/check-python-agent.sh.
- Human judgment: not recorded.
</book:entry>

</book:doc>
