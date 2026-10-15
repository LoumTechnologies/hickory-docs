# A Folder Remembers Its Window

Given a folder previously opened in Hickory Docs,
when it is opened again,
then its open documents, files, active view, pane arrangement, spatial tiles,
drawer pins and widths, prose measures, table sizes and tab zoom return.
Files stays closed if the remembered arrangement left it closed.
A fresh folder uses the normal introductory defaults.

Given a folder is opened in a window already editing notes,
when its remembered arrangement is loaded,
then existing editors are retained, including edited unnamed notes. Only the
untouched startup introduction is replaced. Further layout saves belong to
that selected folder, including when the window began without a folder.

Given the person closes the window,
when the close gate approves closing,
then the latest arrangement is written before the native close request.
Changes are also saved during use, and pagehide requests a keepalive write.

State lives under the user's data directory, keyed by canonical folder path,
and never in project files. Simultaneous windows on the same folder retain
separate slots; reopening normally reuses the first available slot. Draft
recovery remains separate from layout; existing blank-window drafts continue
using their original recovery store during the first folder attachment. A missing or unreadable store permits the window
to open using defaults.

---

Last LLM verification:

- Date: 2026-10-14
- Reviewer: Codex
- Result: verified by implementation review and automated checks; native
  window interaction has not been manually exercised.
- Originating requirement: the user requested layout and open documents per
  folder, returning to the last arrangement when opening that folder.
- Implementation review: `WorkspaceStore::for_project` and `window` isolate
  folders and concurrent windows. `folder_session::Session::attach_folder`
  routes layout to the folder engine while retaining existing document rooms
  and drafts. `useWorkspaceUi` reloads when folder identity changes, cancels
  stale loads, serializes writes and exposes a close flush.
  `restoreFolderLayout` carries existing editors into stored geometry;
  `layout::reid` preserves unnamed recovery identities. `useUntitledSave`
  publishes its named tab before returning success to the close gate.
  `SpatialShellView`
  accepts persisted scene state through `WorkspaceUi`, so reopening on another
  loopback port does not depend on that port's browser storage.
- Witnesses: `useWorkspaceUi.test.tsx`, `restoreFolderLayout.test.ts`,
  `folderPane.test.tsx`, `useUnsavedLifecycle.test.tsx`, `App.test.tsx`, and
  `SpatialShellView.test.tsx` and `useUntitledSave.test.tsx` cover restoration,
  stale loads, quick close,
  preserved editors and spatial center. `serves_one_origin` includes a real
  HTTP check for the first folder attached to a blank window.
- Assumptions and exclusions: folder identity is its canonical path, not its
  basename or Git repository; nested folders and worktrees have independent
  state. Multiple simultaneous windows preserve the existing slot model.
  Layout persistence does not recreate terminated terminal processes or
  persist transient generated readings. Storage failure and abrupt process
  termination before a save completes remain best effort. Unexpected-crash
  recovery of a blank-window draft requires its original recovery store; this
  change does not migrate draft contents into the folder.
- Challenges: replacing a pristine introduction must not replace an edited
  note; a late load must not overwrite another folder; restoration must not
  add Files or Welcome over a deliberately closed arrangement.
- Executed checks: initial TypeScript and 54 frontend tests passed. The
  final `just test-folder-window-web` run passed TypeScript and all 79 tests. `just test-folder-window-state` passed
  the production UI build and the native HTTP regression (1 test).
  `just check-desktop` passed formatting and Clippy with warnings denied.
  `just check-file-length` and `git diff --check` passed. A recovery test
  originally accepted any nonempty editor and could read the previous tab;
  its assertion now waits for the selected draft's exact contents. One
  concurrent run exceeded the test timeout; the web recipe bounds workers.
- Human judgment: not recorded; native window reopening not manually exercised.
- Dependencies: the files named above, at their content hashes recorded below.
  Evidence becomes stale when these paths change.

Content hashes (SHA-256, reviewed working tree):

