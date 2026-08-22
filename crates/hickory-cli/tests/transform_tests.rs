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

/// A transform that summarizes something UPSTREAM — a meeting turn, a decision
/// — must fingerprint the same bytes `hick:paste` would find at weave. Before
/// this test, `hick test` and `hick refresh` parsed the document without
/// resolving `hick:upstream`, so the selection was empty: the passage was
/// stamped over nothing and never went stale when the meeting changed.
/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let meeting = dir.path().join("meeting.hick");
    let note = dir.path().join("note.hick");
    std::fs::write(
        &meeting,
        r##"<hick:copy id="decision" class="decision">Ship on Friday.</hick:copy>
"##,
    )
    .unwrap();
    let note_src = r##"<hick:upstream file="meeting.hick" />

<hick:transform select="#decision" from="FINGERPRINT" instruct="One line.">
We ship Friday.
</hick:transform>
"##;
    let fp = hick_lang::transform_fingerprint("[#decision] Ship on Friday.", "One line.");
    std::fs::write(&note, note_src.replace("FINGERPRINT", &fp)).unwrap();

    let source = std::fs::read_to_string(&note).unwrap();
    let failures = stale_transforms(&note, &source).unwrap();
    assert!(
        failures.is_empty(),
        "stamped over the upstream bytes: {failures:?}"
    );

    // The meeting changes; the note that summarized it is now stale — which is
    // the whole point of a transform that reads another document.
    std::fs::write(
        &meeting,
        r##"<hick:copy id="decision" class="decision">Ship on Monday.</hick:copy>
"##,
    )
    .unwrap();
    let failures = stale_transforms(&note, &source).unwrap();
    assert_eq!(failures.len(), 1, "{failures:?}");
}

/// An included file's transforms are the included file's to check: the
/// includer must not report them (twice) as its own, because `hick refresh`
/// on the includer would then write a passage at the wrong file's offsets.
#[test]
fn an_included_files_transforms_are_not_the_includers() {
    let dir = tempfile::tempdir().unwrap();
    let chapter = dir.path().join("chapter.hick");
    let book = dir.path().join("book.hick");
    std::fs::write(
        &chapter,
        r##"<hick:copy id="facts" class="facts">Three states.</hick:copy>
<hick:transform select="#facts" from="stale000" instruct="Summarize.">
Old summary.
</hick:transform>
"##,
    )
    .unwrap();
    std::fs::write(&book, "<hick:include file=\"chapter.hick\" />\n").unwrap();
    let source = std::fs::read_to_string(&book).unwrap();
    let failures = stale_transforms(&book, &source).unwrap();
    assert!(failures.is_empty(), "{failures:?}");
    let chapter_src = std::fs::read_to_string(&chapter).unwrap();
    assert_eq!(stale_transforms(&chapter, &chapter_src).unwrap().len(), 1);
}

/// `hick:check` is a transform spelled for one question: `claim=` + `against=`
/// are its selection, the built-in instruction is its instruction, and it is
/// fingerprinted, checked, and refreshed exactly like one.
/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn a_check_is_a_transform_with_the_question_built_in() {
    let src = "<hick:copy id=\"m1\" class=\"message\">We ship Friday.</hick:copy>\n<hick:copy id=\"f\" class=\"finding\">Ship date: Friday.</hick:copy>\n<hick:check claim=\"#m1\" against=\".finding\" from=\"FP\">\nBACKED.\n</hick:check>\n";
    let doc = hickory_cli::transform_document(std::path::Path::new("d.hick"), src).unwrap();
    let check = doc.tags().find(|t| t.name == "check").unwrap();
    let (select, instruct) = hickory_cli::transform_spec(check);
    assert_eq!(select, "#m1,.finding");
    assert_eq!(instruct, hickory_cli::CHECK_INSTRUCT);
    let fp = hick_lang::transform_fingerprint(&transform_input(&doc, &select), &instruct);
    let stamped = src.replace("FP", &fp);
    assert!(
        stale_transforms(std::path::Path::new("d.hick"), &stamped)
            .unwrap()
            .is_empty()
    );
    let drifted = stamped.replace("Ship date: Friday.", "Ship date: Monday.");
    assert_eq!(
        stale_transforms(std::path::Path::new("d.hick"), &drifted)
            .unwrap()
            .len(),
        1
    );
}

/// The input a transform reads is citeable: fragments are separated, carry
/// their ids, and a speaker turn names its speaker — so a passage that checks
/// a sentence against a meeting can say which turn backs it.
/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn the_transform_input_is_one_labelled_paragraph_per_fragment() {
    let src = "<hick:transcript id=\"t\" format=\"vtt\">\nWEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Ship it.\n\n00:00:03.000 --> 00:00:04.000\n<v Ana>Not yet.\n</hick:transcript>\n<hick:copy id=\"m1\" class=\"message\">We ship.</hick:copy>\n";
    let doc = hickory_cli::transform_document(std::path::Path::new("d.hick"), src).unwrap();
    assert_eq!(
        transform_input(&doc, "#m1,.said-ana"),
        "[#t-u2] Ana (00:00:03.000): Not yet.\n\n[#m1] We ship."
    );
    assert_eq!(
        transform_input(&doc, "#t"),
        "[#t-u1] Sam (00:00:01.000): Ship it.\n[#t-u2] Ana (00:00:03.000): Not yet."
    );
}
