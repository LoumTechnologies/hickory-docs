//! Script-first intent parser — [`MatchIntent`] implementation for the engine.
//!
//! [`ScriptBlockIntentParser`] inspects LLM responses for the `<hick:next>` protocol:
//!
//! - `<hick:next>done</hick:next>` → [`TaskCompleteSignal`]
//! - `<hick:next>code</hick:next>` + code blocks → [`ScriptBlockPayload::Code`]
//! - `<hick:next>…</hick:next>` with wrong structure → [`ScriptBlockPayload::Invalid`]
//! - No `<hick:next>` tag → `None` (engine streams response as text)

use std::any::Any;

use hick_agent_sdk::{MatchIntent, Message, TaskCompleteSignal};

use crate::tools::{CodeBlock, extract_code_blocks};

// ── System prompt ─────────────────────────────────────────────────────────────

/// Default system prompt for the script-first agent strategy.
pub const SYSTEM_PROMPT: &str = r#"You are an AI coding agent. You work by writing and executing code to accomplish tasks.

## Response format (REQUIRED)

Every response MUST begin with one of these two XML tags on the very first line:

  <hick:next>code</hick:next>
  <hick:next>done</hick:next>

Rules:
- `<hick:next>code</hick:next>` — you are going to run code. You MUST include exactly one fenced code block in this response.
- `<hick:next>done</hick:next>` — you are finished. Write your final answer as plain text; do NOT include any code blocks.

The system validates these tags. If you write `<hick:next>code</hick:next>` without a code block, or `<hick:next>done</hick:next>` with a code block, you will be asked to correct your response.

Write ONE fenced code block per response, then STOP and wait for the real execution output. Do NOT predict or guess what the output will be — the system runs your code and shows you the actual stdout/stderr.

## Executable fenced blocks (runner map)

Pick a fence whose runner exists in your environment (see the runtime probe):

- Shell: ```bash, ```sh, ```shell, ```posix-sh — saved as POSIX `sh`; use for `apk add`, `ls`, pipelines, POSIX scripts, installers, compilers, etc.
- Python: ```python — `python3`
- JavaScript (Node): ```javascript / ```js — `node`
- C#: ```csharp / ```cs — `dotnet-script`
- TypeScript: ```typescript / ```ts — `tsx`

You may install language runtimes or packages yourself inside the sandbox (e.g. `apk add …`, `npm install -g …`, pip, curling binaries) whenever you need them. No separate permission step — go ahead and fix the environment if something is missing.

## Available languages

### Shell (recommended for discovery and installs)
- Prefer ```bash when you want simple commands: `find`, `ls`, `apk add python3 py3-pip`, `wget`, etc.
- Scripts run as POSIX `sh` (shebang lines are ignored). Call another interpreter explicitly (`python3 …`, `bash …`) once installed.

### Python
- Read/write files with `open()`, `os`, `pathlib`
- Run shell commands with `subprocess.run()`
- Install packages: `subprocess.run(['pip', 'install', ...])` (after installing pip if needed)

### JavaScript (Node)
- Runs with `node`. Use `.mjs` style or plain CommonJS depending on what's available after install.

### C# (dotnet-script)
- Write C# script files (.csx) — no Main() or class wrapper needed
- Install NuGet packages: `#r "nuget: Newtonsoft.Json, 13.0.3"`
- Requires `dotnet` / `dotnet-script` when you choose this fence.

### TypeScript (tsx)
- Write TypeScript; requires `tsx` (install globally or via npm/npx depending on probe output).

## Key concepts

1. **Print your results.** Your script's stdout is what you see as observation. Use print/Console.WriteLine/console.log in typed languages, or echo/printf for shell fences, to report findings.
"#;

/// Appended to the system prompt when code runs in the WASM sandbox (normal mode).
pub const SANDBOX_EXECUTION_CONTEXT: &str = r#"

## Execution environment (sandbox)

Your fenced code blocks run in an isolated Linux/WASM container. The project is mounted at **`/workspace`** and shell commands start from there (`cd /workspace`, etc.).

In every script, use **`/workspace`**, **`WORKSPACE`** (set for Python), or paths **relative to `/workspace`**. **Do not** use host absolute paths such as `/home/...`, `/Users/...`, or `~/...` — those paths do not exist inside the container.

The prompt includes a short **automatic runtime probe** from this container listing which binaries are installed; trust that output over assumptions from training data. If something is missing, install it yourself in-code (apk, npm, etc.) unless the error clearly blocks that.
"#;

