# A Generated File Starts At Its First Byte

Given a `<hick:file path="…">` whose content begins on the line *after* the
open tag — which is how every readable document writes one — when the file is
produced, then its first byte is the first byte of the content, not the line
break that ended the tag's line.

The block

```
<hick:file path="chart.svg">
<hick:exec container="r" show="output">
…
</hick:exec>
</hick:file>
```

writes a file beginning `<?xml`, not `\n<?xml`.

## Why this is a correctness rule and not a tidiness one

A leading blank line is untidy in a `.py`. It is **fatal** in three cases this
product produces routinely:

- **An XML declaration must start at byte zero.** Firefox rejects the file
  outright — *"XML or text declaration not at start of entity"* — so every SVG
  hick generated was a chart its own weave could draw and a browser could not
  open. That is the worst shape a bug can have here: the app looked right
  while the file on disk was broken.
- **A `#!` line is only a shebang on line one.** A generated script with a
  blank line above it is a script the kernel will not run.
- **A file whose first line is significant** — a `.csv` header a tool reads
  positionally, a `---` frontmatter fence — is silently misread rather than
  rejected.

## The rule, precisely

**Exactly one line break, only the first text child, and only when it is
really there.** `\n` and `\r\n` both count, one of them, once. Content written
on the tag's own line (`<hick:file path="a.txt">hello</hick:file>`) keeps
every byte, because there is no tag-line break to remove and taking a real one
would corrupt the file.

**The trailing break is left alone.** The `\n` before `</hick:file>` is the
file's final newline, which is what a text file should end with.

**The span moves with the text.** The trimmed text keeps a source span
pointing at the first byte that survived — start advanced past the break,
start line incremented, column zero — so a reverse edit from the generated
file still lands on the right bytes of the document. Trimming without moving
the span would put every edit one character early.

---

Last LLM verification:
- Date: 2026-08-26
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `strip_opening_break` in `crates/hick-literate/src/lib.rs`, applied
  by `process_file_children` to the child at position 0 only. It returns the
  text unchanged when there is no leading break, and otherwise returns the
  remainder with a `hick_lang::SourceSpan` whose `start` is advanced by the
  bytes taken (1 or 2), `start_line` incremented, and `start_col` zeroed;
  `file_id` is carried through, so a span spliced in by
  `<hick:include>`/`<hick:upstream>` still indexes the included file.
- Test coverage: `crates/hick-literate/tests/file_first_byte.rs` — an SVG
  whose declaration lands at byte zero, a `#!` script, content on the tag's
  own line keeping its bytes, `\r\n`, and the trailing newline surviving.
  End to end, the repository's own examples are the regression: `hick test
  examples/` compares every generated file against the committed bytes, and
  `examples/bootstrap-histogram.svg` and `examples/regression-explorer.html`
  are committed starting at `<svg` and `<!DOCTYPE html>`.
- Caveat requiring LLM review: this covers `<hick:file>`. The woven markdown
  target takes a different path (`process_weave_output`) and is not governed
  by this rule; a `.md` beginning with a blank line is harmless, but if a
  document ever weaves to a format with a byte-zero requirement, that path
  needs the same treatment and does not have it.
