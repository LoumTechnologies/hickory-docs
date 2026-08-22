//! Links between documents: what they become in the weave, and whether the
//! prose around them keeps its lineage.
//!
//! Two guarantees are at stake here and they are easy to satisfy one at a
//! time and lose the other:
//!
//!  - `docs/guarantees/authoring/a-link-between-documents-lands-in-the-weave.md`
//!  - `docs/guarantees/lineage/a-rewritten-link-keeps-its-paragraph-attached.md`
//!
//! Rewriting the destination is the easy half. The hard half is that the
//! rewrite must not turn the surrounding paragraph synthetic — which it did,
//! for every substitution, until `ProvenanceTransformNode` learned to narrow
//! a passthrough segment's span to the bytes it covers.

use hick_exec::node::SourceOrigin;

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

#[tokio::test]
async fn a_link_to_another_document_points_at_the_markdown_it_weaves() {
    let src = hick_doc("See [the plan](plan.hick) and [the chart](assets/c.png).");
    let result = hick_literate::run_pipeline(&[("notes.hick", &src)], &[])
        .await
        .unwrap();
    let woven = result.files.get("notes.md").unwrap().to_string();
    assert!(
        woven.contains("[the plan](plan.md)"),
        "the document link should point at the weave: {woven}"
    );
    assert!(
        woven.contains("[the chart](assets/c.png)"),
        "everything that is not a document link is untouched: {woven}"
    );
}

#[tokio::test]
async fn a_fragment_and_a_url_survive_the_rewrite_intact() {
    let src = hick_doc(
        "Jump to [risks](plan.hick#risks), [here](#top), or [away](https://example.com/a.hick).",
    );
    let result = hick_literate::run_pipeline(&[("notes.hick", &src)], &[])
        .await
        .unwrap();
    let woven = result.files.get("notes.md").unwrap().to_string();
    assert!(woven.contains("[risks](plan.md#risks)"), "{woven}");
    assert!(woven.contains("[here](#top)"), "{woven}");
    assert!(
        woven.contains("[away](https://example.com/a.hick)"),
        "{woven}"
    );
}

#[tokio::test]
async fn the_prose_around_a_rewritten_link_keeps_its_ribbon() {
    // The label and the words on both sides of the link came from the
    // document and must still say so — that is what draws a ribbon from the
    // sentence in the source to the sentence in the weave. Only the four
    // bytes the weaver changed are allowed to be synthetic.
    let src = hick_doc("Before [the plan](plan.hick) after.");
    let result = hick_literate::run_pipeline(&[("notes.hick", &src)], &[])
        .await
        .unwrap();
    let woven = result.files.get("notes.md").unwrap().to_string();
    let map = result.provenance_maps.get("notes.md").unwrap();

    let literal_at = |needle: &str| {
        let at = woven
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not in {woven}"));
        matches!(map.origin_at(at), Some(SourceOrigin::Literal { .. }))
    };
    assert!(
        literal_at("Before "),
        "the words before the link lost their origin"
    );
    assert!(literal_at("the plan"), "the link's label lost its origin");
    assert!(
        literal_at(" after."),
        "the words after the link lost their origin"
    );

    // And the rewritten destination is honestly marked as the weaver's.
    let at = woven.find("plan.md").unwrap();
    assert!(
        matches!(map.origin_at(at), Some(SourceOrigin::Synthetic)),
        "the rewritten destination is the weaver's, not the author's"
    );
}

#[tokio::test]
async fn a_span_after_a_rewritten_link_still_points_at_the_right_source_bytes() {
    // The failure this catches is a silent one: if the transform advanced by
    // the OUTPUT length rather than the length of what it replaced, every
    // span after the first link would slide by the difference — a reverse
    // edit would land two characters off, in the wrong place, with no error.
    let src = hick_doc("x [a](one.hick) y");
    let result = hick_literate::run_pipeline(&[("notes.hick", &src)], &[])
        .await
        .unwrap();
    let woven = result.files.get("notes.md").unwrap().to_string();
    let map = result.provenance_maps.get("notes.md").unwrap();
    let at = woven.find(") y").unwrap();
    let Some(SourceOrigin::Literal { span, .. }) = map.origin_at(at) else {
        panic!("the tail of the sentence should still be literal: {woven}");
    };
    // The span covers the rest of the prose node, so it is the START that
    // pins the alignment: a slide would leave it a character or two off.
    assert!(
        src[span.start..span.end].starts_with(") y"),
        "the span must index the bytes it claims, got {:?}",
        &src[span.start..span.end]
    );
}
