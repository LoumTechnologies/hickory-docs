//! Integration tests: OS-level and language-specific package installation.
//!
//! Verifies that the sandbox container supports installing packages via
//! apt-get, pip, and npm, and that installed packages are usable.
//!
//! Run with: cargo test -p hick-agent --features integration-test --test package_install -- --test-threads=1

#![cfg(feature = "integration-test")]

mod common;

use common::{create_executor, finished_results, run_agent};

#[tokio::test(flavor = "multi_thread")]
async fn apt_get_install_and_use() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Install jq via apt-get
            "```python\nimport subprocess\nresult = subprocess.run(['apt-get', 'update'], capture_output=True, text=True)\nprint('update:', result.returncode)\nresult = subprocess.run(['apt-get', 'install', '-y', 'jq'], capture_output=True, text=True)\nprint('install:', result.returncode)\n```",
            // Use jq to process JSON
            r#"```python
import subprocess
result = subprocess.run(
    ['jq', '.name'],
    input='{"name": "hickory", "version": 1}',
    capture_output=True, text=True
)
print(result.stdout.strip())
```"#,
            "jq installed and working.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].exit_code, Some(0));
    assert!(
        results[1].stdout.contains("\"hickory\""),
        "stdout: {:?}",
        results[1].stdout
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pip_install_requests() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Install requests
            "```python\nimport subprocess\nresult = subprocess.run(['pip', 'install', 'requests'], capture_output=True, text=True)\nprint('exit:', result.returncode)\n```",
            // Verify it imports
            "```python\nimport requests\nprint('requests version:', requests.__version__)\n```",
            "requests installed.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].stdout.contains("exit: 0"));
    assert!(results[1].stdout.contains("requests version:"));
    assert_eq!(results[1].exit_code, Some(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn npm_install_and_verify() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Init npm project and install a package
            "```python\nimport subprocess\nsubprocess.run(['npm', 'init', '-y'], capture_output=True)\nresult = subprocess.run(['npm', 'install', 'chalk@4'], capture_output=True, text=True)\nprint('exit:', result.returncode)\n```",
            // Verify it's installed
            "```python\nimport subprocess, json\nresult = subprocess.run(['node', '-e', 'console.log(JSON.stringify(Object.keys(require(\"chalk\"))))'], capture_output=True, text=True)\nprint(result.stdout)\n```",
            "npm package installed.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].stdout.contains("exit: 0"));
    assert_eq!(results[1].exit_code, Some(0));
}
