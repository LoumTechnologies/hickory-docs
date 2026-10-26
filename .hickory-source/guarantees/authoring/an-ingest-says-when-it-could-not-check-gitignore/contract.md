<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Ingest Says When It Could Not Check `.gitignore`

For: Authors editing executable documents and the files those documents produce.

Given a directory that is not a git repository, when `hick ingest --from`
runs there, then it says on stderr that `.gitignore` filtering was skipped
and every produced file was ingested unfiltered — never silently, and never
indistinguishable from a real run that checked and found nothing to skip.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-ingest-says-when-it-could-not-check-gitignore.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted`

This is the maintained test fragment from `crates/hickory-cli/tests/ingest_scaffold.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6682c6282b3c-ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted"></a>
### `ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted`

Protects docs/guarantees/authoring/an-ingest-says-when-it-could-not-check-gitignore.md

<storycode:copy id="f-6682c6282b3c-ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted">
// Protects docs/guarantees/authoring/an-ingest-says-when-it-could-not-check-gitignore.md
#[test]
fn ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted() {
    // No `repo()` call: this directory is deliberately not a git repository,
    // which is exactly the case `gitignored()` returns `None` for — "could
    // not check" rather than "checked, found nothing to skip".
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("app.md");
    std::fs::write(&doc, doc_source()).unwrap();

    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc)
        .output()
        .expect("run hick");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "ingest failed: {stderr}\n{stdout}");

    // Filtering never ran, so `obj/build.log` — which a real project would
    // gitignore — is ingested unfiltered rather than silently dropped.
    let source = std::fs::read_to_string(&doc).unwrap();
    assert!(source.contains("skipped=\"0\""), "{source}");
    assert!(source.contains("path=\"app/obj/build.log\""), "{source}");

    // And the reason `skipped="0"` means "never checked" here, not "checked,
    // found nothing" — which the document's own attributes cannot
    // distinguish — is on stderr instead.
    assert!(
        stderr.contains("not a git repository") && stderr.contains(".gitignore filtering"),
        "{stderr}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/ingest_exec.rs](../../../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#output-4e4b92369bcd)

### Tests and fixtures

- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#output-6682c6282b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
