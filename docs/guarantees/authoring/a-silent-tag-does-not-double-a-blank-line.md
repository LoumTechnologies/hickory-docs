# A Silent Tag Does Not Double A Blank Line

Given a `<hick:copy>`, `<hick:cut>`, `<hick:container>`, `<hick:volume>`,
`<hick:feature>`, `<hick:needs>`, or `<hick:allow>` tag standing alone
between two paragraphs, when the document is woven, then exactly one blank
line separates the paragraphs — never the three or more a naive
concatenation of the surrounding prose's own paragraph-break newlines would
leave behind.

## Why

None of these seven tags contribute a single byte to the woven markdown:
`copy`/`cut` register content for a later `hick:paste` and return
`TagResult::Declaration` (confirmed by reading
`crates/hick-handlers/src/handlers/copy.rs` directly); `container`,
`volume`, `feature`, `needs`, and `allow` have no registered weave handler
at all — they are pipeline/DAG declarations with nothing to render. But the
prose immediately before and after the tag still carries its own paragraph
break: `"prose A.\n\n<hick:copy…/>\n\nprose B"` concatenates, with the tag
contributing nothing, to `"prose A.\n\n\n\nprose B"` — three blank lines
where a human who left the declaration out entirely would have written one.

Found comparing a real `.hick` tutorial against a plain-Markdown control
written for the same content: multiple runs of 3–5 blank lines throughout
the woven output, every one traced to a silent declaration tag sitting on
its own line. Confirmed at scale by re-weaving five of this repository's own
committed examples — every one dropped 2–6 bytes of pure redundant
whitespace, with zero change to any other content.

## The fix, and what it deliberately does not touch

`process_weave_content` (`crates/hick-literate/src/weave.rs`) tracks whether
the node just processed was one of the seven `SILENT_DECLARATION_TAGS`; if
so, the NEXT text node strips up to one blank line's worth of its own
leading newlines (`strip_up_to_one_blank_line`) before being added — the
same span-adjustment shape as the pre-existing `strip_opening_break`, so
lineage stays byte-precise rather than degrading to synthetic. Only the
following side is touched (the preceding text is already committed to the
weave and cannot be edited after the fact) — sufficient, because one side
owning the correct single blank line is enough to remove the redundant rest.

This is a closed, verified list of tag names, not a general "did this tag
add anything" check — `InsertionPoint` (`hick-flow`) exposes no count to
test that against from outside the weave-emission code. A future declaration
tag with the same silent shape would need adding to the list by hand; the
consequence of missing one is the OLD (correct, just noisier) spacing
behavior, not a new defect.

A tag that DOES emit something — `hick:file`, `hick:exec`, `hick:diagram`,
etc. — never triggers this at all: the strip only fires for names in the
closed list, so real content's own spacing is never touched.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hick-literate/src/weave.rs`'s `SILENT_DECLARATION_TAGS`,
  the `pending_blank_strip` tracking in `process_weave_content`, and
  `strip_up_to_one_blank_line`. Verified end to end against a real
  ingest-based tutorial (`.../scratchpad/tutorial-hick/tutorial.hick`) and
  against five of this repository's own committed examples
  (`ai-transform`, `architecture-that-draws-itself`, `bootstrap-ci`,
  `planned-messages`, `text-tools-tour`) — every re-weave's diff against the
  previously-committed `.md` consists ENTIRELY of removed blank lines, no
  other byte changed.
- Test coverage: `strip_up_to_one_blank_line_tests` in
  `crates/hick-literate/src/weave.rs` (5 unit tests: two leading newlines
  fully removed; a single leading newline — the shape between two adjacent
  silent tags — fully removed rather than left dangling; three or more
  leading newlines keep the real paragraph break, taking only two; text
  with no leading newline is untouched; a span moves by exactly the bytes
  taken). `weave_reads_naturally.rs` in `crates/hickory-cli/tests/` (3 of
  its 4 tests, driving the real binary: a silent tag between two paragraphs
  leaves exactly one blank line; two CONSECUTIVE silent tags still collapse
  to one, not two; a tag that DOES render — `hick:file` — is confirmed
  untouched). Confirmed load-bearing by temporarily disabling the strip and
  observing both the unit and integration tests fail.
- Caveat requiring LLM review: `examples/scaffolded-service.hick` and
  `examples/grand-tour.hick` also drift under this fix (both declare
  `hick:container`/`hick:volume`), but their committed `.md` files were
  deliberately NOT regenerated in this pass — `scaffolded-service.hick`'s
  `ls` output order differs on macOS/LocalExecutor from whatever produced
  its current recording (an unrelated, pre-existing environment mismatch),
  and `grand-tour.hick` needs `duckdb`, not installed on this machine.
  Regenerating either from an environment with the mismatch present would
  bake in wrong, machine-specific content. Both need a proper re-run (Linux,
  with `duckdb`) before `hick test examples/` is fully clean again.
