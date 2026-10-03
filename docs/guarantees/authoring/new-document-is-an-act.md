# New Document Is An Act, Not A Place

Given the app open on any folder, when a person asks for a new document —
File → New Document, the `+` at the top of the file tree, or the welcome
page's "New document…" — then an untitled buffer opens and takes focus,
every time they ask: the first time, the second time while the first
Untitled tab is still open (it is re-activated rather than duplicated),
and again after that tab has been closed.

It did not. Every entry point was `navigate("/new")`, and the hash is a
place: setting it to where it already is changes nothing, so once the
address said `#/new` — which it did from the first New Document onward —
every further New Document did nothing at all. Closing the Untitled tab
and asking again was the sharpest form: nothing happened, with no message.

Two properties hold it up:

1. **One function, one event.** `router.ts`'s `newDocument()` fires
   `hickory-new-document` on `window` and then navigates to `/new`. All
   three entry points call it; none navigates on its own.
2. **The workspace answers the event, not only the route.** The route
   effect still opens an untitled buffer when the address changes to
   `/new` (a deep link, a restart), and a listener opens one whenever the
   event fires. `openUntitledTab` re-activates an existing Untitled tab
   rather than making a second, so the two firing together on a first
   request is harmless.

## Boundary

One Untitled buffer at a time is deliberate, as with the scratchpad: a
second unnamed buffer splits a thought across two places. File → New begins blank and never reads an older unnamed draft. The app's
startup buffer instead contains an editable introduction to Hickory Docs. Typing never creates
`untitled.md`; **Save** and **Save As** open the platform's file-save dialog,
initially in the folder Hickory Docs has open, and only then create the
document. The first Markdown heading supplies its suggested filename; without
one, the suggestion is `untitled.md` (then `untitled-2.md`, and so on).
The dialog remains free to choose any disk location. Saving outside the open
folder changes this window to that file's folder, because one Hickory session
owns one folder. Once it contains text, its tab reads `Untitled *`. Closing
the tab or window asks whether to save; closing without saving discards the
unnamed text.

---

Last LLM verification:
- Date: 2026-09-16
- Reviewer: Codex (GPT-5)
- Result: verified
- Evidence: `apps/web/src/router.ts` — `NEW_DOCUMENT_EVENT`, `newDocument`;
  `apps/web/src/App.tsx` menu case `"new"`; `apps/web/src/views/WorkspaceView.tsx`
  — the new-document event, `saveUntitled`, and the native Save/Save As
  routing; `apps/web/src/views/useUntitledSave.ts` `useUntitledSave`;
  `apps/web/src/lib/newDoc.ts` `untitledSaveName`; `serve/shell.rs`
  `save_file_dialog`; and the desktop `server.rs` shell hook.
- Test coverage: `apps/web/src/router.test.ts` ("fires the event even when
  the hash is already #/new"); `apps/web/src/views/workspaceTabs.test.tsx`
  ("does not create a project file on its first keystroke" and no prior-draft
  restore); `apps/web/src/lib/newDoc.test.ts` (heading filename suggestion);
  and `apps/web/src/views/useUnsavedLifecycle.test.tsx` (discard wording and
  close behavior).
  The native dialog is checked by the desktop crate's typecheck; it has no
  headless platform-dialog test.
  The native prompt and document-creation handoff are covered by typecheck
  and implementation review; they do not yet have a browser-level test.
