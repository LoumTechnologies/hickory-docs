# Document-View Styling Never Mutates the Source

Given any `.hick` document open in the web app's Document view (the
Typora-style WYSIWYG editor), when the view styles headings, inline markdown,
hick tag chrome, cell frames, or attaches cell widgets, then the underlying
editor text is byte-for-byte the raw source: styling uses CodeMirror mark,
line, and block-widget decorations only — never text replacement — so cursor
movement into any styled region cannot corrupt the document, and malformed
input (unclosed tags, stray closers, arbitrary garbage) degrades to plain
text without throwing.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/web/src/editor/wysiwyg.ts` — all decorations are
  `Decoration.mark`/`Decoration.line`/`Decoration.widget`; no `replace`
  decorations exist. `apps/web/src/editor/hickDoc.ts` wraps the parse in a
  catch-all so structure computation cannot throw.
- Test coverage: `apps/web/src/editor/DocumentEditor.test.tsx` ("never
  corrupts text: decorations leave the document unchanged on malformed
  docs"); `apps/web/src/editor/hickDoc.test.ts` ("never throws on garbage",
  unclosed-block and stray-closer cases).
