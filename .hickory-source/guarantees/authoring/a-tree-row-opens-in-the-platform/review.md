<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Tree Row Hands Its File To The Rest Of The Machine: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.tsx.md)
- [apps/web/src/shell/TreeContextMenu.tsx](files/apps/web/src/shell/TreeContextMenu.tsx.md)
- [apps/web/src/shell/treeMenu.ts](../../../shared/files/apps/web/src/shell/treeMenu.ts.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)
- [crates/hickory-cli/src/serve/reveal.rs](files/crates/hickory-cli/src/serve/reveal.rs.md)

## Tests and fixtures

- [apps/web/src/shell/FolderTreePane.test.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.test.tsx.md)
- [apps/web/src/shell/treeMenu.test.ts](../../../shared/files/apps/web/src/shell/treeMenu.test.ts.md)
- [crates/hickory-cli/tests/serve_files.rs](../../../shared/files/crates/hickory-cli/tests/serve_files.rs.md)
- [crates/hickory-cli/tests/serve_reveal.rs](files/crates/hickory-cli/tests/serve_reveal.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
