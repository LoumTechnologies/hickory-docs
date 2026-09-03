# A `hick:file` Is A Generated File However Deeply It Is Nested

Given a `<hick:file path="…">` anywhere in a document — at the top level, or
inside a `hick:exec`, a `hick:ingested`, or anything else — when the document
is read by the language server, the debugger, or a `<hick:capture>`, then that
block is one of the document's generated files: it has an embedded language
server, a breakpoint gutter, and a place in the debugger's weave. The rule is
one rule, stated once: **the file blocks the engine's own weave writes**
(`all_tags`), never a subset of them.

And the bytes are the engine's bytes. A virtual file's *content* opens with
the newline that follows its tag — which is what makes its line 0 the tag's
own line and every line after it line up with the document, exactly right for
a reader that never writes the file down. Anything that **writes** the file
drops that first line (`VirtualFile::written_content`) and shifts the map with
it (`PositionMap::without_first_line`), because a leading blank line moves
whatever has to be at byte 0: a `#!` line, a UTF-8 BOM, an XML declaration.
The two halves are one decision and live next to each other for that reason.

## The bug this is written from (2026-09-03)

Debugging a C# program a document had scaffolded answered:

```
this document does not generate app/Program.cs. It generates:
```

— and then nothing, about a file sitting in the document being read. Two
independent disagreements, both between the engine and the view built on
`hick_lsp`:

1. **Which blocks are files.** `hick-literate` has always asked
   `doc.all_tags()`; `build_virtual_files` asked `doc.find_tags`, which is
   the top level only. `hick ingest --from '#cell'` writes
   `exec > ingested > file`, so *every* scaffold a document owned was written
   to disk by `hick run` and invisible to the language server and the
   debugger.
2. **What the bytes are.** The debugger's weave wrote `content()`, a leading
   newline included. `dotnet new` writes its `.csproj` with a BOM, which then
   sat at line 2 position 1, and MSBuild refused the file outright:
   `MSB4025: The project file could not be loaded. Data at the root level is
   invalid.`

Only the first was visible in the message. Fixing it moved the failure to the
second, which is the shape these two always have: one wrong answer hiding
another.

## Boundary

This is about `hick:file` blocks. `hick-literate/src/pipeline.rs`'s
`extract_owned_files` — the `_hick.yml`-driven pipeline, a separate and older
feature — still asks `find_tags` and is not covered here.

Nothing about *rendering* changes: an ingested block is still drawn as
ingested (`an-ingested-block-is-marked-in-the-document-view-too.md`), and the
origin of those bytes is still "this arrived from that run", not "you wrote
this".

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified, including a live C# session against the seeded document
  that produced the original report
- Evidence: `crates/hick-lsp/src/virtual_file.rs` — `build_virtual_files`
  now walks `doc.all_tags()`, matching `hick-literate/src/lib.rs`'s
  `cell_filled_files`, `literal_products` and `unstable_outputs`; and
  `VirtualFile::written_content`. `crates/hick-lsp/src/position_map.rs` —
  `PositionMap::without_first_line`. `crates/hick-dap/src/program.rs` —
  `weave_into` writes `written_content`;
  `crates/hick-dap/src/session.rs` — `Mapping::for_document` pairs it with
  `without_first_line`.
- Test coverage: `crates/hick-dap/src/program.rs`
  (`a_file_a_document_owns_is_generated_however_deeply_it_is_nested`,
  `what_has_to_be_at_byte_zero_is_at_byte_zero` — over an `exec > ingested >
  file` document with BOMs, the shape `hick ingest` writes);
  `crates/hick-lsp/src/document.rs`
  (`a_file_block_is_a_virtual_file_however_deeply_it_is_nested`, with its
  language detected); `crates/hick-dap/src/session.rs`
  (`a_document_line_maps_into_the_file_it_generates`, which now asserts the
  written bytes and the mapping **together** so they cannot drift again);
  `crates/hick-dap/tests/live_session.rs` (10 live Python sessions — its
  fixture now weaves through `weave_into` rather than writing the files
  itself, which is how it came to disagree with the mapping in the first
  place) and `live_capture.rs`.
- Caveats: the live suites skip without an adapter installed. The C# session
  that produced the report was verified by hand against
  `.dev/project/scaffolding.hick` — breakpoint set on the document's line,
  bound, hit, and the frame reported back on that same line — but no checked-in
  test drives a C# session end to end from a document; `live_session.rs` is
  Python.
