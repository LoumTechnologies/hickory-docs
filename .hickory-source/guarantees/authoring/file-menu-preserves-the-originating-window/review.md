<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The File Menu Preserves The Originating Window: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/views/folderPane.ts](../../../support/files/apps/web/src/views/folderPane.ts.md)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../shared/files/crates/hickory-cli/src/serve/plain_file.rs.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/first_folder.rs](files/apps/desktop/src-tauri/tests/first_folder.rs.md)
- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
