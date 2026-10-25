<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/python_agent_profile.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Python profiles keep function scope and measurement semantics](../../../../contract.md)

Captured SHA-256: `5cf365a4207241e78190cfa8b2726b58f28658cd892fe6f867d324237435c0f5`.

<book:entry path="crates/hickory-cli/tests/python_agent_profile.rs">
//! Focused #75 integration: real confined worker and host cProfile, scripted model/UI transport.
//! Guarantees: agent/ui-action-dispatch.md; execution/python-profile-keeps-function-semantics.md.
use anyhow::{Result, ensure};
use hickory_agent::{LlmClient, Message};
use hickory_cli::{
    ExecutorChoice,
    python_agent::{Config, Proposal, Review, run},
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{collections::VecDeque, path::PathBuf, sync::Mutex};

struct Decisions(Mutex<VecDeque<String>>);
#[async_trait::async_trait]
impl LlmClient for Decisions {
    async fn complete(&self, _: Vec<Message>) -> Result<String> {
        self.0
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("Unexpected model turn"))
    }
    fn model_name(&self) -> &str {
        "scripted-profile-no-network"
    }
    fn provider_name(&self) -> &str {
        "test"
    }
}
struct ProfileReview {
    base: String,
    before: String,
    after: String,
    approvals: usize,
    captures: Vec<Value>,
}
#[async_trait::async_trait]
impl Review for ProfileReview {
    fn ui_tools(&self) -> Vec<Value> {
        // The test supplies only the UI transport endpoint. Production React
        // registration/reading is covered separately by uiRuntimeActions tests.
        vec![
            json!({"name":"ui_act","description":"Invoke the disclosed profile-current test handle","inputSchema":{"type":"object","properties":{"handle":{"type":"string"},"arguments":{"type":"object"},"request_id":{"type":"string"}},"required":["handle","arguments","request_id"]}}),
        ]
    }
    async fn ui_call(&mut self, name: &str, args: Value) -> Result<Value> {
        ensure!(
            name == "ui_act" && args["handle"] == "profile-current",
            "Unknown fixture handle"
        );
        ensure!(self.captures.len() < 2, "Unexpected capture");
        ensure!(
            self.approvals == self.captures.len(),
            "Remeasure requires independent edit review"
        );
        let response = reqwest::Client::new()
            .post(format!("{}/api/runtime/python", self.base))
            .json(&json!({"document":"hotspot.md","program":"main.py","seconds":10,"profile":true}))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let run = response["run"].clone();
        ensure!(run["status"] == "completed", "Actual Python capture failed");
        self.captures.push(run.clone());
        // Native capture returns a compact receipt. Full profile records must
        // be read through bounded pages rather than repeated in model frames.
        Ok(
            json!({"status":"result","outcome":{"status":"completed","value":{"run":run["id"],"status":run["status"],"sources":run["sources"].as_array().unwrap().iter().map(|source| json!({"uri":source["uri"],"revision":source["revision"]})).collect::<Vec<_>>(),"events":run["events"].as_array().unwrap().len(),"scope":"Actual host capture through a scripted UI transport; not a model inference"}}}),
        )
    }
    async fn approve(&mut self, proposal: &Proposal) -> Result<bool> {
        ensure!(
            proposal.tool == "edit_doc"
                && proposal.before == self.before
                && proposal.after == self.after,
            "Unexpected optimization proposal"
        );
        ensure!(
            self.captures.len() == 1 && self.approvals == 0,
            "Profile before edit review"
        );
        self.approvals += 1;
        Ok(true) // Test driver review; never a claim of human judgment.
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn confined_agent_profiles_reviewed_optimization_and_remeasures_exact_source() {
    let python = PathBuf::from(
        std::env::var_os("HICKORY_AGENT_TEST_PYTHON")
            .expect("Set the configured worker interpreter for this explicit integration test"),
    );
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let before = "<hick:file path=\"main.py\">\ndef subtotal(n):\n    return sum(range(n))\nprint(sum(subtotal(100) for _ in range(25)))\n</hick:file>\n".to_owned();
    let after = before.replace(
        "sum(subtotal(100) for _ in range(25))",
        "subtotal(100) * 25",
    );
    std::fs::write(root.join("hotspot.md"), &before).unwrap();
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
    let action = |code: String| json!({"python":code}).to_string();
    let edit = json!({"doc":"hotspot.md","whole":"true","snapshot":hickory_agent::hashline::LineIndex::new(&before).snapshot,"input":after});
    let decisions = Decisions(Mutex::new([
        action("tool('ui_act', {'handle':'profile-current','arguments':{},'request_id':'before'})".into()),
        action("tool('read_doc', {'doc':'hotspot.md'})".into()),
        action(format!("import json\ntool('edit_doc', json.loads({:?}))", edit.to_string())),
        action("tool('ui_act', {'handle':'profile-current','arguments':{},'request_id':'after'})".into()),
        action("import json\ns = json.loads(tool('changeset_status', {})['content'][0]['text'])\ntool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})".into()),
        json!({"done":"The scripted optimization reduced subtotal calls while preserving output. No timing speedup is claimed."}).to_string(),
    ].into()));
    let mut review = ProfileReview {
        base: base.clone(),
        before: before.clone(),
        after: after.clone(),
        approvals: 0,
        captures: vec![],
    };
    let result = run(Config { python, root: root.clone(), doc: Some("hotspot.md".into()), prompt: "Profile, propose the repeated-call optimization for independent review, and remeasure the same saved application.".into(), max_turns: 8 }, &decisions, &mut review).await;
    result.unwrap();
    assert_eq!(review.approvals, 1);
    assert_eq!(review.captures.len(), 2);
    let (first, second) = (&review.captures[0], &review.captures[1]);
    assert_ne!(first["id"], second["id"]);
    for (run, source, calls) in [(first, &before, 25), (second, &after, 1)] {
        let bytes = run["sources"][0]["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect::<Vec<_>>();
        assert_eq!(bytes, source.as_bytes());
        assert_eq!(
            run["sources"][0]["revision"],
            hickory_agent::hashline::LineIndex::new(source).snapshot
        );
        let events = run["events"].as_array().unwrap();
        assert!(
            events
                .iter()
                .any(|event| event["observation"]["kind"] == "profile"
                    && event["observation"]["function"]["name"] == "subtotal"
                    && event["observation"]["function"]["total_calls"] == calls)
        );
        let output = events
            .iter()
            .filter(|event| event["observation"]["kind"] == "output")
            .flat_map(|event| {
                event["observation"]["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u8)
            })
            .collect::<Vec<_>>();
        assert_eq!(String::from_utf8(output).unwrap().trim(), "123750");
        let retained: Value = reqwest::Client::new()
            .get(format!("{base}/api/runtime/runs"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(
            retained["runs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["run"] == *run)
        );
    }
    assert_eq!(
        std::fs::read_to_string(root.join("hotspot.md")).unwrap(),
        after
    );
    server.abort();
}
</book:entry>

</book:doc>
