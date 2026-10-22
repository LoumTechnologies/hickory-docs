#![cfg(unix)]
//! Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
//! Actual daemon process, authenticated attach, MCP and managed-command requests.
use hick_literate::prediction::{OutputEncoding, output_target};
use hick_observe::prediction::*;
use hickory_cli::{ExecutorChoice, engine::Attach, serve::PrivatePredictionStartup};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};
struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[tokio::test]
async fn every_engine_client_inherits_private_policy_and_mcp_and_command_paths_cannot_bypass() {
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let source = "<hick:container name=\"c\" image=\"alpine\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\">printf 160<hick:expect>100</hick:expect></hick:exec></hick:file>";
    let path = root.path().join("doc.md");
    std::fs::write(&path, source).unwrap();
    std::fs::write(root.path().join("cached.txt"), "PRIVATE-HELD-CACHE").unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let actor = Actor {
        kind: "agent".into(),
        id: "engine-predictor".into(),
    };
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!([80, 80])).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([("test-host".into(), digest(&"local-printf").unwrap())]),
    };
    let target = output_target("c", 2, OutputEncoding::Json);
    let sealed = ledger
        .seal(
            0,
            Seal {
                identity: identity.clone(),
                target: target.clone(),
                predicted: Predicate::Equals(json!(100)),
                rationale: "Attempts share a total budget".into(),
                scenario: Scenario {
                    family: "engine-total-budget".into(),
                    inputs: json!([80, 80]),
                    requirement_reference: "single 100ms total".into(),
                    requirement: Predicate::NumberAtMost(100.0),
                    identity_known: true,
                },
            },
            actor.clone(),
        )
        .unwrap();
    let profile = PrivatePredictionStartup {
        ledger_root: private.path().to_path_buf(),
        seal: sealed.id,
        request_id: "fault".into(),
        identity,
        target,
        actor,
        encoding: OutputEncoding::Json,
        question: None,
        runtime: None,
        inputs: None,
    };
    let model_actions = Arc::new(std::sync::Mutex::new(
        std::collections::VecDeque::<String>::new(),
    ));
    let actions = model_actions.clone();
    let model_router = axum::Router::new().route(
        "/",
        axum::routing::post(move |axum::Json(body): axum::Json<Value>| {
            let actions = actions.clone();
            async move {
                assert!(!body.to_string().contains("PRIVATE-HELD-CACHE"));
                assert_eq!(body["max_completion_tokens"], 2048);
                let raw = actions
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("Unexpected provider request");
                (
                    [("content-type", "text/event-stream")],
                    format!(
                        "data: {}\n\ndata: [DONE]\n\n",
                        json!({"choices":[{"delta":{"content":raw},"finish_reason":"stop"}]})
                    ),
                )
            }
        }),
    );
    let model_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let model_url = format!("http://{}/", model_listener.local_addr().unwrap());
    let model_server = tokio::spawn(async move {
        axum::serve(model_listener, model_router).await.unwrap();
    });
    let mut daemon = Daemon(
        Command::new(env!("CARGO_BIN_EXE_hick"))
            .arg("__engine")
            .env("HICKORY_STATE_DIR", state.path())
            .env("OPENAI_API_KEY", "synthetic-test-key-not-a-credential")
            .env("OPENAI_BASE_URL", &model_url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let endpoint_path = state.path().join("engine/endpoint.json");
    for _ in 0..250 {
        if endpoint_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let endpoint: Value = serde_json::from_slice(&std::fs::read(endpoint_path).unwrap()).unwrap();
    let base = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let client = reqwest::Client::new();
    // Failure after acquiring the workspace lock must release it so trusted
    // recovery can immediately attach the corrected profile to the same root.
    let mut unsupported = Attach::new(path.clone(), ExecutorChoice::Local);
    let mut bad_profile = profile.clone();
    bad_profile.inputs = Some(hick_literate::prediction::SourceInputs {
        value: json!([80, 80]),
        anchors: vec![std::fs::read_to_string(&path).unwrap()],
    });
    bad_profile.runtime = Some(
        hick_literate::prediction::environment::Profile::PythonDebugpy {
            project: root.path().into(),
        },
    );
    unsupported.prediction = Some(bad_profile);
    let refused = client
        .post(format!("{base}/attach"))
        .bearer_auth(token)
        .json(&unsupported)
        .send()
        .await
        .unwrap();
    assert!(!refused.status().is_success());
    let mut first = Attach::new(path.clone(), ExecutorChoice::Local);
    first.client = "private-first".into();
    first.prediction = Some(profile);
    first.python_runtime_path = std::env::var_os("HICKORY_AGENT_TEST_PYTHON").map(Into::into);
    let attached = client
        .post(format!("{base}/attach"))
        .bearer_auth(token)
        .json(&first)
        .send()
        .await
        .unwrap();
    assert!(
        attached.status().is_success(),
        "{}",
        attached.text().await.unwrap()
    );
    // An ordinary second attach inherits the binding, rather than opening an escape.
    let mut second = Attach::new(root.path().to_path_buf(), ExecutorChoice::Local);
    second.client = "private-second".into();
    let attached = client
        .post(format!("{base}/attach"))
        .bearer_auth(token)
        .json(&second)
        .send()
        .await
        .unwrap();
    assert!(
        attached.status().is_success(),
        "{}",
        attached.text().await.unwrap()
    );
    let blocked = client
        .get(format!(
            "{base}/clients/private-second/api/file?path=cached.txt"
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(blocked.status().as_u16(), 422);
    let options = json!({"root":root.path(),"doc":path,"params":[],"executor":ExecutorChoice::Local,"session":null});
    for name in [
        "read_file",
        "read_output",
        "debug_start",
        "debug_eval",
        "search",
        "create_doc",
        "submit_changeset",
    ] {
        let response:Value=client.post(format!("{base}/tool")).bearer_auth(token).json(&json!({"client":"private-second","options":options,"params":{"name":name,"arguments":{"path":"cached.txt"}}})).send().await.unwrap().json().await.unwrap();
        assert_eq!(response["isError"], true, "{name}: {response}");
        assert!(!response.to_string().contains("PRIVATE-HELD-CACHE"));
    }
    // The actual stdio client advertises the guarded set and can submit a
    // prediction through the same bound session. No worker-marker fallback.
    let mut mcp = Command::new(env!("CARGO_BIN_EXE_hick"))
        .args(["mcp", "--doc", "doc.md"])
        .current_dir(root.path())
        .env("HICKORY_STATE_DIR", state.path())
        .env("HICKORY_EXECUTOR", "local")
        .env("HICKORY_ENGINE_WORKER", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        let mut input = mcp.stdin.take().unwrap();
        for request in [
            json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"cached.txt"}}}),
            json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"seal_prediction","arguments":{"expected_source_hash":format!("{:x}",Sha256::digest(source.as_bytes())),"predicted":{"kind":"equals","value":100},"rationale":"The same total deadline applies"}}}),
        ] {
            writeln!(input, "{request}").unwrap();
        }
    }
    let transcript = mcp.wait_with_output().unwrap();
    assert!(
        transcript.status.success(),
        "{}",
        String::from_utf8_lossy(&transcript.stderr)
    );
    let replies: Vec<Value> = String::from_utf8(transcript.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let names: Vec<_> = replies[0]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["read_doc", "edit_doc", "verify", "seal_prediction"]
    );
    assert_eq!(replies[1]["result"]["isError"], true);
    assert!(!replies[1].to_string().contains("PRIVATE-HELD-CACHE"));
    assert_eq!(replies[2]["result"]["isError"], false, "{}", replies[2]);
    let verified:Value=client.post(format!("{base}/tool")).bearer_auth(token).json(&json!({"client":"private-second","options":options,"params":{"name":"verify","arguments":{}}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(verified["isError"], true);
    let summary: Value =
        serde_json::from_str(verified["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(summary["summary"]["classification"], "execution_rejected");
    assert!(!root.path().join("answer.txt").exists());
    let command=client.post(format!("{base}/command")).bearer_auth(token).json(&json!({"executable":"/bin/sh","args":["-c","printf PRIVATE-COMMAND-ANSWER"],"cwd":root.path(),"env":[],"input":[]})).send().await.unwrap();
    assert_eq!(command.status().as_u16(), 422);
    assert!(
        !command
            .text()
            .await
            .unwrap()
            .contains("PRIVATE-COMMAND-ANSWER")
    );
    let mut child = Attach::new(root.path().join("subdir"), ExecutorChoice::Local);
    std::fs::create_dir(&child.target).unwrap();
    child.client = "overlap".into();
    assert_eq!(
        client
            .post(format!("{base}/attach"))
            .bearer_auth(token)
            .json(&child)
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        422
    );
    // Standalone Python starts cannot bypass the active policy through their
    // independent live file/editor context service. Refuse before worker launch.
    let blocked = Command::new(env!("CARGO_BIN_EXE_hick"))
        .args([
            "agent",
            "investigate",
            "--provider",
            "openai",
            "--python",
            "/not-a-runtime",
            "--dir",
        ])
        .arg(root.path())
        .env("HICKORY_STATE_DIR", state.path())
        .env("OPENAI_API_KEY", "synthetic-test-key-not-a-credential")
        .output()
        .unwrap();
    assert!(!blocked.status.success());
    assert!(
        String::from_utf8_lossy(&blocked.stderr).contains("agent context path is unavailable"),
        "{}",
        String::from_utf8_lossy(&blocked.stderr)
    );
    if let Some(python) = std::env::var_os("HICKORY_AGENT_TEST_PYTHON") {
        let worker = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "private_worker_child", "--nocapture"])
            .env("HICKORY_STATE_DIR", state.path())
            .env("HICK_TEST_PRIVATE_WORKER_ROOT", root.path())
            .env("HICKORY_AGENT_TEST_PYTHON", python)
            .output()
            .unwrap();
        assert!(
            worker.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&worker.stdout),
            String::from_utf8_lossy(&worker.stderr)
        );
        assert_eq!(ledger.history(None).unwrap().attempts.len(), 2);
        let current = std::fs::read_to_string(&path).unwrap();
        let proposal = json!({"snapshot":hickory_agent::hashline::LineIndex::new(&current).snapshot,"whole":"true","input":format!("# Proposed\n{current}")});
        model_actions.lock().unwrap().extend([
            json!({"python":"tool('read_doc', {})"}).to_string(),
            json!({"python":format!("import json\ntool('edit_doc', json.loads({:?}))",proposal.to_string())}).to_string(),
            json!({"done":"The human declined the source proposal; no current certification is claimed."}).to_string(),
        ]);
        let private_api = format!("{base}/clients/private-second/api/prediction");
        let started=client.post(format!("{private_api}/agent")).bearer_auth(token).json(&json!({"prompt":"Investigate the retained evidence and propose a source note","provider":"openai"})).send().await.unwrap();
        assert!(
            started.status().is_success(),
            "{}",
            started.text().await.unwrap()
        );
        let mut reviewed = false;
        let mut finished = false;
        for _ in 0..300 {
            let status: Value = client
                .get(format!("{private_api}/agent"))
                .bearer_auth(token)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            assert!(!status.to_string().contains("PRIVATE-HELD-CACHE"));
            if !reviewed
                && let Some(change) = status["edits"]["changes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|change| change["status"] == "pending")
            {
                let decision = client
                    .post(format!("{private_api}/agent/decision"))
                    .bearer_auth(token)
                    .json(&json!({"id":change["id"],"accepted":false}))
                    .send()
                    .await
                    .unwrap();
                assert!(decision.status().is_success());
                reviewed = true;
            }
            if status["run"]["status"] == "finished" {
                finished = true;
                break;
            }
            assert_ne!(status["run"]["status"], "failed", "{status}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(reviewed && finished);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), current);
        assert!(model_actions.lock().unwrap().is_empty());
    } else {
        eprintln!(
            "Private worker witness not run: set HICKORY_AGENT_TEST_PYTHON to the installed confined worker"
        );
        assert_eq!(ledger.history(None).unwrap().attempts.len(), 1);
    }
    let refused_end = client
        .post(format!("{base}/private/end"))
        .bearer_auth(token)
        .json(&json!({"client":"private-first"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        refused_end.status().as_u16(),
        422,
        "Other clients must be closed before ending"
    );
    // Retain an approved, not-yet-resealed repair revision through the same
    // source controller. Restart must restore it unsealed, never execute the old seal.
    let approved = format!(
        "{}\n\nApproved source-only continuation.\n",
        std::fs::read_to_string(&path).unwrap()
    );
    let current_hash = format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()));
    let edited=client.post(format!("{base}/clients/private-first/api/prediction/tool")).bearer_auth(token)
        .json(&json!({"name":"edit_doc","args":[["whole","true"],["snapshot",current_hash]],"input":approved})).send().await.unwrap();
    assert!(
        edited.status().is_success(),
        "{}",
        edited.text().await.unwrap()
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), approved);
    let before_restart = ledger.history(None).unwrap().attempts.len();
    model_server.abort();
    daemon.0.kill().unwrap();
    daemon.0.wait().unwrap();
    std::fs::remove_file(state.path().join("engine/endpoint.json")).unwrap();
    daemon.0 = Command::new(env!("CARGO_BIN_EXE_hick"))
        .arg("__engine")
        .env("HICKORY_STATE_DIR", state.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..250 {
        if state.path().join("engine/endpoint.json").exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let restarted: Value =
        serde_json::from_slice(&std::fs::read(state.path().join("engine/endpoint.json")).unwrap())
            .unwrap();
    let url = restarted["url"].as_str().unwrap();
    let auth = restarted["token"].as_str().unwrap();
    let policy: Value = client
        .get(format!("{url}/prediction-policy"))
        .bearer_auth(auth)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["active"], true);
    let mut ordinary = Attach::new(root.path().into(), ExecutorChoice::Local);
    ordinary.client = "restored-private".into();
    // Disk changes without an approved controller receipt are never adopted.
    std::fs::write(&path, format!("{approved}External mutation\n")).unwrap();
    let refused = client
        .post(format!("{url}/attach"))
        .bearer_auth(auth)
        .json(&ordinary)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status().as_u16(), 422);
    assert!(
        refused
            .text()
            .await
            .unwrap()
            .contains("outside the approved session")
    );
    std::fs::write(&path, &approved).unwrap();
    let restored = client
        .post(format!("{url}/attach"))
        .bearer_auth(auth)
        .json(&ordinary)
        .send()
        .await
        .unwrap();
    assert!(
        restored.status().is_success(),
        "{}",
        restored.text().await.unwrap()
    );
    let acknowledged: Value = restored.json().await.unwrap();
    assert!(acknowledged["prediction"].is_string());
    let restored_api = format!("{url}/clients/restored-private/api/prediction");
    let captured: Value = client
        .post(format!("{restored_api}/tool"))
        .bearer_auth(auth)
        .json(&json!({"name":"read_doc","args":[]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(captured["source"], approved);
    let restored_state: Value = client
        .get(format!("{restored_api}/state"))
        .bearer_auth(auth)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let run: Value = client
        .post(format!(
            "{url}/clients/restored-private/api/docs/{}/run",
            restored_state["document"].as_str().unwrap()
        ))
        .bearer_auth(auth)
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let run_id = run["run_id"].as_str().unwrap();
    let mut stopped = false;
    for _ in 0..100 {
        let result: Value = client
            .get(format!("{url}/clients/restored-private/api/runs/{run_id}"))
            .bearer_auth(auth)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if result["status"] != "running" {
            assert_eq!(result["status"], "failed");
            stopped = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(stopped, "Restored unsealed Run must terminate");
    assert_eq!(
        ledger.history(None).unwrap().attempts.len(),
        before_restart,
        "Recovery must not execute an old seal against the repaired source"
    );
    assert_eq!(client.post(format!("{url}/command")).bearer_auth(auth).json(&json!({"executable":"/bin/sh","args":["-c","true"],"cwd":root.path(),"env":[],"input":[]})).send().await.unwrap().status().as_u16(),422);
    let ended = client
        .post(format!("{url}/private/end"))
        .bearer_auth(auth)
        .json(&json!({"client":"restored-private"}))
        .send()
        .await
        .unwrap();
    assert!(
        ended.status().is_success(),
        "{}",
        ended.text().await.unwrap()
    );
    let receipt: Value = ended.json().await.unwrap();
    assert_eq!(receipt["root_still_private"], true);
    let inactive: Value = client
        .get(format!("{url}/prediction-policy"))
        .bearer_auth(auth)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(inactive["active"], false);
    assert_eq!(
        client
            .post(format!("{url}/attach"))
            .bearer_auth(auth)
            .json(&ordinary)
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        422,
        "Retired client context cannot reconnect"
    );
    assert_eq!(client.post(format!("{url}/command")).bearer_auth(auth).json(&json!({"executable":"/bin/sh","args":["-c","true"],"cwd":root.path(),"env":[],"input":[]})).send().await.unwrap().status().as_u16(),422,"Ending must not turn the private root into an ordinary execution root");
    let normal = tempfile::tempdir().unwrap();
    assert!(client.post(format!("{url}/command")).bearer_auth(auth).json(&json!({"executable":"/bin/sh","args":["-c","true"],"cwd":normal.path(),"env":[],"input":[]})).send().await.unwrap().status().is_success());
    ordinary.client = "fresh-private-context".into();
    let reopened = client
        .post(format!("{url}/attach"))
        .bearer_auth(auth)
        .json(&ordinary)
        .send()
        .await
        .unwrap();
    assert!(
        reopened.status().is_success(),
        "{}",
        reopened.text().await.unwrap()
    );
    assert!(reopened.json::<Value>().await.unwrap()["prediction"].is_string());
    assert_eq!(ledger.history(None).unwrap().attempts.len(), before_restart);
}

#[tokio::test]
async fn private_worker_child() {
    let Some(root) = std::env::var_os("HICK_TEST_PRIVATE_WORKER_ROOT") else {
        return;
    };
    use hickory_agent::{LlmClient, Message};
    use hickory_cli::python_agent::{Config, Conversation, Proposal, Review, run_conversation};
    struct Model(std::sync::Mutex<std::collections::VecDeque<String>>);
    #[async_trait::async_trait]
    impl LlmClient for Model {
        async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
            for message in messages {
                assert!(!message.content.contains("PRIVATE-HELD-CACHE"));
            }
            Ok(self
                .0
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected inference"))
        }
        fn model_name(&self) -> &str {
            "scripted-private-test-no-network"
        }
        fn provider_name(&self) -> &str {
            "test"
        }
    }
    struct AcceptCaptured;
    #[async_trait::async_trait]
    impl Review for AcceptCaptured {
        fn private_prediction_safe(&self) -> bool {
            true
        }
        async fn approve(&mut self, proposal: &Proposal) -> anyhow::Result<bool> {
            // Synthetic test review decision; never presented as human approval evidence.
            assert_eq!(
                proposal.after,
                proposal.before.replace("printf 160", "printf 100")
            );
            Ok(true)
        }
        async fn editor_heads(
            &mut self,
        ) -> anyhow::Result<Option<Vec<hickory_cli::python_agent::EditorHead>>> {
            panic!("Private mode must not request ordinary editor context")
        }
    }
    let root = std::path::PathBuf::from(root);
    let before = std::fs::read_to_string(root.join("doc.md")).unwrap();
    let after = before.replace("printf 160", "printf 100");
    let edit = json!({"doc":"doc.md","snapshot":hickory_agent::hashline::LineIndex::new(&before).snapshot,"whole":"true","input":after});
    let seal = json!({"expected_source_hash":format!("{:x}",Sha256::digest(after.as_bytes())),"predicted":{"kind":"equals","value":100},"rationale":"The unchanged total-budget prediction follows the shared requirement"});
    let model = Model(std::sync::Mutex::new(std::collections::VecDeque::from([
        json!({"done":"The implementation already meets the requirement"}).to_string(),
        json!({"python":"tool('read_doc', {})"}).to_string(),
        json!({"python":"tool('verify', {})"}).to_string(),
        json!({"python":format!("import json\ntool('edit_doc', json.loads({:?}))",edit.to_string())}).to_string(),
        json!({"python":format!("import json\ntool('seal_prediction', json.loads({:?}))",seal.to_string())}).to_string(),
        json!({"python":"tool('verify', {})"}).to_string(),
        json!({"done":"The retained comparison is non-passing; target answers remain withheld."}).to_string(),
    ])));
    let storage = tempfile::tempdir().unwrap();
    let outcome = run_conversation(
        Config {
            python: std::env::var_os("HICKORY_AGENT_TEST_PYTHON")
                .unwrap()
                .into(),
            root: root.clone(),
            doc: Some(root.join("doc.md")),
            prompt: "Investigate with private tools".into(),
            max_turns: 8,
        },
        &model,
        &mut AcceptCaptured,
        Conversation {
            private_prediction: true,
            require_private_verification: true,
            session_storage: Some(storage.path().into()),
            ..Default::default()
        },
        &mut |_| {},
    )
    .await
    .unwrap();
    assert!(outcome.artifact.is_none());
    assert!(outcome.summary.contains("no current certification"));
    assert_eq!(std::fs::read_to_string(root.join("doc.md")).unwrap(), after);
    assert!(!root.join("answer.txt").exists());
    let narrative = std::fs::read_to_string(outcome.session).unwrap();
    assert!(narrative.contains("python-private-completion"));
    assert!(narrative.contains("verified_after_correction"));
    assert!(!narrative.contains("python-verified-changeset"));
    assert!(!narrative.contains("PRIVATE-HELD-CACHE"));
}
