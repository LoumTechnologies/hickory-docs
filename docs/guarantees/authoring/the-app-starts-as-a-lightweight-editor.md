# The App Starts As A Lightweight Editor

Given the app is launched without an explicit file or folder, when its window
opens, then no folder is selected, even if an older launch remembered a folder.
It opens the editable introduction as an unsaved Untitled document, supports
Agent and Save, and shows no repository setup banner. Recent files and folders
remain available for explicit opening. File → New Window uses the same editor
surface. Closing a folderless window checks unsaved work through the same close
gate as a folder window.

Given Hickory Docs opens a folder with remembered window state, the open
editors and arrangement return as recorded in
[a-folder-remembers-its-window.md](a-folder-remembers-its-window.md).
A folder with no remembered arrangement starts with the editable introduction
and Files; Agent and Welcome remain closed. The editor takes keyboard focus.
Typing creates no document in the folder; Save or Save As names a .md file.
Opening Files or Agent preserves the introduction's edits.

Given the introduction has been closed, when File → New Document is chosen,
then a blank untitled document opens. The introduction is startup content,
not a template inserted into every new document.

Given `apps/web/src/content/startup.md` is edited in a Markdown editor, when
the frontend is rebuilt, then the startup introduction contains that file's
exact contents. The Markdown file is the source for the bundled introduction.

---

Last LLM verification:
- Startup followup (2026-10-17, Codex): workspace preparation no longer gates
  desktop editor bootstrap; file operations retain their initialization gate.
  `lib/focusWhenShown.ts` focuses a new unnamed editor when its initially hidden
  spatial pane is revealed. The Chromium startup flow verifies typing during
  preparation, restoring the note and its ruler, opening Agent, and keyboard
  focus in a newly created blank note. Full startup evidence and exclusions are
  in `the-editor-does-not-wait-for-workspace-preparation.md`.
- Amendment (2026-10-14, Codex): the user requested folder-specific restoration.
  Stored arrangements now take precedence over the introductory fallback;
  the historical verification below predates this amendment. `App.test.tsx`
  exercises both fresh-folder introduction and remembered-folder restoration.
- Files default (2026-10-03, Codex): `useFolderPane` waits for layout
  hydration and confirmed folder visibility, then ensures Files is present
  once per folder session without moving keyboard focus. `App.test.tsx`
  verifies Files alongside the focused startup editor and keeps the
  folderless checks. `just test-agent-web` passed TypeScript and all 1,912
  tests; `just check-file-length` and `git diff --check` passed. The updated
  Playwright startup expectations were not run against a live app.
- Date: 2026-10-03
- Reviewer: Codex
- Result: verified in the real editor under jsdom and in the installed native
  app. An ordinary launch after a remembered `project` session opens Untitled
  with no merge banner; the live `/api/files` answers `folder_open: false`.
  Opening Agent shows `Context: Untitled (unsaved)` with no folder label.
  `just local-install` builds, signs, and installs the app.
- Evidence: `App.tsx`, `WorkspaceView.tsx`, `workspaceState.ts`, `workspaceTabs.tsx`,
  `lib/newDoc.ts`, and `apps/web/src/content/startup.md`. The introduction is
  imported with Vite's `?raw` suffix and passed to the startup buffer unchanged.
- Desktop launch: `apps/desktop/src-tauri/src/lib.rs::launch` opens internal
  editor storage when no path is requested; `open_blank` uses the introductory
  route, and all windows pass through `CloseGate`.
- Desktop HTTP coverage: `serves_one_origin::a_blank_window_has_editor_apis_but_no_open_folder`
  starts with a remembered folder and verifies editor APIs, no open folder,
  and an inapplicable merge-driver check. Native launch is verified separately
  because that HTTP test does not invoke Tauri's launch handler.
- Test coverage: `apps/web/src/App.test.tsx` mounts the real startup workspace
  and editor, tests editing and native Save/Files/Agent/New commands, and
  overrides stored pane layouts and the Welcome preference.
  `apps/web/e2e/startup.spec.ts` exercises startup, editing,
  opening tools, reopening the app, and creating a blank note;
  `workspaceState.test.ts` checks the single-pane default and on-demand Agent.
