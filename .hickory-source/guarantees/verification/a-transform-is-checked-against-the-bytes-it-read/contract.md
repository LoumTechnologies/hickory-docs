<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Transform Is Checked Against The Bytes It Read, Wherever They Live

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a `<hick:transform select="…" instruct="…" from="…">`, when `hick test`
checks it or `hick refresh` rewrites it, then the fingerprint is taken over
the **same bytes `hick:paste` would find at weave** — including fragments the
document reaches through `hick:upstream` or `hick:include` — so a passage
summarizing a meeting turn in another document goes stale the moment that
turn changes. When `hick refresh` writes a passage, then it stamps `from=`
**and** the `provider=` and `model=` that wrote it, by the tag's own span, so
a document that binds another prefix (`<slack:transform>`) is restamped too.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-51aecdb4aa76"></a>
## `crates/hickory-cli/src/restamp_tests.rs`

Related contracts: [A Transform Is Checked Against The Bytes It Read, Wherever They Live](contract.md).

<a id="f-51aecdb4aa76-restamp_rewrites_an_existing_fingerprint_and_inserts_the_model"></a>
### `restamp_rewrites_an_existing_fingerprint_and_inserts_the_model`

<storycode:copy id="f-51aecdb4aa76-restamp_rewrites_an_existing_fingerprint_and_inserts_the_model">
use super::restamp;

/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn restamp_rewrites_an_existing_fingerprint_and_inserts_the_model() {
    let src = "<hick:transform select=\"#a\" from=\"old\" instruct=\"x\">\nbody\n</hick:transform>";
    let body = src.find('>').unwrap() + 1;
    let out = restamp(
        src,
        0,
        body,
        &[
            ("from", "new1"),
            ("provider", "anthropic"),
            ("model", "claude-sonnet-5"),
        ],
    );
    assert!(out.starts_with(
        "<hick:transform provider=\"anthropic\" model=\"claude-sonnet-5\" select=\"#a\" from=\"new1\" instruct=\"x\">\nbody"
    ), "{out}");
}

</storycode:copy>
<a id="f-51aecdb4aa76-cited_ids_keeps_only_ids_the_input_labelled"></a>
### `cited_ids_keeps_only_ids_the_input_labelled`

The passage's own `#id` mentions, filtered to what it was shown,
become `cites=` — declared, and only ever what the input labelled.

<storycode:copy id="f-51aecdb4aa76-cited_ids_keeps_only_ids_the_input_labelled">
/// The passage's own `#id` mentions, filtered to what it was shown,
/// become `cites=` — declared, and only ever what the input labelled.
#[test]
fn cited_ids_keeps_only_ids_the_input_labelled() {
    let input = "[#m1] Sentence.\n\n[#transcript-u3] Sam: The SLO.\n\n[#p95] 212 ms.";
    let passage = "BACKED by [#transcript-u3] and #p95; also #p95 again and #nope and #m1.";
    assert_eq!(super::cited_ids(passage, input), "#transcript-u3,#p95,#m1");
    assert_eq!(super::cited_ids("nothing here", input), "");
}

</storycode:copy>
<a id="f-51aecdb4aa76-restamp_works_under_a_rebound_prefix"></a>
### `restamp_works_under_a_rebound_prefix`

A document may bind any prefix to the namespace; refresh must find the
tag it is rewriting by its span, not by the spelling `<hick:transform`.

