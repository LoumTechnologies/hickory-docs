<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Host context observations are immutable and changes are source-backed: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-context-store/src/host.rs](../../../shared/files/crates/hick-context-store/src/host.rs.md)
- [crates/hick-context-store/src/lib.rs](../../../shared/files/crates/hick-context-store/src/lib.rs.md)
- [crates/hickory-agent/src/tools/read_file.rs](../../../shared/files/crates/hickory-agent/src/tools/read_file.rs.md)
- [crates/hickory-agent/src/tools/receipts.rs](files/crates/hickory-agent/src/tools/receipts.rs.md)
- [crates/hickory-cli/src/python_agent.rs](../../../shared/files/crates/hickory-cli/src/python_agent.rs.md)
- [crates/hickory-cli/src/serve/acp/read_receipt.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/read_receipt.rs.md)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../shared/files/crates/hickory-cli/src/serve/python_agent.rs.md)

## Tests and fixtures

- [crates/hick-context-store/tests/lineage.rs](../../../shared/files/crates/hick-context-store/tests/lineage.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
