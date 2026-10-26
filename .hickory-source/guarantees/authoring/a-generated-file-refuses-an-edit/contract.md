<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Generated Text Refuses An Edit, And Says So Before You Type

For: Authors editing executable documents and the files those documents produce.

Given `hick up` running over a folder, when a generated output file contains
no byte that maps back to a document, then that file is marked read-only on
disk for as long as the loop runs; and when an edit is forced into generated
text anyway, then the file is restored to the woven bytes, the document is
left untouched, and the refusal names the line, the reason, and the file to
edit instead.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-generated-file-refuses-an-edit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `a_file_mixing_prose_with_generated_text_stays_writable`

This is the maintained test fragment from `crates/hickory-cli/tests/up_loop.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3098574124c3-a_file_mixing_prose_with_generated_text_stays_writable"></a>
### `a_file_mixing_prose_with_generated_text_stays_writable`

Property 1 of that guarantee: a file mixing document text with generated
text stays WRITABLE, because marking it read-only would block the edits
that are legal. Woven markdown is exactly such a file — its prose is the
document's prose — so this is the case that says so.

<storycode:copy id="f-3098574124c3-a_file_mixing_prose_with_generated_text_stays_writable">
/// Guarantee: `docs/guarantees/authoring/a-generated-file-refuses-an-edit.md`
///
/// Property 1 of that guarantee: a file mixing document text with generated
/// text stays WRITABLE, because marking it read-only would block the edits
/// that are legal. Woven markdown is exactly such a file — its prose is the
/// document's prose — so this is the case that says so.
#[test]
fn a_file_mixing_prose_with_generated_text_stays_writable() {
    let up = Loop::start(DOC);
    let woven_markdown = up.path("greeter.md");
    let meta = std::fs::metadata(&woven_markdown).expect("stat");
    assert!(
        !meta.permissions().readonly(),
        "woven markdown carries the document's prose, which is editable"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../mechanisms/crates/hickory-cli/src/serve/plain_file.rs.md#output-128423f88fa6)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)
- [crates/hickory-cli/src/up/reverse.rs](../../../mechanisms/crates/hickory-cli/src/up/reverse.rs.md#output-0e6cf077f262)
- [crates/hickory-cli/src/up/state.rs](../../../mechanisms/crates/hickory-cli/src/up/state.rs.md#output-d4f59b7595b5)

### Tests and fixtures

- [crates/hickory-cli/tests/up_loop.rs](../../../mechanisms/crates/hickory-cli/tests/up_loop.rs.md#output-3098574124c3)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
