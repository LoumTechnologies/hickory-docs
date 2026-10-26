<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Notebook Asks The Same Questions An Editor Does

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a document open in the notebook — the editor the desktop app hosts —
when the editor asks a language question (hover, definition, references,
completion) or the document changes, then the answer comes from the same
`hick-lsp` that serves Zed, VS Code, Helix, and Neovim, delegating to the same
child language servers, and arrives in **document coordinates**.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/the-notebook-asks-the-same-questions-an-editor-does.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `the_language_channel_answers_in_document_coordinates`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_local.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2b0b4fa19188-the_language_channel_answers_in_document_coordinates"></a>
### `the_language_channel_answers_in_document_coordinates`

The notebook asks language questions over the same socket that carries its
edits, and gets answers in document coordinates.
Protects docs/guarantees/editor-intelligence/the-notebook-asks-the-same-questions-an-editor-does.md

<storycode:copy id="f-2b0b4fa19188-the_language_channel_answers_in_document_coordinates">
/// The notebook asks language questions over the same socket that carries its
/// edits, and gets answers in document coordinates.
///
/// Protects docs/guarantees/editor-intelligence/the-notebook-asks-the-same-questions-an-editor-does.md
#[tokio::test(flavor = "multi_thread")]
async fn the_language_channel_answers_in_document_coordinates() {
    let session = start().await;
    let mut ws = connect(&session).await;

    send_lsp(
        &mut ws,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "hick:///demo.md",
                "languageId": "hick",
                "version": 1,
                "text": DOC,
            }},
        }),
    )
    .await;

    let published = next_lsp(&mut ws, "textDocument/publishDiagnostics").await;
    // The browser never learns where this project lives on disk.
    assert_eq!(published["params"]["uri"], "hick:///demo.md");
    assert!(
        published["params"]["diagnostics"].is_array(),
        "a publish always carries a list, even an empty one: {published}"
    );

    // A question the hick layer answers by itself, so this holds on a machine
    // with no language servers installed at all.
    send_lsp(
        &mut ws,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "hick:///demo.md" },
                // The `<hick:paste select="#greet" />` line.
                "position": { "line": 6, "character": 30 },
            },
        }),
    )
    .await;
    let hover = tokio::time::timeout(Duration::from_secs(20), async {
        while let Some(Ok(msg)) = ws.next().await {
            let TtMessage::Binary(data) = msg else {
                continue;
            };
            if data.first() != Some(&CHANNEL_LSP) {
                continue;
            }
            let message: Value = serde_json::from_slice(&data[1..]).unwrap();
            if message["id"] == 1 {
                return message;
            }
        }
        panic!("the socket closed before the hover answer arrived");
    })
    .await
    .expect("hover should be answered");
    assert!(
        hover.get("result").is_some() || hover.get("error").is_some(),
        "a request must get a reply of some kind: {hover}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/serve/lsp_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/lsp_bridge.rs.md#output-a429fedbabb9)
- [crates/hickory-cli/src/serve/socket.rs](../../../mechanisms/crates/hickory-cli/src/serve/socket.rs.md#output-ce0f0e119d3c)

### Tests and fixtures

- [crates/hickory-cli/tests/serve_local.rs](../../../mechanisms/crates/hickory-cli/tests/serve_local.rs.md#output-2b0b4fa19188)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
