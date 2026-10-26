<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Citation That Points At Nothing Fails

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document with `cites="…"`, when `hick test` runs, then every selector
in that list is resolved **one at a time**, and any selector matching nothing
fails the run — `EXPECTATION FAILED`, exit 3 — naming the document, the line,
and the selector. `hick cites` marks the same selector in its listing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-citation-that-points-at-nothing-fails.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `a_dangling_citation_fails_verification`

This is the maintained test fragment from `crates/hickory-cli/tests/cites.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a19643f3dc7c-a_dangling_citation_fails_verification"></a>
### `a_dangling_citation_fails_verification`

Protects docs/guarantees/lineage/a-citation-that-points-at-nothing-fails.md

<storycode:copy id="f-a19643f3dc7c-a_dangling_citation_fails_verification">
/// Protects docs/guarantees/lineage/a-citation-that-points-at-nothing-fails.md
#[test]
fn a_dangling_citation_fails_verification() {
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("note.md");
    std::fs::write(
        &note,
        "<hick:copy id=\"finding\" class=\"f\">Pool is 40.</hick:copy>\n\n         <hick:claim by=\"nate\" standing=\"judgment\" cites=\"#finding,#missing\">We are fine.</hick:claim>\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&note).unwrap();

    let failures = dangling_citations(&note, &source).unwrap();
    assert_eq!(failures.len(), 1, "{failures:?}");
    match &failures[0] {
        CheckFailure::DanglingCitation { line, selector, .. } => {
            assert_eq!(selector, "#missing");
            assert_eq!(*line, 3);
        }
        other => panic!("wrong failure: {other:?}"),
    }

    // Not drift: nothing to regenerate, and no re-run fixes it. The document
    // points at something that is not there.
    assert_eq!(
        check_outcome(&failures),
        CheckOutcome::ExpectationFailed,
        "a citation to nothing is a statement that is definitely wrong"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/tests/cites.rs](../../../mechanisms/crates/hickory-cli/tests/cites.rs.md#output-a19643f3dc7c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
