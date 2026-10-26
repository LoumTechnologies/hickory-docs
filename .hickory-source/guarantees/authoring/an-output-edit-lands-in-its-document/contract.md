<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Save a generated edit once and retain it when saving fails

For: Authors editing executable documents and the files those documents produce.

Type a second change while the first generated-file save is pending. The next save must use the acknowledged baseline and retain the newer text.

## What must happen

Supported generated-text edits map to their document source. The frontend computes each diff against the last content acknowledged by the backend; successful saves advance that baseline, failed saves leave it unchanged. Saves are serialized. Incoming output does not replace pending local text, and a discard request rechecks the expected buffer before and after awaiting earlier saves. The filesystem watcher also rechecks contributing source before publication and refuses stale or synthetic edits instead of overwriting unrelated work.

## Limits of the evidence

The focused saver witnesses exercise queued callbacks and byte diffs. They do not establish end-to-end watcher convergence or crash-atomic multi-document publication.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-output-edit-lands-in-its-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Unify the watcher and HTTP entry points around acknowledged bytes while retaining their distinct concurrency boundaries. Remove historical duplication anecdotes from the current promise.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `an_edit_saved_in_a_woven_file_lands_in_the_document`

This is the maintained test fragment from `crates/hickory-cli/tests/up_loop.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3098574124c3-an_edit_saved_in_a_woven_file_lands_in_the_document"></a>
### `an_edit_saved_in_a_woven_file_lands_in_the_document`

<storycode:copy id="f-3098574124c3-an_edit_saved_in_a_woven_file_lands_in_the_document">
/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn an_edit_saved_in_a_woven_file_lands_in_the_document() {
    let mut up = Loop::start(DOC);
    let output = up.path("greeter.py");

    let woven = std::fs::read_to_string(&output).expect("read output");
    assert!(
        woven.contains("Hello, {name}!"),
        "unexpected weave: {woven}"
    );
    save_atomically(&output, &woven.replace("Hello, {name}!", "Howdy, {name}!"));

    let doc_path = up.path("greeter.md");
    let doc = up.wait_for(
        &doc_path,
        |c| c.contains("Howdy"),
        "the edit saved in the output never reached the document",
    );
    assert!(
        doc.contains("Howdy, {name}!"),
        "document did not receive the edit: {doc}"
    );
    // The document is still a document: the edit landed inside the block it
    // came from, not appended or pasted over the structure.
    assert!(doc.contains("<hick:file path=\"greeter.py\">"), "{doc}");
    assert!(doc.contains("# Greeter"), "{doc}");
}

</storycode:copy>

## Test the behavior: `an_edit_saved_in_the_document_reaches_the_woven_file`

This is the maintained test fragment from `crates/hickory-cli/tests/up_loop.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3098574124c3-an_edit_saved_in_the_document_reaches_the_woven_file"></a>
### `an_edit_saved_in_the_document_reaches_the_woven_file`

An edit made in the document appears in the generated file without asking.

<storycode:copy id="f-3098574124c3-an_edit_saved_in_the_document_reaches_the_woven_file">
/// An edit made in the document appears in the generated file without asking.
///
/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn an_edit_saved_in_the_document_reaches_the_woven_file() {
    let mut up = Loop::start(DOC);

    let doc_path = up.path("greeter.md");
    let doc = std::fs::read_to_string(&doc_path).expect("read doc");
    save_atomically(
        &doc_path,
        &doc.replace("greet(\"world\")", "greet(\"everyone\")"),
    );

    let output_path = up.path("greeter.py");
    let output = up.wait_for(
        &output_path,
        |c| c.contains("everyone"),
        "the edit saved in the document never reached the output file",
    );
    assert!(output.contains("greet(\"everyone\")"), "{output}");
}

</storycode:copy>

## Test the behavior: `two_regions_edited_in_one_save_both_land`

This is the maintained test fragment from `crates/hickory-cli/tests/up_loop.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3098574124c3-two_regions_edited_in_one_save_both_land"></a>
### `two_regions_edited_in_one_save_both_land`

Two regions edited in one save must both land. A single edit span wide
enough to cover both would also cover what lies between them.

<storycode:copy id="f-3098574124c3-two_regions_edited_in_one_save_both_land">
/// Two regions edited in one save must both land. A single edit span wide
/// enough to cover both would also cover what lies between them.
///
/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn two_regions_edited_in_one_save_both_land() {
    let mut up = Loop::start(DOC);
    let output = up.path("greeter.py");

    let woven = std::fs::read_to_string(&output).expect("read output");
    let edited = woven
        .replace("def greet(name):", "def greet(name, punct=\"!\"):")
        .replace("greet(\"world\")", "greet(\"world\", \"?\")");
    save_atomically(&output, &edited);

    let doc_path = up.path("greeter.md");
    let doc = up.wait_for(
        &doc_path,
        |c| c.contains("punct"),
        "the first of the two edited regions never reached the document",
    );
    assert!(doc.contains("punct=\"!\""), "first region missing: {doc}");
    assert!(
        doc.contains("\"world\", \"?\""),
        "second region missing: {doc}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/lib/outputSave.ts](../../../mechanisms/apps/web/src/lib/outputSave.ts.md#output-ff2a9a5b4231)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)
- [crates/hickory-cli/src/up/reverse.rs](../../../mechanisms/crates/hickory-cli/src/up/reverse.rs.md#output-0e6cf077f262)
- [crates/hickory-cli/src/up/state.rs](../../../mechanisms/crates/hickory-cli/src/up/state.rs.md#output-d4f59b7595b5)
- [crates/hickory-lineage/src/lib.rs](../../../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3)

### Tests and fixtures

- [apps/web/src/lib/outputSave.test.ts](../../../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43)
- [apps/web/src/shell/GeneratedBufferRecovery.test.tsx](../../../mechanisms/apps/web/src/shell/GeneratedBufferRecovery.tsx.md#output-9cef161ebf1f)
- [crates/hickory-cli/tests/serve_local.rs](../../../mechanisms/crates/hickory-cli/tests/serve_local.rs.md#output-2b0b4fa19188)
- [crates/hickory-cli/tests/up_loop.rs](../../../mechanisms/crates/hickory-cli/tests/up_loop.rs.md#output-3098574124c3)
- [crates/hickory-cli/tests/up_stress.rs](../../../mechanisms/crates/hickory-cli/tests/up_stress.rs.md#output-635d4ae171a9)

### Focused witnesses

- `serializes overlapping saves so each diff sees the previous result` in `apps/web/src/lib/outputSave.test.ts`.
- `does not discard generated edits typed while an earlier save is settling` in `apps/web/src/lib/outputSave.test.ts`.

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
