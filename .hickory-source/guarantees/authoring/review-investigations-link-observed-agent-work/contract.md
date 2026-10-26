<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Review investigations link observed agent work without resolving human review

For: Authors editing executable documents and the files those documents produce.

Given a saved review thread, posting a comment or preparing a prompt does not
start an agent. The user separately selects one open conversation and sends an
investigation. Dispatch uses only that conversation's current branch, provider,
context capture and existing edit approval flow. It preserves an unsent draft.
Closed, busy or unconfigured panes report failure rather than broadcasting.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/review-investigations-link-observed-agent-work.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `review_investigation_links_real_response_refused_edits_and_cancellation`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-08f0cd3133cc-review_investigation_links_real_response_refused_edits_and_cancellation"></a>
### `review_investigation_links_real_response_refused_edits_and_cancellation`

<storycode:copy id="f-08f0cd3133cc-review_investigation_links_real_response_refused_edits_and_cancellation">
// Guarantee: docs/guarantees/authoring/review-investigations-link-observed-agent-work.md
#[tokio::test(flavor = "multi_thread")]
async fn review_investigation_links_real_response_refused_edits_and_cancellation() {
    use sha2::{Digest, Sha256};
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    let (_, initial) = app.request("GET", "/api/reviews", Value::Null).await;
    let scope = initial["scope"].clone();
    let (status, root) = app
        .request(
            "POST",
            "/api/reviews",
            json!({"scope":scope,"target":{
        "kind":"source-relationship","path":"note.md","quote":"Original human input",
        "source_hash":format!("{:x}",Sha256::digest("Original human input\n")),"lines":[1,1]},
        "action":"comment","body":"The generated requirement contradicts this source"}),
        )
        .await;
    assert_eq!(status, 200, "{root}");
    let thread = root["thread"].clone();
    app.state.agent.set_llm_override(Model::new([
        json!({"python":"tool('create_doc', {'path':'refused.md','input':'Unapproved correction'})"}),
        json!({"done":"The proposed edit was refused; human review remains open."}),
    ]));
    let id = app.send("Investigate the source objection", None).await;
    let change = app.pending().await;
    app.request(
        "POST",
        &app.route("/python/review"),
        json!({"id":change["id"],"accepted":false,"error":"Human refuses this correction"}),
    )
    .await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "ok");
    assert!(!app.root.join("refused.md").exists());
    let link = json!({"version":1,"id":"investigation-1","conversation":app.doc,"turn":id,"state":"running"});
    assert_eq!(
        app.request(
            "POST",
            "/api/reviews",
            json!({"scope":scope,"thread":thread,"action":"investigate","body":link.to_string()})
        )
        .await
        .0,
        200
    );
    let (_, history) = app.request("GET", "/api/reviews", Value::Null).await;
    assert_eq!(history["investigations"][0]["state"], "responded");
    assert_eq!(history["investigations"][0]["response"], turn["answer"]);
    assert_eq!(history["investigations"][0]["session"], turn["session"]);
    assert!(
        !history["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "resolve")
    );
    std::fs::write(
        app.root.join("note.md"),
        "Original human input\nOriginal human input\n",
    )
    .unwrap();
    let (_, history) = app.request("GET", "/api/reviews", Value::Null).await;
    assert_eq!(history["targets"][0]["current_state"], "ambiguous");
    for action in ["resolve", "reopen"] {
        assert_eq!(app.request("POST", "/api/reviews", json!({"scope":scope,"thread":thread,"action":action,"body":"Human inspected the recorded evidence"})).await.0, 200);
    }
    app.state
        .agent
        .set_llm_override(Model::new([json!({"pending_stream":true})]));
    let cancelled = app.send("Cancelled investigation", Some(&id)).await;
    let link = json!({"version":1,"id":"investigation-2","conversation":app.doc,"turn":cancelled,"state":"running"});
    app.request(
        "POST",
        "/api/reviews",
        json!({"scope":scope,"thread":thread,"action":"investigate","body":link.to_string()}),
    )
    .await;
    app.request("POST", &app.route("/stop"), json!({})).await;
    app.wait(&cancelled).await;
    let (_, history) = app.request("GET", "/api/reviews", Value::Null).await;
    assert_eq!(history["investigations"][1]["state"], "cancelled");
}

</storycode:copy>

<a id="output-f30b69dbca4b"></a>
## `scripts/check-python-agent.sh`

Related contracts: [Review investigations link observed agent work without resolving human review](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/check-python-agent.sh" executable="true">
#!/usr/bin/env bash
# The same locked Python and real confined-worker checks run locally and in CI.
set -euo pipefail
cd "$(dirname "$0")/.."
# Rebuild the worker wheel so source edits cannot be masked by a cached install.
uv sync --project apps/agent --locked --no-editable --reinstall-package hickory-agent-worker
uv run --project apps/agent --locked --no-editable python -m unittest discover -s apps/agent/tests -v
HICKORY_AGENT_TEST_PYTHON="$PWD/apps/agent/.venv/bin/python" \
  cargo test -p hickory-cli --test python_agent --test serve_python_agent -- --nocapture
cargo test -p hickory-executor-sandbox agent_process

(cd apps/web && npx playwright install chromium)
scripts/test-python-agent-ui.sh
</storycode:entry>

## Implementation review

### Implementation

- [scripts/check-python-agent.sh](contract.md#output-f30b69dbca4b)

### Tests and fixtures

- [apps/web/src/components/ReviewPanel.test.tsx](../../../mechanisms/apps/web/src/components/ReviewPanel.tsx.md#output-d6bd48d3bc5c)
- [apps/web/src/elements/session/evidence.test.tsx](../../../mechanisms/apps/web/src/elements/session/evidence.test.tsx.md#output-4f5d36dc9d9d)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_python_agent.rs.md#output-08f0cd3133cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
