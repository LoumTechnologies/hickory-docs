<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/literate-projects/replay-is-exact-and-atomic.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `d2277577b16f446c8b2b5a64567f34b73ab865967c4890867b4d39c15b8d737a`.

<book:entry path="docs/guarantees/literate-projects/replay-is-exact-and-atomic.md">
# A project patch replays exactly as one act

Given a virtual project and an explicit namespace-prefixed patch containing
unified diff bytes, when replay is requested, then every hunk applies at its
specified old and new offsets with exact context. Wrong context, malformed
counts, overlaps, path collisions, unsupported input encoding, and incomplete
file deletion fail the complete act. The input tree remains unchanged. No fuzzy
search, process, model call, or filesystem write occurs. Untouched bytes retain
their origins; inserted bytes map to the patch payload. BOM, CRLF, Unicode, and
final-newline state remain data. Ordinary Markdown diff fences are inert.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified within the textual unified-diff subset.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, exact
  replay and virtual-tree milestones. A raw body closes at its bound namespace's
  patch delimiter; rebinding is required if that delimiter occurs in the data.
- Evidence: `hick-lang/src/project/patch.rs::apply_project_patch`, `tree.rs`, and
  `parser.rs::is_raw_content_tag`; parser/WASM share the grammar.
- Witnesses and challenges: `hick-lang/tests/project.rs` tests BOM/CRLF/Unicode,
  no-newline markers, create/delete/rename, incorrect offsets and partial deletes.
- Assumptions and exclusions: headers use `a/`, `b/` or `/dev/null`; Git extended
  headers, fuzzy diffs, binary textual patches, and mode-only patches are refused.
  Binary bytes and modes can be constructed explicitly in maintained source.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: the implementation and test paths above; edits make this review stale.
- Human judgment: not recorded. Byte equality does not certify prose or runtime behavior.
</book:entry>

</book:doc>
