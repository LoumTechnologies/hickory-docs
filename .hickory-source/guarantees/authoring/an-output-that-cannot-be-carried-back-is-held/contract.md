<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep divergent output until an explicit decision names the write

For: Authors editing executable documents and the files those documents produce.

A generated file differs from the document and cannot be mapped back. Leaving the warning must not silently resolve the conflict.

## What must happen

Unreproducible generated output is retained as divergent rather than overwritten. The surface names the write consequence: generated files offer Leave disk unchanged or Regenerate from document; plain buffers offer Overwrite disk with this buffer or Reload from disk. Merge uses the recorded base and current sides under their source guards. Dismissing a notice is not resolution. Kept unrecorded output and an external change under an unsaved buffer remain distinct reasons for divergence.

Pending output captures add an explicit ownership decision before generic
merge/regeneration. Those older controls refuse to dismiss a pending capture;
use `hick project edits`, `new-patch` or an authored amendment followed by
`accept`. Reverting a capture requires recording and accepting the reverted
observation. Acceptance emits a targeted event so a running watcher refreshes
its hold without a further source edit.

## Limits of the evidence

Retaining a divergent file is not verification. Per-target guards do not form a filesystem transaction across all products.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the common divergence mechanism and replace ambiguous keep-mine/take-theirs labels with the later explicit-consequence wording.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `interleaved_document_and_output_edits_never_corrupt`

This is the maintained test fragment from `crates/hickory-cli/tests/up_stress.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-635d4ae171a9-interleaved_document_and_output_edits_never_corrupt"></a>
### `interleaved_document_and_output_edits_never_corrupt`

Both ends at once
Edits arriving from both directions at once must not corrupt the document.
This does not assert which side wins — with two writers and no ordering
between them, "the document still parses" is the strongest honest claim.
The two sides either agree, or an output edit that was computed against
a document that had already moved is HELD: the file keeps what was typed
into it, the loop says so, and nothing is undone. A test that demanded a
particular winner would be asserting a scheduling accident; one that
demanded agreement after a hold would be asking for the destruction the
hold exists to prevent
(docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md).

<storycode:copy id="f-635d4ae171a9-interleaved_document_and_output_edits_never_corrupt">
// ---------------------------------------------------------------------------
// Both ends at once
// ---------------------------------------------------------------------------

/// Edits arriving from both directions at once must not corrupt the document.
///
/// This does not assert which side wins — with two writers and no ordering
/// between them, "the document still parses" is the strongest honest claim.
/// The two sides either agree, or an output edit that was computed against
/// a document that had already moved is HELD: the file keeps what was typed
/// into it, the loop says so, and nothing is undone. A test that demanded a
/// particular winner would be asserting a scheduling accident; one that
/// demanded agreement after a hold would be asking for the destruction the
/// hold exists to prevent
/// (docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md).
#[test]
fn interleaved_document_and_output_edits_never_corrupt() {
    let up = Loop::start("notes.md", DOC, false);
    let doc = up.path("notes.md");
    let output = up.path("notes.py");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    for i in 0..12 {
        if i % 2 == 0 {
            set_marker(&doc, &format!("d{i}"));
        } else {
            set_marker(&output, &format!("o{i}"));
        }
        std::thread::sleep(Duration::from_millis(40));
        assert_parses(&doc);
    }

    // Let it quiesce. Then either the two sides agree, or the output is
    // held with an edit typed into it and the loop said so.
    std::thread::sleep(Duration::from_secs(2));
    let doc_text = std::fs::read_to_string(&doc).expect("read doc");
    let out_text = std::fs::read_to_string(&output).expect("read output");
    let doc_marker = marker_of(&doc_text).expect("document has a marker");
    let out_marker = marker_of(&out_text).expect("output has a marker");
    if doc_marker != out_marker {
        let log = up.log();
        assert!(
            log.contains("diverged (held)"),
            "document and output disagree and the loop never said it was holding the \
             file:\ndoc={doc_marker}\nout={out_marker}\n{log}"
        );
        assert!(
            marker_value(&out_text).is_some_and(|v| v.starts_with('o')),
            "a held file must hold what was typed into it, not a weave: {out_marker}"
        );
    }
    assert_parses(&doc);
}

</storycode:copy>

## Test the behavior: `a_refused_edit_holds_the_file_with_the_edit_in_it`

This is the maintained test fragment from `crates/hickory-cli/tests/up_stress.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-635d4ae171a9-a_refused_edit_holds_the_file_with_the_edit_in_it"></a>
### `a_refused_edit_holds_the_file_with_the_edit_in_it`

