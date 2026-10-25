<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Debugger That Lives In A Language Server Is Asked For: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-dap/src/adapter.rs](../../../shared/files/crates/hick-dap/src/adapter.rs.md)
- [crates/hick-dap/src/discovery.rs](../../../shared/files/crates/hick-dap/src/discovery.rs.md)
- [crates/hick-dap/src/java.rs](files/crates/hick-dap/src/java.rs.md)
- [crates/hick-plugins/src/dap_discovery.rs](../../../support/files/crates/hick-plugins/src/dap_discovery.rs.md)
- [crates/hick-plugins/src/java.rs](../../../support/files/crates/hick-plugins/src/java.rs.md)
- [crates/hick-plugins/src/java_debug.rs](../../../support/files/crates/hick-plugins/src/java_debug.rs.md)
- [crates/hickory-cli/src/java_debug.rs](files/crates/hickory-cli/src/java_debug.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/live_session_java.rs](files/crates/hickory-cli/tests/live_session_java.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
