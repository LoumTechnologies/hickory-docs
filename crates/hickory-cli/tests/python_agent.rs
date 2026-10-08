//! Guarantees: docs/guarantees/agent/python-execution-has-two-boundaries.md
use anyhow::Result;
use hickory_agent::{LlmClient, Message};
use hickory_cli::python_agent::{Config, Proposal, Review, run};
use serde_json::json;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

struct Scripted(Mutex<VecDeque<String>>);
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
        let client = Scripted(Mutex::new(VecDeque::from([
            json!({"python":"tool('record_assumption', {'statement':'The requested note can be new', 'reason':'Avoid altering original input', 'affects':['create_doc']})"}).to_string(),
            json!({"python":"tool('create_doc', {'path':'proposal.md', 'input':'AI proposal'})"}).to_string(),
            json!({"done":"Stopped; checks were not run."}).to_string(),
        ])));
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
        assert!(source.contains("\"verification\":\"unrun\""));
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
    let client = Scripted(Mutex::new(VecDeque::from([
        json!({"shell":"touch hacked"}).to_string(),
        json!({"python":"tool('verify', {})"}).to_string(),
        json!({"done":"Both unsupported actions were refused."}).to_string(),
    ])));
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
    let client = Scripted(Mutex::new(VecDeque::from([
        json!({"python":"tool('read_doc', {'doc':'human.md'})"}).to_string(),
        json!({"python":"tool('edit_doc', {'doc':'human.md', 'after':'^', 'input':'AI prefix'})"})
            .to_string(),
        json!({"done":"The source changed; edit refused."}).to_string(),
    ])));
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
