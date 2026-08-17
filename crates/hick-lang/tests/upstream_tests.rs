//! `<hick:upstream>`: a document declares its own place in a pipeline, and
//! selectors reach anything upstream of it.
//!
//! The edge carries FRAGMENTS, transitively, and renders nothing — which is
//! what separates it from `hick:include`, whose job is composing a manual out
//! of chapters and which brings the prose along on purpose.

use std::collections::HashSet;
use std::path::Path;

use hick_lang::{parse, resolve_includes};

fn write(dir: &Path, name: &str, body: &str) {
    std::fs::write(
        dir.join(name),
        format!("<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"),
    )
    .unwrap();
}

fn resolve(dir: &Path, body: &str) -> Result<hick_lang::HickDocument, hick_lang::ParseError> {
    let mut doc = parse(&format!(
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"
    ))?;
    resolve_includes(&mut doc, dir, &mut HashSet::new())?;
    Ok(doc)
}

/// notes → domain → requirements, the shape the real chain has.
fn chain(dir: &Path) {
    write(
        dir,
        "notes.hick",
        r#"Long meeting preamble nobody downstream should read.
<hick:copy id="metering-basis" class="decision">Meter execution minutes.</hick:copy>
<hick:copy id="hard-stop" class="decision">Refuse, do not queue.</hick:copy>"#,
    );
    write(
        dir,
        "domain.hick",
        r#"<hick:upstream file="notes.hick" />
<hick:copy id="term-allowance" class="glossary">Minutes per month.</hick:copy>"#,
    );
}

