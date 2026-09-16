# Open File Tabs Follow Renames

Given a plain file open in an editor pane, when it is renamed from Hickory's
Files tree, then its existing tab keeps its pane, position, and editor state
while its path and displayed filename change to the new name. It is one file
moving, never a close followed by a second editor opening.

---

Last LLM verification:

- Date: 2026-09-16
- Reviewer: Codex (GPT-5)
- Result: verified
- Evidence: `FolderTreePane.tsx` publishes the `{from, to}` rename; `WorkspaceView.tsx`
  applies it to the live layout; `workspaceState.ts` `renameFileTab` changes only
  the matching file-tab identity and caption.
- Test coverage: `workspaceState.test.ts` verifies tab and pane identity survive
  while the path and caption change. The event wiring is covered by typecheck.
