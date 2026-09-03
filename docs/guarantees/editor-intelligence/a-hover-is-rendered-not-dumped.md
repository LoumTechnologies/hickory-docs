# A Hover Is Rendered, Not Dumped

Given a language server's answer to `textDocument/hover`, when the tooltip is
drawn, then its Markdown is **rendered**: a fenced block becomes a code block
without its backticks or its language word, `---` becomes a rule, a link
becomes its own text, and inline code and emphasis become `<code>`, `<strong>`
and `<em>`. Prose is drawn in the proportional UI font; the fenced signature
stays monospace.

The tooltip has a **maximum height** and scrolls inside it, so a hover can
never grow past the window with its own end unreachable.

Nothing the server sends is ever interpreted as HTML. Every node is built and
its content set as text: the string comes from another program, and
`innerHTML` on it would be an injection with extra steps.

## Why

The tooltip set the server's answer as `textContent`. Every real language
server answers in Markdown — rust-analyzer wraps the signature in a ```rust
fence, rules off the sections with `---`, and links types to docs.rs — so what
a person saw was the backticks, the word `rust`, a row of hyphens, and a URL.
Both halves were noted while dogfooding on 2026-09-02, the second being that
a long hover simply ran off the screen.

This is deliberately **not** a Markdown library. A hover is a paragraph, a
fenced block, a rule and some inline emphasis; the failure mode of a full
parser here is a tooltip that renders a table. Links are shown as their text
rather than as anchors because a tooltip is not somewhere you can click, and
an underlined thing that does nothing is worse than plain words.

Two smaller decisions that each cost a test to pin down: `**bold**` must be
taken off before `*em*` or it reads as two emphases around nothing, and a
fence the server never closed still renders — swallowing a signature because
a backtick is missing is worse than showing it.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/lsp/hoverMarkdown.ts` — `renderHoverMarkdown`,
  `inline`, `isRule`, and the `LINK` rule; `apps/web/src/lsp/cmLsp.ts` — the
  hover appends the fragment instead of assigning `textContent`;
  `apps/web/src/styles.css` — `.cm-lsp-hover` gains `max-height` and
  `overflow-y: auto`, with `.cm-lsp-hover-text`, `.cm-lsp-hover-code`, and
  the `hr`/`code` rules beside it.
- Test coverage: `apps/web/src/lsp/hoverMarkdown.test.ts` (10), against
  rust-analyzer's real answer shape — "renders a fence as a code block,
  without its backticks" (which asserts the three things a person used to see
  are gone), "shows a link as its text", "does not read bold as two
  emphases", "never interprets the server's text as HTML", "renders a fence
  nobody closed rather than swallowing it".
- Caveat requiring LLM review: the height ceiling is asserted only by reading
  the stylesheet — jsdom has no layout, so no test can observe that a tall
  hover scrolls rather than overflows. Lists and tables are not rendered;
  their Markdown source shows through as text, which is the same failure this
  guarantee fixed, narrowed to two constructs no language server has been
  observed to send here.
