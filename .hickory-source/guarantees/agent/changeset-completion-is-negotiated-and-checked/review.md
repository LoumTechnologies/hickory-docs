<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Finish an agent turn against its captured project boundary: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ChangesetDocument.tsx](files/apps/web/src/components/ChangesetDocument.tsx.md)
- [apps/web/src/elements/patch/Editor.tsx](../../../shared/files/apps/web/src/elements/patch/Editor.tsx.md)
- [crates/hickory-cli/src/changeset.rs](files/crates/hickory-cli/src/changeset.rs.md)
- [crates/hickory-cli/src/changeset_mcp.rs](files/crates/hickory-cli/src/changeset_mcp.rs.md)
- [crates/hickory-cli/src/serve/acp/client.rs](files/crates/hickory-cli/src/serve/acp/client.rs.md)
- [crates/hickory-cli/src/serve/acp/edits.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/edits.rs.md)
- [crates/hickory-cli/src/serve/acp/mcp.rs](files/crates/hickory-cli/src/serve/acp/mcp.rs.md)
- [crates/hickory-cli/src/serve/acp/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/mod.rs.md)
- [crates/hickory-cli/src/serve/agent_projects.rs](files/crates/hickory-cli/src/serve/agent_projects.rs.md)
- [crates/hickory-cli/src/workspace_capture.rs](files/crates/hickory-cli/src/workspace_capture.rs.md)

## Tests and fixtures

- [apps/web/e2e/workspace-changesets.spec.ts](../../../shared/files/apps/web/e2e/workspace-changesets.spec.ts.md)
- [apps/web/src/components/ChangesetDocument.test.tsx](../../../shared/files/apps/web/src/components/ChangesetDocument.test.tsx.md)
- [crates/hickory-cli/tests/agent_changesets.rs](../../../shared/files/crates/hickory-cli/tests/agent_changesets.rs.md)
- [crates/hickory-cli/tests/byo_agent_surface.rs](../../../shared/files/crates/hickory-cli/tests/byo_agent_surface.rs.md)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../shared/files/crates/hickory-cli/tests/literate_agent_projects.rs.md)
- [crates/hickory-cli/tests/python_agent.rs](../../../shared/files/crates/hickory-cli/tests/python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
