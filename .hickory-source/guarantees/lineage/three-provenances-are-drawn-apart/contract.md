<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Three Provenances Are Drawn Apart, And Any Subset Can Be Shown

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document on screen, when the overlay draws where its text came from,
then it draws up to three families — **lineage** (the weave's byte-exact
derivation), **context** (what was in front of the model when an agent wrote
these lines, from the session record), and **declared** (what the author says
it cites, `cites="…"`) — each in its own hue AND its own stroke (solid
palette / dotted amber / dashed grey), each with a title that says what it is
and what it cannot claim; and display settings select any subset, remembered per browser. The status bar
only labels families painted in the current view; the editor and command search
retain access to all display settings, including disabled families.

The quiet-workspace requirement supersedes permanent status-bar toggles. Other provenance distinctions remain retained; the preserved original records earlier placement.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/three-provenances-are-drawn-apart.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep provenance hue/stroke distinctions and remembered display preferences. The quiet-workspace contract supersedes permanent status-bar toggles; the retained detail below records the earlier placement. Focused checks and live acceptance remain separate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="f-a19643f3dc7c-cites_resolve_across_the_chain_and_report_dangling_ones"></a>
### `cites_resolve_across_the_chain_and_report_dangling_ones`

Declared provenance: `cites="…"` is what an author SAYS an element rests
on. It resolves to places so it can be drawn, and it weaves as an
assertion in words — never as lineage, never as a check.

<storycode:copy id="f-a19643f3dc7c-cites_resolve_across_the_chain_and_report_dangling_ones">
//! Declared provenance: `cites="…"` is what an author SAYS an element rests
//! on. It resolves to places so it can be drawn, and it weaves as an
//! assertion in words — never as lineage, never as a check.
//! Guarantee: docs/guarantees/lineage/three-provenances-are-drawn-apart.md

use hickory_cli::{CheckFailure, CheckOutcome, check_outcome, dangling_citations, declared_cites};

#[test]
fn cites_resolve_across_the_chain_and_report_dangling_ones() {
    let dir = tempfile::tempdir().unwrap();
    let meeting = dir.path().join("meeting.md");
    std::fs::write(
        &meeting,
        "<hick:transcript id=\"t\" format=\"vtt\">\nWEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Forty is safe.\n</hick:transcript>\n",
    )
    .unwrap();
    let note = dir.path().join("note.md");
    std::fs::write(
        &note,
        "<hick:upstream file=\"meeting.md\" />\n\n<hick:copy id=\"finding\" class=\"f\">Pool is 40.</hick:copy>\n\n<hick:claim by=\"nate\" standing=\"judgment\" cites=\"#t-u1,#finding,#missing\">We are fine.</hick:claim>\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&note).unwrap();
    let cites = declared_cites(&note, &source).unwrap();
    assert_eq!(cites.len(), 1, "{cites:?}");
    let c = &cites[0];
    assert_eq!(c.from.element, "claim");
    assert_eq!((c.from.first_line, c.from.last_line), (5, 5));
    assert_eq!(
        c.to.len(),
        2,
        "the dangling #missing resolves to nothing: {:?}",
        c.to
    );
    // NAMED, not merely absent. This assertion is the point of the test's
    // own title and was missing: `to` having two entries says nothing about
    // WHICH of the three selectors contributed them, so `#missing` was
    // dropped in silence and wove beside the two that resolved.
    assert_eq!(c.dangling, vec!["#missing".to_string()], "{c:?}");
    let turn =
        c.to.iter()
            .find(|p| p.element == "said")
            .expect("the meeting turn");
    assert!(turn.path.ends_with("meeting.md"), "{}", turn.path);
    assert_eq!(turn.first_line, 5);
    let finding = c.to.iter().find(|p| p.element == "copy").unwrap();
    assert!(finding.path.ends_with("note.md"));
    assert_eq!(finding.id.as_deref(), Some("finding"));
}

</storycode:copy>

</storycode:doc>
