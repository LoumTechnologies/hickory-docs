# Markdown Quotes And Task Lists Are Read As Prose

Given any buffer this app styles as markdown — a `.hick` document's prose, a
woven `.md` output pane, a plain markdown file — when a line opens with one or
more `>` markers, then that line is drawn as a quote at its nesting depth; and
when a list item's marker is followed by `[ ]` or `[x]`, then its box is drawn
as a checkbox that a single click toggles in the source.

Both are prose features, and "prose" is decided by the same scan that already
decides what a heading is. That is the whole point of putting them in
`scanMarkdownProse` rather than beside it: a `- [ ]` inside an exec cell's
payload is an argument to a command, a `>` inside a fenced block is a shell
prompt, and neither may grow a checkbox or a quote rule. A feature that
guessed separately would eventually disagree with headings about where prose
is, and the disagreement would show up as a checkbox in the middle of a
command.

Four rules keep it:

1. **A quote prefix is stripped before anything else reads the line.** So
   `> ## Heading` is a heading inside a quote and `> - [x] done` is a
   completed task inside a quote, rather than three features each refusing to
   see the others. What remains is scanned at its real document offset, so
   every range the scan returns still points at the bytes it describes.
2. **Quotes are scanned per line, not per paragraph.** The decoration that
   tints a quote is a line decoration, and a person splitting a quote in half
   as they type wants both halves to stay quoted — which paragraph grouping
   would fight on every keystroke.
3. **The styling half never hides text.** `markdownStyling.ts` emits line
   classes and marks only. The `>` markers and the `[ ]` box stay in the
   buffer, dimmed as chrome, exactly as heading `#` marks already are.
4. **The checkbox is an inline replacement, and it steps aside for the
   caret.** `taskList.ts` replaces the three characters of the box with a
   checkbox of the same span — never a block widget, so no screen row appears
   without a document line behind it (see
   [the-gutters-never-skip-a-number](the-gutters-never-skip-a-number.md)) —
   and the box on any line a selection touches is left as source, because
   text you cannot see is text you cannot fix.

A click toggles only the middle character of the box. `[X]` therefore clears
to `[ ]`, and an empty box fills to the lowercase `[x]` every markdown
renderer agrees about; the bullet, the indent, and the item's text are never
rewritten.

## Boundary

Nesting deeper than three `>` reuses the third tint. The indent still grows,
so depth is still visible; a fourth shade of the same grey carries nothing a
reader could act on.

The toggle rescans the buffer to find the task under the click rather than
trusting an offset captured when the widget was built. A widget survives edits
made above it, and a two-second-old offset can point into the middle of a
word.

This says nothing about *rendering* a quote or a task list to HTML. Weaving is
`hick weave`'s job and goes through a real markdown renderer; this guarantee
is about what the editor draws over the source.

Ordered-list task items (`1. [ ]`) are recognised, but the number is left
alone when the box is toggled — renumbering a list is an edit nobody asked
for.

---

Last LLM verification:
- Date: 2026-08-20
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `apps/web/src/editor/hickDoc.ts` — `QUOTE_RE`/`TASK_RE` and the
  rewritten `scanMarkdownProse`, which strips the quote prefix into `base`
  before the heading, task, and inline scans run; `QuoteLine`/`TaskItem` are
  carried on `HickDocStructure` so the `.hick` editor and the plain-markdown
  panes share one definition of where prose is.
  `apps/web/src/editor/markdownStyling.ts` — `mdQuoteLines`, `mdTaskLine`,
  `mdTaskDoneLine`, and the two loops added to `proseDecorationRanges`;
  decoration-only, as the module header states.
  `apps/web/src/editor/taskList.ts` — `toggleTaskAt` (pure, text in / change
  out), `CheckboxWidget` (inline replacement, `posAtDOM` at click time), and
  `taskCheckboxes(source)`, whose `update` rebuilds on selection as well as
  on content so the caret-reveal works.
  Wired at `apps/web/src/editor/DocumentEditor.tsx` (with
  `structureOf(state).tasks`, so cell payloads are excluded),
  `apps/web/src/components/OutputEditorPane.tsx`, and
  `apps/web/src/components/PlainFilePane.tsx`.
  Styles: `.cm-md-quote*`, `.cm-md-task*`, `.cm-md-checkbox*` in
  `apps/web/src/styles.css`.
- Test coverage: `apps/web/src/editor/taskList.test.ts` (13 tests) — the scan
  (bullet/ordered/checked, the bare-bracket rejection, the whitespace-after
  rule, a task inside a quote, and the exec-payload exclusion asserted
  through `parseHickDoc`), the toggle (both directions, a click anywhere on
  the line, `[X]` normalisation, null off a task line), and the decoration
  mounted in a real `EditorView` (caret-line reveal, glyphs, a `mousedown`
  that edits the document).
  `apps/web/src/editor/markdownStyling.test.ts` — the six "block quotes"
  cases: depth and marker run, prose styled inside a quote, a heading inside
  a quote, the fenced-code exclusion, the emitted classes, and the depth cap.
- Caveat requiring review: the visual result is asserted only as class names
  and DOM structure — jsdom does no layout, so the quote rules' indent and the
  checkbox's baseline alignment were checked by eye rather than by a test.
  Lazy continuation (a quote paragraph whose second line omits its `>`) is
  not implemented: such a line is styled as ordinary prose, which is what the
  source literally says but not what CommonMark renders.
