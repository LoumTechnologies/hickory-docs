<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Claim Says Who Is Asserting It, And Never Looks Verified

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:claim>` in a document, when the document is woven, then it
emits an attribution naming who made the claim, on what standing, and about
what scope — and the claim's own prose passes through unchanged. When the
marking is incomplete or uses a standing the document does not know, then a
**warning** names the problem, the value, and the vocabulary; never an error.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-claim-says-who-is-asserting-it.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a87e30ba2c23"></a>
## `crates/hickory-cli/tests/claims.rs`

Related contracts: [A Claim Says Who Is Asserting It, And Never Looks Verified](contract.md).

<a id="f-a87e30ba2c23-hick"></a>
### `hick`

Claims: who asserted something, and on what footing.
Protects docs/guarantees/authoring/a-claim-says-who-is-asserting-it.md
A claim is the one thing in a note that nothing can verify — no fingerprint,
no re-derivation, no blame. So the two things worth testing are that it
survives into what a reader actually opens, and that the marking is never
allowed to look more authoritative than it is.

<storycode:copy id="f-a87e30ba2c23-hick">
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

</storycode:copy>
<a id="f-a87e30ba2c23-parse"></a>
### `parse`

<storycode:copy id="f-a87e30ba2c23-parse">
fn parse(source: &str) -> hick_lang::HickDocument {
    hick_lang::parse(source).expect("the document parses")
}

</storycode:copy>
<a id="f-a87e30ba2c23-note"></a>
### `note`

<storycode:copy id="f-a87e30ba2c23-note">
fn note(body: &str) -> String {
    format!("# Sync\n\n{body}\n")
}

</storycode:copy>
<a id="f-a87e30ba2c23-a_claim_weaves_an_attribution_a_reader_can_see"></a>
### `a_claim_weaves_an_attribution_a_reader_can_see`

<storycode:copy id="f-a87e30ba2c23-a_claim_weaves_an_attribution_a_reader_can_see">
#[test]
fn a_claim_weaves_an_attribution_a_reader_can_see() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("sync.md");
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

</storycode:copy>
<a id="f-a87e30ba2c23-an_unattributed_claim_is_warned_about"></a>
### `an_unattributed_claim_is_warned_about`

<storycode:copy id="f-a87e30ba2c23-an_unattributed_claim_is_warned_about">
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

</storycode:copy>
<a id="f-a87e30ba2c23-a_claim_without_a_standing_is_warned_about"></a>
### `a_claim_without_a_standing_is_warned_about`

<storycode:copy id="f-a87e30ba2c23-a_claim_without_a_standing_is_warned_about">
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

</storycode:copy>
<a id="f-a87e30ba2c23-an_unknown_standing_names_the_value_and_the_vocabulary"></a>
### `an_unknown_standing_names_the_value_and_the_vocabulary`

<storycode:copy id="f-a87e30ba2c23-an_unknown_standing_names_the_value_and_the_vocabulary">
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

</storycode:copy>
<a id="f-a87e30ba2c23-a_document_may_extend_the_standing_vocabulary"></a>
### `a_document_may_extend_the_standing_vocabulary`

<storycode:copy id="f-a87e30ba2c23-a_document_may_extend_the_standing_vocabulary">
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

</storycode:copy>
<a id="f-a87e30ba2c23-expertise_without_a_scope_is_warned_about"></a>
### `expertise_without_a_scope_is_warned_about`

<storycode:copy id="f-a87e30ba2c23-expertise_without_a_scope_is_warned_about">
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

</storycode:copy>
<a id="f-a87e30ba2c23-a_well_formed_claim_is_silent"></a>
### `a_well_formed_claim_is_silent`

<storycode:copy id="f-a87e30ba2c23-a_well_formed_claim_is_silent">
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

</storycode:copy>
<a id="f-a87e30ba2c23-unmarked_prose_is_never_warned_about"></a>
### `unmarked_prose_is_never_warned_about`

<storycode:copy id="f-a87e30ba2c23-unmarked_prose_is_never_warned_about">
#[test]
fn unmarked_prose_is_never_warned_about() {
    // Sparse by design. If ordinary prose produced warnings, every note would
    // become a wall of them and people would stop writing notes in the tool.
    let warnings = hickory_cli::claim_warnings(&parse(&note(
        "Sam thinks the index is fine. We'll wait for three requests.",
    )));
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/claims.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/claims.rs"><storycode:paste select="#f-a87e30ba2c23-hick"/><storycode:paste select="#f-a87e30ba2c23-parse"/><storycode:paste select="#f-a87e30ba2c23-note"/><storycode:paste select="#f-a87e30ba2c23-a_claim_weaves_an_attribution_a_reader_can_see"/><storycode:paste select="#f-a87e30ba2c23-an_unattributed_claim_is_warned_about"/><storycode:paste select="#f-a87e30ba2c23-a_claim_without_a_standing_is_warned_about"/><storycode:paste select="#f-a87e30ba2c23-an_unknown_standing_names_the_value_and_the_vocabulary"/><storycode:paste select="#f-a87e30ba2c23-a_document_may_extend_the_standing_vocabulary"/><storycode:paste select="#f-a87e30ba2c23-expertise_without_a_scope_is_warned_about"/><storycode:paste select="#f-a87e30ba2c23-a_well_formed_claim_is_silent"/><storycode:paste select="#f-a87e30ba2c23-unmarked_prose_is_never_warned_about"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hickory-cli/tests/claims.rs](contract.md#output-a87e30ba2c23)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
