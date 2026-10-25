<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Included Span Names Its Own File, And An Edit Through It Lands There: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-handlers/src/handlers/copy.rs](../../../shared/files/crates/hick-handlers/src/handlers/copy.rs.md)
- [crates/hick-handlers/src/lib.rs](files/crates/hick-handlers/src/lib.rs.md)
- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)
- [crates/hickory-agent/src/tools/mod.rs](../../../shared/files/crates/hickory-agent/src/tools/mod.rs.md)
- [crates/hickory-cli/src/up/mod.rs](../../../shared/files/crates/hickory-cli/src/up/mod.rs.md)
- [crates/hickory-lineage/src/lib.rs](../../../shared/files/crates/hickory-lineage/src/lib.rs.md)

## Tests and fixtures

- [crates/hick-lang/tests/upstream_tests.rs](files/crates/hick-lang/tests/upstream_tests.rs.md)
- [crates/hickory-cli/tests/include_lineage.rs](files/crates/hickory-cli/tests/include_lineage.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
