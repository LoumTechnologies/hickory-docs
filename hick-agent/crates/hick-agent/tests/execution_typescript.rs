//! Integration tests: TypeScript code execution inside WASM sandbox.
//!
//! These tests verify that the agent can write and execute TypeScript code
//! (via `npx tsx`) inside a WASM Linux container.
//!
//! Run with: cargo test -p hick-agent --features integration-test --test execution_typescript -- --test-threads=1

#![cfg(feature = "integration-test")]

mod common;

use common::{create_executor, finished_results, run_agent};

#[tokio::test(flavor = "multi_thread")]
async fn typescript_hello_world() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```typescript\nconsole.log('hello from typescript')\n```",
            "The output was hello from typescript.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(
        results[0].stdout.contains("hello from typescript"),
        "stdout: {:?}",
        results[0].stdout
    );
    assert_eq!(results[0].exit_code, Some(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn typescript_ts_alias() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```ts\nconst x: number = 42;\nconsole.log(`answer is ${x}`);\n```",
            "Got the answer.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].stdout.contains("answer is 42"));
}

#[tokio::test(flavor = "multi_thread")]
async fn typescript_type_features() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            r#"```typescript
interface Person {
  name: string;
  age: number;
}

function greet(p: Person): string {
  return `Hello ${p.name}, age ${p.age}`;
}

const p: Person = { name: "Alice", age: 30 };
console.log(greet(p));
```"#,
            "TypeScript interfaces work.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].stdout.contains("Hello Alice, age 30"));
    assert_eq!(results[0].exit_code, Some(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn typescript_file_io() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Write a file
            r#"```typescript
import { writeFileSync } from 'fs';
writeFileSync('test.txt', 'typescript wrote this');
console.log('written');
```"#,
            // Read it back
            r#"```typescript
import { readFileSync } from 'fs';
const content = readFileSync('test.txt', 'utf-8');
console.log(content);
```"#,
            "File round-trip works.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].stdout.contains("written"));
    assert!(results[1].stdout.contains("typescript wrote this"));
}

#[tokio::test(flavor = "multi_thread")]
async fn typescript_npm_install_and_use() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Turn 1: init and install lodash (via Python subprocess — npm needs shell)
            "```python\nimport subprocess\nsubprocess.run(['npm', 'init', '-y'], capture_output=True)\nresult = subprocess.run(['npm', 'install', 'lodash'], capture_output=True, text=True)\nprint(result.stdout)\nprint(result.stderr)\n```",
            // Turn 2: use lodash from TypeScript
            r#"```typescript
import _ from 'lodash';
const arr = [3, 1, 4, 1, 5, 9];
console.log(`sorted: ${_.sortBy(arr)}`);
```"#,
            "npm package used from TypeScript.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].exit_code, Some(0));
    assert!(
        results[1].stdout.contains("sorted: 1,1,3,4,5,9"),
        "stdout: {:?}",
        results[1].stdout
    );
}
