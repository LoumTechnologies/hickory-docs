<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Assumptions remain auditable after the work: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/AssumptionAudit.tsx](../../../support/files/apps/web/src/components/AssumptionAudit.tsx.md)
- [crates/hickory-agent/src/audit.rs](files/crates/hickory-agent/src/audit.rs.md)
- [crates/hickory-cli/src/serve/sessions.rs](../an-answer-in-the-agent-pane-has-ribbons/files/crates/hickory-cli/src/serve/sessions.rs.md)

## Tests and fixtures

- [apps/web/src/components/AssumptionAudit.test.tsx](files/apps/web/src/components/AssumptionAudit.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