An output edit made while the *document* is also changing is refused — the
spans it was computed against no longer describe the document — and the
file is HELD with the edit in it, never put back.
It used to be restored to agree with the document, on the argument that
a silent fork is worse than a refusal. The fork is now loud instead of
silent — the loop names the file and why, the tree and the pane show it —
and what was typed stays, because the loop has nothing truer to put in
its place. Guarantee:

<storycode:copy id="f-635d4ae171a9-a_refused_edit_holds_the_file_with_the_edit_in_it">
/// An output edit made while the *document* is also changing is refused — the
/// spans it was computed against no longer describe the document — and the
/// file is HELD with the edit in it, never put back.
///
/// It used to be restored to agree with the document, on the argument that
/// a silent fork is worse than a refusal. The fork is now loud instead of
/// silent — the loop names the file and why, the tree and the pane show it —
/// and what was typed stays, because the loop has nothing truer to put in
/// its place. Guarantee:
/// docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md
#[test]
fn a_refused_edit_holds_the_file_with_the_edit_in_it() {
    let up = Loop::start("notes.md", DOC, false);
    let doc = up.path("notes.md");
    let output = up.path("notes.py");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    // Both sides inside one debounce window, so the output edit is computed
    // against a document that has already moved.
    set_marker(&doc, "from-document");
    set_marker(&output, "from-output");

    std::thread::sleep(Duration::from_secs(3));

    let doc_text = std::fs::read_to_string(&doc).expect("read doc");
    let out_text = std::fs::read_to_string(&output).expect("read output");
    let log = up.log();
    match (
        marker_value(&doc_text).as_deref(),
        marker_value(&out_text).as_deref(),
    ) {
        // The output edit won the race and was carried back: both agree.
        (Some("from-output"), Some("from-output")) => {}
        // The document edit won and the output edit was refused: the file
        // keeps what was typed, the document keeps its own, and the loop
        // said it was holding the file.
        (Some("from-document"), Some("from-output")) => {
            assert!(
                log.contains("diverged (held)"),
                "a refused edit was not reported as held:\n{log}"
            );
        }
        // The document edit landed and its weave reached the file before the
        // output edit was even seen: agreement, nothing refused.
        (Some("from-document"), Some("from-document")) => {}
        other => panic!("an outcome that undid somebody's typing: {other:?}\n{log}"),
    }
    assert_parses(&doc);
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/api/types.ts](../../../mechanisms/apps/web/src/api/types.ts.md#output-8a4c3188ff6c)
- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/output_resolution.rs](../../../mechanisms/crates/hickory-cli/src/serve/output_resolution.rs.md#output-37563d3bf652)
- [crates/hickory-cli/src/serve/watch.rs](../../../mechanisms/crates/hickory-cli/src/serve/watch.rs.md#output-c9fa53b5e3bf)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)
- [crates/hickory-cli/src/up/reverse.rs](../../../mechanisms/crates/hickory-cli/src/up/reverse.rs.md#output-0e6cf077f262)
- [crates/hickory-cli/src/up/state.rs](../../../mechanisms/crates/hickory-cli/src/up/state.rs.md#output-d4f59b7595b5)

### Tests and fixtures

- [apps/web/src/components/DivergedBanner.test.tsx](../../../mechanisms/apps/web/src/components/DivergedBanner.test.tsx.md#output-c3c47782ddb8)
- [crates/hickory-cli/src/up/state/resolution_tests.rs](../../agent/ui-action-dispatch/implementation/crates/hickory-cli/src/up/state/resolution_tests.rs.md#output-3b41d39d831f)
- [crates/hickory-cli/tests/round_trip.rs](../../../mechanisms/crates/hickory-cli/tests/round_trip.rs.md#output-82b3411e044d)
- [crates/hickory-cli/tests/serve_output_resolution.rs](../../agent/ui-action-dispatch/implementation/crates/hickory-cli/tests/serve_output_resolution.rs.md#output-074b9e71856c)
- [crates/hickory-cli/tests/up_loop.rs](../../../mechanisms/crates/hickory-cli/tests/up_loop.rs.md#output-3098574124c3)
- [crates/hickory-cli/tests/up_stress.rs](../../../mechanisms/crates/hickory-cli/tests/up_stress.rs.md#output-635d4ae171a9)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
