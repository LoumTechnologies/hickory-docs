<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Python observations stay confined and source bound: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/observation.rs](../../../shared/files/crates/hickory-cli/src/observation.rs.md)
- [examples/demo/app/main.py](../../../support/files/examples/demo/app/main.py.md)

## Tests and fixtures

- [apps/web/src/debug/runtimeHistory.test.tsx](../../../shared/files/apps/web/src/debug/runtimeHistory.test.tsx.md)
- [crates/hickory-cli/tests/serve_runtime_runs.rs](../../../shared/files/crates/hickory-cli/tests/serve_runtime_runs.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