/// Probe command run once per session inside the sandbox to guide the LLM.
pub const SANDBOX_RUNTIME_PROBE: &str = r#"echo '### Sandbox runtime probe'; echo '---'; uname -a 2>&1 || true; echo '--- interpreters / tools ---'; for c in sh ash bash python3 python dotnet dotnet-script node npm npx tsx bun deno rustc cargo ruby lua apk apt-get wget curl git; do printf '%s: ' "$c"; command -v "$c" >/dev/null 2>&1 && command -v "$c" || echo '(not found)'; done"#;

// ── Response protocol ─────────────────────────────────────────────────────────

/// Structural correctness check for a `<hick:next>` response.
///
/// Returns `None` when the response is valid, or a short error description
/// when it violates the declared intent.
pub fn validate_response(response: &str) -> Option<String> {
    let t = response.trim_start();
    let is_code = t.starts_with("<hick:next>code</hick:next>");
    let is_done = t.starts_with("<hick:next>done</hick:next>");
    let has_code = !extract_code_blocks(response).is_empty();
    match (is_code, is_done, has_code) {
        (true, _, true) => None,
        (_, true, false) => None,
        (true, _, false) => Some(
            "declared <hick:next>code</hick:next> but did not include a code block".into(),
        ),
        (_, true, true) => Some(
            "declared <hick:next>done</hick:next> but included a code block".into(),
        ),
        _ => Some(
            "response did not start with <hick:next>code</hick:next> or <hick:next>done</hick:next>".into(),
        ),
    }
}

/// Build the re-prompt correction message to inject into history.
pub fn correction_message(reason: &str) -> String {
    format!(
        "Your last response was invalid: {reason}.\n\n\
         Please try again. Start your response with exactly one of:\n\
         - `<hick:next>code</hick:next>` followed immediately by a fenced code block \
           (shell: bash/sh/shell — or python, javascript/js, csharp/cs, typescript/ts)\n\
         - `<hick:next>done</hick:next>` followed by your final plain-text answer with no code blocks"
    )
}

// ── ScriptBlockPayload ────────────────────────────────────────────────────────

/// Payload produced by [`ScriptBlockIntentParser`] for code-execution responses.
#[derive(Debug)]
pub enum ScriptBlockPayload {
    /// Valid `<hick:next>code</hick:next>` response with extracted block(s).
    Code {
        /// Code blocks to execute (first block only, per the one-block-per-turn rule).
        blocks: Vec<CodeBlock>,
        /// Reasoning text before the first code block, if any.
        thought: Option<String>,
    },
    /// `<hick:next>` was present but the response violated the protocol.
    ///
    /// The executor should inject [`correction_message`] into history so the
    /// LLM can self-correct on the next turn.
    Invalid {
        /// Short description of the violation.
        reason: String,
    },
}

// ── ScriptBlockIntentParser ───────────────────────────────────────────────────

/// [`MatchIntent`] impl for the script-first agent strategy.
///
/// Matches any response that begins with a `<hick:next>` tag. Returns:
/// - [`TaskCompleteSignal`] for valid `<hick:next>done</hick:next>` responses.
/// - [`ScriptBlockPayload::Code`] for valid `<hick:next>code</hick:next>` responses.
/// - [`ScriptBlockPayload::Invalid`] when the protocol is violated.
/// - `None` when no `<hick:next>` tag is present (engine treats as free-form text).
pub struct ScriptBlockIntentParser;

impl MatchIntent for ScriptBlockIntentParser {
    fn name(&self) -> &'static str {
        "script_first"
    }

    fn parse(&self, message: &Message) -> Option<Box<dyn Any + Send>> {
        let response = &message.content;
        let trimmed = response.trim_start();

        if !trimmed.starts_with("<hick:next>") {
            return None;
        }

        if trimmed.starts_with("<hick:next>done</hick:next>") {
            if !extract_code_blocks(response).is_empty() {
                return Some(Box::new(ScriptBlockPayload::Invalid {
                    reason: "declared <hick:next>done</hick:next> but included a code block"
                        .into(),
                }));
            }
            let summary = extract_thought(response).unwrap_or_default();
            return Some(Box::new(TaskCompleteSignal { summary }));
        }

        if trimmed.starts_with("<hick:next>code</hick:next>") {
            let all_blocks = extract_code_blocks(response);
            if all_blocks.is_empty() {
                return Some(Box::new(ScriptBlockPayload::Invalid {
                    reason: "declared <hick:next>code</hick:next> but did not include a code block"
                        .into(),
                }));
            }
            let thought = extract_thought(response);
            let first_block = all_blocks.into_iter().next().unwrap();
            return Some(Box::new(ScriptBlockPayload::Code {
                blocks: vec![first_block],
                thought,
            }));
        }

        Some(Box::new(ScriptBlockPayload::Invalid {
            reason: "response did not start with <hick:next>code</hick:next> or <hick:next>done</hick:next>".into(),
        }))
    }
}

