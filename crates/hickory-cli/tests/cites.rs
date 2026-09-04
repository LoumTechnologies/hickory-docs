//! Declared provenance: `cites="…"` is what an author SAYS an element rests
//! on. It resolves to places so it can be drawn, and it weaves as an
//! assertion in words — never as lineage, never as a check.
//! Guarantee: docs/guarantees/lineage/three-provenances-are-drawn-apart.md

use hickory_cli::{CheckFailure, CheckOutcome, check_outcome, dangling_citations, declared_cites};

#[test]
fn cites_resolve_across_the_chain_and_report_dangling_ones() {
    let dir = tempfile::tempdir().unwrap();
    let meeting = dir.path().join("meeting.hick");
    std::fs::write(
        &meeting,
        "<hick:transcript id=\"t\" format=\"vtt\">\nWEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Forty is safe.\n</hick:transcript>\n",
    )
    .unwrap();
    let note = dir.path().join("note.hick");
    std::fs::write(
        &note,
        "<hick:upstream file=\"meeting.hick\" />\n\n<hick:copy id=\"finding\" class=\"f\">Pool is 40.</hick:copy>\n\n<hick:claim by=\"nate\" standing=\"judgment\" cites=\"#t-u1,#finding,#missing\">We are fine.</hick:claim>\n",
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
    assert!(turn.path.ends_with("meeting.hick"), "{}", turn.path);
    assert_eq!(turn.first_line, 5);
    let finding = c.to.iter().find(|p| p.element == "copy").unwrap();
    assert!(finding.path.ends_with("note.hick"));
    assert_eq!(finding.id.as_deref(), Some("finding"));
}

#[test]
fn a_cited_claim_weaves_its_citation_as_words() {
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("n.hick");
    std::fs::write(
        &note,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"n.md\">\n<hick:copy id=\"a\">x</hick:copy>\n<hick:claim by=\"nate\" standing=\"report\" cites=\"#a\">Said so.</hick:claim>\n</hick:doc>\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
        .args(["weave", note.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("n.md")).unwrap();
    assert!(woven.contains("Said so."), "{woven}");
    assert!(woven.contains("*cites: #a*"), "{woven}");
    assert!(!woven.contains("✓"), "never a checkmark: {woven}");
}

/// Protects docs/guarantees/lineage/a-citation-that-points-at-nothing-fails.md
#[test]
fn a_dangling_citation_fails_verification() {
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("note.hick");
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

#[test]
fn a_citation_that_resolves_is_not_reported() {
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("note.hick");
    std::fs::write(
        &note,
        "<hick:copy id=\"finding\" class=\"f\">Pool is 40.</hick:copy>\n\n         <hick:claim by=\"nate\" standing=\"judgment\" cites=\"#finding,.f\">We are fine.</hick:claim>\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&note).unwrap();
    assert!(dangling_citations(&note, &source).unwrap().is_empty());
    // Two selectors, one fragment: the same place cited twice is listed once.
    let cites = declared_cites(&note, &source).unwrap();
    assert_eq!(cites[0].to.len(), 1, "{:?}", cites[0].to);
    assert!(cites[0].dangling.is_empty());
}
