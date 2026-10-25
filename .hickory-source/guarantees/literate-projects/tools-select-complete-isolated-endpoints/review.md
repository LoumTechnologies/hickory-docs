<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep language tools and debug launches tied to their captured project: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-dap/src/project.rs](files/crates/hick-dap/src/project.rs.md)
- [crates/hick-lsp/src/document.rs](../../../shared/files/crates/hick-lsp/src/document.rs.md)
- [crates/hick-lsp/src/location_map.rs](../../../support/files/crates/hick-lsp/src/location_map.rs.md)
- [crates/hick-lsp/src/position_map.rs](../../../shared/files/crates/hick-lsp/src/position_map.rs.md)
- [crates/hick-lsp/src/project.rs](../../../shared/files/crates/hick-lsp/src/project.rs.md)
- [crates/hick-lsp/src/project_contexts.rs](../../../support/files/crates/hick-lsp/src/project_contexts.rs.md)
- [crates/hick-lsp/src/project_positions.rs](../../../support/files/crates/hick-lsp/src/project_positions.rs.md)
- [crates/hickory-cli/src/debug_sessions.rs](../../../shared/files/crates/hickory-cli/src/debug_sessions.rs.md)
- [crates/hickory-cli/src/project_documents.rs](../../../shared/files/crates/hickory-cli/src/project_documents.rs.md)

## Tests and fixtures

- [apps/web/src/components/ComparisonTools.test.tsx](../../../shared/files/apps/web/src/components/ComparisonTools.test.tsx.md)
- [crates/hick-dap/tests/project.rs](../../../shared/files/crates/hick-dap/tests/project.rs.md)
- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)
- [crates/hick-lsp/tests/project.rs](../../../shared/files/crates/hick-lsp/tests/project.rs.md)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../shared/files/crates/hickory-cli/tests/literate_agent_projects.rs.md)
- [crates/hickory-cli/tests/literate_project_csharp.rs](files/crates/hickory-cli/tests/literate_project_csharp.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
