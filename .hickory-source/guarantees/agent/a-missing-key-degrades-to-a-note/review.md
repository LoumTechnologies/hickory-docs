<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Missing Key Degrades to a Note, Never a Crash: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/ChatDock.tsx](../../../shared/files/apps/web/src/components/ChatDock.tsx.md)
- [crates/hickory-agent/src/provider.rs](../../../shared/files/crates/hickory-agent/src/provider.rs.md)
- [crates/hickory-cli/src/serve/agent.rs](../../../shared/files/crates/hickory-cli/src/serve/agent.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/serve_agent.rs](../../../shared/files/crates/hickory-cli/tests/serve_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
