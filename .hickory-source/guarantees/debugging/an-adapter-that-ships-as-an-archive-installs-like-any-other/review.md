<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Install a supported debugger archive into its declared prefix: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-dap/src/discovery.rs](../../../shared/files/crates/hick-dap/src/discovery.rs.md)
- [crates/hick-plugins/src/dap_discovery.rs](../../../support/files/crates/hick-plugins/src/dap_discovery.rs.md)
- [crates/hickory-cli/src/dap_install.rs](../../../shared/files/crates/hickory-cli/src/dap_install.rs.md)
- [crates/hickory-cli/src/tool_install.rs](files/crates/hickory-cli/src/tool_install.rs.md)

## Tests and fixtures

- [crates/hick-dap/tests/live_session_csharp.rs](../../../shared/files/crates/hick-dap/tests/live_session_csharp.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
