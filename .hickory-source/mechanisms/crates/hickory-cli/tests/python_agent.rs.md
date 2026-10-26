<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/python_agent.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9bda4649fd61"></a>
## `crates/hickory-cli/tests/python_agent.rs`

Related contracts: [Finish an agent turn against its captured project boundary](../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md), [Separate model snippets from trusted host operations](../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md).

<a id="f-9bda4649fd61-Scripted"></a>
### `Scripted`

<storycode:copy id="f-9bda4649fd61-Scripted">
//! Guarantees: docs/guarantees/agent/python-execution-has-two-boundaries.md
use anyhow::Result;
use hickory_agent::{LlmClient, Message};
use hickory_cli::python_agent::{Config, Proposal, Review, run};
use serde_json::json;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

struct Scripted(Mutex<VecDeque<String>>);
</storycode:copy>
<a id="f-9bda4649fd61-scripted"></a>
### `scripted`

<storycode:copy id="f-9bda4649fd61-scripted">
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
</storycode:copy>
<a id="f-9bda4649fd61-SUBMIT"></a>
### `SUBMIT`

<storycode:copy id="f-9bda4649fd61-SUBMIT">
const SUBMIT: &str = r#"import json
s = json.loads(tool('changeset_status', {})['content'][0]['text'])
tool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})"#;

</storycode:copy>
<a id="f-9bda4649fd61-impl-LlmClient-for-Scripted"></a>
### `impl-LlmClient for Scripted`

<storycode:copy id="f-9bda4649fd61-impl-LlmClient-for-Scripted">
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
</storycode:copy>
<a id="f-9bda4649fd61-FixedReview"></a>
### `FixedReview`

<storycode:copy id="f-9bda4649fd61-FixedReview">
struct FixedReview {
    accept: bool,
    count: usize,
}
</storycode:copy>
<a id="f-9bda4649fd61-impl-Review-for-FixedReview"></a>
### `impl-Review for FixedReview`

<storycode:copy id="f-9bda4649fd61-impl-Review-for-FixedReview">
#[async_trait::async_trait]
impl Review for FixedReview {
    async fn approve(&mut self, proposal: &Proposal) -> Result<bool> {
        assert_eq!(proposal.after, "AI proposal\n");
        assert_eq!(proposal.surface, "proposal.md");
        self.count += 1;
        Ok(self.accept)
    }
}
</storycode:copy>
<a id="f-9bda4649fd61-python"></a>
### `python`

<storycode:copy id="f-9bda4649fd61-python">
fn python() -> Option<PathBuf> {
    let result = std::env::var_os("HICKORY_AGENT_TEST_PYTHON").map(PathBuf::from);
    if result.is_none() {
        eprintln!(
            "SKIP confined Python runtime: run scripts/check-python-agent.sh to install and exercise the real worker"
        );
    }
    result
}

</storycode:copy>
<a id="f-9bda4649fd61-terminal_review_can_be_cancelled_without_waiting_for_input"></a>
### `terminal_review_can_be_cancelled_without_waiting_for_input`

<storycode:copy id="f-9bda4649fd61-terminal_review_can_be_cancelled_without_waiting_for_input">
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

</storycode:copy>
<a id="f-9bda4649fd61-real_confined_worker_requires_review_and_preserves_the_audit"></a>
### `real_confined_worker_requires_review_and_preserves_the_audit`

<storycode:copy id="f-9bda4649fd61-real_confined_worker_requires_review_and_preserves_the_audit">
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

</storycode:copy>
<a id="f-9bda4649fd61-model_cannot_invoke_shell_or_verify_through_the_broker"></a>
### `model_cannot_invoke_shell_or_verify_through_the_broker`

<storycode:copy id="f-9bda4649fd61-model_cannot_invoke_shell_or_verify_through_the_broker">
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

</storycode:copy>
<a id="f-9bda4649fd61-os_profile_denies_workspace_bytes_and_network_even_to_trusted_python"></a>
### `os_profile_denies_workspace_bytes_and_network_even_to_trusted_python`

<storycode:copy id="f-9bda4649fd61-os_profile_denies_workspace_bytes_and_network_even_to_trusted_python">
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

</storycode:copy>
<a id="f-9bda4649fd61-approval_is_invalidated_when_the_source_changes"></a>
### `approval_is_invalidated_when_the_source_changes`

<storycode:copy id="f-9bda4649fd61-approval_is_invalidated_when_the_source_changes">
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

</storycode:copy>
[Read the test `model_claims_and_worker_prints_cannot_certify_completion` beside its requirement](../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md#f-9bda4649fd61-model_claims_and_worker_prints_cannot_certify_completion).
[Read the test `a_write_after_submission_invalidates_completion` beside its requirement](../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md#f-9bda4649fd61-a_write_after_submission_invalidates_completion).
[Read the test `indirect_execution_and_outside_proposals_are_not_capabilities` beside its requirement](../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md#f-9bda4649fd61-indirect_execution_and_outside_proposals_are_not_capabilities).
[Read the test `approved_executable_document_content_does_not_execute_during_reads_or_completion` beside its requirement](../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md#f-9bda4649fd61-approved_executable_document_content_does_not_execute_during_reads_or_completion).

### Assemble `crates/hickory-cli/tests/python_agent.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/python_agent.rs"><storycode:paste select="#f-9bda4649fd61-Scripted"/><storycode:paste select="#f-9bda4649fd61-scripted"/><storycode:paste select="#f-9bda4649fd61-SUBMIT"/><storycode:paste select="#f-9bda4649fd61-impl-LlmClient-for-Scripted"/><storycode:paste select="#f-9bda4649fd61-FixedReview"/><storycode:paste select="#f-9bda4649fd61-impl-Review-for-FixedReview"/><storycode:paste select="#f-9bda4649fd61-python"/><storycode:paste select="#f-9bda4649fd61-terminal_review_can_be_cancelled_without_waiting_for_input"/><storycode:paste select="#f-9bda4649fd61-real_confined_worker_requires_review_and_preserves_the_audit"/><storycode:paste select="#f-9bda4649fd61-model_cannot_invoke_shell_or_verify_through_the_broker"/><storycode:paste select="#f-9bda4649fd61-os_profile_denies_workspace_bytes_and_network_even_to_trusted_python"/><storycode:paste select="#f-9bda4649fd61-approval_is_invalidated_when_the_source_changes"/><storycode:paste select="#f-9bda4649fd61-model_claims_and_worker_prints_cannot_certify_completion"/><storycode:paste select="#f-9bda4649fd61-a_write_after_submission_invalidates_completion"/><storycode:paste select="#f-9bda4649fd61-indirect_execution_and_outside_proposals_are_not_capabilities"/><storycode:paste select="#f-9bda4649fd61-approved_executable_document_content_does_not_execute_during_reads_or_completion"/></storycode:file>

</storycode:doc>
