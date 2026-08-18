//! Claims: who asserted something, and on what footing.
//!
//! Protects docs/guarantees/authoring/a-claim-says-who-is-asserting-it.md
//!
//! A claim is the one thing in a note that nothing can verify — no fingerprint,
//! no re-derivation, no blame. So the two things worth testing are that it
//! survives into what a reader actually opens, and that the marking is never
//! allowed to look more authoritative than it is.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

fn parse(source: &str) -> hick_lang::HickDocument {
    hick_lang::parse(source).expect("the document parses")
}

fn note(body: &str) -> String {
    format!("# Sync\n\n{body}\n")
}

#[test]
fn a_claim_weaves_an_attribution_a_reader_can_see() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("sync.hick");
    std::fs::write(
        &doc,
        note(
            r#"<hick:claim by="sam" standing="expert" scope="postgres">
A partial index is safe at this write volume.
</hick:claim>"#,
        ),
    )
    .unwrap();

    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let woven = std::fs::read_to_string(dir.path().join("sync.md")).unwrap();
    assert!(woven.contains("**sam**"), "no attribution in:\n{woven}");
    assert!(woven.contains("expert"), "no standing in:\n{woven}");
    assert!(woven.contains("postgres"), "no scope in:\n{woven}");
    // The prose is carried through unchanged — the header is the marking, and
    // rewriting the claim's own bytes would cost it its lineage.
    assert!(
        woven.contains("A partial index is safe at this write volume."),
        "claim body missing:\n{woven}"
    );
    // The tag is source, not output.
    assert!(!woven.contains("hick:claim"), "tag leaked into:\n{woven}");
}

#[test]
fn an_unattributed_claim_is_warned_about() {
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        r#"<hick:claim standing="judgment">We will wait for three requests.</hick:claim>"#,
    )));
    assert!(
        warnings.iter().any(|w| w.contains("who is making it")),
        "expected an attribution warning, got: {warnings:?}"
    );
}

#[test]
fn a_claim_without_a_standing_is_warned_about() {
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        r#"<hick:claim by="nate">We will wait for three requests.</hick:claim>"#,
    )));
    let joined = warnings.join("\n");
    assert!(
        joined.contains("no `standing`"),
        "expected a standing warning, got: {warnings:?}"
    );
    // The warning has to teach the vocabulary, or nobody learns it.
    for standing in hick_lang::STANDINGS {
        assert!(
            joined.contains(standing),
            "{standing} missing from: {joined}"
        );
    }
}

#[test]
fn an_unknown_standing_names_the_value_and_the_vocabulary() {
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        r#"<hick:claim by="nate" standing="vibes">Ship it.</hick:claim>"#,
    )));
    let joined = warnings.join("\n");
    assert!(joined.contains("vibes"), "value not named: {joined}");
    assert!(joined.contains("expert"), "vocabulary not named: {joined}");
    assert!(
        joined.contains("standings:"),
        "the escape hatch is not mentioned: {joined}"
    );
}

#[test]
fn a_document_may_extend_the_standing_vocabulary() {
    // The escape hatch the previous test advertises has to actually work.
    let source = format!(
        "---\nstandings:\n  - vibes\n---\n\n{}",
        note(r#"<hick:claim by="nate" standing="vibes">Ship it.</hick:claim>"#)
    );
    let warnings = hickory_cli::claim_warnings(&parse(&source));
    assert!(
        !warnings.iter().any(|w| w.contains("vibes")),
        "declared standing still warned: {warnings:?}"
    );

    let inline = format!(
        "---\nstandings: [vibes]\n---\n\n{}",
        note(r#"<hick:claim by="nate" standing="vibes">Ship it.</hick:claim>"#)
    );
    assert!(
        !hickory_cli::claim_warnings(&parse(&inline))
            .iter()
            .any(|w| w.contains("vibes")),
        "the inline list spelling must work too"
    );
}

#[test]
fn expertise_without_a_scope_is_warned_about() {
    // Expertise is never global. `standing="expert"` with no scope is the
    // unfalsifiable form of exactly the distinction this feature exists for.
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        r#"<hick:claim by="sam" standing="expert">Everything is fine.</hick:claim>"#,
    )));
    assert!(
        warnings.iter().any(|w| w.contains("scope")),
        "expected a scope warning, got: {warnings:?}"
    );
}

#[test]
fn a_well_formed_claim_is_silent() {
    // Marking must not be nagging: a claim that says who, on what footing, and
    // about what has nothing left to warn about.
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        r#"<hick:claim by="sam" standing="expert" scope="postgres">Fine.</hick:claim>
<hick:claim by="nate" standing="judgment">Wait for three requests.</hick:claim>
<hick:claim by="nate" standing="report">Sam said the index is safe.</hick:claim>
<hick:claim by="nate" standing="assumption">Write volume stays flat.</hick:claim>"#,
    )));
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
}

#[test]
fn unmarked_prose_is_never_warned_about() {
    // Sparse by design. If ordinary prose produced warnings, every note would
    // become a wall of them and people would stop writing notes in the tool.
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        "Sam thinks the index is fine. We'll wait for three requests.",
    )));
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
}
