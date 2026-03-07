//! Integration tests for hick-agent's multi-language code execution.
//!
//! These tests exercise the full ReactLoop with a mock LLM that returns
//! predetermined responses. Real code execution happens (Python, TypeScript,
//! C#) — only the LLM is mocked.
//!
//! Tests that require a language runtime check availability at startup and
//! skip gracefully if the runtime is not installed.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use tokio::sync::{Notify, mpsc};

use hick_agent::{
    Agent, AgentConfig, AgentEvent, Language, LlmClient, Message, ReactLoop, ScriptResult,
};

// ── Mock LLM ────────────────────────────────────────────────────────

/// Deterministic LLM client that returns responses from a pre-programmed sequence.
///
/// When all responses are consumed, returns a plain text message (no code block)
/// which terminates the ReAct loop.
struct ScriptedLlmClient {
    responses: Vec<String>,
    call_count: AtomicUsize,
}

impl ScriptedLlmClient {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            call_count: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl LlmClient for ScriptedLlmClient {
    async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
        let i = self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .responses
            .get(i)
            .cloned()
            .unwrap_or_else(|| "All tasks complete.".to_string()))
    }

    fn provider_name(&self) -> &str {
        "test"
    }

    fn model_name(&self) -> &str {
        "scripted"
    }
}

// ── Test helpers ────────────────────────────────────────────────────

/// Run the agent with scripted LLM responses and collect all events.
async fn run_with_responses(
    responses: Vec<String>,
    project_dir: &Path,
) -> anyhow::Result<Vec<AgentEvent>> {
    let llm = ScriptedLlmClient::new(responses);
    let config = AgentConfig {
        project_dir: project_dir.to_path_buf(),
        ..Default::default()
    };
    let agent = Agent::new(config, Box::new(llm));
    let mut react_loop = ReactLoop::new(agent, project_dir.to_path_buf());
    react_loop.stream_output = false;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let interrupt = Notify::new();

    react_loop.run("test request", &tx, &interrupt).await?;

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    Ok(events)
}

/// Extract all ScriptFinished results from events.
fn script_results(events: &[AgentEvent]) -> Vec<&ScriptResult> {
    events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::ScriptFinished(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// Extract all ScriptStarted languages from events.
fn script_languages(events: &[AgentEvent]) -> Vec<Language> {
    events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::ScriptStarted { language, .. } => Some(*language),
            _ => None,
        })
        .collect()
}

/// Extract the Done message from events.
fn done_message(events: &[AgentEvent]) -> Option<&str> {
    events.iter().find_map(|e| match e {
        AgentEvent::Done(msg) => Some(msg.as_str()),
        _ => None,
    })
}

/// Check if a language runtime is available on this system.
fn runtime_available(language: Language) -> bool {
    match language {
        Language::Python => std::process::Command::new("python3")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
        Language::TypeScript => std::process::Command::new("npx")
            .args(["tsx", "--version"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
        Language::CSharp => std::process::Command::new("dotnet-script")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
    }
}

/// Skip the test if the required runtime is not available.
macro_rules! require_runtime {
    ($lang:expr) => {
        if !runtime_available($lang) {
            eprintln!(
                "SKIPPED: {} runtime not available",
                $lang.display_name()
            );
            return;
        }
    };
}

// ── Python tests ────────────────────────────────────────────────────

#[tokio::test]
async fn python_hello_world() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "Let me print hello.\n\n```python\nprint('hello from python')\n```".into(),
            "The script printed hello from python.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success());
    assert_eq!(results[0].stdout.trim(), "hello from python");
    assert!(done_message(&events).is_some());
}

#[tokio::test]
async fn python_file_io() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            // Turn 1: write a file
            "Writing a file.\n\n```python\nimport os\npath = os.path.join(os.environ['WORKSPACE'], 'test_output.txt')\nwith open(path, 'w') as f:\n    f.write('file content here')\nprint('wrote file')\n```".into(),
            // Turn 2: read it back
            "Reading the file.\n\n```python\nimport os\npath = os.path.join(os.environ['WORKSPACE'], 'test_output.txt')\nwith open(path) as f:\n    content = f.read()\nprint(f'read: {content}')\n```".into(),
            // Final answer
            "File I/O verified successfully.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].success());
    assert!(results[0].stdout.contains("wrote file"));
    assert!(results[1].success());
    assert!(results[1].stdout.contains("read: file content here"));
}

