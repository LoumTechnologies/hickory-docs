<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A New Project Is A Recipe Commit, Made As One Act: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/server.rs](../../../shared/files/apps/desktop/src-tauri/src/server.rs.md)
- [apps/web/src/App.tsx](../../../shared/files/apps/web/src/App.tsx.md)
- [apps/web/src/api/client.ts](../../../shared/files/apps/web/src/api/client.ts.md)
- [apps/web/src/components/NewProjectDialog.tsx](../../../shared/files/apps/web/src/components/NewProjectDialog.tsx.md)
- [crates/hick-plugins/src/scaffold.rs](../../../support/files/crates/hick-plugins/src/scaffold.rs.md)
- [crates/hickory-cli/src/scaffold.rs](../../../shared/files/crates/hickory-cli/src/scaffold.rs.md)
- [crates/hickory-cli/src/scaffold_commit.rs](../../../shared/files/crates/hickory-cli/src/scaffold_commit.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)
- [crates/hickory-cli/src/serve/scaffold.rs](../../../shared/files/crates/hickory-cli/src/serve/scaffold.rs.md)

## Tests and fixtures

- [apps/web/src/components/NewProjectDialog.test.tsx](../../../shared/files/apps/web/src/components/NewProjectDialog.test.tsx.md)
- [apps/web/src/lib/scaffold.test.ts](files/apps/web/src/lib/scaffold.test.ts.md)
- [crates/hickory-cli/tests/scaffold_templates.rs](../../../shared/files/crates/hickory-cli/tests/scaffold_templates.rs.md)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../shared/files/crates/hickory-cli/tests/serve_scaffold.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
