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
