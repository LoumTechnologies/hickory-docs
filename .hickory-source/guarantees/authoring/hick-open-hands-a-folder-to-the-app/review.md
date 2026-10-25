<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# hick Opens The App While Explicit Verbs Keep Their CLI Behavior: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/cli_launch.rs](files/crates/hickory-cli/src/cli_launch.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/open_app.rs](files/crates/hickory-cli/tests/open_app.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
