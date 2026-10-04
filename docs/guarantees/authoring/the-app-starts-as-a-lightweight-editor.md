# The App Starts As A Lightweight Editor

Given the app is launched without an explicit file or folder, when its window
opens, then no folder is selected, even if an older launch remembered a folder.
It opens the editable introduction as an unsaved Untitled document, supports
Agent and Save, and shows no repository setup banner. Recent files and folders
remain available for explicit opening. File → New Window uses the same editor
surface. Closing a folderless window checks unsaved work through the same close
gate as a folder window.

Given Hickory Docs starts on a folder, even one with existing documents and
previously stored Files and Agent panes, when the main window opens, then it
shows one untitled, unsaved literate document containing an editable
introduction to Hickory Docs. Files opens beside it by default once the folder
and stored layout have loaded; Agent and Welcome remain closed. Files can be
closed for the rest of that window's folder session.
The editor takes keyboard focus. Opening the app and typing into this buffer
create no document in the folder; Save or Save As names a .md file.

Given that startup buffer, when the person opens Files or Agent, then the
chosen pane opens beside the document and the document's edits survive.
On another launch, the introduction opens with Files if a folder is open.
Explicit document routes can still restore their workspace arrangement.

Given the introduction has been closed, when File → New Document is chosen,
then a blank untitled document opens. The introduction is startup content,
not a template inserted into every new document.

Given `apps/web/src/content/startup.md` is edited in a Markdown editor, when
the frontend is rebuilt, then the startup introduction contains that file's
exact contents. The Markdown file is the source for the bundled introduction.

---

Last LLM verification:
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
