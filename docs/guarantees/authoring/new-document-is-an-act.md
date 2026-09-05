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
second unnamed buffer splits a thought across two places. A person who
wants two new documents saves the first, which names it.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/router.ts` — `NEW_DOCUMENT_EVENT`, `newDocument`;
  `apps/web/src/App.tsx` menu case `"new"`; `apps/web/src/views/WorkspaceView.tsx`
  — the tree's `onNewDocument`, the welcome action, and the
  `NEW_DOCUMENT_EVENT` listener beside the other window events;
  `apps/web/src/views/workspaceState.ts` `openUntitledTab`.
- Test coverage: `apps/web/src/router.test.ts` ("fires the event even when
  the hash is already #/new"). Found by reproducing in the built app on
  2026-09-05: dispatching the menu's `hickory-menu: new` twice, and once
  more after closing the Untitled tab.
