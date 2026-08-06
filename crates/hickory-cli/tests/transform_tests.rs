//! `<hick:transform>`: an LLM-written passage that a document can still verify.
//!
//! The claim a transform makes is deliberately weaker than a paste's. A paste
//! says "these bytes reproduce"; a transform says "these bytes were written
//! from EXACTLY those bytes under EXACTLY this instruction, and neither has
//! changed since". That claim is a fingerprint comparison, which is why
//! checking one is free, offline, and deterministic — the properties that let
//! LLM-written prose live inside a verified document at all.

use hickory_cli::{CheckFailure, stale_transforms, transform_input};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:copy id="limits" class="limits">    "open": 300,
    "pro": 2000,
</hick:copy>

<hick:transform select="#limits" from="FINGERPRINT"
                instruct="One sentence. Numbers exact.">
Open allows 300 execution minutes a month; Pro allows 2,000.
</hick:transform>
</hick:doc>
"##;

/// The document with a correct fingerprint stamped in.
fn stamped() -> String {
    let doc = hick_lang::parse(DOC).unwrap();
    let input = transform_input(&doc, "#limits");
    let fp = hick_lang::transform_fingerprint(&input, "One sentence. Numbers exact.");
    DOC.replace("FINGERPRINT", &fp)
}

#[test]
fn a_passage_whose_input_is_unchanged_is_not_stale() {
    let failures = stale_transforms(std::path::Path::new("d.hick"), &stamped()).unwrap();
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn changing_the_input_makes_the_passage_stale() {
    // The exact failure this element exists to close: a number in the machine
    // form moves and the prose beside it keeps saying the old one.
    let drifted = stamped().replace(r#""open": 300,"#, r#""open": 500,"#);
    let failures = stale_transforms(std::path::Path::new("d.hick"), &drifted).unwrap();
    assert_eq!(failures.len(), 1);
    match &failures[0] {
        CheckFailure::StaleTransform { select, line, .. } => {
            assert_eq!(select, "#limits");
            assert!(*line > 0, "the failure must point at a line");
        }
        other => panic!("expected StaleTransform, got {other:?}"),
    }
}

#[test]
fn changing_the_instruction_makes_the_passage_stale() {
    // Otherwise "summarize" -> "summarize briefly" would leave stale prose
    // still carrying its attestation.
    let reworded = stamped().replace("One sentence. Numbers exact.", "One sentence. Be terse.");
    let failures = stale_transforms(std::path::Path::new("d.hick"), &reworded).unwrap();
    assert_eq!(
        failures.len(),
        1,
        "the instruction must be inside the fingerprint"
    );
}

#[test]
fn editing_the_passage_by_hand_does_not_make_it_stale() {
    // A transform is not reproducible, so hand-edited wording is legitimate
    // authorship — not drift. Only the INPUTS are attested.
    let edited = stamped().replace(
        "Open allows 300 execution minutes a month; Pro allows 2,000.",
        "Open gives you 300 minutes of execution each month. Pro gives 2,000.",
    );
    let failures = stale_transforms(std::path::Path::new("d.hick"), &edited).unwrap();
    assert!(
        failures.is_empty(),
        "hand-editing a passage is not drift: {failures:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn weaving_emits_the_pinned_passage_and_calls_no_model() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("d.hick");
    std::fs::write(&doc_path, stamped()).unwrap();

    // No ANTHROPIC_API_KEY is set here: weaving must not need one.
    let run = hickory_cli::run_doc(
        &doc_path,
        &[],
        hickory_cli::RunMode::Weave,
        hickory_cli::ExecutorChoice::Local,
    )
    .await
    .unwrap();
    hickory_cli::write_outputs(&run, None).unwrap();

    let woven = std::fs::read_to_string(dir.path().join("out.md")).unwrap();
    assert!(
        woven.contains("Open allows 300 execution minutes a month; Pro allows 2,000."),
        "the pinned passage must weave verbatim:\n{woven}"
    );
}