#[tokio::test]
async fn python_subprocess() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "Running a subprocess.\n\n```python\nimport subprocess\nresult = subprocess.run(['echo', 'subprocess works'], capture_output=True, text=True)\nprint(result.stdout.strip())\n```".into(),
            "Subprocess execution verified.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success());
    assert_eq!(results[0].stdout.trim(), "subprocess works");
}

#[tokio::test]
async fn python_script_error() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            // Script with a NameError
            "```python\nprint(undefined_variable)\n```".into(),
            // LLM acknowledges the error
            "There was an error — undefined_variable is not defined.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(!results[0].success());
    assert!(results[0].stderr.contains("NameError"));
    assert_eq!(results[0].exit_code, Some(1));
}

#[tokio::test]
async fn python_workspace_env_var() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "```python\nimport os\nprint(os.environ['WORKSPACE'])\n```".into(),
            "Workspace confirmed.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert!(results[0].success());
    assert_eq!(
        results[0].stdout.trim(),
        dir.path().to_str().unwrap()
    );
}

#[tokio::test]
async fn python_pip_install_and_use() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    // Install a tiny package into a local directory, then import it
    let install_script = format!(
        "```python\nimport subprocess, os\n\
         pkg_dir = os.path.join(os.environ['WORKSPACE'], '.packages')\n\
         result = subprocess.run(\n\
             ['pip', 'install', '--target', pkg_dir, '--quiet', 'six'],\n\
             capture_output=True, text=True\n\
         )\n\
         if result.returncode == 0:\n\
             print('installed')\n\
         else:\n\
             print(f'error: {{result.stderr}}')\n\
         ```"
    );

    let use_script = format!(
        "```python\nimport sys, os\n\
         sys.path.insert(0, os.path.join(os.environ['WORKSPACE'], '.packages'))\n\
         import six\n\
         print(f'six version: {{six.__version__}}')\n\
         ```"
    );

    let events = run_with_responses(
        vec![
            install_script,
            use_script,
            "Package installed and verified.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].success(), "pip install failed: {}", results[0].stderr);
    assert!(results[0].stdout.contains("installed"));
    assert!(results[1].success(), "import failed: {}", results[1].stderr);
    assert!(results[1].stdout.contains("six version:"));
}

#[tokio::test]
async fn multi_turn_python() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            // Turn 1: create a file
            "First, create the data.\n\n```python\nimport os, json\ndata = {'count': 0, 'items': []}\npath = os.path.join(os.environ['WORKSPACE'], 'data.json')\nwith open(path, 'w') as f:\n    json.dump(data, f)\nprint('created data.json')\n```".into(),
            // Turn 2: modify it
            "Now update the data.\n\n```python\nimport os, json\npath = os.path.join(os.environ['WORKSPACE'], 'data.json')\nwith open(path) as f:\n    data = json.load(f)\ndata['count'] = 3\ndata['items'] = ['a', 'b', 'c']\nwith open(path, 'w') as f:\n    json.dump(data, f)\nprint(f'updated: count={data[\"count\"]}')\n```".into(),
            // Turn 3: verify
            "Finally, verify the data.\n\n```python\nimport os, json\npath = os.path.join(os.environ['WORKSPACE'], 'data.json')\nwith open(path) as f:\n    data = json.load(f)\nprint(f'count={data[\"count\"]} items={data[\"items\"]}')\n```".into(),
            // Done
            "Data pipeline verified: created, updated, and read back successfully.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 3);
    assert!(results[0].success());
    assert!(results[0].stdout.contains("created data.json"));
    assert!(results[1].success());
    assert!(results[1].stdout.contains("updated: count=3"));
    assert!(results[2].success());
    assert!(results[2].stdout.contains("count=3"));
    assert!(results[2].stdout.contains("items=['a', 'b', 'c']"));
}

// ── TypeScript tests ────────────────────────────────────────────────

#[tokio::test]
async fn typescript_hello_world() {
    require_runtime!(Language::TypeScript);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "Let me print hello in TypeScript.\n\n```typescript\nconsole.log('hello from typescript')\n```".into(),
            "TypeScript printed hello.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success(), "tsx failed: {}", results[0].stderr);
    assert_eq!(results[0].stdout.trim(), "hello from typescript");

    let langs = script_languages(&events);
    assert_eq!(langs, vec![Language::TypeScript]);
}

