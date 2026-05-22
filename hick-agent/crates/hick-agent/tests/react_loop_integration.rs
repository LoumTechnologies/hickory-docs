//! Integration tests: Full ReactLoop multi-turn conversations.
//!
//! Run with: cargo test -p hick-agent --features integration-test --test react_loop_integration -- --test-threads=1

#![cfg(feature = "integration-test")]

mod common;

use common::{create_executor, done_message, finished_results, run_agent};
use hick_agent_script_first::AgentEvent;

#[tokio::test(flavor = "multi_thread")]
async fn single_turn_no_code() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // LLM responds with plain text — no code block → Done immediately
            "This is the final answer with no code.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert!(results.is_empty(), "No scripts should have run");
    assert_eq!(
        done_message(&events),
        Some("This is the final answer with no code.")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_turn_python_conversation() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Turn 1: explore
            "Let me check what's here.\n\n```python\nimport os\nprint(os.listdir('.'))\n```",
            // Turn 2: create a file
            "I see the directory is empty. Let me create a file.\n\n```python\nwith open('hello.txt', 'w') as f:\n    f.write('created by agent')\nprint('done')\n```",
            // Turn 3: verify
            "```python\nwith open('hello.txt') as f:\n    print(f.read())\n```",
            // Final answer
            "I created hello.txt with the content 'created by agent'.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 3);

    for (i, r) in results.iter().enumerate() {
        assert_eq!(r.exit_code, Some(0), "script {i} failed: {r:?}");
    }

    assert!(results[2].stdout.contains("created by agent"));
    assert!(done_message(&events).unwrap().contains("created by agent"));
}

#[tokio::test(flavor = "multi_thread")]
async fn cross_language_multi_turn() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Turn 1: Python writes a JSON config
            r#"```python
import json
config = {"app": "hickory", "version": 2, "features": ["sandbox", "multi-lang"]}
with open('config.json', 'w') as f:
    json.dump(config, f)
print('config written')
```"#,
            // Turn 2: TypeScript reads and transforms it
            r#"```typescript
import { readFileSync, writeFileSync } from 'fs';

const config = JSON.parse(readFileSync('config.json', 'utf-8'));
const summary = `${config.app} v${config.version}: ${config.features.join(', ')}`;
writeFileSync('summary.txt', summary);
console.log(summary);
```"#,
            // Final answer
            "Cross-language pipeline complete. Python wrote JSON, TypeScript transformed it.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].stdout.contains("config written"));
    assert!(
        results[1].stdout.contains("hickory v2: sandbox, multi-lang"),
        "stdout: {:?}",
        results[1].stdout
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn error_recovery_multi_turn() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Turn 1: script with runtime error
            "```python\nx = 1 / 0\n```",
            // Turn 2: LLM sees the error and tries again
            "I see a ZeroDivisionError. Let me fix that.\n\n```python\nprint('recovered')\n```",
            // Final answer
            "Fixed the error.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);

    // First script should fail
    assert_ne!(results[0].exit_code, Some(0));

    // Second script should succeed
    assert_eq!(results[1].exit_code, Some(0));
    assert!(results[1].stdout.contains("recovered"));
}

#[tokio::test(flavor = "multi_thread")]
async fn events_emitted_in_order() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec!["```python\nprint('step1')\n```", "All done."],
    )
    .await;

    let event_names: Vec<&str> = events
        .iter()
        .map(|e| match e {
            AgentEvent::Thinking => "Thinking",
            AgentEvent::Token(_) => "Token",
            AgentEvent::ResponseComplete(_) => "ResponseComplete",
            AgentEvent::ScriptStarted { .. } => "ScriptStarted",
            AgentEvent::ScriptFinished(_) => "ScriptFinished",
            AgentEvent::Done(_) => "Done",
            AgentEvent::Error(_) => "Error",
            _ => "Other",
        })
        .collect();

    assert_eq!(event_names[0], "Thinking");
    assert!(event_names.contains(&"ScriptStarted"));
    assert!(event_names.contains(&"ScriptFinished"));
    assert_eq!(*event_names.last().unwrap(), "Done");
}