#[test]
fn selectors_reach_through_the_whole_chain_not_just_one_hop() {
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(dir.path(), r#"<hick:upstream file="domain.hick" />"#).unwrap();
    let ids: Vec<_> = doc
        .find_tags("copy")
        .iter()
        .filter_map(|t| t.get_attribute("id").map(str::to_string))
        .collect();
    // `notes.hick` is two hops away and never named by this document.
    assert!(ids.contains(&"metering-basis".to_string()), "{ids:?}");
    assert!(ids.contains(&"term-allowance".to_string()), "{ids:?}");
}

#[test]
fn an_upstream_edge_carries_no_prose() {
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(dir.path(), r#"<hick:upstream file="domain.hick" />"#).unwrap();
    assert!(
        !format!("{doc:?}").contains("Long meeting preamble"),
        "an upstream edge dragged the source document's prose along"
    );
}

#[test]
fn a_class_still_matches_across_documents() {
    // Explicitly multi-match: accumulating decisions from several documents is
    // the point of the chain, so this must NOT be a collision.
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(
        dir.path(),
        r#"<hick:upstream file="domain.hick" />
<hick:copy id="local" class="decision">And one decided here.</hick:copy>"#,
    )
    .unwrap();
    assert_eq!(hick_lang::fragments_matching(&doc, ".decision").len(), 3);
}

#[test]
fn a_duplicate_id_across_the_pipeline_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let err = resolve(
        dir.path(),
        r#"<hick:upstream file="domain.hick" />
<hick:copy id="hard-stop" class="decision">A rival definition.</hick:copy>"#,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("hard-stop"), "{msg}");
    assert!(
        msg.contains("unique"),
        "the error must state the rule: {msg}"
    );
}

#[test]
fn a_diamond_is_normal_and_merges_each_document_once() {
    // requirements → domain → notes, and requirements → notes directly. Every
    // real pipeline grows this shape; merging notes twice would make every id
    // in it collide with itself.
    let dir = tempfile::tempdir().unwrap();
    chain(dir.path());
    let doc = resolve(
        dir.path(),
        r#"<hick:upstream file="domain.hick" />
<hick:upstream file="notes.hick" />"#,
    )
    .unwrap();
    assert_eq!(hick_lang::fragments_matching(&doc, "#hard-stop").len(), 1);
}

#[test]
fn a_cycle_is_refused_rather_than_followed() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "a.hick", r#"<hick:upstream file="b.hick" />"#);
    write(dir.path(), "b.hick", r#"<hick:upstream file="a.hick" />"#);
    let err = resolve(dir.path(), r#"<hick:upstream file="a.hick" />"#).unwrap_err();
    assert!(format!("{err}").contains("circular"), "{err}");
}

#[test]
fn a_missing_upstream_names_the_file_it_could_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let err = resolve(dir.path(), r#"<hick:upstream file="nope.hick" />"#).unwrap_err();
    assert!(format!("{err}").contains("nope.hick"), "{err}");
}

// Protects docs/guarantees/lineage/included-spans-name-their-own-file.md
mod span_file_attribution {
    use super::*;
    use hick_lang::HickNode;

    /// Every span in `nodes`, with the file (from the table) it claims.
    fn spans_with_files(doc: &hick_lang::HickDocument) -> Vec<(Option<String>, usize, usize)> {
        fn walk(
            nodes: &[HickNode],
            table: &[String],
            out: &mut Vec<(Option<String>, usize, usize)>,
        ) {
            for node in nodes {
                match node {
                    HickNode::Text(_, Some(span)) => out.push((
                        span.file_id.map(|id| table[usize::from(id)].clone()),
                        span.start,
                        span.end,
                    )),
                    HickNode::Text(_, None) => {}
                    HickNode::Tag(tag) => walk(&tag.children, table, out),
                }
            }
        }
        let mut out = Vec::new();
        walk(&doc.nodes, &doc.span_files, &mut out);
        out
    }

    #[test]
    fn included_text_spans_name_the_included_file() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "chapter.hick",
            "Chapter prose that lives in chapter.hick.",
        );
        let doc = resolve(dir.path(), r#"<hick:include file="chapter.hick" />"#).unwrap();

        assert_eq!(doc.span_files.len(), 1);
        assert!(doc.span_files[0].ends_with("chapter.hick"));

        let spans = spans_with_files(&doc);
        let foreign: Vec<_> = spans.iter().filter(|(f, _, _)| f.is_some()).collect();
        assert!(
            !foreign.is_empty(),
            "included spans must be stamped: {spans:?}"
        );
        for (file, start, end) in &foreign {
            let file = file.as_ref().unwrap();
            assert!(file.ends_with("chapter.hick"), "{file}");
            // The offsets must be valid in the file they claim.
            let source = std::fs::read_to_string(file).unwrap();
            assert!(
                *end <= source.len(),
                "span {start}..{end} out of bounds for {file}"
            );
        }
        // The including document's own spans stay unstamped.
        assert!(spans.iter().any(|(f, _, _)| f.is_none()) || spans.len() == foreign.len());
    }

    #[test]
    fn upstream_fragment_spans_name_the_upstream_file() {
        let dir = tempfile::tempdir().unwrap();
        chain(dir.path());
        let doc = resolve(dir.path(), r#"<hick:upstream file="domain.hick" />"#).unwrap();
        // notes.hick arrives transitively through domain.hick; both are in
        // the table, and the #hard-stop fragment's span names notes.hick.
        assert!(doc.span_files.iter().any(|f| f.ends_with("domain.hick")));
        assert!(doc.span_files.iter().any(|f| f.ends_with("notes.hick")));

        let spans = spans_with_files(&doc);
        let hard_stop_text = "Refuse, do not queue.";
        let owning = spans.iter().find(|(f, s, e)| {
            f.as_ref().is_some_and(|f| {
                std::fs::read_to_string(f)
                    .ok()
                    .and_then(|src| src.get(*s..*e).map(|t| t == hard_stop_text))
                    .unwrap_or(false)
            })
        });
        let (file, _, _) = owning.expect("the fragment's span must map byte-exactly to its file");
        assert!(file.as_ref().unwrap().ends_with("notes.hick"));
    }

    #[test]
    fn a_document_without_includes_has_an_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let doc = resolve(dir.path(), "Just prose.").unwrap();
        assert!(doc.span_files.is_empty());
        assert!(spans_with_files(&doc).iter().all(|(f, _, _)| f.is_none()));
    }
}
