<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Reject a patch that does not fit its exact input: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

The patch evaluator constructs a candidate tree in memory. It publishes that candidate only after the complete act succeeds. It never searches fuzzily for a similar place to apply a hunk.

## Implementation

- [crates/hick-lang/src/parser.rs](../../language/prefix-only-parsing-no-escaping/files/crates/hick-lang/src/parser.rs.md)
- [crates/hick-lang/src/project/patch.rs](files/crates/hick-lang/src/project/patch.rs.md)
- [crates/hick-lang/src/project/tree.rs](../../../shared/files/crates/hick-lang/src/project/tree.rs.md)

## Tests and fixtures

- [apps/web/src/elements/patch/view.test.tsx](../../../shared/files/apps/web/src/elements/patch/view.test.tsx.md)
- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)

## Focused witnesses

- `patch_preserves_bom_crlf_unicode_and_untouched_origins` in `crates/hick-lang/tests/project.rs`.
- `newline_markers_add_delete_rename_and_failure_are_atomic` in `crates/hick-lang/tests/project.rs`.

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
