<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A project environment offers its own sync: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/environments/EnvironmentPanel.tsx](../../../shared/files/apps/web/src/environments/EnvironmentPanel.tsx.md)

## Tests and fixtures

- [apps/web/src/environments/EnvironmentPanel.test.tsx](../../../shared/files/apps/web/src/environments/EnvironmentPanel.test.tsx.md)
- [apps/web/src/environments/useEnvironments.test.tsx](files/apps/web/src/environments/useEnvironments.test.tsx.md)
- [crates/hick-project-env/tests/providers.rs](files/crates/hick-project-env/tests/providers.rs.md)
- [crates/hickory-cli/tests/serve_environments.rs](files/crates/hickory-cli/tests/serve_environments.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</book:doc>