// ── extract_thought ───────────────────────────────────────────────────────────

/// Extract the "thought" text from a response — everything before the first
/// code block, with the `<hick:next>` intent tag stripped.
pub fn extract_thought(response: &str) -> Option<String> {
    let stripped = response
        .trim_start()
        .trim_start_matches("<hick:next>code</hick:next>")
        .trim_start_matches("<hick:next>done</hick:next>")
        .trim_start();
    let text = if let Some(pos) = stripped.find("```") {
        stripped[..pos].trim().to_string()
    } else {
        stripped.trim().to_string()
    };
    if text.is_empty() { None } else { Some(text) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_agent_sdk::Role;

    fn msg(content: &str) -> Message {
        Message { role: Role::Assistant, content: content.to_string() }
    }

    #[test]
    fn extract_thought_from_response() {
        let response =
            "Let me check the file structure first.\n\n```python\nimport os\nprint(os.listdir('.'))\n```";
        let thought = extract_thought(response);
        assert_eq!(thought.unwrap(), "Let me check the file structure first.");
    }

    #[test]
    fn no_thought_when_starts_with_code() {
        let response = "```python\nprint('hello')\n```";
        assert!(extract_thought(response).is_none());
    }

    #[test]
    fn extract_thought_no_code_blocks() {
        let response = "This is the final answer with no code.";
        assert_eq!(
            extract_thought(response).unwrap(),
            "This is the final answer with no code."
        );
    }

    #[test]
    fn parser_returns_none_for_no_hick_next_tag() {
        let parser = ScriptBlockIntentParser;
        let result = parser.parse(&msg("Just a plain response with no protocol tag."));
        assert!(result.is_none());
    }

    #[test]
    fn parser_returns_task_complete_for_done() {
        let parser = ScriptBlockIntentParser;
        let result = parser.parse(&msg("<hick:next>done</hick:next>\nAll finished!"));
        let payload = result.expect("expected Some");
        assert!(payload.downcast::<TaskCompleteSignal>().is_ok());
    }

    #[test]
    fn parser_returns_code_payload_for_valid_code() {
        let parser = ScriptBlockIntentParser;
        let result = parser.parse(&msg(
            "<hick:next>code</hick:next>\n```python\nprint('hi')\n```",
        ));
        let payload = result.expect("expected Some");
        let script = payload
            .downcast::<ScriptBlockPayload>()
            .expect("expected ScriptBlockPayload");
        assert!(matches!(*script, ScriptBlockPayload::Code { .. }));
    }

    #[test]
    fn parser_returns_invalid_for_code_without_block() {
        let parser = ScriptBlockIntentParser;
        let result = parser.parse(&msg("<hick:next>code</hick:next>\nNo block here."));
        let payload = result.expect("expected Some");
        let script = payload
            .downcast::<ScriptBlockPayload>()
            .expect("expected ScriptBlockPayload");
        assert!(matches!(*script, ScriptBlockPayload::Invalid { .. }));
    }

    #[test]
    fn parser_returns_invalid_for_done_with_block() {
        let parser = ScriptBlockIntentParser;
        let result = parser.parse(&msg(
            "<hick:next>done</hick:next>\nDone!\n```python\nprint('oops')\n```",
        ));
        let payload = result.expect("expected Some");
        let script = payload
            .downcast::<ScriptBlockPayload>()
            .expect("expected ScriptBlockPayload");
        assert!(matches!(*script, ScriptBlockPayload::Invalid { .. }));
    }

    #[test]
    fn validate_response_accepts_valid_code() {
        let result = validate_response(
            "<hick:next>code</hick:next>\n```python\nprint('hi')\n```",
        );
        assert!(result.is_none());
    }

    #[test]
    fn validate_response_accepts_valid_done() {
        let result = validate_response("<hick:next>done</hick:next>\nAll done.");
        assert!(result.is_none());
    }

    #[test]
    fn validate_response_rejects_code_without_block() {
        let result = validate_response("<hick:next>code</hick:next>\nNo block.");
        assert!(result.is_some());
    }
}
