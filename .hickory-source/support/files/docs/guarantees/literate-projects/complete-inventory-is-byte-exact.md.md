<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `e02167903c5da68b80ede2060ff446b1de514da5c47bcd6c6582495adb035776`.

<book:entry path="docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md">
# Complete source inventory includes bytes and metadata

Given a declared project's generated and expected virtual trees, when equality
is checked, then missing, extra, differing bytes, entry kinds, executable modes
and symlink targets fail. The Markdown inventory policy excludes its source
closure, declared woven reading targets, fixed VCS/Hick metadata directories,
and named non-source paths. No extension heuristic or Git ignore file
silently removes expected source files. Unknown native entry kinds fail. Paths
are normalized relative paths and ambiguous platform names/collisions fail.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified within the portable entry model.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, complete
  inventory, binary assets, lockfiles, migrations, metadata and virtual comparison.
- Evidence: `hick-lang/src/project/tree.rs::{ProjectTree,ProjectEntry}`, inventory
  in `evaluate.rs`, directory/Git adapters in `hickory-cli/src/project_revision.rs`.
  `hick-literate::equiv::compare_outputs` now compares actual binary bytes through
  the shared entry equality rather than binary display placeholders.
- Witnesses and challenges: `hick-lang/tests/project.rs` varies binary bytes,
  executable modes and adds an unexplained lockfile; CLI fresh-folder test adds
  an omitted migration; `hick-literate/tests/project.rs` compares two distinct
  same-length binary assets and mutation/deletion of a binary temporary file
  after pinning a baseline; `freeze_outputs` keeps pinned bytes in memory.
- Assumptions and exclusions: relevant portable permissions are executable versus
  ordinary, not ownership or all POSIX bits. Native executable-mode capture on
  Windows and symlink materialization there are unsupported; source generation
  retains portable metadata. Native symlink targets must stay relative and have
  a supported unambiguous spelling before materialization.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; inventory/entry-model edits invalidate this review.
- Human judgment: not recorded. Explicit exclusions are an authored inventory decision.
</book:entry>

</book:doc>
