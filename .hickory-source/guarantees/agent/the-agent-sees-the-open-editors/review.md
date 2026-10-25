<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Select model context without saving open buffers: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/contextProjection.ts](../../../support/files/apps/web/src/lib/contextProjection.ts.md)
- [crates/hick-literate/src/session_receipts.rs](../../../shared/files/crates/hick-literate/src/session_receipts.rs.md)
- [crates/hickory-agent/src/tools/read_file.rs](../../../shared/files/crates/hickory-agent/src/tools/read_file.rs.md)

## Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../shared/files/apps/desktop/src-tauri/tests/serves_one_origin.rs.md)
- [apps/web/src/lib/contextProjection.test.ts](../expanded-reading-projection/files/apps/web/src/lib/contextProjection.test.ts.md)
- [apps/web/src/views/WorkspaceChat.test.tsx](../../../shared/files/apps/web/src/views/WorkspaceChat.test.tsx.md)
- [crates/hickory-cli/tests/serve_acp.rs](../../../shared/files/crates/hickory-cli/tests/serve_acp.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
