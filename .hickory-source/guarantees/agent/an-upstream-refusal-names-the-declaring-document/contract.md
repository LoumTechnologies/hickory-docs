<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Upstream Refusal Names The Declaring Document

For: Authors using an agent to change documents while retaining review control.

Given an edit to a generated file whose bytes were pasted from a fragment
declared in an **upstream** document, when `edit_output` refuses that edit as
unreproducible, then the refusal names the document where the fragment is
**declared** — not the document where it is pasted — and shows the fragment,
so the next call needs no search.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-upstream-refusal-names-the-declaring-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `a_refusal_across_an_upstream_edge_names_the_upstream_document`

This is the maintained test fragment from `crates/hickory-agent/tests/tools_edit_session.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-35f0054c98d4-a_refusal_across_an_upstream_edge_names_the_upstream_document"></a>
### `a_refusal_across_an_upstream_edge_names_the_upstream_document`

A refusal must send you to the document that DECLARES the fragment, which
across a `hick:upstream` edge is a different file from the paste site.
Protects docs/guarantees/agent/an-upstream-refusal-names-the-declaring-document.md

<storycode:copy id="f-35f0054c98d4-a_refusal_across_an_upstream_edge_names_the_upstream_document">
/// A refusal must send you to the document that DECLARES the fragment, which
/// across a `hick:upstream` edge is a different file from the paste site.
///
/// Protects docs/guarantees/agent/an-upstream-refusal-names-the-declaring-document.md
#[tokio::test(flavor = "multi_thread")]
async fn a_refusal_across_an_upstream_edge_names_the_upstream_document() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("up.md"),
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="up.md">
<hick:copy id="greet">def greet(name):
    return f"Hello, {name}!"
</hick:copy>
</hick:doc>
"##,
    )
    .unwrap();
    let doc_path = write_doc(
        dir.path(),
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="down.md">
<hick:upstream file="up.md" />
<hick:file path="gen.rs">
<hick:paste select="#greet" />
</hick:file>
</hick:doc>
"##,
    );
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    let refused = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            // The WHOLE pasted block, which is what triggers the shared-source
            // refusal: a single line inside it takes a different path (the
            // mapped edit would break the document, and is refused there).
            &[
                ("path", "gen.rs"),
                (
                    "run",
                    &format!(
                        "{}..{}",
                        line_hash("def greet(name):"),
                        line_hash("    return f\"Hello, {name}!\"")
                    ),
                ),
            ],
            Some("def greet(name):\n    return f\"Howdy, {name}!\""),
        ),
    )
    .await;

    assert!(!refused.ok, "{}", refused.text);
    assert!(
        refused.text.contains("up.md"),
        "the refusal must name the document that DECLARES the fragment, not the \
         paste site — an agent that obeys the old message edited the <hick:paste> \
         tag in the wrong file: {}",
        refused.text
    );
    assert!(
        refused.text.contains("hick:copy id=\"greet\""),
        "and it should show the fragment, so the next call needs no search: {}",
        refused.text
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)

### Tests and fixtures

- [crates/hickory-agent/tests/tools_edit_session.rs](../../../mechanisms/crates/hickory-agent/tests/tools_edit_session.rs.md#output-35f0054c98d4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
