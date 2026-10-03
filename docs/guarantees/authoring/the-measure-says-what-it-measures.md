# The Measure Says What It Measures

Given the ruler above a document editor, when it is measuring prose then it
names its unit (`chars/line`), shades the range that is comfortable to read
(45–75), and says in its tooltip that the number is an average of a
proportional face rather than a count of columns. Given a caret inside a
table, the same strip stops measuring prose entirely and names the table's
columns instead.

Given an untitled document, including the initial introduction, when the
reader releases a dragged marker then it stays at that measure and prose
wraps there. The measure is stored in workspace UI state under `untitled`
and restored on the next launch, just as a saved document's measure is stored
under its path. Moving the marker does not change document bytes.

## The question this answers

The strip used to be a row of unexplained numbers, and the reasonable readings
of it were all wrong. Not inches: a CSS pixel is not a physical unit, so an
inch scale on a screen is a number that looks authoritative and measures
nothing. Not monospace columns either — this editor sets prose in the system's
proportional face, so an `i` and an `m` are different widths and there is no
column grid for a tick to fall on.

What it measures is the typographer's measure: characters per line. That is
the number five centuries of setting text actually cares about, and the only
one that transfers between faces. It is an average by construction —
CodeMirror's `defaultCharacterWidth` is the content element's real font
measured over a sample — so "72" means "about 72 characters of this face, at
this size, fit on a line". A line of `l`s holds more and a line of `M`s fewer.
That is a property of the question, not a defect in the answer, and the ruler
says so rather than implying a precision it does not have.

## The rules

1. **The unit is written on the ruler**, at its left end, in the lane the
   gutter occupies below. A measuring stick whose unit is not on it is
   furniture people learn to ignore.
2. **45–75 is shaded.** A number with no band around it is a number nobody can
   judge; with one, the marker's position means something at a glance.
3. **The word is "about".** In the tooltip and in `aria-valuetext`, so it
   reaches a screen reader too.
4. **The marker is still a real control** — a slider with a value, arrow keys,
   and shift for ten — because a margin you can only set by dragging is one
   nobody can set precisely and nobody without a mouse can set at all.
5. **Inside a table, the prose furniture goes away entirely.** The unit label,
   the band, the ticks, and the marker are all hidden: a measure over a grid
   is measuring the wrong thing, and leaving any of it visible would be a
   claim about a block that never wraps.
6. **Code is not measured.** The measure applies to prose lines; a code line
   drops it, keeps its lines, and takes the whole pane.

## Boundary

The ruler measures the editor's own content element, so it follows a font-size
change (the zoom control) and a pane divider without being told. It does not
follow a font change made outside the app mid-session — nothing does, and the
next layout picks it up.

Nothing persists the reader's opinion of the band. 45–75 is the range every
typography text gives for continuous reading, and making it configurable would
be a setting to defend rather than a fact to state.

---

Last LLM verification:
- Date: 2026-10-03
- Reviewer: Codex
- Result: verified for the untitled wrap-setting path by implementation review
  and regression tests.
- Evidence: `WorkspaceView.tsx` passes `workspaceUi.wrapFor(tab.target)` and
  `workspaceUi.setWrap` to `UntitledTab`; `workspaceTabs.tsx` forwards both to
  `DocumentEditor`. `EditorRuler` reports the final drag value, synchronizes
  the CodeMirror wrap field, and `useWorkspaceUi` debounces persistence.
- Test coverage: `apps/web/src/App.test.tsx` drives the real startup ruler
  through pointer down/move/up, checks the released value and editor wrap
  field, unchanged source bytes, and stored UI state. `workspaceTabs.test.tsx`
  covers forwarding; `apps/web/e2e/startup.spec.ts` covers browser dragging,
  persistence, and restoration after reload against the live dev app.

Earlier verification of the ruler's labels and shaded range:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `apps/web/src/editor/EditorRuler.tsx` — the module doc (which is
  where the reasoning above lives in the code), `READABLE_MIN` /
  `READABLE_MAX`, the `band` computation from the same `metrics.charWidth` the
  ticks and the marker use, the `editor-ruler__unit` label, and the marker's
  `aria-valuetext` and `data-tip`. The `naming` flag hides all of it inside a
  table (rule 5).
  `apps/web/src/editor/wrapColumn.ts` — the measure itself, and `.cm-code-line`
  opting out of it (rule 6).
  `apps/web/src/styles.css` — `.editor-ruler__unit`, `.editor-ruler__band`.
- Test coverage: `apps/web/src/editor/EditorRuler.test.tsx` (10) — the slider's
  value and bounds, arrow keys, the clamp, the measure reaching the editor,
  the unit label, the "average, not a column count" wording in both the
  tooltip and `aria-valuetext`, the shaded range named in the tooltip, and the
  band being absent inside a table.
- Caveat requiring review: the band's PIXEL placement is not asserted — jsdom
  does no layout, so `defaultCharacterWidth` is zero and the band has zero
  width there. That it lands under the right ticks follows from sharing one
  `metrics` object with them, and was checked by reading rather than by test.
