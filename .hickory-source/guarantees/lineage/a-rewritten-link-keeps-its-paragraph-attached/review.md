<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Rewritten Link Keeps Its Paragraph Attached: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-flow/src/transform.rs](../../../shared/files/crates/hick-flow/src/transform.rs.md)
- [crates/hick-literate/src/links.rs](../../../shared/files/crates/hick-literate/src/links.rs.md)
- [crates/hickory-lineage/src/lib.rs](../../../shared/files/crates/hickory-lineage/src/lib.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