#[tokio::test]
async fn typescript_file_io() {
    require_runtime!(Language::TypeScript);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            // Write a file using Node fs
            "```typescript\nimport { writeFileSync } from 'fs';\nimport { join } from 'path';\nconst dir = process.env.WORKSPACE!;\nwriteFileSync(join(dir, 'ts_test.txt'), 'written by typescript');\nconsole.log('wrote file');\n```".into(),
            // Read it back
            "```typescript\nimport { readFileSync } from 'fs';\nimport { join } from 'path';\nconst dir = process.env.WORKSPACE!;\nconst content = readFileSync(join(dir, 'ts_test.txt'), 'utf-8');\nconsole.log(`read: ${content}`);\n```".into(),
            "File I/O verified in TypeScript.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].success(), "write failed: {}", results[0].stderr);
    assert!(results[0].stdout.contains("wrote file"));
    assert!(results[1].success(), "read failed: {}", results[1].stderr);
    assert!(results[1].stdout.contains("read: written by typescript"));
}

#[tokio::test]
async fn typescript_ts_shorthand() {
    require_runtime!(Language::TypeScript);
    let dir = tempfile::tempdir().unwrap();

    // Use ```ts instead of ```typescript
    let events = run_with_responses(
        vec![
            "```ts\nconst x: number = 42;\nconsole.log(`the answer is ${x}`);\n```".into(),
            "Done.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success(), "tsx failed: {}", results[0].stderr);
    assert!(results[0].stdout.contains("the answer is 42"));

    let langs = script_languages(&events);
    assert_eq!(langs, vec![Language::TypeScript]);
}

// ── C# tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn csharp_hello_world() {
    require_runtime!(Language::CSharp);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "Let me print hello in C#.\n\n```csharp\nConsole.WriteLine(\"hello from csharp\");\n```".into(),
            "C# printed hello.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success(), "dotnet-script failed: {}", results[0].stderr);
    assert_eq!(results[0].stdout.trim(), "hello from csharp");

    let langs = script_languages(&events);
    assert_eq!(langs, vec![Language::CSharp]);
}

#[tokio::test]
async fn csharp_file_io() {
    require_runtime!(Language::CSharp);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            // Write
            "```csharp\nvar dir = Environment.GetEnvironmentVariable(\"WORKSPACE\");\nvar path = Path.Combine(dir, \"cs_test.txt\");\nFile.WriteAllText(path, \"written by csharp\");\nConsole.WriteLine(\"wrote file\");\n```".into(),
            // Read
            "```csharp\nvar dir = Environment.GetEnvironmentVariable(\"WORKSPACE\");\nvar path = Path.Combine(dir, \"cs_test.txt\");\nvar content = File.ReadAllText(path);\nConsole.WriteLine($\"read: {content}\");\n```".into(),
            "File I/O verified in C#.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].success(), "write failed: {}", results[0].stderr);
    assert!(results[0].stdout.contains("wrote file"));
    assert!(results[1].success(), "read failed: {}", results[1].stderr);
    assert!(results[1].stdout.contains("read: written by csharp"));
}

#[tokio::test]
async fn csharp_cs_shorthand() {
    require_runtime!(Language::CSharp);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "```cs\nvar x = 42;\nConsole.WriteLine($\"the answer is {x}\");\n```".into(),
            "Done.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success(), "dotnet-script failed: {}", results[0].stderr);
    assert!(results[0].stdout.contains("the answer is 42"));
}

#[tokio::test]
async fn csharp_nuget_package() {
    require_runtime!(Language::CSharp);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "```csharp\n#r \"nuget: Newtonsoft.Json, 13.0.3\"\nusing Newtonsoft.Json;\nvar obj = new { Name = \"test\", Value = 42 };\nvar json = JsonConvert.SerializeObject(obj);\nConsole.WriteLine(json);\n```".into(),
            "NuGet package used successfully.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 1);
    assert!(results[0].success(), "dotnet-script failed: {}", results[0].stderr);
    assert!(results[0].stdout.contains("\"Name\":\"test\""));
    assert!(results[0].stdout.contains("\"Value\":42"));
}

// ── Cross-language tests ────────────────────────────────────────────

#[tokio::test]
async fn mixed_language_turns() {
    require_runtime!(Language::Python);
    require_runtime!(Language::TypeScript);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            // Turn 1: Python writes a JSON file
            "Writing data with Python.\n\n```python\nimport os, json\npath = os.path.join(os.environ['WORKSPACE'], 'shared.json')\nwith open(path, 'w') as f:\n    json.dump({'language': 'python', 'value': 123}, f)\nprint('wrote shared.json')\n```".into(),
            // Turn 2: TypeScript reads it
            "Reading with TypeScript.\n\n```typescript\nimport { readFileSync } from 'fs';\nimport { join } from 'path';\nconst data = JSON.parse(readFileSync(join(process.env.WORKSPACE!, 'shared.json'), 'utf-8'));\nconsole.log(`language=${data.language} value=${data.value}`);\n```".into(),
            "Cross-language data sharing verified.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert_eq!(results.len(), 2);
    assert!(results[0].success());
    assert!(results[0].stdout.contains("wrote shared.json"));
    assert!(results[1].success(), "tsx failed: {}", results[1].stderr);
    assert!(results[1].stdout.contains("language=python"));
    assert!(results[1].stdout.contains("value=123"));

    let langs = script_languages(&events);
    assert_eq!(langs, vec![Language::Python, Language::TypeScript]);
}

// ── Event structure tests ───────────────────────────────────────────

#[tokio::test]
async fn event_sequence_single_turn() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    let events = run_with_responses(
        vec![
            "```python\nprint('hi')\n```".into(),
            "Done.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    // Verify expected event sequence
    let event_types: Vec<&str> = events
        .iter()
        .map(|e| match e {
            AgentEvent::Thinking => "Thinking",
            AgentEvent::Token(_) => "Token",
            AgentEvent::ResponseComplete(_) => "ResponseComplete",
            AgentEvent::ScriptStarted { .. } => "ScriptStarted",
            AgentEvent::ScriptFinished(_) => "ScriptFinished",
            AgentEvent::Done(_) => "Done",
            AgentEvent::Info(_) => "Info",
            AgentEvent::Interrupted => "Interrupted",
            AgentEvent::Error(_) => "Error",
        })
        .collect();

    // Turn 1: Thinking → Token → ResponseComplete → ScriptStarted → ScriptFinished
    // Turn 2: Thinking → Token → ResponseComplete → Done
    assert!(event_types.contains(&"Thinking"));
    assert!(event_types.contains(&"ScriptStarted"));
    assert!(event_types.contains(&"ScriptFinished"));
    assert!(event_types.contains(&"Done"));

    // ScriptStarted should come before ScriptFinished
    let start_pos = event_types.iter().position(|e| *e == "ScriptStarted").unwrap();
    let finish_pos = event_types.iter().position(|e| *e == "ScriptFinished").unwrap();
    assert!(start_pos < finish_pos);
}

#[tokio::test]
async fn final_answer_no_code_block() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    // LLM responds with plain text immediately — no code blocks
    let events = run_with_responses(
        vec!["This is a direct answer with no code to run.".into()],
        dir.path(),
    )
    .await
    .unwrap();

    let results = script_results(&events);
    assert!(results.is_empty(), "no scripts should have been executed");
    assert_eq!(
        done_message(&events).unwrap(),
        "This is a direct answer with no code to run."
    );
}

#[tokio::test]
async fn script_file_written_to_tmp_dir() {
    require_runtime!(Language::Python);
    let dir = tempfile::tempdir().unwrap();

    run_with_responses(
        vec![
            "```python\nprint('check tmp')\n```".into(),
            "Done.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    // Verify the script file was written
    let script_path = dir.path().join(".hick").join("tmp").join("script_0.py");
    assert!(script_path.exists(), "script file should exist at {:?}", script_path);
    let content = std::fs::read_to_string(&script_path).unwrap();
    assert_eq!(content, "print('check tmp')");
}

#[tokio::test]
async fn typescript_script_file_extension() {
    require_runtime!(Language::TypeScript);
    let dir = tempfile::tempdir().unwrap();

    run_with_responses(
        vec![
            "```typescript\nconsole.log('ext check')\n```".into(),
            "Done.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let script_path = dir.path().join(".hick").join("tmp").join("script_0.ts");
    assert!(script_path.exists(), "TS script should have .ts extension at {:?}", script_path);
}

#[tokio::test]
async fn csharp_script_file_extension() {
    require_runtime!(Language::CSharp);
    let dir = tempfile::tempdir().unwrap();

    run_with_responses(
        vec![
            "```csharp\nConsole.WriteLine(\"ext check\");\n```".into(),
            "Done.".into(),
        ],
        dir.path(),
    )
    .await
    .unwrap();

    let script_path = dir.path().join(".hick").join("tmp").join("script_0.csx");
    assert!(script_path.exists(), "C# script should have .csx extension at {:?}", script_path);
}
