//! Code-block extraction and script execution through the [`Executor`] trait.
//!
//! The LLM writes code in fenced markdown blocks; this module extracts those
//! blocks, writes them into the agent's container workspace, runs them, and
//! reads back stdout/stderr/exit-code from the executor's transcript.

use std::fmt;

use anyhow::Result;
use hickory_executor::{Executor, TranscriptEvent};
use serde::{Deserialize, Serialize};

/// Execution backend for an extracted fenced code block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Shell,
    Python,
}

impl Language {
    /// File extension used when writing the script to the workspace.
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Shell => "sh",
            Language::Python => "py",
        }
    }

    /// Language identifier recorded on `<hick:action lang="...">` elements.
    /// Matches what `hick run` uses to replay the action.
    pub fn hick_lang(&self) -> &'static str {
        match self {
            Language::Shell => "sh",
            Language::Python => "python",
        }
    }

    /// Shell command to execute a script file at `script_path` in the
    /// container.
    pub fn run_command(&self, script_path: &str) -> String {
        match self {
            Language::Shell => format!("sh {script_path}"),
            Language::Python => format!("python3 {script_path}"),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Shell => write!(f, "shell"),
            Language::Python => write!(f, "python"),
        }
    }
}

/// A code block extracted from an LLM response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub language: Language,
    pub code: String,
}

/// Result of executing a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl ScriptResult {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Format as an observation string for the LLM.
    pub fn as_observation(&self) -> String {
        let mut parts = Vec::new();
        if !self.stdout.is_empty() {
            parts.push(format!("stdout:\n{}", self.stdout));
        }
        if !self.stderr.is_empty() {
            parts.push(format!("stderr:\n{}", self.stderr));
        }
        let status = match self.exit_code {
            Some(0) => "exit: 0 (success)".to_string(),
            Some(code) => format!("exit: {code} (error)"),
            None => "exit: unknown (process may have been killed)".to_string(),
        };
        parts.push(status);
        parts.join("\n\n")
    }
}

/// Fence tag → Language mapping. Longer tags before shorter prefixes.
const FENCE_TAGS: &[(&str, Language)] = &[
    ("bash", Language::Shell),
    ("shell", Language::Shell),
    ("posix-sh", Language::Shell),
    ("sh", Language::Shell),
    ("python", Language::Python),
    ("py", Language::Python),
];

/// Extract fenced code blocks tagged with a supported language
/// (`bash`/`sh`/`shell`/`posix-sh` → shell, `python`/`py` → python).
/// Returns blocks in document order; unsupported fences are ignored.
pub fn extract_code_blocks(text: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut remaining = text;

    while let Some((content_start, language)) = find_next_fence(remaining) {
        let after_fence = &remaining[content_start..];
        if let Some(end) = after_fence.find("\n```") {
            let code = &after_fence[..end];
            if !code.trim().is_empty() {
                blocks.push(CodeBlock {
                    language,
                    code: code.to_string(),
                });
            }
            remaining = &after_fence[end + 4..];
        } else {
            let code = after_fence;
            if !code.trim().is_empty() {
                blocks.push(CodeBlock {
                    language,
                    code: code.to_string(),
                });
            }
            break;
        }
    }

    blocks
}

/// Find the next fenced code block with a supported language tag. Returns
/// `(content_start_index, Language)` where the index is the byte offset in
/// `text` where the code content begins.
fn find_next_fence(text: &str) -> Option<(usize, Language)> {
    let mut best: Option<(usize, usize, Language)> = None;

    for &(tag, language) in FENCE_TAGS {
        for sep in ["", " "] {
            for eol in ["\n", "\r\n"] {
                let pattern = format!("```{sep}{tag}{eol}");
                if let Some(pos) = text.find(&pattern) {
                    let content_start = pos + pattern.len();
                    if best.is_none() || pos < best.unwrap().0 {
                        best = Some((pos, content_start, language));
                    }
                }
            }
        }
    }

    best.map(|(_, content_start, lang)| (content_start, lang))
}

// ---------------------------------------------------------------------------
// Execution through the Executor trait
// ---------------------------------------------------------------------------

/// The container name the agent runs in.
pub const AGENT_CONTAINER: &str = "agent";

