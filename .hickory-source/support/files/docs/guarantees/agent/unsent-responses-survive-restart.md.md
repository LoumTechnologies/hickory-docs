<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/unsent-responses-survive-restart.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `51454c4b5643118a14805e6d50ddd513c29777ee9dab936e74a350efe6e7b02c`.

<book:entry path="docs/guarantees/agent/unsent-responses-survive-restart.md">
# Unsent responses survive restart

Given text in **Your response**, editing saves recovery to personal workspace
storage. Closing the Agent pane retains it. Reopening the pane or restarting the
app restores the response to its workspace, window slot, editor subject and
conversation thread. Recovery never creates a repository file, names an untitled
editor, or writes response text into an immutable recorded session.

The subject uses the existing agent subject identity: canonical document path,
literate-view identity, or the workspace conversation. Untitled editors, including
windows without an open folder, use that workspace conversation; switching the
focused editor does not change its identity. The existing canonical folder and
window-slot store supplies workspace/window isolation. Attaching the first folder
starts that folder's workspace conversation; queued writes from the blank window
are rejected by their old scope instead of entering the folder's conversation.
Blank-window recovery remains in its original personal store.

Within a subject, each root turn has its own response. Rewinding and switching
branches inside that root share a response. **New thread** and a second
conversation use a separate opening-response slot until an accepted send assigns
a real root turn. The selected tip is restored too. Unknown historical thread
identities are retained rather than assigned to an unrelated conversation.

A late restoration cannot overwrite typing, discard, or thread selection already
made in the pane. Persistence is ordered across pane closure/reopening. An older
queued save cannot follow a newer accepted-send clear or explicit discard.
**Discard response** removes that response's recovery text; closing the pane alone
does not. Other threads' responses remain available.

A rejected send preserves the response. Only acceptance of the request clears the
submitted revision; text edited while sending survives, including an identical
replacement. A newly accepted root carries newer response text to that root.
Recovery does not affect agent execution, review defaults, or finalization.

Saving happens during editing, with retry during use, and flushes on pane unmount,
pagehide and document hiding. The native close gate waits for pending response
persistence. Unavailable, damaged, or full storage leaves typing usable and shows
an actionable recovery warning. A successful later save clears the warning.

## Limits

Unexpected interruption retains the last completed atomic write. Bytes still in
flight, unsent HTTP requests, or storage failures cannot be recovered. The store
inherits its existing size limit (20 MiB), atomic-file replacement and window-slot
reuse rules. Simultaneous independent clients sharing a single window slot do not
have a distributed editing/merge protocol. Moving a document or removing its
conversation can orphan recovery under the old subject/thread identity. Deleted
conversation evidence is not reconstructed from a draft. Recovery is always on
for responses, independently of the preference for recovering saved documents.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: verified within the stated limits by implementation review and automated checks.
- Originating requirement: user request for durable unsent responses in the agents
  pane, surviving closure, restart and interrupted shutdown without changing agent
  execution, review defaults or finalization.
- Implementation review: `ResponseRecovery`, `useResponseRecovery`, `ChatDock`,
  `WorkspaceChat`, `useUnsavedLifecycle`, `serve/workspace::{response_store,
  get_response,put_response}`, `WorkspaceStore::{responses,response,save_draft}`.
- Assumptions and alternatives: a response belongs to a root conversation rather
  than each rewind tip; the user explicitly requested this decision. Reusing the
  existing workspace subject for unnamed editors follows `agent_context::subject`
  and avoids invented filenames or unstable buffer provenance URIs. A separate
  response namespace reuses atomic draft storage without teaching ordinary
  document recovery to open synthetic response keys. The scope token binds writes
  to the personal store and subject resolved by the server.
- Witnesses and challenges: `responseRecovery.test.ts` exercises cold-controller
  restoration, separate roots, shared rewinds, early typing, remount ordering,
  delayed writes, rejection/acceptance, newer text, corrupt storage and retry.
  `ChatDock.test.tsx` drives rejection and acceptance through the actual composer.
  `WorkspaceChat.test.tsx` exercises the folderless untitled-editor context.
  `hickory-workspace::response_tests` performs actual disk round trips, reopens the
  store, isolates folders/windows/subjects, and challenges partial/corrupt files.
  `e2e/responseRecovery.spec.ts` drives Chromium against real HTTP/file persistence,
  recreates the page, rejects a foreign scope, discards explicitly, and checks
  that response text entered neither document drafts nor conversation evidence.
- Dependencies: implementation and test paths above in this commit; later edits
  to identity resolution, store routing, close handling or these paths stale this
  review. Native OS window closing and sudden machine power loss are reviewed,
  not driven by the browser test. Audit tools are unavailable in this session;
  these notes retain the consequential decisions and evidence instead.
- Executed checks: `just test-response-recovery` passed TypeScript, 70 frontend
  tests, 26 workspace-store tests and the CLI compile check.
  `just test-response-recovery-ui` passed both Chromium tests against a real
  engine and personal persistence. `just check-response-recovery` passed Rust
  formatting, Clippy with warnings denied and file-length checks.
  `git diff --check` passed. These checks ran on this implementation before
  commit; detached post-commit checks have their own recorded status.
- Human judgment: not recorded.

Inspected dependency SHA-256 (working tree used for the checks):

```text
b461327b0c310e9b412c2caa00dc64c97898964e668dbef019d06fc76264aa72  apps/web/src/lib/responseRecovery.ts
dd4b8366da484d79187bfd66a1c09e40d4521d5bd7c4ca9cad57217e3a2d6f53  apps/web/src/lib/responseRecovery.test.ts
23564d6fdec29992cb091bf16816a2aee5952dc0d8494a0c5df0d73063d2f705  apps/web/src/components/ChatDock.tsx
ec14d257b76d0b7e4e1e88ecb5b3fde0dc17d136b83b4c38270fcc1c2e982b50  apps/web/src/components/ChatDock.test.tsx
90eac8c74d392b0dbfc95ffe411b97670432b790a677aadc3d373a71bc7ea0aa  apps/web/src/views/WorkspaceChat.tsx
b364fe18fb45967734f79025b740b05e6b55a8c6ae96b02870bcb5742ab5851f  apps/web/src/views/WorkspaceChat.test.tsx
f4435e3bd58ec94b4e4e14b2a7147bfd73d6f88a1c59dd96c6a532726277a983  apps/web/src/views/useUnsavedLifecycle.ts
826ade25bb8f77b6693c51a86006553f2fd4d8f8ead7c60cb7b5961abcaf4d73  apps/web/e2e/responseRecovery.spec.ts
e2d75be02db915005383ae07ab8675062be7f4998c5b56b6f05d7651d7da0a22  crates/hickory-cli/src/serve/workspace.rs
9d682fc0f649915d366f1e893e9691c606c8c5c7cf7691c5f5b05234b1c5004e  crates/hickory-workspace/src/lib.rs
5ae743e60d00e3a7d8033bd602cff77712918fc95984ed32cbc6a687b8efbe7a  crates/hickory-cli/src/serve/agent_context.rs
6f7d0cc69662f6f644cfcef45dc1dacf7babef995595ab54d2c4a8c000711506  apps/desktop/src-tauri/src/folder_session.rs
```
</book:entry>

</book:doc>
