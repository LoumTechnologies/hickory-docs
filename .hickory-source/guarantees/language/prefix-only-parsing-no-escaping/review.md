<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Bind the document language before quoting code: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

Prefix resolution chooses the bound marker before parsing children. Raw entry payloads preserve their bytes until the matching closer. The construction generator uses an explicit book root and hex encoding when a payload cannot be represented by its raw delimiter.

## Implementation

- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hick-lang/src/parser.rs](files/crates/hick-lang/src/parser.rs.md)

## Tests and fixtures

- [apps/web/src/editor/hickDoc.test.ts](../../../shared/files/apps/web/src/editor/hickDoc.test.ts.md)
- [crates/hick-lang/src/lib/tests/tests.rs](../../../shared/files/crates/hick-lang/src/lib/tests/tests.rs.md)

## Focused witnesses

- `parse_text_content_with_special_chars` in `crates/hick-lang/src/lib/tests/tests.rs`.

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
