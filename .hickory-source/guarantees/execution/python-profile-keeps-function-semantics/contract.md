<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Python profiles keep function scope and measurement semantics

For: Authors running documents on their own machine and reviewers assessing their side effects.

## Requirements

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/python-profile-keeps-function-semantics.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-e94b83d3cd71"></a>
## `apps/web/src/debug/decodeRuntimeRun.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Python profiles keep function scope and measurement semantics](contract.md).

<a id="f-e94b83d3cd71-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e94b83d3cd71-expression_statement">
// Guarantee: docs/guarantees/execution/python-profile-keeps-function-semantics.md
import { expect, it } from "vitest";
import record from "./fixtures/python-profile.json";
import { decodeRuntimeRun } from "./decodeRuntimeRun";
it("imports the actual selected native profile as captured data without executing it", async () => {
  const run = await decodeRuntimeRun(JSON.stringify(record));
  expect(run.id).toBe(record.id);
  expect(run.events).toEqual(record.events);
  expect(new TextDecoder().decode(new Uint8Array(run.sources[0].bytes))).toContain("subtotal");
});
</storycode:copy>
<a id="f-e94b83d3cd71-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e94b83d3cd71-expression_statement-2">
it("refuses altered source bytes and malformed profile state rather than manufacturing a valid capture", async () => {
  const changed = structuredClone(record); changed.sources[0].bytes[0] ^= 1;
  await expect(decodeRuntimeRun(JSON.stringify(changed))).rejects.toThrow("source hash");
  const malformed = structuredClone(record); malformed.events[0].observation.value = "not a measurement" as never;
  await expect(decodeRuntimeRun(JSON.stringify(malformed))).rejects.toThrow("Invalid recorded runtime evidence");
  await expect(decodeRuntimeRun(JSON.stringify({ ...record, extra_actual: "unrecognized" }))).rejects.toThrow();
});
</storycode:copy>

### Assemble `apps/web/src/debug/decodeRuntimeRun.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/decodeRuntimeRun.test.ts"><storycode:paste select="#f-e94b83d3cd71-expression_statement"/><storycode:paste select="#f-e94b83d3cd71-expression_statement-2"/></storycode:file>

<a id="output-7f6da3233440"></a>
## `apps/web/src/debug/profileRows.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Python profiles keep function scope and measurement semantics](contract.md).

<a id="f-7f6da3233440-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7f6da3233440-expression_statement">
// Guarantee: docs/guarantees/execution/python-profile-keeps-function-semantics.md
import { expect, it } from "vitest";
import captured from "./fixtures/python-profile.json";
import { profileRows } from "./profileRows";
import { runtimeReading } from "./runtimeHistory";
import type { RuntimeRun } from "./runtimeObservations";
it("parses actual cProfile function aggregates without summing inclusive times", () => {
  const run = captured as RuntimeRun;
  const rows = profileRows(run.events), subtotal = rows.find(row => row.function.name === "subtotal")!;
  expect(subtotal.function.total_calls).toBe(500); expect(subtotal.function.primitive_calls).toBe(500);
  const recursive = rows.find(row => row.function.name === "recursive")!;
  expect(recursive.function.total_calls).toBe(10); expect(recursive.function.primitive_calls).toBe(1);
  const measured = subtotal.events.find(event => event.observation.kind === "profile" && event.observation.metric === "self_seconds")!.observation;
  expect(measured.kind).toBe("profile");
  if (measured.kind === "profile") expect(subtotal.self).toBe(measured.value);
  expect(subtotal.events).toHaveLength(2);
  const reading = runtimeReading(run);
  expect(reading.provider_annotations!.every(a => a.provider_version === "3.9.6")).toBe(true);
  expect(reading.provider_annotations!.every(a => a.origin.kind === "runtime" && a.origin.stop === null)).toBe(true);
  expect(rows.some(row => row.events.every(event => event.emission === null))).toBe(true);
});
</storycode:copy>

### Assemble `apps/web/src/debug/profileRows.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/profileRows.test.ts"><storycode:paste select="#f-7f6da3233440-expression_statement"/></storycode:file>

<a id="output-6d332b06f9dc"></a>
## `crates/hickory-cli/tests/python_agent_profile.rs`

Related contracts: [Python profiles keep function scope and measurement semantics](contract.md).

<a id="f-6d332b06f9dc-Decisions"></a>
### `Decisions`

Focused #75 integration: real confined worker and host cProfile, scripted model/UI transport.

<storycode:copy id="f-6d332b06f9dc-Decisions">
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
</storycode:copy>
<a id="f-6d332b06f9dc-impl-LlmClient-for-Decisions"></a>
### `impl-LlmClient for Decisions`

<storycode:copy id="f-6d332b06f9dc-impl-LlmClient-for-Decisions">
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
</storycode:copy>
<a id="f-6d332b06f9dc-ProfileReview"></a>
### `ProfileReview`

<storycode:copy id="f-6d332b06f9dc-ProfileReview">
struct ProfileReview {
    base: String,
    before: String,
    after: String,
    approvals: usize,
    captures: Vec<Value>,
}
</storycode:copy>
<a id="f-6d332b06f9dc-impl-Review-for-ProfileReview"></a>
### `impl-Review for ProfileReview`

<storycode:copy id="f-6d332b06f9dc-impl-Review-for-ProfileReview">
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

</storycode:copy>
<a id="f-6d332b06f9dc-confined_agent_profiles_reviewed_optimization_and_remeasures_exact_source"></a>
### `confined_agent_profiles_reviewed_optimization_and_remeasures_exact_source`

<storycode:copy id="f-6d332b06f9dc-confined_agent_profiles_reviewed_optimization_and_remeasures_exact_source">
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/python_agent_profile.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/python_agent_profile.rs"><storycode:paste select="#f-6d332b06f9dc-Decisions"/><storycode:paste select="#f-6d332b06f9dc-impl-LlmClient-for-Decisions"/><storycode:paste select="#f-6d332b06f9dc-ProfileReview"/><storycode:paste select="#f-6d332b06f9dc-impl-Review-for-ProfileReview"/><storycode:paste select="#f-6d332b06f9dc-confined_agent_profiles_reviewed_optimization_and_remeasures_exact_source"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/observation.rs](../../../mechanisms/crates/hickory-cli/src/observation.rs.md#output-131492bc8594)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/debug/decodeRuntimeRun.test.ts](contract.md#output-e94b83d3cd71)
- [apps/web/src/debug/profileRows.test.ts](contract.md#output-7f6da3233440)
- [apps/web/src/debug/runtimeHistory.test.tsx](../../../mechanisms/apps/web/src/debug/runtimeHistory.ts.md#output-b01fe4e29028)
- [crates/hickory-cli/tests/python_agent_profile.rs](contract.md#output-6d332b06f9dc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
