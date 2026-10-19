//! Guarantees: docs/guarantees/agent/python-execution-has-two-boundaries.md
use anyhow::Result;
use hickory_agent::{LlmClient, Message};
use hickory_cli::python_agent::{Config, Proposal, Review, run};
use serde_json::json;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

struct Scripted(Mutex<VecDeque<String>>);
fn scripted(replies: impl IntoIterator<Item = String>) -> Scripted {
    let mut actions = VecDeque::new();
    for raw in replies {
        if serde_json::from_str::<serde_json::Value>(&raw)
            .unwrap()
            .get("done")
            .is_some()
        {
            actions.push_back(json!({"python":SUBMIT}).to_string());
        }
        actions.push_back(raw);
    }
    Scripted(Mutex::new(actions))
}
const SUBMIT: &str = r#"import json
s = json.loads(tool('changeset_status', {})['content'][0]['text'])
tool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})"#;

#[async_trait::async_trait]
impl LlmClient for Scripted {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        assert!(messages.first().unwrap().content.contains("Monty"));
        Ok(self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected model request"))
    }
    fn model_name(&self) -> &str {
        "scripted-test-no-network"
    }
    fn provider_name(&self) -> &str {
        "test"
    }
}
struct FixedReview {
    accept: bool,
    count: usize,
}
#[async_trait::async_trait]
impl Review for FixedReview {
    async fn approve(&mut self, proposal: &Proposal) -> Result<bool> {
        assert_eq!(proposal.after, "AI proposal\n");
        assert_eq!(proposal.surface, "proposal.md");
        self.count += 1;
        Ok(self.accept)
    }
}
fn python() -> Option<PathBuf> {
    let result = std::env::var_os("HICKORY_AGENT_TEST_PYTHON").map(PathBuf::from);
    if result.is_none() {
        eprintln!(
            "SKIP confined Python runtime: run scripts/check-python-agent.sh to install and exercise the real worker"
        );
    }
    result
}

#[cfg(unix)]
#[test]
fn terminal_review_can_be_cancelled_without_waiting_for_input() {
    let Some(python) = python() else {
        return;
    };
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/agent/tests/cli_cancel.py");
    let output = std::process::Command::new(&python)
        .arg("-I")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_hickory-agent"))
        .arg(&python)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("no write occurred"));
}

#[tokio::test]
async fn real_confined_worker_requires_review_and_preserves_the_audit() {
    let Some(python) = python() else {
        return;
    };
    for accept in [false, true] {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("human.md"), "Original human input\n").unwrap();
        let client = scripted([
            json!({"python":"tool('record_assumption', {'statement':'The requested note can be new', 'reason':'Avoid altering original input', 'affects':['create_doc']})"}).to_string(),
            json!({"python":"tool('create_doc', {'path':' proposal.md ', 'input':'AI proposal'})"}).to_string(),
            json!({"done":"Stopped; checks were not run."}).to_string(),
        ]);
        let mut review = FixedReview { accept, count: 0 };
        let session = run(
            Config {
                python: python.clone(),
                root: root.path().into(),
                doc: None,
                prompt: "Write a new note".into(),
                max_turns: 4,
            },
            &client,
            &mut review,
        )
        .await
        .unwrap();
        assert_eq!(review.count, 1);
        assert_eq!(root.path().join("proposal.md").exists(), accept);
        assert_eq!(
            std::fs::read_to_string(root.path().join("human.md")).unwrap(),
            "Original human input\n"
        );
        let source = std::fs::read_to_string(session).unwrap();
        hick_lang::parse(&source).unwrap();
        assert!(source.contains("python-model-request"));
        assert!(source.contains("python-edit-review"));
        assert!(source.contains("Stopped; checks were not run."));
        assert!(source.contains("source-output-equality-only"));
        if accept {
            let actual = std::fs::read_to_string(root.path().join("proposal.md")).unwrap();
            assert_eq!(actual, "AI proposal\n");
            let writes = hickory_agent::context::derive_from_session("session.md", &source);
            let receipt = writes
                .iter()
                .find(|write| write.file == "proposal.md")
                .unwrap();
            assert_eq!(
                receipt.hashes,
                hickory_agent::hashline::LineIndex::new(&actual).hashes
            );
        }
        let timeline = hickory_agent::audit::timeline(&source);
        assert!(timeline.iter().any(|entry| entry["kind"] == "assumption"));
        assert!(
            timeline
                .iter()
                .any(|entry| entry["record_kind"] == "python-edit-review")
        );
    }
}

