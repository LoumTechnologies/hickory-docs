<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Review investigations link observed agent work without resolving human review: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [scripts/check-python-agent.sh](files/scripts/check-python-agent.sh.md)

## Tests and fixtures

- [apps/web/src/components/ReviewPanel.test.tsx](../../../shared/files/apps/web/src/components/ReviewPanel.test.tsx.md)
- [apps/web/src/elements/session/evidence.test.tsx](../../../shared/files/apps/web/src/elements/session/evidence.test.tsx.md)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../shared/files/crates/hickory-cli/tests/serve_python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
