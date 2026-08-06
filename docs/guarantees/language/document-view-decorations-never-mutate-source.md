# Document-View Styling Never Mutates the Source

Given any `.hick` document open in the web app's Document view (the
Typora-style WYSIWYG editor), when the view styles headings, inline markdown,
hick tag chrome, cell/file/fragment frames, conditional banners, paste chips,
inline directive tokens, embedded-language syntax highlighting inside block
bodies, or attaches cell/env/fragment/banner widgets, then the underlying
editor text is byte-for-byte the raw source: styling uses CodeMirror mark,
line, and block-widget decorations only — never text replacement — clicking a
paste chip only dispatches a selection, so cursor movement into any styled
region cannot corrupt the document, and malformed input (unclosed tags, stray
closers, arbitrary garbage) degrades to plain text without throwing.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/web/src/editor/wysiwyg.ts` — all decorations are
  `Decoration.mark`/`Decoration.line`/`Decoration.widget`; no `replace`
  decorations exist; the paste-chip mousedown handler dispatches only a
  selection. `apps/web/src/editor/embedded.ts` computes highlight spans as
  pure data over slices of the source. `apps/web/src/editor/hickDoc.ts`
  wraps the parse in a catch-all so structure computation cannot throw.
- Test coverage: `apps/web/src/editor/DocumentEditor.test.tsx` ("never
  corrupts text: decorations leave the document unchanged on malformed
  docs"; "renders fragment chips, when banners, paste chips and embedded
  highlighting" asserts the raw source stays present under the new
  decorations); `apps/web/src/editor/hickDoc.test.ts` ("never throws on
  garbage", "never throws on nasty new-element inputs", no-escaping raw
  `< > &` cases); `apps/web/src/editor/embedded.test.ts` (garbage input
  never throws, unknown languages yield no spans).
