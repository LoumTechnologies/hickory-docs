<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Transcript Derives Its Speaker Turns, And Cannot Disagree With Itself: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-handlers/src/handlers/copy.rs](../../../shared/files/crates/hick-handlers/src/handlers/copy.rs.md)
- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/weave.rs](../../../shared/files/crates/hick-literate/src/weave.rs.md)
- [crates/hick-transcript/src/lib.rs](files/crates/hick-transcript/src/lib.rs.md)

## Tests and fixtures

- [crates/hick-transcript/tests/upstream.rs](files/crates/hick-transcript/tests/upstream.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
