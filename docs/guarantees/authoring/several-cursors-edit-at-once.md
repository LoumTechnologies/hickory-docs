# Several Cursors Edit At Once

Given any editor in the app — a document, a plain file, a generated file —
when the person Alt+clicks, Alt+drags a column, or presses Ctrl+D (Cmd+D) on
a selection, then a second cursor or selection is added rather than the first
moved, Ctrl+Shift+L selects every occurrence, and typing, deleting and
pasting act at every cursor at once.

CodeMirror holds one selection unless told otherwise, and the browser can
draw only one. The three parts — allow many, draw them, let a rectangle
become a column — ship together, because any one alone is a feature that
looks broken.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/editor/multiCursor.ts` (`multipleCursors`:
  `allowMultipleSelections`, `drawSelection`, `rectangularSelection`,
  `crosshairCursor`), mounted in `apps/web/src/editor/DocumentEditor.tsx`,
  `apps/web/src/components/PlainFilePane.tsx` and
  `apps/web/src/components/OutputEditorPane.tsx`; the Ctrl+D and
  Ctrl+Shift+L bindings are `@codemirror/search`'s `selectNextOccurrence` and
  `selectSelectionMatches` in the `searchKeymap` every editor already wears.
- Test coverage: `apps/web/src/editor/multiCursor.test.ts`.
- Caveats: the mouse gestures are CodeMirror's own and are not driven by a
  test here.
