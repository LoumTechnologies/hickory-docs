# Markdown tables use the table editor

Given a top-level Markdown pipe table in a document's prose,
when the document opens,
then the table uses the same grid as a CSV table, with the first row as its
header and a rail action to reveal its Markdown source.

When a person edits cells or resizes the table through that grid,
then the edit lands in the document as a Markdown pipe table. Cell edits keep
the alignment delimiter row, unchanged rows, and surrounding prose intact.
Pipes entered into cells are escaped; multiline values become `<br>` in the
Markdown source. A table remains editable after an insertion above it, and a
header edit keeps the grid mounted.

Tables inside fenced or indented code, verbatim hick element bodies, block
quotes, and lists are not replaced by this grid. Markdown tables are a prose
presentation; rendering them introduces no hick element or generated dataset.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: verified
- Evidence: `editor/markdownTables.ts` uses the existing CodeMirror Markdown
  parser, masking hick tags and verbatim ranges while retaining offsets.
  `elements/index.ts`, `editor/cards.ts`, and `editor/rendered.ts` use those
  presentation blocks for default rendering and the source toggle.
  `components/MarkdownTable.tsx` adapts the existing `TablePanel` through
  `lib/markdownTable.ts`; `DocumentEditor.replaceBlockContent` writes the
  Markdown block and retains its rendered state.
- Test coverage: `components/MarkdownTable.test.tsx` covers real grid cell
  commits, alignment and unchanged-row preservation, CRLF, pipe escaping,
  structural edits, Markdown detection, and code exclusion.
  `editor/markdownTables.test.tsx` opens the real DocumentEditor, inserts prose
  above its table, commits a body and header edit, checks document bytes and
  change notifications, and reveals source. `just test-markdown-tables`
  also checks existing rendered-slot and rail-card regressions.
- Caveats: tests use jsdom; native layout, pointer drags, and OS clipboard
  behavior are covered only by the existing grid's verification. Column
  insertion/deletion preserves alignment markers by position. Nested tables
  in lists and block quotes remain source text.