<storycode:copy id="f-51aecdb4aa76-restamp_works_under_a_rebound_prefix">
/// A document may bind any prefix to the namespace; refresh must find the
/// tag it is rewriting by its span, not by the spelling `<hick:transform`.
#[test]
fn restamp_works_under_a_rebound_prefix() {
    let src = "intro\n<slack:transform select=\"#a\" instruct=\"x\">\nbody\n</slack:transform>";
    let open = src.find("<slack:").unwrap();
    let body = src.find('>').unwrap() + 1;
    let out = restamp(src, open, body, &[("from", "abcd1234")]);
    assert!(
        out.contains("<slack:transform from=\"abcd1234\" select=\"#a\""),
        "{out}"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/restamp_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/restamp_tests.rs"><storycode:paste select="#f-51aecdb4aa76-restamp_rewrites_an_existing_fingerprint_and_inserts_the_model"/><storycode:paste select="#f-51aecdb4aa76-cited_ids_keeps_only_ids_the_input_labelled"/><storycode:paste select="#f-51aecdb4aa76-restamp_works_under_a_rebound_prefix"/></storycode:file>

<a id="output-c791fc613faa"></a>
## `crates/hickory-cli/tests/transform_tests.rs`

Related contracts: [A Transform Is Checked Against The Bytes It Read, Wherever They Live](contract.md).

<a id="f-c791fc613faa-DOC"></a>
### `DOC`

`<hick:transform>`: an LLM-written passage that a document can still verify.
The claim a transform makes is deliberately weaker than a paste's. A paste
says "these bytes reproduce"; a transform says "these bytes were written
from EXACTLY those bytes under EXACTLY this instruction, and neither has
changed since". That claim is a fingerprint comparison, which is why
checking one is free, offline, and deterministic — the properties that let
LLM-written prose live inside a verified document at all.

<storycode:copy id="f-c791fc613faa-DOC">
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

</storycode:copy>
<a id="f-c791fc613faa-stamped"></a>
### `stamped`

The document with a correct fingerprint stamped in.

<storycode:copy id="f-c791fc613faa-stamped">
/// The document with a correct fingerprint stamped in.
fn stamped() -> String {
    let doc = hick_lang::parse(DOC).unwrap();
    let input = transform_input(&doc, "#limits");
    let fp = hick_lang::transform_fingerprint(&input, "One sentence. Numbers exact.");
    DOC.replace("FINGERPRINT", &fp)
}

</storycode:copy>
<a id="f-c791fc613faa-a_passage_whose_input_is_unchanged_is_not_stale"></a>
### `a_passage_whose_input_is_unchanged_is_not_stale`

<storycode:copy id="f-c791fc613faa-a_passage_whose_input_is_unchanged_is_not_stale">
#[test]
fn a_passage_whose_input_is_unchanged_is_not_stale() {
    let failures = stale_transforms(std::path::Path::new("d.md"), &stamped()).unwrap();
    assert!(failures.is_empty(), "{failures:?}");
}

</storycode:copy>
<a id="f-c791fc613faa-changing_the_input_makes_the_passage_stale"></a>
### `changing_the_input_makes_the_passage_stale`

<storycode:copy id="f-c791fc613faa-changing_the_input_makes_the_passage_stale">
#[test]
fn changing_the_input_makes_the_passage_stale() {
    // The exact failure this element exists to close: a number in the machine
    // form moves and the prose beside it keeps saying the old one.
    let drifted = stamped().replace(r#""open": 300,"#, r#""open": 500,"#);
    let failures = stale_transforms(std::path::Path::new("d.md"), &drifted).unwrap();
    assert_eq!(failures.len(), 1);
    match &failures[0] {
        CheckFailure::StaleTransform { select, line, .. } => {
            assert_eq!(select, "#limits");
            assert!(*line > 0, "the failure must point at a line");
        }
        other => panic!("expected StaleTransform, got {other:?}"),
    }
}

</storycode:copy>
<a id="f-c791fc613faa-changing_the_instruction_makes_the_passage_stale"></a>
### `changing_the_instruction_makes_the_passage_stale`

<storycode:copy id="f-c791fc613faa-changing_the_instruction_makes_the_passage_stale">
#[test]
fn changing_the_instruction_makes_the_passage_stale() {
    // Otherwise "summarize" -> "summarize briefly" would leave stale prose
    // still carrying its attestation.
    let reworded = stamped().replace("One sentence. Numbers exact.", "One sentence. Be terse.");
    let failures = stale_transforms(std::path::Path::new("d.md"), &reworded).unwrap();
    assert_eq!(
        failures.len(),
        1,
        "the instruction must be inside the fingerprint"
    );
}

</storycode:copy>
<a id="f-c791fc613faa-editing_the_passage_by_hand_does_not_make_it_stale"></a>
### `editing_the_passage_by_hand_does_not_make_it_stale`

<storycode:copy id="f-c791fc613faa-editing_the_passage_by_hand_does_not_make_it_stale">
#[test]
fn editing_the_passage_by_hand_does_not_make_it_stale() {
    // A transform is not reproducible, so hand-edited wording is legitimate
    // authorship — not drift. Only the INPUTS are attested.
    let edited = stamped().replace(
        "Open allows 300 execution minutes a month; Pro allows 2,000.",
        "Open gives you 300 minutes of execution each month. Pro gives 2,000.",
    );
    let failures = stale_transforms(std::path::Path::new("d.md"), &edited).unwrap();
    assert!(
        failures.is_empty(),
        "hand-editing a passage is not drift: {failures:?}"
    );
}

</storycode:copy>
<a id="f-c791fc613faa-weaving_emits_the_pinned_passage_and_calls_no_model"></a>
### `weaving_emits_the_pinned_passage_and_calls_no_model`

<storycode:copy id="f-c791fc613faa-weaving_emits_the_pinned_passage_and_calls_no_model">
#[tokio::test(flavor = "multi_thread")]
async fn weaving_emits_the_pinned_passage_and_calls_no_model() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("d.md");
    let source = stamped()
        .replace(
            "<hick:transform select",
            "<hick:file path=\"out.md\"><hick:transform select",
        )
        .replace("</hick:transform>", "</hick:transform></hick:file>");
    std::fs::write(&doc_path, source).unwrap();

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

</storycode:copy>
<a id="f-c791fc613faa-a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes"></a>
### `a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes`

A transform that summarizes something UPSTREAM — a meeting turn, a decision
— must fingerprint the same bytes `hick:paste` would find at weave. Before
this test, `hick test` and `hick refresh` parsed the document without
resolving `hick:upstream`, so the selection was empty: the passage was
stamped over nothing and never went stale when the meeting changed.

<storycode:copy id="f-c791fc613faa-a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes">
/// A transform that summarizes something UPSTREAM — a meeting turn, a decision
/// — must fingerprint the same bytes `hick:paste` would find at weave. Before
/// this test, `hick test` and `hick refresh` parsed the document without
/// resolving `hick:upstream`, so the selection was empty: the passage was
/// stamped over nothing and never went stale when the meeting changed.
/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let meeting = dir.path().join("meeting.md");
    let note = dir.path().join("note.md");
    std::fs::write(
        &meeting,
        r##"<hick:copy id="decision" class="decision">Ship on Friday.</hick:copy>
"##,
    )
    .unwrap();
    let note_src = r##"<hick:upstream file="meeting.md" />

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

</storycode:copy>
<a id="f-c791fc613faa-an_included_files_transforms_are_not_the_includers"></a>
### `an_included_files_transforms_are_not_the_includers`

An included file's transforms are the included file's to check: the
includer must not report them (twice) as its own, because `hick refresh`
on the includer would then write a passage at the wrong file's offsets.

<storycode:copy id="f-c791fc613faa-an_included_files_transforms_are_not_the_includers">
/// An included file's transforms are the included file's to check: the
/// includer must not report them (twice) as its own, because `hick refresh`
/// on the includer would then write a passage at the wrong file's offsets.
#[test]
fn an_included_files_transforms_are_not_the_includers() {
    let dir = tempfile::tempdir().unwrap();
    let chapter = dir.path().join("chapter.md");
    let book = dir.path().join("book.md");
    std::fs::write(
        &chapter,
        r##"<hick:copy id="facts" class="facts">Three states.</hick:copy>
<hick:transform select="#facts" from="stale000" instruct="Summarize.">
Old summary.
</hick:transform>
"##,
    )
    .unwrap();
    std::fs::write(&book, "<hick:include file=\"chapter.md\" />\n").unwrap();
    let source = std::fs::read_to_string(&book).unwrap();
    let failures = stale_transforms(&book, &source).unwrap();
    assert!(failures.is_empty(), "{failures:?}");
    let chapter_src = std::fs::read_to_string(&chapter).unwrap();
    assert_eq!(stale_transforms(&chapter, &chapter_src).unwrap().len(), 1);
}

</storycode:copy>
<a id="f-c791fc613faa-a_check_is_a_transform_with_the_question_built_in"></a>
### `a_check_is_a_transform_with_the_question_built_in`

`hick:check` is a transform spelled for one question: `claim=` + `against=`
are its selection, the built-in instruction is its instruction, and it is
fingerprinted, checked, and refreshed exactly like one.

<storycode:copy id="f-c791fc613faa-a_check_is_a_transform_with_the_question_built_in">
/// `hick:check` is a transform spelled for one question: `claim=` + `against=`
/// are its selection, the built-in instruction is its instruction, and it is
/// fingerprinted, checked, and refreshed exactly like one.
/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn a_check_is_a_transform_with_the_question_built_in() {
    let src = "<hick:copy id=\"m1\" class=\"message\">We ship Friday.</hick:copy>\n<hick:copy id=\"f\" class=\"finding\">Ship date: Friday.</hick:copy>\n<hick:check claim=\"#m1\" against=\".finding\" from=\"FP\">\nBACKED.\n</hick:check>\n";
    let doc = hickory_cli::transform_document(std::path::Path::new("d.md"), src).unwrap();
    let check = doc.tags().find(|t| t.name == "check").unwrap();
    let (select, instruct) = hickory_cli::transform_spec(check);
    assert_eq!(select, "#m1,.finding");
    assert_eq!(instruct, hickory_cli::CHECK_INSTRUCT);
    let fp = hick_lang::transform_fingerprint(&transform_input(&doc, &select), &instruct);
    let stamped = src.replace("FP", &fp);
    assert!(
        stale_transforms(std::path::Path::new("d.md"), &stamped)
            .unwrap()
            .is_empty()
    );
    let drifted = stamped.replace("Ship date: Friday.", "Ship date: Monday.");
    assert_eq!(
        stale_transforms(std::path::Path::new("d.md"), &drifted)
            .unwrap()
            .len(),
        1
    );
}

</storycode:copy>
<a id="f-c791fc613faa-the_transform_input_is_one_labelled_paragraph_per_fragment"></a>
### `the_transform_input_is_one_labelled_paragraph_per_fragment`

The input a transform reads is citeable: fragments are separated, carry
their ids, and a speaker turn names its speaker — so a passage that checks
a sentence against a meeting can say which turn backs it.

<storycode:copy id="f-c791fc613faa-the_transform_input_is_one_labelled_paragraph_per_fragment">
/// The input a transform reads is citeable: fragments are separated, carry
/// their ids, and a speaker turn names its speaker — so a passage that checks
/// a sentence against a meeting can say which turn backs it.
/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn the_transform_input_is_one_labelled_paragraph_per_fragment() {
    let src = "<hick:transcript id=\"t\" format=\"vtt\">\nWEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Ship it.\n\n00:00:03.000 --> 00:00:04.000\n<v Ana>Not yet.\n</hick:transcript>\n<hick:copy id=\"m1\" class=\"message\">We ship.</hick:copy>\n";
    let doc = hickory_cli::transform_document(std::path::Path::new("d.md"), src).unwrap();
    assert_eq!(
        transform_input(&doc, "#m1,.said-ana"),
        "[#t-u2] Ana (00:00:03.000): Not yet.\n\n[#m1] We ship."
    );
    assert_eq!(
        transform_input(&doc, "#t"),
        "[#t-u1] Sam (00:00:01.000): Ship it.\n[#t-u2] Ana (00:00:03.000): Not yet."
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/transform_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/transform_tests.rs"><storycode:paste select="#f-c791fc613faa-DOC"/><storycode:paste select="#f-c791fc613faa-stamped"/><storycode:paste select="#f-c791fc613faa-a_passage_whose_input_is_unchanged_is_not_stale"/><storycode:paste select="#f-c791fc613faa-changing_the_input_makes_the_passage_stale"/><storycode:paste select="#f-c791fc613faa-changing_the_instruction_makes_the_passage_stale"/><storycode:paste select="#f-c791fc613faa-editing_the_passage_by_hand_does_not_make_it_stale"/><storycode:paste select="#f-c791fc613faa-weaving_emits_the_pinned_passage_and_calls_no_model"/><storycode:paste select="#f-c791fc613faa-a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes"/><storycode:paste select="#f-c791fc613faa-an_included_files_transforms_are_not_the_includers"/><storycode:paste select="#f-c791fc613faa-a_check_is_a_transform_with_the_question_built_in"/><storycode:paste select="#f-c791fc613faa-the_transform_input_is_one_labelled_paragraph_per_fragment"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/src/restamp_tests.rs](contract.md#output-51aecdb4aa76)
- [crates/hickory-cli/tests/transform_tests.rs](contract.md#output-c791fc613faa)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
