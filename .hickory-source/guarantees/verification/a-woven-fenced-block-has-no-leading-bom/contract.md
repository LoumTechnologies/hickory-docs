<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Woven Fenced Block Has No Leading BOM

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a `<hick:file>` (ingested or hand-authored) whose content begins with
a UTF-8 byte-order mark, when the document is woven, then the fenced code
block a reader sees begins with the file's first real character — never
three invisible-but-copy-pasteable bytes before it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `strip_leading_bom_removes_the_bom_and_advances_the_span`

This is the maintained test fragment from `crates/hick-literate/src/lib/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-23d11bf7cb82-strip_leading_bom_removes_the_bom_and_advances_the_span"></a>
### `strip_leading_bom_removes_the_bom_and_advances_the_span`

Protects docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md

<storycode:copy id="f-23d11bf7cb82-strip_leading_bom_removes_the_bom_and_advances_the_span">
// Protects docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md
#[test]
fn strip_leading_bom_removes_the_bom_and_advances_the_span() {
    let span = hick_lang::SourceSpan {
        start: 50,
        end: 90,
        start_line: 3,
        start_col: 0,
        file_id: None,
    };
    let (text, moved) = strip_leading_bom("\u{feff}// real content", Some(&span));
    assert_eq!(text, "// real content");
    let moved = moved.expect("a span in must be a span out");
    // The BOM is 3 UTF-8 bytes (U+FEFF), never 1.
    assert_eq!(moved.start, 53);
    assert_eq!(moved.end, 90);
    assert_eq!(moved.start_line, 3, "no newline was crossed");
    assert_eq!(moved.start_col, 1);
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)

### Tests and fixtures

- [crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs](../../../support/files/crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs.md)
- [crates/hick-literate/src/lib/tests/tests.rs](../../../mechanisms/crates/hick-literate/src/lib/tests/tests.rs.md#output-23d11bf7cb82)
- [crates/hickory-cli/tests/weave_reads_naturally.rs](../../../mechanisms/crates/hickory-cli/tests/weave_reads_naturally.rs.md#output-a118c221b58a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
