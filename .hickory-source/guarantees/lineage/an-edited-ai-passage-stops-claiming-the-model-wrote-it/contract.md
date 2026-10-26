<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Edited AI Passage Stops Claiming The Model Wrote It

For: Authors editing generated text and reviewers tracing a result to its source.

Given a `hick:transform` carrying `wrote=` — a fingerprint of the passage as
the model produced it — when the passage's bytes are no longer those bytes,
then the weave says *written by a model, and edited by hand since*, and
`hick test` reports it, naming the model.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `edited_passages`

This is the maintained test fragment from `crates/hickory-cli/tests/cites.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a19643f3dc7c-edited_passages"></a>
### `edited_passages`

Protects docs/guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it.md

<storycode:copy id="f-a19643f3dc7c-edited_passages">
// Protects docs/guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it.md
mod edited_passages {
    use hickory_cli::{CheckFailure, CheckOutcome, check_outcome, edited_transforms};

    /// A transform holding exactly what the model wrote, `wrote=` and all.
    fn doc(passage: &str, wrote: &str) -> String {
        format!(
            "<hick:copy id=\"f\" class=\"incident\">The release was red.</hick:copy>\n\n\
             <hick:transform model=\"a/b\" select=\".incident\" instruct=\"Summarize.\" \
             from=\"deadbeef\" wrote=\"{wrote}\">\n{passage}\n</hick:transform>\n"
        )
    }

    fn write(dir: &std::path::Path, source: &str) -> std::path::PathBuf {
        let path = dir.join("note.md");
        std::fs::write(&path, source).unwrap();
        path
    }

    #[test]
    fn the_models_own_words_are_not_marked() {
        let dir = tempfile::tempdir().unwrap();
        let passage = "The release was red for two days.";
        let wrote = hick_lang::passage_fingerprint(passage);
        let path = write(dir.path(), &doc(passage, &wrote));
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(edited_transforms(&path, &source).unwrap().is_empty());
    }

    #[test]
    fn a_hand_edit_is_marked_and_names_the_model() {
        let dir = tempfile::tempdir().unwrap();
        // Stamped for what the model wrote; the document holds something else.
        let wrote = hick_lang::passage_fingerprint("The release was red for two days.");
        let path = write(
            dir.path(),
            &doc("Everything was fine and nobody noticed.", &wrote),
        );
        let source = std::fs::read_to_string(&path).unwrap();
        let found = edited_transforms(&path, &source).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        match &found[0] {
            CheckFailure::EditedTransform { model, .. } => {
                assert_eq!(model.as_deref(), Some("a/b"));
            }
            other => panic!("wrong failure: {other:?}"),
        }
        // Editing is the author's business. The document telling the truth
        // about itself is not a failure of verification.
        assert_eq!(check_outcome(&found), CheckOutcome::Verified);
    }

    #[test]
    fn whitespace_around_the_passage_is_not_an_edit() {
        // Re-indenting a document must not read as somebody rewriting the
        // model: the newlines around a passage are the element's formatting.
        let dir = tempfile::tempdir().unwrap();
        let passage = "The release was red for two days.";
        let wrote = hick_lang::passage_fingerprint(passage);
        let path = write(dir.path(), &doc(&format!("\n{passage}   "), &wrote));
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(edited_transforms(&path, &source).unwrap().is_empty());
    }

    #[test]
    fn a_passage_written_before_this_existed_claims_nothing() {
        // Silence, not a mark: a document with no `wrote=` never said whose
        // words these are, and inventing an answer for it would be worse
        // than saying nothing.
        let dir = tempfile::tempdir().unwrap();
        let path = write(
            dir.path(),
            "<hick:transform select=\".x\" instruct=\"Summarize.\" from=\"deadbeef\">\n\
             anything at all\n</hick:transform>\n",
        );
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(edited_transforms(&path, &source).unwrap().is_empty());
    }
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-handlers/src/handlers/transform.rs](../../../mechanisms/crates/hick-handlers/src/handlers/transform.rs.md#output-399538a999c4)
- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
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
