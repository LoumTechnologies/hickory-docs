<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Runtime history keeps runs and invocation occurrences separate

For: Authors running documents on their own machine and reviewers assessing their side effects.

The initial display policy selects the latest captured run, with explicit
history, pin/unpin and separate comparison. A newer failed/partial run exposes
its actual observations rather than retaining an older green result as current.
Historical selections remain explicit. Editing source preserves observations
but marks them stale and removes their live-editor decorations.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `runtime_history_http_keeps_evidence_immutable_and_pins_explicit`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_runtime_runs.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-b659fc307bcd-runtime_history_http_keeps_evidence_immutable_and_pins_explicit"></a>
### `runtime_history_http_keeps_evidence_immutable_and_pins_explicit`

<storycode:copy id="f-b659fc307bcd-runtime_history_http_keeps_evidence_immutable_and_pins_explicit">
//! Guarantee: docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md
use hick_observe::{Run, Source, Status};
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn runtime_history_http_keeps_evidence_immutable_and_pins_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
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
    let mut run = Run::new(
        vec![Source::capture(
            "document".into(),
            "example.md".into(),
            b"console.log('hello')".to_vec(),
        )],
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    );
    run.finish(Status::Failed).unwrap();
    let url = format!("{base}/api/runtime/runs");
    assert_eq!(
        client.post(&url).json(&run).send().await.unwrap().status(),
        200
    );
    let id_url = format!("{url}/{}", run.id);
    assert_eq!(
        client
            .patch(&id_url)
            .json(&json!({"pinned":true}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client.post(&url).json(&run).send().await.unwrap().status(),
        200
    );
    let listing: Value = client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(listing["runs"][0]["run"]["pinned"], true);
    assert_eq!(listing["runs"][0]["run"]["status"], "failed");
    let mut corrupt = run.clone();
    corrupt.sources[0].bytes.push(b'!');
    assert_eq!(
        client
            .post(&url)
            .json(&corrupt)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert!(root.join(format!(".hick/runs/{}.json", run.id)).exists());
    assert_eq!(client.delete(&id_url).send().await.unwrap().status(), 200);
    let listing: Value = client.get(&url).send().await.unwrap().json().await.unwrap();
    assert!(listing["runs"].as_array().unwrap().is_empty());
    server.abort();
}

</storycode:copy>

## Implementation review

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/debug/runtimeHistory.test.tsx](../../../mechanisms/apps/web/src/debug/runtimeHistory.ts.md#output-b01fe4e29028)
- [crates/hickory-cli/tests/serve_runtime_runs.rs](../../../mechanisms/crates/hickory-cli/tests/serve_runtime_runs.rs.md#output-b659fc307bcd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
