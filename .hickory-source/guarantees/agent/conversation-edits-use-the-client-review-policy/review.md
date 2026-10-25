<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Conversation edits use the client's review policy: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-agent/src/tools/preview.rs](../../../shared/files/crates/hickory-agent/src/tools/preview.rs.md)
- [crates/hickory-cli/src/mcp_preview.rs](../../../support/files/crates/hickory-cli/src/mcp_preview.rs.md)
- [crates/hickory-cli/src/serve/acp/edits.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/edits.rs.md)
- [crates/hickory-cli/src/serve/agent_review.rs](../../../support/files/crates/hickory-cli/src/serve/agent_review.rs.md)

## Tests and fixtures

- [apps/web/e2e/python-agent.spec.ts](../../../shared/files/apps/web/e2e/python-agent.spec.ts.md)
- [apps/web/src/components/AgentChanges.test.tsx](../../../shared/files/apps/web/src/components/AgentChanges.test.tsx.md)
- [apps/web/src/lib/agentEdit.test.ts](../../../shared/files/apps/web/src/lib/agentEdit.test.ts.md)
- [apps/web/src/lib/diff.test.ts](files/apps/web/src/lib/diff.test.ts.md)
- [crates/hickory-cli/tests/serve_acp.rs](../../../shared/files/crates/hickory-cli/tests/serve_acp.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
