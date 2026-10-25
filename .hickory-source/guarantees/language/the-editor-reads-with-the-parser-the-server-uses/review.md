<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Editor Reads A Document With The Parser The Server Uses: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/editor/hickDoc.ts](../../../shared/files/apps/web/src/editor/hickDoc.ts.md)
- [apps/web/src/editor/hickLang.ts](../../../shared/files/apps/web/src/editor/hickLang.ts.md)
- [apps/web/src/test-setup.ts](../../../shared/files/apps/web/src/test-setup.ts.md)
- [crates/hick-lang-wasm/src/lib.rs](../../../shared/files/crates/hick-lang-wasm/src/lib.rs.md)
- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hick-lang/src/structure.rs](files/crates/hick-lang/src/structure.rs.md)
- [scripts/check-codegen.sh](../../../shared/files/scripts/check-codegen.sh.md)

## Tests and fixtures

- [apps/web/src/editor/hickDoc.test.ts](../../../shared/files/apps/web/src/editor/hickDoc.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
