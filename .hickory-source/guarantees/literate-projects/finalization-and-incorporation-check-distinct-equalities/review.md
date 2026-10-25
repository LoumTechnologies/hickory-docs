<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Check the agent's transition separately from maintained-source reconstruction: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-lang/src/project/reconstruct.rs](files/crates/hick-lang/src/project/reconstruct.rs.md)
- [crates/hickory-cli/src/serve/agent_projects.rs](../../agent/changeset-completion-is-negotiated-and-checked/files/crates/hickory-cli/src/serve/agent_projects.rs.md)
- [crates/hickory-cli/src/serve/project_incorporation.rs](../../../support/files/crates/hickory-cli/src/serve/project_incorporation.rs.md)
- [crates/hickory-cli/src/serve/project_publish.rs](../../../support/files/crates/hickory-cli/src/serve/project_publish.rs.md)

## Tests and fixtures

- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../shared/files/crates/hickory-cli/tests/literate_agent_projects.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
