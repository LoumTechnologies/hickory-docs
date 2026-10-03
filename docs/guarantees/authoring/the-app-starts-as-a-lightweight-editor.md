# The App Starts As A Lightweight Editor

Given Hickory Docs starts on a folder, even one with existing documents and
previously stored Files and Agent panes, when the main window opens, then it
shows one untitled, unsaved literate document containing an editable
introduction to Hickory Docs. No Files, Agent, or Welcome pane opens beside it.
The editor takes keyboard focus. Opening the app and typing into this buffer
create no document in the folder; Save or Save As names a .md file.

Given that startup buffer, when the person opens Files or Agent, then the
chosen pane opens beside the document and the document's edits survive.
On another launch, the introduction opens in the same small starting layout.
Explicit document routes can still restore their workspace arrangement.

Given the introduction has been closed, when File → New Document is chosen,
then a blank untitled document opens. The introduction is startup content,
not a template inserted into every new document.

Given `apps/web/src/content/startup.md` is edited in a Markdown editor, when
the frontend is rebuilt, then the startup introduction contains that file's
exact contents. The Markdown file is the source for the bundled introduction.

---

Last LLM verification:
- Date: 2026-10-03
- Reviewer: Codex
- Result: verified in the real editor under jsdom; `just test-startup-web`
  passes all 45 tests and `just build-web` bundles the Markdown import.
  Live desktop/browser testing was not rerun for this extraction.
- Evidence: `App.tsx`, `WorkspaceView.tsx`, `workspaceState.ts`, `workspaceTabs.tsx`,
  `lib/newDoc.ts`, and `apps/web/src/content/startup.md`. The introduction is
  imported with Vite's `?raw` suffix and passed to the startup buffer unchanged.
- Test coverage: `apps/web/src/App.test.tsx` mounts the real startup workspace
  and editor, tests editing and native Save/Files/Agent/New commands, and
  overrides stored pane layouts and the Welcome preference.
  `apps/web/e2e/startup.spec.ts` exercises startup, editing,
  opening tools, reopening the app, and creating a blank note;
  `workspaceState.test.ts` checks the single-pane default and on-demand Agent.
