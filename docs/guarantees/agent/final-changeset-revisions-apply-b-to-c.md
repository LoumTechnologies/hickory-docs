# A final changeset revision applies B to C

Given a finalized agent artifact constructing accepted B from captured A,
when the user chooses Revise New, then the accepted closure and snapshots remain
immutable. A separately labeled authored draft permits New code edits with Old
fixed. Editing uses ordinary patch regeneration with captured A, re-evaluates the
entire supplied closure, and writes no destination. A downstream-breaking edit
remains unchecked and disables Apply. Discard removes the draft without writes.

Given a successfully constructed C, when Apply revision is requested, then the
host re-evaluates C against frozen A and checks the current complete destination
inventory against B, including editor identities, untitled window-instance URIs and modeled metadata. It
publishes only the B-to-C differences through guarded text publication and
identified editor transactions. A stale destination is refused. Untitled editor
addresses never become filenames or an implicit save. Creations, deletions and
renames in the current inventory invalidate the expected state.

Given a partial application failure, then the result names landed disk paths,
retains their before/after recovery bytes, and reports editor successes and
failures separately. It does not claim batch atomicity or silently roll back
across newer work. Editor undo remains the editor's transaction history where
supported. Construction success, application success and runtime checks are
separate; editing clears inherited verification evidence. Agent and authored
revision records stay distinct without asserting human authorship or semantic
correctness.

Given a constructed draft patch with native tools available, then tools receive
its complete evaluated captured project. Unchecked edited bytes lose the old
native context. No missing dependency is supplied from the current worktree.
Virtual editor addresses and folderless captures have no native executable
context and refuse those tools.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: verified within the modeled tree and per-destination limits below.
- Originating requirement: user request to revise New in accepted final changesets
  and apply the revision through shared guarded writes, preserving A/B/C.
- Evidence: `ChangesetDocument`, `PatchEditor`, `AgentProject`,
  `serve/changeset_drafts.rs::apply`, `project_publish::publish_text`,
  `WorkspaceCapture::current`, `WorkspaceChat::applyAgentChange`, `applyAgentEdit`
  (local user-event marking and an isolated undo transaction).
- Witnesses: real HTTP tests in `literate_agent_projects.rs`, ACP subprocess
  B-to-C test in `agent_changesets.rs`, captured regeneration/downstream tests in
  `ChangesetDocument.test.tsx`, real Chromium untitled workflow in
  `workspace-changesets.spec.ts` with and without a folder.
- Challenges: concurrent bytes, inventory changes, editor save/rename/close,
  downstream failure, read-only second target after a first write, and reuse of
  an already-applied revision. These must preserve newer work and the original.
- Assumptions and exclusions: snapshots observe stable reads, not an OS-wide
  transaction. External writers can race the final check or perform ABA writes.
  Mode comparison covers executable bits and symlink targets; ownership,
  timestamps, other permission bits and empty directories are outside the model.
  New code editing targets textual patch comparisons with explicit IDs and
  preserves entry kind/mode. Other construction kinds retain captured read-only
  rendering. Revised symlink/binary or executable
  metadata publication is refused before writes. Drafts are window-local and can
  be lost on close/restart. A publication receipt is recovery data, not an
  automatic cross-file undo. A client disconnect after disk publication can leave
  unapplied editor changes; its separate revision record retains the candidate.
- Dependencies: paths above in this commit; changes invalidate this review.
- Executed checks: `CARGO_INCREMENTAL=0 just test-agent-changesets` passed
  26 native cases; `just test-agent-changesets-web` passed typechecking and
  72 tests; `just test-web` passed 2,198 tests in 239 files. The final
  `CARGO_INCREMENTAL=0 just test-workspace-changesets-browser` passed both real
  Chromium flows, including stale refusal, Discard, Apply and undo with no save.
  `CARGO_INCREMENTAL=0 just check-agent-changesets`, the file-length ratchet and
  whitespace checks passed. These are development-tree observations; subsequent
  edits to the dependencies above stale this evidence. Native runtime execution
  of a user's revised program is not implied by these checks.
- Human judgment: requirement supplied; no visual acceptance recorded.
