# An Equation Renders And Shows Its Source

Given LaTeX written in a document this app styles as prose, when the caret is
not in it, then it is typeset where it was written; and when the reader wants
the LaTeX back, there is always a gesture that returns it without retyping
anything.

Which gesture depends on how big the equation is, and that split is the point:

- **Inline and prose maths** (`$x^2$`, and `$$…$$` written in prose) reveals
  its source when a selection touches its line — the same rule the task
  checkbox follows, for the same reason. An equation you cannot see the source
  of is an equation you cannot fix, and moving the caret into it is the
  cheapest possible way to ask.
- **A `<hick:math>` block** is a paragraph, and gets the heavier treatment: it
  is a rendered block with a `∑` icon on the action rail beside it, exactly as
  a `<hick:diagram>` has `◈`. A reader who wants the LaTeX of a displayed
  equation should not have to put the cursor inside it to get it, and the rail
  is the one column that exists to be clicked.

Three rules keep it honest:

1. **Rendered maths replaces its own source and nothing else.** Inline maths
   is an inline replacement inside its line. Display maths is a *block*
   replacement — a fold, taking the rows it occupies rather than adding one —
   and only when its `$$` markers own their lines; display maths written
   mid-line renders inline instead, because a block replacement that does not
   cover whole lines is not something CodeMirror can take out of its height
   map. Either way the gutters keep counting truthfully (see
   [the-gutters-never-skip-a-number](the-gutters-never-skip-a-number.md)).
2. **A dollar sign is a dollar sign until proven otherwise.** `$5 and $6` is
   money and `$PATH` is a shell variable, and both are far more common in
   these documents than inline maths. Maths opens only on a non-space, closes
   only on a non-space, never closes immediately before a digit, never crosses
   a line, and never appears inside a verbatim range — which is what keeps
   `echo $PATH` in an exec cell a command.
3. **The woven markdown carries the equation, not its machinery.** A
   `<hick:math>` block weaves to `$$…$$`, which GitHub and every editor
   preview already render. The tag does not survive the weave.

## Boundary

KaTeX is loaded lazily, stylesheet and fonts included, because the marketing
site builds from this source tree and a page with no equations must not carry
a typesetting engine. Until it arrives — and on any build where it fails to
load — the widget shows the LaTeX itself, marked as untypeset. Nothing ever
flashes empty.

LaTeX that does not parse is *not* an error state here. Half-typed maths is
the normal condition of maths being written, so KaTeX runs with
`throwOnError: false` and draws the parts it understood.

This says nothing about numbering equations, `\ref`, or a macro preamble
shared across a document. Each `<hick:math>` block and each `$…$` span is
typeset alone.

`\(…\)` and `\[…\]` are not recognised. One notation for each of inline and
display is enough, and it is the one the woven markdown already uses.

---

Last LLM verification:
- Date: 2026-08-20
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `apps/web/src/lib/math.ts` — `mathSpans`, and the false-positive
  rules in `findInlineClose`, which are the whole reason this is a hand-rolled
  scanner rather than a regex.
  `apps/web/src/editor/mathRender.ts` — the lazy engine loader (shared by the
  block panel), `MathWidget` (identity is the LaTeX, never the position),
  `ownsItsLines` (the block-vs-inline decision), and `renderedMath(source)`,
  whose `update` rebuilds on selection so the caret-reveal works.
  `apps/web/src/components/MathPanel.tsx` — the `<hick:math>` block's panel;
  no controls, because the rail owns the verbs.
  Card and fold wiring: `apps/web/src/editor/cards.ts` (`"math"` card kind),
  `apps/web/src/lib/railActions.ts` (`["source"]`),
  `apps/web/src/editor/CardRail.tsx` (the `∑` glyph),
  `apps/web/src/editor/rendered.ts` (`renderableBlocks`, the `math` count and
  its own `estimatedHeight`), `apps/web/src/editor/DocumentEditor.tsx` (the
  math slot portal, and `renderedMath` fed the document's `verbatimRanges`).
  Weave: `crates/hick-literate/src/weave.rs` — the `"math"` arm, whose
  children go through `process_file_children_to_weave` so a `<hick:paste>`
  inside an equation resolves.
  Insert menu, both halves: `apps/web/src/lib/insertCatalog.ts` and
  `apps/desktop/src-tauri/src/lib.rs::INSERT_GROUPS` (kept in step by
  `apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs`).
  Styles: `.math-panel`, `.math-figure`, `.cm-math*` in
  `apps/web/src/styles.css`; `katex` is now a declared dependency rather than
  a thing that happened to be in the tree under mermaid.
- Test coverage: `apps/web/src/lib/math.test.ts` (9 tests) — the two
  notations, and six cases that are all about NOT finding maths: a pair of
  prices, a space after the opener, a space before the closer, an unclosed
  `$` that must not swallow the document, escaped delimiters, and a verbatim
  range.
  `apps/web/src/editor/mathRender.test.ts` (7 tests) — inline replacement,
  the caret reveal, display maths as a block `DIV`, the pre-engine fallback
  text, no widget where there is no maths, `$PATH` in a cell left alone via
  `parseHickDoc` + `verbatimRanges`, and the `<hick:math>` card's single verb.
  `crates/hickory-cli/tests/math.rs` (3 tests) — the `$$` block in the weave,
  the tag not surviving it, and the surrounding prose untouched.
- Caveat requiring review: the typeset OUTPUT is not asserted anywhere. jsdom
  never resolves the dynamic `import("katex")`, so every editor test observes
  the fallback path — that the right element is created, with the right class
  and the source as its text. That KaTeX draws `e = mc^2` correctly is
  KaTeX's own test suite's business, but that our stylesheet lets it inherit
  the document's colour in a dark theme was checked by eye, not by a test.
