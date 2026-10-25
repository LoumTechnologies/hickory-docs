<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-transcript/tests/upstream.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Transcript Derives Its Speaker Turns, And Cannot Disagree With Itself](../../../../contract.md)

Captured SHA-256: `4c5d6ea02e09dc7d75fd30f2a9670894fc4b1debce41621a5824e5e8426e5c18`.

<book:entry path="crates/hick-transcript/tests/upstream.rs">
//! A meeting upstream of a note: the transcript travels the `hick:upstream`
//! edge like any fragment, and its turns are derived AFTER the splice.

use std::collections::HashSet;

fn write(dir: &std::path::Path, name: &str, body: &str) {
    std::fs::write(
        dir.join(name),
        format!("<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"),
    )
    .unwrap();
}

fn resolve(dir: &std::path::Path, body: &str) -> hick_lang::HickDocument {
    let mut doc = hick_lang::parse(&format!(
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"
    ))
    .unwrap();
    hick_lang::resolve_includes(&mut doc, dir, &mut HashSet::new()).unwrap();
    doc
}

/// A meeting note upstream of a document is quotable turn by turn: the
/// transcript travels the edge like any fragment, its turns are derived after
/// the splice, and they keep the meeting file's stamp so lineage points at
/// the meeting, not the quoting document.
/// Guarantee: docs/guarantees/authoring/a-transcript-derives-its-speaker-turns.md
#[test]
fn a_transcript_upstream_is_selectable_turn_by_turn() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "meeting.md",
        "<hick:transcript id=\"t\" format=\"vtt\">\nWEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Ship it.\n\n00:00:03.000 --> 00:00:04.000\n<v Ana>Not yet.\n</hick:transcript>",
    );
    let mut doc = resolve(dir.path(), r#"<hick:upstream file="meeting.md" />"#);
    hick_transcript::expand(&mut doc);
    let turn = hick_lang::fragments_matching(&doc, "#t-u2");
    assert_eq!(turn.len(), 1);
    assert_eq!(turn[0].text_content().trim(), "Not yet.");
    assert_eq!(hick_lang::fragments_matching(&doc, ".said-sam").len(), 1);
    let span = match &turn[0].children[0] {
        hick_lang::HickNode::Text(_, Some(span)) => *span,
        other => panic!("{other:?}"),
    };
    let file = &doc.span_files[usize::from(span.file_id.expect("stamped"))];
    assert!(file.ends_with("meeting.md"), "{file}");
}
</book:entry>

</book:doc>
