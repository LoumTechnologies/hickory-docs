//! Integration tests: C# code execution inside WASM sandbox.
//!
//! These tests verify that the agent can write and execute C# scripts
//! (.csx files via dotnet-script) inside a WASM Linux container.
//!
//! Run with: cargo test -p hick-agent --features integration-test --test execution_csharp -- --test-threads=1

#![cfg(feature = "integration-test")]

mod common;

use common::{create_executor, finished_results, run_agent};

#[tokio::test(flavor = "multi_thread")]
async fn csharp_hello_world() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```csharp\nConsole.WriteLine(\"hello from csharp\");\n```",
            "The output was hello from csharp.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(
        results[0].stdout.contains("hello from csharp"),
        "stdout: {:?}",
        results[0].stdout
    );
    assert_eq!(results[0].exit_code, Some(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn csharp_cs_alias() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            "```cs\nvar x = 40 + 2;\nConsole.WriteLine($\"answer is {x}\");\n```",
            "Got the answer.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].stdout.contains("answer is 42"));
}

#[tokio::test(flavor = "multi_thread")]
async fn csharp_file_io() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Write a file
            "```csharp\nSystem.IO.File.WriteAllText(\"test.txt\", \"csharp wrote this\");\nConsole.WriteLine(\"written\");\n```",
            // Read it back
            "```csharp\nvar content = System.IO.File.ReadAllText(\"test.txt\");\nConsole.WriteLine(content);\n```",
            "File round-trip works.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].stdout.contains("written"));
    assert!(results[1].stdout.contains("csharp wrote this"));
}

#[tokio::test(flavor = "multi_thread")]
async fn csharp_nuget_package() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor(workdir.path());

    let events = run_agent(
        workdir.path(),
        executor,
        &image,
        vec![
            // Use a NuGet package via #r directive (dotnet-script feature)
            r#"```csharp
#r "nuget: Newtonsoft.Json, 13.0.3"

using Newtonsoft.Json;

var obj = new { Name = "test", Value = 42 };
var json = JsonConvert.SerializeObject(obj);
Console.WriteLine(json);
```"#,
            "JSON serialization via NuGet works.",
        ],
    )
    .await;

    let results = finished_results(&events);
    assert_eq!(results.len(), 1);
    assert!(
        results[0].stdout.contains("\"Name\":\"test\"")
            || results[0].stdout.contains("\"Name\": \"test\""),
        "stdout: {:?}",
        results[0].stdout
    );
    assert_eq!(results[0].exit_code, Some(0));
}
