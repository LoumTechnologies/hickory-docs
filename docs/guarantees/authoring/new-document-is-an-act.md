# New Document Creates Another Unnamed Buffer

Given the app has any number of unnamed documents open, when a person chooses
File → New Document, its keyboard shortcut, or the tree's New document action,
then another blank unnamed tab opens and takes focus. Existing tabs and their
contents remain open. Tabs have distinct identities and captions (Untitled,
Untitled 2, and so on), like a conventional text editor.

The explicit request is `newDocument()`'s `hickory-new-document` event. The
workspace creates a new tab for each request. Navigation to `/new` only ensures
an unnamed buffer exists, so the event and its route change do not create two
tabs or switch focus back to an older buffer.

Each tab owns its editor, text, Save target, and recovery draft. Saving names
only the chosen tab; a Save dialog or agent edit cannot accidentally target a
different unnamed editor. Recovery keys are per tab, outside the project.
Save all visits each unnamed tab, and closing the window with Save all visits
every dirty unnamed tab. Saving outside the workspace keeps the document in
the current window. Older
single-buffer recovery drafts are migrated without losing their contents.

The startup introduction remains an unnamed document. New Document begins
blank. Typing never creates a file; Save and Save As name a `.md` document.
The first Markdown heading suggests the filename, otherwise `untitled.md`
and its numbered successors do. An untouched introduction closes without a
save prompt. Edited unnamed tabs show an asterisk and offer retention when
closed without saving.

---

Last LLM verification:
- Date: 2026-10-08
- Reviewer: Codex
- Result: verified by automated checks and implementation review.
- Validation: `just test-web` passed all 2,017 tests; the real HTTP
  `open_selected_file` test passed for folderless and folder sessions;
  `just check-desktop` passed; the native app built, passed signature
  verification, and was installed locally. Native menu/picker clicks were
  not exercised by automation.
- Evidence: `workspaceState.ts`, `useNewDocument.ts`, `useUntitledRecovery.ts`,
  `useUntitledSave.ts`, `useUnsavedLifecycle.ts`, `editorBufferTargets.ts`,
  `WorkspaceChat.tsx`, and the keyed `UntitledTab` in `WorkspaceView.tsx`.
- Tests: `just test-file-menu` exercises repeated New requests, independent
  text, per-buffer drafts and restoration, route idempotence, and close/save
  behavior using the real CodeMirror editors under jsdom.
- Boundary: native platform dialogs are typechecked and reviewed; these tests
  do not click the operating system's menu or Save dialog.