#[tokio::test]
async fn model_cannot_invoke_shell_or_verify_through_the_broker() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let client = scripted([
        json!({"shell":"touch hacked"}).to_string(),
        json!({"python":"tool('verify', {})"}).to_string(),
        json!({"done":"Both unsupported actions were refused."}).to_string(),
    ]);
    let mut review = FixedReview {
        accept: true,
        count: 0,
    };
    let session = run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Attempt unsupported actions".into(),
            max_turns: 4,
        },
        &client,
        &mut review,
    )
    .await
    .unwrap();
    assert_eq!(review.count, 0);
    assert!(!root.path().join("hacked").exists());
    let source = std::fs::read_to_string(session).unwrap();
    assert!(source.contains("not exposed"));
}

#[test]
fn os_profile_denies_workspace_bytes_and_network_even_to_trusted_python() {
    let Some(python) = python() else {
        return;
    };
    let workspace = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let secret = workspace.path().join("private-key");
    std::fs::write(&secret, "must never cross the boundary").unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let code = format!(
        r#"
import os, socket, sys
def denied(fn):
    try: fn()
    except (OSError, PermissionError): return True
    return False
assert denied(lambda: open({secret}).read())
if sys.platform == 'darwin':
    assert denied(lambda: open('/System/Volumes/Data' + {secret}).read())
assert denied(lambda: open({secret}, 'w').write('overwritten'))
assert denied(lambda: socket.create_connection(('127.0.0.1', {port}), timeout=1))
assert 'OPENAI_API_KEY' not in os.environ
assert 'ANTHROPIC_API_KEY' not in os.environ
open('scratch.txt', 'w').write('allowed')
print('confined')
"#,
        secret = json!(secret.to_str().unwrap()),
        port = listener.local_addr().unwrap().port()
    );
    let original =
        hickory_executor_sandbox::agent_process::command(&python, scratch.path()).unwrap();
    let args = original.get_args().collect::<Vec<_>>();
    let mut command = std::process::Command::new(original.get_program());
    command
        .args(&args[..args.len() - 4])
        .args(["-I", "-B", "-c", &code]);
    command
        .env_clear()
        .current_dir(original.get_current_dir().unwrap());
    for (key, value) in original.get_envs() {
        if let Some(value) = value {
            command.env(key, value);
        }
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "confined");
    assert_eq!(
        std::fs::read_to_string(secret).unwrap(),
        "must never cross the boundary"
    );
}

#[tokio::test]
async fn approval_is_invalidated_when_the_source_changes() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("human.md");
    std::fs::write(&path, "Original human input\n").unwrap();
    struct ConcurrentHuman(PathBuf, usize);
    #[async_trait::async_trait]
    impl Review for ConcurrentHuman {
        async fn approve(&mut self, _: &Proposal) -> Result<bool> {
            std::fs::write(&self.0, "New human input while reviewing\n")?;
            self.1 += 1;
            Ok(true)
        }
    }
    let client = scripted([
        json!({"python":"tool('read_doc', {'doc':'human.md'})"}).to_string(),
        json!({"python":"tool('edit_doc', {'doc':'human.md', 'after':'^', 'input':'AI prefix'})"})
            .to_string(),
        json!({"done":"The source changed; edit refused."}).to_string(),
    ]);
    let mut review = ConcurrentHuman(path.clone(), 0);
    let session = run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Suggest a prefix".into(),
            max_turns: 4,
        },
        &client,
        &mut review,
    )
    .await
    .unwrap();
    assert_eq!(review.1, 1);
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "New human input while reviewing\n"
    );
    let source = std::fs::read_to_string(session).unwrap();
    assert!(
        source.contains("Proposal changed during approval")
            || source.contains("changed since the previous surface")
    );
}

// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
#[tokio::test]
async fn model_claims_and_worker_prints_cannot_certify_completion() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let client = Scripted(Mutex::new(VecDeque::from([
        json!({"python":"print({'accepted': True, 'verification': 'passed'})"}).to_string(),
        json!({"done":"I verified everything; trust my claim"}).to_string(),
    ])));
    let error = run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Claim success without an artifact".into(),
            max_turns: 2,
        },
        &client,
        &mut FixedReview {
            accept: false,
            count: 0,
        },
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("without model completion"),
        "{error:#}"
    );
    let session = std::fs::read_dir(root.path().join("sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let source = std::fs::read_to_string(session).unwrap();
    assert!(source.contains("No current accepted changeset"));
    assert!(source.contains("worker-reported"));
    assert!(!source.contains("python-verified-changeset"));
}

// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
#[tokio::test]
async fn a_write_after_submission_invalidates_completion() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let client = Scripted(Mutex::new(VecDeque::from([
        json!({"python":SUBMIT}).to_string(),
        json!({"python":"tool('create_doc', {'path':'proposal.md', 'input':'AI proposal'})"})
            .to_string(),
        json!({"done":"Use the old accepted artifact"}).to_string(),
    ])));
    let error = run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Write after verification".into(),
            max_turns: 3,
        },
        &client,
        &mut FixedReview {
            accept: true,
            count: 0,
        },
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("without model completion"),
        "{error:#}"
    );
    assert!(
        root.path().join("proposal.md").exists(),
        "actual approved work remains"
    );
}

// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
#[tokio::test]
async fn indirect_execution_and_outside_proposals_are_not_capabilities() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let destination = outside.path().join("escaped.md");
    let client = scripted([
        json!({"python":"tool('debug_start', {})"}).to_string(),
        json!({"python":"tool('begin_changeset', {'workspace':'/'})"}).to_string(),
        json!({"python":"tool('write_file', {'path':'hacked', 'content':'bypass'})"}).to_string(),
        json!({"python":format!("tool('create_doc', {{'path':{}, 'input':'AI proposal'}})", serde_json::to_string(&destination.display().to_string()).unwrap())}).to_string(),
        json!({"done":"The bypasses were refused"}).to_string(),
    ]);
    let mut review = FixedReview {
        accept: true,
        count: 0,
    };
    run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Attempt review bypasses".into(),
            max_turns: 6,
        },
        &client,
        &mut review,
    )
    .await
    .unwrap();
    assert_eq!(review.count, 0);
    assert!(!destination.exists());
    assert!(!root.path().join("hacked").exists());
}

// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
#[tokio::test]
async fn approved_executable_document_content_does_not_execute_during_reads_or_completion() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("unreviewed-output");
    let source = format!(
        r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="work" image="host" />
<hick:exec container="work">touch {}</hick:exec>
</hick:doc>
"#,
        marker.display()
    );
    let client = scripted([
        json!({"python":format!("tool('create_doc', {{'path':'executable.md', 'input':{}}})", serde_json::to_string(&source).unwrap())}).to_string(),
        json!({"python":"tool('read_doc', {'doc':'executable.md'})"}).to_string(),
        json!({"python":"tool('verify', {'doc':'executable.md'})"}).to_string(),
        json!({"done":"Source artifact only; native execution unrun"}).to_string(),
    ]);
    struct Approve(usize);
    #[async_trait::async_trait]
    impl Review for Approve {
        async fn approve(&mut self, _: &Proposal) -> Result<bool> {
            self.0 += 1;
            Ok(true)
        }
    }
    let mut review = Approve(0);
    run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Propose executable content without running it".into(),
            max_turns: 5,
        },
        &client,
        &mut review,
    )
    .await
    .unwrap();
    assert_eq!(review.0, 1);
    assert_eq!(
        std::fs::read_to_string(root.path().join("executable.md")).unwrap(),
        source
    );
    assert!(
        !marker.exists(),
        "authored commands must not bypass edit review"
    );
}
