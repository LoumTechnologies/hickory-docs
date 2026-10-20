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

The startup introduction is an unnamed fallback when no draft is restored.
Restoring unnamed documents reuses that fallback for the first draft instead
of adding another introduction on each launch, regardless of draft order or
changes to the bundled introduction. New Document begins
blank. Typing never creates a file; Save and Save As name a `.md` document.
The first Markdown heading suggests the filename, otherwise `untitled.md`
and its numbered successors do. An untouched introduction closes without a
save prompt. Edited unnamed tabs show an asterisk and offer Save, Discard, or Cancel.
Discard removes their recovery draft; explicitly closed notes do not return.

---

Last LLM verification:
- Date: 2026-10-08
- Reviewer: Codex
- Result: verified by automated checks and implementation review.
- Follow-up validation: `just test-web` passed all 2,021 tests and
  `just test-file-menu` passed 74 frontend tests and its real HTTP test. This
  includes three consecutive recovery launches without a duplicate introduction,
  explicit tab/window discard, and in-flight recovery write ordering. The updated
  app built, passed signature verification, and was installed locally.
- Original validation: `just test-web` passed all 2,017 tests; the real HTTP
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

## New-file creation refuses overwrite — 2026-10-20

Naming an unnamed buffer creates a new file; it does not truncate an existing
file, even when a native chooser returns an absolute existing path. Both absolute
and folder-relative document creation use atomic `create_new`, so another writer
winning after target inspection causes a conflict. Final symlinks, including
ones whose targets are absent, are refused rather than followed for creation.
Open the existing file to edit it, or choose a new filename. A failure after
creation reports that partial new bytes may remain; it does not assert no effect.

Retrospective correction: the route's earlier refusal comment was stronger than
its implementation. The absolute branch used an overwriting write, and the
relative branch's existence check was not atomic with writing. This review found
and corrected those paths while auditing issue #72's native chooser equivalents.
No model request is treated as native overwrite consent.

LLM review: partially verified. Inspected `serve/api.rs::create_doc` dispatching
both branches to `serve/api/create_new.rs::write`. Two focused Rust cases passed
via `cargo test -p hickory-cli --lib serve::api::create_new::tests --quiet`:
concurrent temporary-file creates produce exactly one complete winner and a
conflict, and Unix final/dangling symlinks preserve existing bytes/absent targets.
These are real temporary filesystem operations, not an HTTP/native-dialog
witness. Parent-directory symlink changes, disk-full partial writes, power-loss
durability and Windows symlink behavior are not certified. Human judgment and
real native chooser validation are unrecorded.

Evidence dependency SHA-256 values (changed bytes invalidate this checkpoint):

- `crates/hickory-cli/src/serve/api.rs`: `b54aff5268ce5a0595aab3d79bdf5ea75046dcf93c4deae4940058302fb8ee82`
- `crates/hickory-cli/src/serve/api/create_new.rs`: `516e58a44d1cc9c3a1cf083169ebb4513fede55aacb33d135b9a73b803428e8d`
