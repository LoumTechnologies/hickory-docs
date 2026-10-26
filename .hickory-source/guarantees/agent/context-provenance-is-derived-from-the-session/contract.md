<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Context Provenance Is Derived From The Session, Never From The Model

For: Authors using an agent to change documents while retaining review control.

Given an agent session, when a tool shows the model a file (`read_doc`,
`read_output`, `read_file`), then the session records `<hick:read>` — the
path, the content's SHA-256, the repository commit when there is one, and the
lines shown; when an edit tool writes, then the session records
`<hick:wrote>` — the file, the lines as they stand after the edit, and their
hashline hashes; and when anyone asks (`hick context`, `GET /api/docs/:id/context`,
the app's Context layer), native writes use the recorded message envelope supplied to the provider client and
reads from this turn. Adapter writes use recorded observations from this turn,
with opaque adapter-managed context explicitly outside that scope. Abandoned
branches are retained historically but are never counted as supplied inputs.
Availability is not evidence that a claim follows from a source.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/context-provenance-is-derived-from-the-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `read_file_shows_a_project_file_and_records_the_read`

This is the maintained test fragment from `crates/hickory-agent/tests/tools_edit_session.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-35f0054c98d4-read_file_shows_a_project_file_and_records_the_read"></a>
### `read_file_shows_a_project_file_and_records_the_read`

Context provenance: what the tools showed, and what the edits wrote
`read_file` shows a project file and records the read; it refuses to look
outside the project, and an edit records the lines it wrote.

<storycode:copy id="f-35f0054c98d4-read_file_shows_a_project_file_and_records_the_read">
// ---------------------------------------------------------------------------
// Context provenance: what the tools showed, and what the edits wrote
// ---------------------------------------------------------------------------

/// `read_file` shows a project file and records the read; it refuses to look
/// outside the project, and an edit records the lines it wrote.
/// Guarantee: docs/guarantees/agent/context-provenance-is-derived-from-the-session.md
#[tokio::test(flavor = "multi_thread")]
async fn read_file_shows_a_project_file_and_records_the_read() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    std::fs::create_dir_all(dir.path().join("data")).unwrap();
    std::fs::write(dir.path().join("data/export.csv"), "a,b\n1,2\n3,4\n").unwrap();
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    let out = execute_tool(
        &mut session,
        executor(),
        &inv("read_file", &[("path", "data/export.csv")], None),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    assert!(out.text.contains("|1,2"), "hashline-rendered: {}", out.text);
    assert_eq!(out.reads.len(), 1);
    let read = &out.reads[0];
    assert_eq!(read.path, "data/export.csv");
    assert_eq!((read.first_line, read.last_line), (1, 3));
    assert_eq!(read.sha256.len(), 64);

    // A range is a range, and is recorded as one.
    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "read_file",
            &[("path", "data/export.csv"), ("from", "2"), ("to", "2")],
            None,
        ),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    assert!(
        out.text.contains("|1,2") && !out.text.contains("|3,4"),
        "{}",
        out.text
    );
    assert_eq!((out.reads[0].first_line, out.reads[0].last_line), (2, 2));

    // Nothing above the project.
    let out = execute_tool(
        &mut session,
        executor(),
        &inv("read_file", &[("path", "../../../../etc/hostname")], None),
    )
    .await;
    assert!(!out.ok);
    assert!(
        out.text.contains("outside the project") || out.text.contains("cannot read"),
        "{}",
        out.text
    );
    assert!(out.reads.is_empty());

    // read_doc records the document itself as shown.
    let out = execute_tool(&mut session, executor(), &inv("read_doc", &[], None)).await;
    assert!(out.ok);
    assert_eq!(out.reads.len(), 1);
    assert!(
        out.reads[0].path.ends_with("doc.md"),
        "{}",
        out.reads[0].path
    );

    // An edit records the lines it wrote, as they stand afterwards.
    let doc_out = execute_tool(&mut session, executor(), &inv("read_doc", &[], None)).await;
    let last_hash = doc_out
        .text
        .lines()
        .filter_map(|l| l.split_once('|'))
        .map(|(h, _)| h.to_string())
        .rfind(|h| h.len() == 64)
        .unwrap();
    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_doc",
            &[("after", &last_hash)],
            Some("A new closing line.\n"),
        ),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    let wrote = out.wrote.expect("an edit records what it wrote");
    assert!(wrote.file.ends_with("doc.md"));
    assert!(wrote.first_line <= wrote.last_line, "{wrote:?}");
    assert!(
        wrote
            .hashes
            .contains(&hickory_agent::hashline::line_hash("A new closing line.")),
        "{wrote:?}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-agent/src/context.rs](../../../mechanisms/crates/hickory-agent/src/context.rs.md#output-92769b9184f8)
- [crates/hickory-agent/src/session.rs](../../../mechanisms/crates/hickory-agent/src/session.rs.md#output-9c7ae0f28c14)
- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)
- [crates/hickory-agent/src/tools/read_file.rs](../../../mechanisms/crates/hickory-agent/src/tools/read_file.rs.md#output-7845fffcfd2e)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/serve/acp/read_receipt.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/read_receipt.rs.md#output-bc08d0c441ad)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)

### Tests and fixtures

- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [crates/hickory-agent/tests/tools_edit_session.rs](../../../mechanisms/crates/hickory-agent/tests/tools_edit_session.rs.md#output-35f0054c98d4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