/// Directory (relative to the container workdir) that scripts are written to.
const SCRIPT_DIR: &str = ".hickory-agent";

/// Run one extracted code block through `executor` inside `container`.
///
/// `container` is normally one the DOCUMENT declares, so the agent's scripts
/// and the document's own `hick:exec` cells share a filesystem and a toolchain
/// — an agent that writes a file is writing it where the cells will find it.
/// [`AGENT_CONTAINER`] is the fallback for documents that declare none.
///
/// The script is written to `.hickory-agent/action-<n>.<ext>` in the
/// container workspace and executed with the language's runner. The result
/// (stdout, stderr, exit code) is recovered from the executor's transcript,
/// so a non-zero exit is an observation, not an error.
pub async fn run_script(
    executor: &dyn Executor,
    container: &str,
    block: &CodeBlock,
    action_index: usize,
) -> Result<ScriptResult> {
    let script_path = format!(
        "{SCRIPT_DIR}/action-{action_index}.{}",
        block.language.extension()
    );

    // Write the script via stdin so no quoting of the code is needed.
    executor
        .execute_with_stdin(
            container,
            &format!("mkdir -p {SCRIPT_DIR} && cat > {script_path}"),
            &block.code,
        )
        .await?;

    // Run it. A non-zero exit surfaces as Err from the executor, but the
    // transcript entry is recorded first — recover the result from there.
    let command = block.language.run_command(&script_path);
    let _ = executor.execute(container, &command).await;

    let transcripts = executor.transcripts();
    let entry = transcripts
        .get(container)
        .and_then(|entries| entries.last())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("executor recorded no transcript for the script run"))?;

    let mut stderr = String::new();
    let mut exit_code = None;
    for event in &entry.events {
        match event {
            TranscriptEvent::Err { data, .. } => stderr.push_str(data),
            TranscriptEvent::Exit { code, .. } => exit_code = Some(*code),
            _ => {}
        }
    }

    Ok(ScriptResult {
        stdout: entry.output,
        stderr,
        exit_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_executor::LocalExecutor;

    #[test]
    fn extract_single_python_block() {
        let text = "Some text\n```python\nprint('hello')\n```\nMore text";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Python);
        assert_eq!(blocks[0].code, "print('hello')");
    }

    #[test]
    fn extract_shell_aliases() {
        for tag in ["bash", "sh", "shell", "posix-sh"] {
            let text = format!("```{tag}\nls -la\n```");
            let blocks = extract_code_blocks(&text);
            assert_eq!(blocks.len(), 1, "tag {tag}");
            assert_eq!(blocks[0].language, Language::Shell);
        }
    }

    #[test]
    fn unsupported_language_ignored() {
        let text = "```rust\nfn main() {}\n```\n```python\nprint('hi')\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Python);
    }

    #[test]
    fn no_blocks() {
        assert!(extract_code_blocks("Just plain text.").is_empty());
    }

    #[test]
    fn observation_format() {
        let ok = ScriptResult {
            stdout: "hello".into(),
            stderr: String::new(),
            exit_code: Some(0),
        };
        assert!(ok.as_observation().contains("success"));
        let bad = ScriptResult {
            stdout: String::new(),
            stderr: "boom".into(),
            exit_code: Some(1),
        };
        let obs = bad.as_observation();
        assert!(obs.contains("boom"));
        assert!(obs.contains("error"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run_script_captures_stdout_and_exit() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started(AGENT_CONTAINER, "host").await.unwrap();
        let block = CodeBlock {
            language: Language::Shell,
            code: "echo out; echo err >&2".into(),
        };
        let result = run_script(&ex, AGENT_CONTAINER, &block, 0).await.unwrap();
        assert_eq!(result.stdout, "out\n");
        assert_eq!(result.stderr, "err\n");
        assert_eq!(result.exit_code, Some(0));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run_script_nonzero_exit_is_observation() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started(AGENT_CONTAINER, "host").await.unwrap();
        let block = CodeBlock {
            language: Language::Shell,
            code: "echo oops >&2; exit 3".into(),
        };
        let result = run_script(&ex, AGENT_CONTAINER, &block, 0).await.unwrap();
        assert_eq!(result.exit_code, Some(3));
        assert!(result.stderr.contains("oops"));
    }
}
