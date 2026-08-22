# A Keystroke Redraws Only The Editor

Given a document open in the workspace beside any number of other panes, when
a character is typed into it, then the only React render that keystroke causes
is inside the editor pane that received it — the workspace chrome (the other
panes, the folder tree, the rulers, the rails, the ribbon overlay, the status
bar's other fields) is not re-rendered, and nothing that was already on screen
is re-measured against the DOM.

This is what "snappy" means mechanically. The editor itself is CodeMirror and
was always cheap per keystroke; what made typing feel heavy was everything
around it waking up: the document session re-publishing itself to the whole
window on every change, the overlay re-deriving every ribbon from provenance
and re-reading every pane's geometry, the embedded-language highlighter
re-parsing every code block on screen, and a caret poll re-rendering the
workspace eight times a second whether or not anything had moved.

Four rules keep it true:

1. **A session is published by value, not by render.** `SessionRegistry.publish`
   compares the new session's fields to the one it holds and stays silent when
   they are all the same. The session hook therefore keeps every field
   referentially stable unless it changed: callbacks that need the live text
   read it through a ref rather than closing over it, and composite values
   (the debugger's state) are memoized into one object per distinct state.
2. **The one thing that changes per keystroke lives alone.** The caret
   position is polled by its own component inside the status bar, and it
   re-renders only when the line or column actually changed. Nothing above it
   in the tree holds it as state. Inside the editor pane, the only per-keystroke
   state is *which rendered table the caret is in* — the ruler needs that —
   never the caret position itself.
3. **Derived work is keyed by content, not by document version.** Embedded
   highlighting is cached by (language, code); a keystroke in the prose above
   a block does not re-parse the block. Byte↔char offset conversion takes the
   identity fast path for ASCII text instead of walking the string for every
   provenance span.
4. **Measurements that repeat must commit only differences.** The ribbon
   overlay measures on a timer as a last resort; the measurement sets state
   only when a shape actually moved, and its source object is memoized so a
   render of the workspace is not itself a reason to re-measure. Memoized
   components (the ruler) are handed stable callbacks.

A fifth rule follows from the first: **a render of the workspace is allowed to
be expensive**, because it is rare — it happens when a pane opens, a layout
changes, a run finishes, the server's copy of a document catches up. It must
not happen because a key was pressed.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude (Fable 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `apps/web/src/views/documentSession.tsx` — `SessionRegistry.publish`
  and `sameSession`; `liveSourceRef` feeding `openTarget`, `wordAt`,
  `makeOutputLsp`. `apps/web/src/debug/useDebugger.ts` — the memoized return.
  `apps/web/src/shell/StatusBar.tsx` — `CaretPosition`, the poll and its
  equality check. `apps/web/src/editor/DocumentEditor.tsx` — `caretTableAt`
  replacing the caret-position state; `onWrapColumnStable` for the memoized
  `EditorRuler` (`apps/web/src/editor/EditorRuler.tsx`).
  `apps/web/src/editor/embedded.ts` — the (language, code) cache.
  `apps/web/src/lib/offsets.ts` — the ASCII fast path.
  `apps/web/src/shell/Ribbons.tsx` — `sameShapes`; `apps/web/src/views/WorkspaceView.tsx`
  — `ribbonSource` memo, the lazily loaded terminal pane.
  Tests: `apps/web/src/views/sessionRegistry.test.ts` (publish dedupe),
  `apps/web/src/lib/offsets.test.ts` (fast path equals the walk),
  `apps/web/src/editor/embedded.test.ts` (cache identity),
  `apps/web/src/shell/StatusBar.test.tsx` (the caret is the bar's own).
- Measured, in the Vite dev build under Playwright, typing 47 characters into
  the seeded `weave-demo.hick` with the folder tree and a Welcome tab open:
  before, two long tasks of 220 ms and 242 ms and ~296 ms of React work;
  after, no long task at all and ~67 ms of React work, with the ribbon
  derivation and the embedded re-highlight gone from the profile entirely.
- Caveat requiring review: "no React render outside the editor pane" is
  argued from the state graph and the profile, not asserted by a test that
  counts renders; a future field added to `DocSession` that changes identity
  per render (an inline object or closure) would silently reopen the path, and
  `sessionRegistry.test.ts` would not notice. Whole-document work that still
  runs per keystroke inside CodeMirror — the structure parse, the layout and
  widget decoration fields — is linear in document size and was not the cost
  measured here; it is the next thing to look at if a very long note lags.
