<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Image Dropped In Becomes A File And A Reference: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/web/src/editor/DocumentEditor.tsx](../../../shared/files/apps/web/src/editor/DocumentEditor.tsx.md)
- [apps/web/src/editor/mdLinks.ts](../../../shared/files/apps/web/src/editor/mdLinks.ts.md)
- [apps/web/src/editor/mdPaste.ts](../../../shared/files/apps/web/src/editor/mdPaste.ts.md)
- [crates/hickory-cli/src/serve/asset.rs](files/crates/hickory-cli/src/serve/asset.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)

## Tests and fixtures

- [apps/web/src/editor/mdLinks.test.ts](../../../shared/files/apps/web/src/editor/mdLinks.test.ts.md)
- [apps/web/src/editor/mdPaste.test.ts](../../../shared/files/apps/web/src/editor/mdPaste.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
