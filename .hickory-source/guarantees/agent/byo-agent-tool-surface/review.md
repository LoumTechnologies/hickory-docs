<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Use the same document tools from an external coding agent: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-agent/src/session.rs](../../../shared/files/crates/hickory-agent/src/session.rs.md)
- [crates/hickory-cli/src/doc_tools.rs](files/crates/hickory-cli/src/doc_tools.rs.md)
- [crates/hickory-cli/src/init.rs](../../../shared/files/crates/hickory-cli/src/init.rs.md)
- [crates/hickory-cli/src/mcp.rs](../../../shared/files/crates/hickory-cli/src/mcp.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/byo_agent_surface.rs](../../../shared/files/crates/hickory-cli/tests/byo_agent_surface.rs.md)
- [crates/hickory-cli/tests/init_tests.rs](../../../shared/files/crates/hickory-cli/tests/init_tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</book:doc>