```text
241c1904a6a28146d9fdecf1e56cf72e9a38fdbd94d9e56b8395472cf0619f19  crates/hickory-workspace/src/lib.rs
6f7d0cc69662f6f644cfcef45dc1dacf7babef995595ab54d2c4a8c000711506  apps/desktop/src-tauri/src/folder_session.rs
ccb3f90ef57026a27a2cd60ac162f325dd3698a465086102861e2c6053063ff3  apps/desktop/src-tauri/tests/serves_one_origin.rs
3cb4397863b2b9057a1e5974bc9772af6e4bcf02bd3b5e5b579b2c77b4aa0d4a  apps/web/src/views/WorkspaceView.tsx
2a3878108d11dd46171b627dda6bccac8cf42fa40093a0da00d357eebba46a31  apps/web/src/views/useWorkspaceUi.ts
d0cf53cdf9001699d923e47d76201e3d529710caf7ad6c646525b98ccbd6d3b2  apps/web/src/views/useWorkspaceUi.test.tsx
29a87326d282960a8decab58adf05e8c7b0bce747f1e91479adf998076d52798  apps/web/src/views/restoreFolderLayout.ts
d329014a92e7c110e306e476321594204083be244736cd69de71149a890f914e  apps/web/src/views/restoreFolderLayout.test.ts
075bcbc88f959eb53ec36f9119a97c0fb3cb603a6ba2a2c437f62c692de91a85  apps/web/src/views/useUnsavedLifecycle.ts
69d4103309f42725ee5eda10f1c9572a42249830f8119f65fc69335d8f8487c8  apps/web/src/views/useUnsavedLifecycle.test.tsx
4130751891a62209ed3be7eee0686e732215f430a09c5984f0492fb997d07762  apps/web/src/views/useUntitledSave.ts
0dd7ef755ff0dba810e71dcb5e3709329ac29804982e1fd78d586b69432b4188  apps/web/src/views/useUntitledSave.test.tsx
acea8d622c940fcbd73565dd8d9e54a1b3e15f805e75682b4b48b4fc64485990  apps/web/src/shell/SpatialShellView.tsx
600ae74c1cabe86cf227dbb546139df9bf0c34cca25421d0825557883d7acea5  apps/web/src/shell/SpatialShellView.test.tsx
5ac708d1a397e2caede886c5a3d1df25d5b9e8021bedcf3edb9e84e11212944e  apps/web/src/shell/layout.ts
b1e6d19151dad26a80ef5c819fbaba097666bd34f6eb41dd259a1373d6f25de4  apps/web/src/lib/uiState.ts
715c4d4a2663fb62bea47a496de36175914111a395a2c847f471bd60a9997841  apps/web/src/App.test.tsx
```

Integration review (2026-10-15, Codex): merged with master at `63149bc`.
The spatial editor retains the newer drawer-context and pin-alignment behavior.
`just test-folder-window-web` passed 79 tests and TypeScript; `just test-spatial`
passed 65 tests and TypeScript in the main checkout. File-length and diff checks
passed. The spatial editor hash above describes the pre-merge revision; its
merged SHA-256 is `58c2977e98a176d064b42a4d1680589bb3075e02c578d2877334c6a3a92ed954`.
Native window interaction has not been manually exercised.

Integration correction (2026-10-15, Codex): the full pre-commit suite exposed
a delayed editor-ready callback focusing a hidden restored unnamed note after
the person selected another note. `UntitledTab` now focuses its ready editor
only when it has no hidden ancestor. `workspaceTabs.test.tsx` verifies that a
hidden editor stays unfocused while a visible editor still receives focus.
The earlier targeted results did not cover this delayed initialization path.
- `apps/web/src/views/workspaceTabs.tsx` SHA-256: `44b9c887e930465dfa7f25c370b906550c146656222ddd5dfcb464550f6bc848`.
- `apps/web/src/views/workspaceTabs.test.tsx` SHA-256: `5d7616e48f11a76057ed805f9a55480afd03fefabfbf08c23aafe05cef6c5ba6`.

After the focus correction, `just test-folder-window-web` passed TypeScript
and 88 tests in the main checkout. The full pre-commit run before that
correction had one failing editor-focus regression out of 2,124 web tests;
its browser-embedding checks (33), collab tests (7), and workspace-filesystem
tests (8) passed. The merge commit reruns the full hook with the fix.

A subsequent full-hook attempt exhausted temporary disk space (`ENOSPC`):
69 web test files could not load, while 1,412 executed tests passed. Generated
Rust build directories belonging to the implementation worktree were removed
to recover approximately 14 GB before retrying. This was an environment
failure, not verification of the skipped tests.
