<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Verify Covers the Whole Editable Set: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-literate/src/run_transcripts.rs](files/crates/hick-literate/src/run_transcripts.rs.md)
- [crates/hick-project-env/src/tree.rs](../../../shared/files/crates/hick-project-env/src/tree.rs.md)
- [crates/hickory-agent/src/tools/mod.rs](../../../shared/files/crates/hickory-agent/src/tools/mod.rs.md)
- [crates/hickory-agent/src/tools/verify.rs](../../../support/files/crates/hickory-agent/src/tools/verify.rs.md)
- [crates/hickory-agent/src/tools/verify_publish.rs](../../../support/files/crates/hickory-agent/src/tools/verify_publish.rs.md)

## Tests and fixtures

- [crates/hickory-agent/tests/verify_publication.rs](files/crates/hickory-agent/tests/verify_publication.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</book:doc>
