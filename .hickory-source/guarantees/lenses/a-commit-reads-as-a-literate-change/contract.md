<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A commit reads as a literate change

For: Authors reviewing history and choosing where an edit should land.

Given a commit in History or Story, **Read commit** opens a read-only document
reading with its complete message above its changed files. Markdown reads as
prose; other text files read as code. Additions and removals use the same
`comparisonField` as literate Git comparisons and agent proposals. Removed
passages render through the literate formatting layer and are expanded in a
read-only comparison. They never enter the current buffer or its saved bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/a-commit-reads-as-a-literate-change.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `commit_reading_preserves_messages_blobs_and_the_working_tree`

This is the maintained test fragment from `crates/hickory-cli/tests/git_ops.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-34c22c3c39d6-commit_reading_preserves_messages_blobs_and_the_working_tree"></a>
### `commit_reading_preserves_messages_blobs_and_the_working_tree`

<storycode:copy id="f-34c22c3c39d6-commit_reading_preserves_messages_blobs_and_the_working_tree">
// Guarantee: docs/guarantees/lenses/a-commit-reads-as-a-literate-change.md
#[tokio::test]
async fn commit_reading_preserves_messages_blobs_and_the_working_tree() {
    let session = start(true).await;
    let (status, initial) = get(&session, "/api/git/reading?sha=HEAD").await;
    assert_eq!(status, 200, "{initial}");
    assert!(initial["parent"].is_null());
    assert_eq!(initial["message"].as_str().unwrap().trim(), "First");
    assert!(
        initial["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["before"] == "")
    );
    git(&session.root, &["mv", "a.txt", "renamed.txt"]);
    std::fs::write(
        session.root.join("note.md"),
        "# Résumé 📝\n\nA **new** note.\n",
    )
    .unwrap();
    std::fs::write(session.root.join("binary.bin"), [0, 255, 1]).unwrap();
    git(&session.root, &["rm", "notes.md"]);
    git(&session.root, &["add", "-A"]);
    git(
        &session.root,
        &[
            "commit",
            "-m",
            "Explain the change\n\nThis is the full message body.\n\nA second paragraph.",
        ],
    );
    let sha = git(&session.root, &["rev-parse", "HEAD"]).trim().to_owned();
    std::fs::write(session.root.join("note.md"), "unsaved on disk\n").unwrap();
    let before_status = git(&session.root, &["status", "--porcelain"]);
    let (status, reading) = get(&session, &format!("/api/git/reading?sha={sha}")).await;
    assert_eq!(status, 200, "{reading}");
    assert!(
        reading["message"]
            .as_str()
            .unwrap()
            .contains("A second paragraph.")
    );
    let files = reading["files"].as_array().unwrap();
    let note = files.iter().find(|f| f["path"] == "note.md").unwrap();
    assert_eq!(note["after"], "# Résumé 📝\n\nA **new** note.\n");
    let renamed = files.iter().find(|f| f["path"] == "renamed.txt").unwrap();
    assert_eq!(renamed["from"], "a.txt");
    assert_eq!(renamed["before"], "one\ntwo\n");
    assert_eq!(renamed["before"], renamed["after"]);
    let deleted = files.iter().find(|f| f["path"] == "notes.md").unwrap();
    assert!(!deleted["before"].as_str().unwrap().is_empty());
    assert_eq!(deleted["after"], "");
    assert_eq!(
        files.iter().find(|f| f["path"] == "binary.bin").unwrap()["binary"],
        true
    );
    assert_eq!(
        git(&session.root, &["status", "--porcelain"]),
        before_status
    );
    assert_eq!(
        std::fs::read_to_string(session.root.join("note.md")).unwrap(),
        "unsaved on disk\n"
    );
    let (status, _) = get(&session, "/api/git/reading?sha=--all").await;
    assert_eq!(status, 422);
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/serve/commit_reading.rs](../../../mechanisms/crates/hickory-cli/src/serve/commit_reading.rs.md#output-4798664e2dcf)

### Tests and fixtures

- [apps/web/src/editor/comparison.test.ts](../../../mechanisms/apps/web/src/editor/comparison.ts.md#output-a13b9a534b46)
- [crates/hickory-cli/tests/git_ops.rs](../../../mechanisms/crates/hickory-cli/tests/git_ops.rs.md#output-34c22c3c39d6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
