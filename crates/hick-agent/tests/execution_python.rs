//! Integration tests: Python code execution inside WASM sandbox.
//!
//! These tests verify that the agent can write and execute Python code
//! inside a WASM Linux container, with no LLM calls (uses ScriptedLlm).
//!
//! Run with: cargo test -p hick-agent --features integration-test --test execution_python -- --test-threads=1

#![cfg(feature = "integration-test")]

mod common;

use common::{create_executor, done_message, finished_results, run_agent};

#[tokio::test(flavor = "multi_thread")]
async fn python_hello_world() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```python\nprint('hello from python')\n```",
            "The output was hello from python.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(
        results[0].stdout.contains("hello from python"),
        "stdout: {:?}",
        results[0].stdout
    );
    assert_eq!(results[0].exit_code, Some(0));
    assert!(done_message(&events).is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn python_file_io() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Turn 1: write a file
            "```python\nwith open('test_output.txt', 'w') as f:\n    f.write('hello file io')\nprint('written')\n```",
            // Turn 2: read it back
            "```python\nwith open('test_output.txt') as f:\n    print(f.read())\n```",
            // Turn 3: final answer
            "File round-trip successful.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].stdout.contains("written"));
    assert!(results[1].stdout.contains("hello file io"));
}

#[tokio::test(flavor = "multi_thread")]
async fn python_syntax_error() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```python\ndef broken(\n```",
            "Got a syntax error as expected.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert_ne!(results[0].exit_code, Some(0), "should have non-zero exit");
}

#[tokio::test(flavor = "multi_thread")]
async fn python_multiline_with_imports() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```python\nimport json\ndata = {'key': 'value', 'num': 42}\nprint(json.dumps(data, sort_keys=True))\n```",
            "JSON serialization works.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].stdout.contains(r#""key": "value""#));
    assert!(results[0].stdout.contains(r#""num": 42"#));
    assert_eq!(results[0].exit_code, Some(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn python_pip_install_and_use() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Turn 1: install a small package
            "```python\nimport subprocess\nresult = subprocess.run(['pip', 'install', 'cowsay'], capture_output=True, text=True)\nprint(result.stdout)\nprint(result.stderr)\n```",
            // Turn 2: use it
            "```python\nimport cowsay\ncowsay.cow('moo')\nprint('cowsay works')\n```",
            // Final answer
            "Package installed and used successfully.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    // The install should succeed
    assert_eq!(results[0].exit_code, Some(0));
    // The usage should produce output
    assert!(
        results[1].stdout.contains("cowsay works"),
        "stdout: {:?}",
        results[1].stdout
    );
}
