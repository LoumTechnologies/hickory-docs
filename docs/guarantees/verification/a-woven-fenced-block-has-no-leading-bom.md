# A Woven Fenced Block Has No Leading BOM

Given a `<hick:file>` (ingested or hand-authored) whose content begins with
a UTF-8 byte-order mark, when the document is woven, then the fenced code
block a reader sees begins with the file's first real character — never
three invisible-but-copy-pasteable bytes before it.

## Why

`dotnet new` (and other .NET tooling) writes a BOM at the start of every
generated source file. `hick ingest` correctly keeps it: the document's own
`<hick:ingested>` bytes must stay byte-exact
(`docs/guarantees/authoring/ingest-keeps-the-original-bytes.md`), and
stripping it there would make a re-ingest's hash never match what the
scaffolder actually produced. But that byte-exactness is a claim about the
DOCUMENT's own bytes, not about how the WOVEN markdown should display them —
a reader looking at a fenced ```csharp block has no use for three bytes that
render as nothing, copy, and then either silently vanish or corrupt
whatever they're pasted into.

Found reading a real ingested `Program.cs` in a woven tutorial: the fenced
block's first line read `﻿// See https://aka.ms/new-console-template…` with
a literal, invisible `\u{feff}` before the comment.

## What changed, and what stayed exactly the same

`strip_leading_bom` (`crates/hick-literate/src/lib.rs`) is the same shape as
the pre-existing `strip_opening_break`: it strips a leading BOM and advances
the text's span by exactly 3 bytes (`\u{feff}`'s UTF-8 encoding), so lineage
stays byte-precise instead of degrading to synthetic. It is called from
`process_file_children_to_weave` (`crates/hick-literate/src/weave.rs`) only
at `position == 0` — a BOM only ever appears at the true start of a file,
never mid-content — right after the existing `strip_opening_break` call, in
the same per-position match.

This is a WEAVE-time display fix only. Three things are deliberately
unaffected, verified directly:
- The document's own `<hick:file>`/`<hick:ingested>` bytes keep the BOM —
  `hick ingest`'s byte-exactness guarantee is untouched.
- The real output file written to disk (what a build actually compiles)
  keeps the BOM its content actually has.
- Only the fenced block inside the woven `.md` loses it.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hick-literate/src/lib.rs`'s `strip_leading_bom`, wired
  into `crates/hick-literate/src/weave.rs`'s
  `process_file_children_to_weave` alongside `strip_opening_break`.
  Verified end to end against the real ingested `Program.cs` in
  `.../scratchpad/tutorial-hick/tutorial.hick` — both fenced blocks
  (`app/Program.cs`, `app/TodoApp.csproj`) lost their leading BOM with no
  other byte changed.
- Test coverage: unit tests in `crates/hick-literate/src/lib.rs`'s `tests`
  module (BOM removed and span advanced by exactly 3 bytes with no line
  crossed; ordinary text untouched; a BOM sitting mid-file, however it got
  there, is left alone — this function only ever looks at the very start).
  `weave_reads_naturally.rs` in `crates/hickory-cli/tests/`'s
  `a_file_starting_with_a_bom_weaves_its_fenced_block_without_one`, driving
  the real binary: confirms all three claims above in one test — the
  document's own bytes, the real output file, AND the woven fence — checking
  each keeps or loses the BOM correctly. Confirmed load-bearing by
  temporarily disabling the strip and observing the BOM reappear in the
  woven output.
- Caveat requiring LLM review: this repository's own `hick:exec` TRANSCRIPT
  rendering (the `$ cat Program.cs` shell-style block, a different code path
  from the fenced-file display this fix covers) still shows a BOM when the
  command it ran genuinely printed one — confirmed present in
  `tutorial.hick`'s Step 0 transcript after this fix. Left alone
  deliberately: a transcript is a claim about what a real command really
  printed, and this codebase treats manufacturing drift from a transcript's
  own fidelity as a worse failure than a stray invisible character. Whether
  that tradeoff should also apply to a fenced FILE display (a claim about
  file content, not about a command's output) was the judgment call this fix
  made — differently — and is worth revisiting if a transcript's own BOM is
  ever found to cause real confusion.
