<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A final changeset revision applies B to C: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/serve/changeset_drafts.rs](../../../support/files/crates/hickory-cli/src/serve/changeset_drafts.rs.md)

## Tests and fixtures

- [apps/web/e2e/workspace-changesets.spec.ts](../../../shared/files/apps/web/e2e/workspace-changesets.spec.ts.md)
- [apps/web/src/components/ChangesetDocument.test.tsx](../../../shared/files/apps/web/src/components/ChangesetDocument.test.tsx.md)
- [apps/web/src/lib/agentEdit.test.ts](../../../shared/files/apps/web/src/lib/agentEdit.test.ts.md)
- [crates/hickory-cli/tests/agent_changesets.rs](../../../shared/files/crates/hickory-cli/tests/agent_changesets.rs.md)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../shared/files/crates/hickory-cli/tests/literate_agent_projects.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
