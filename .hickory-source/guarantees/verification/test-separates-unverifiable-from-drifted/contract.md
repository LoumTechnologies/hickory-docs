<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Report whether verification found drift, no baseline or a false expectation

For: Reviewers deciding what an output, recording or prediction actually establishes.

CI must distinguish a stale recording that needs regeneration from a cell with no baseline and from an expectation that actually failed.

## What must happen

Verification reports verified, drifted, unverifiable or expectation-failed with their distinct public exit codes. A stale recording counts as drift; a wholly missing baseline counts as unverifiable. A frozen valid recording is a baseline. Unverifiable cells do not evaluate expectations or supply trustworthy drift comparisons. When several outcomes occur, expectation-failed outranks unverifiable, which outranks drifted, which outranks verified; all findings remain visible.

## Limits of the evidence

Exit status summarizes evidence, not application correctness. A failed expectation needs a requirement/code decision rather than automatic baseline regeneration.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the stale-recording amendment into the four-outcome contract. Retain the public code mapping and severity ordering from the original detailed requirements.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `a_cell_with_no_container_can_be_named_and_keyed`

This is the maintained test fragment from `crates/hick-literate/src/lib/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-23d11bf7cb82-a_cell_with_no_container_can_be_named_and_keyed"></a>
### `a_cell_with_no_container_can_be_named_and_keyed`

Protects the "key does not assume a container" clause of

<storycode:copy id="f-23d11bf7cb82-a_cell_with_no_container_can_be_named_and_keyed">
// Protects the "key does not assume a container" clause of
// docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.
#[test]
fn a_cell_with_no_container_can_be_named_and_keyed() {
    let agentish = CellId::containerless(12);
    assert_eq!(agentish.container(), None);
    assert_eq!(agentish.to_string(), "line 12");

    let exec = CellId::exec("build", 12);
    assert_eq!(exec.container(), Some("build"));
    assert_eq!(exec.to_string(), "line 12 (container 'build')");

    // Same line, different cells: the key distinguishes them.
    let mut never_run = NeverRun::new();
    never_run.insert(agentish.clone(), NoBaseline::NotExecuted);
    never_run.insert(exec.clone(), NoBaseline::NotExecuted);
    assert_eq!(never_run.len(), 2);
    assert!(never_run.contains_key(&agentish));
    assert!(never_run.contains_key(&exec));
}

</storycode:copy>

## Test the behavior: `frozen_command`

This is the maintained test fragment from `crates/hickory-cli/tests/test_command_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-87d6e5478294-frozen_command"></a>
### `frozen_command`

Four outcomes: verified / drifted / unverifiable / expectation-failed
These protect
The frozen cell's command, shared between the document and the recording
so the cache key the pipeline computes is the one the test wrote.

<storycode:copy id="f-87d6e5478294-frozen_command">
// ---------------------------------------------------------------------------
// Four outcomes: verified / drifted / unverifiable / expectation-failed
//
// These protect
// docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.
// ---------------------------------------------------------------------------

/// The frozen cell's command, shared between the document and the recording
/// so the cache key the pipeline computes is the one the test wrote.
fn frozen_command() -> String {
    format!("\n{}\n", two_lines())
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hick-literate/src/lib/tests/tests.rs](../../../mechanisms/crates/hick-literate/src/lib/tests/tests.rs.md#output-23d11bf7cb82)
- [crates/hickory-cli/tests/test_command_tests.rs](../../../mechanisms/crates/hickory-cli/tests/test_command_tests.rs.md#output-87d6e5478294)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
