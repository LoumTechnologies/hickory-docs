//! Declared provenance: `cites="…"` is what an author SAYS an element rests
//! on. It resolves to places so it can be drawn, and it weaves as an
//! assertion in words — never as lineage, never as a check.
//! Guarantee: docs/guarantees/lineage/three-provenances-are-drawn-apart.md

use hickory_cli::declared_cites;

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
