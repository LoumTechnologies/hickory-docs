<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# One loop owns a directory and never reacts to itself

For: Authors editing executable documents and the files those documents produce.

Given an engine watching a folder, its own output writes are not treated as user
edits. App and CLI clients share the engine for that folder; the directory lock
prevents a second writer loop from starting.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/one-loop-owns-a-directory.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `a_second_session_on_the_same_folder_attaches`

This is the maintained test fragment from `apps/desktop/src-tauri/tests/serves_one_origin.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e7ec95ab75f6-a_second_session_on_the_same_folder_attaches"></a>
### `a_second_session_on_the_same_folder_attaches`

<storycode:copy id="f-e7ec95ab75f6-a_second_session_on_the_same_folder_attaches">
/// Guarantee: docs/guarantees/authoring/one-loop-owns-a-directory.md
#[tokio::test(flavor = "multi_thread")]
async fn a_second_session_on_the_same_folder_attaches() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (first, http) = start_in(dir.path()).await;
    let second = hickory_desktop_lib::server::start(dir.path(), None, None)
        .await
        .unwrap();
    for session in [&first, &second] {
        assert!(
            http.get(format!("{}/api/health", session.url))
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
        );
    }
}

</storycode:copy>

## Test the behavior: `a_second_client_on_the_same_directory_attaches`

This is the maintained test fragment from `crates/hickory-cli/tests/up_loop.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3098574124c3-a_second_client_on_the_same_directory_attaches"></a>
### `a_second_client_on_the_same_directory_attaches`

<storycode:copy id="f-3098574124c3-a_second_client_on_the_same_directory_attaches">
/// Guarantee: `docs/guarantees/authoring/one-loop-owns-a-directory.md`
#[test]
fn a_second_client_on_the_same_directory_attaches() {
    let up = Loop::start(DOC);
    let mut second = hick()
        .arg("up")
        .arg(up.dir.path())
        .stderr(Stdio::null())
        .spawn()
        .expect("start another client");
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        second.try_wait().unwrap().is_none(),
        "the second client refused to attach"
    );
    let _ = second.kill();
    let _ = second.wait();
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/engine/mod.rs](../../../mechanisms/crates/hickory-cli/src/engine/mod.rs.md#output-e55203129482)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)
- [crates/hickory-cli/src/up/state.rs](../../../mechanisms/crates/hickory-cli/src/up/state.rs.md#output-d4f59b7595b5)

### Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../mechanisms/apps/desktop/src-tauri/tests/serves_one_origin.rs.md#output-e7ec95ab75f6)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../mechanisms/crates/hickory-cli/tests/engine_lifecycle.rs.md#output-6ea35164aaa4)
- [crates/hickory-cli/tests/literate_views.rs](../../../mechanisms/crates/hickory-cli/tests/literate_views.rs.md#output-3df513c37b3c)
- [crates/hickory-cli/tests/up_loop.rs](../../../mechanisms/crates/hickory-cli/tests/up_loop.rs.md#output-3098574124c3)
- [crates/hickory-cli/tests/up_stress.rs](../../../mechanisms/crates/hickory-cli/tests/up_stress.rs.md#output-635d4ae171a9)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
