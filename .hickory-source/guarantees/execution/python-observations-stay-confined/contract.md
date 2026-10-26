<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Python observations stay confined and source bound

For: Authors running documents on their own machine and reviewers assessing their side effects.

## Requirements

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/python-observations-stay-confined.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Test the behavior: `user_python_capture_is_retained_and_refuses_workspace_escape`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_runtime_runs.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-b659fc307bcd-user_python_capture_is_retained_and_refuses_workspace_escape"></a>
### `user_python_capture_is_retained_and_refuses_workspace_escape`

<storycode:copy id="f-b659fc307bcd-user_python_capture_is_retained_and_refuses_workspace_escape">
// Guarantee: docs/guarantees/execution/python-observations-stay-confined.md
#[tokio::test(flavor = "multi_thread")]
async fn user_python_capture_is_retained_and_refuses_workspace_escape() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("a.py"), "print('from actual Python')\n").unwrap();
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/runtime/python"))
        .json(&json!({"document":"a.py","profile":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let result: Value = response.json().await.unwrap();
    assert_eq!(
        result["run"]["status"], "completed",
        "{}",
        result["run"]["detail"]
    );
    assert!(
        result["run"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["observation"]["kind"] == "profile")
    );
    assert_eq!(result["run"]["sources"][0]["uri"], "a.py");
    assert!(!result["run"]["events"].as_array().unwrap().is_empty());
    let listing: Value = client
        .get(format!("{base}/api/runtime/runs"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(listing["runs"][0]["run"]["id"], result["run"]["id"]);
    let outside = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        client
            .post(format!("{base}/api/runtime/python"))
            .json(&json!({"document":outside.path()}))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    server.abort();
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/observation.rs](../../../mechanisms/crates/hickory-cli/src/observation.rs.md#output-131492bc8594)
- [examples/demo/app/main.py](../../../mechanisms/examples/demo/app/main.py.md#output-2a5787996a2d)

### Tests and fixtures

- [apps/web/src/debug/runtimeHistory.test.tsx](../../../mechanisms/apps/web/src/debug/runtimeHistory.ts.md#output-b01fe4e29028)
- [crates/hickory-cli/tests/serve_runtime_runs.rs](../../../mechanisms/crates/hickory-cli/tests/serve_runtime_runs.rs.md#output-b659fc307bcd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
