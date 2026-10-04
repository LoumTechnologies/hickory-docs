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

Given a Markdown table without saved manual column widths or an explicit
setting, when it is rendered, then its columns share the draggable prose
soft-wrap measure and its text wraps to show all words, including long tokens.
Changing that measure reflows the table. CSV fenced blocks and other CSV tables
keep their declared widths by default.

Every table has a **Table settings** gear with **Wrap to prose width**. The
setting is remembered per table and can enable or disable this behavior for
Markdown and CSV alike. An explicit setting takes precedence over the default.
Changing a column width, by drag or column auto-fit, disables automatic prose
fitting and freezes the other columns at their current widths. The first drag
starts from the live width, even immediately after the prose measure changes.
Re-enabling the setting restores responsive columns and automatic row heights.
These choices change workspace presentation, never the table's Markdown or CSV.

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
- Browser verification: Playwright Chromium checked a two-column Markdown
  table at prose widths of 320px and 460px, with wrapped prose and an unbroken
  280-character token. All cells reported no horizontal or vertical clipping.
  A 40px column drag started from the live 183.6px width, saved 224px for that
  column, preserved the other column, and disabled fitting. The gear restored
  fitting on Markdown and enabled/disabled it on CSV. Selected-row fitting
  was also rechecked after these transitions.
- Caveats: automated component tests use jsdom; OS clipboard behavior is
  covered only by the existing grid's verification. Column
  insertion/deletion preserves alignment markers by position. Nested tables
  in lists and block quotes remain source text.
