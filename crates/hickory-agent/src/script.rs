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

/// Resource limits applied to every agent script.
///
/// Both are load-bearing, and both were missing. An agent's script is
/// written by a model, so "it will finish" and "it will print a reasonable
/// amount" are assumptions, not facts: a `cargo test` that waits on a prompt
/// runs forever, and one `grep -r` over a large repo returns megabytes that
/// would be pasted verbatim into the next request.
#[derive(Debug, Clone, Copy)]
pub struct ScriptLimits {
    /// Wall-clock limit for one script.
    pub timeout: std::time::Duration,
    /// Maximum bytes of stdout (and, separately, stderr) kept as the
    /// observation.
    pub max_output_bytes: usize,
}

impl Default for ScriptLimits {
    fn default() -> Self {
        Self {
            // Long enough for a real build or test suite; short enough that
            // a wedged script does not consume the whole run.
            timeout: std::time::Duration::from_secs(600),
            // ~16k of text is far more than any useful observation and far
            // less than a context-destroying dump.
            max_output_bytes: 16 * 1024,
        }
    }
}

/// Keep the head and tail of `text`, dropping the middle.
///
/// Both ends matter and for different reasons: the head shows what the
/// script was doing, the tail holds the error and the exit. Truncating to
/// the head alone (the obvious implementation) throws away the part the
/// model actually needs. The marker states the byte count dropped, so the
/// model can tell a truncated observation from a complete one and narrow
/// its next command instead of assuming it saw everything.
fn clamp_output(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let half = max_bytes / 2;
    let head_end = floor_char_boundary(text, half);
    let tail_start = ceil_char_boundary(text, text.len() - half);
    let dropped = tail_start - head_end;
    format!(
        "{}\n\n… {dropped} bytes omitted; re-run narrowed if you need the middle …\n\n{}",
        &text[..head_end],
        &text[tail_start..]
    )
}

/// Largest char boundary at or below `i` (`str::floor_char_boundary` is
/// still unstable).
fn floor_char_boundary(s: &str, i: usize) -> usize {
    let mut i = i.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Smallest char boundary at or above `i`.
fn ceil_char_boundary(s: &str, i: usize) -> usize {
    let mut i = i.min(s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

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
    limits: ScriptLimits,
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

    // Run it under `timeout` INSIDE the container. Racing an outer future
    // against the call would return control while the runaway process kept
    // holding the container's CPU and files; `timeout` actually kills it,
    // and `-k` follows with SIGKILL for anything that ignores SIGTERM.
    // Exit 124 is the documented "timed out" code.
    let secs = limits.timeout.as_secs().max(1);
    let command = format!(
        "timeout -k 5 {secs} {}",
        block.language.run_command(&script_path)
    );
    // A backstop for the executor itself hanging (a lost connection to a
    // remote microVM never reaches `timeout` in the guest). Generous, so it
    // only ever fires when the in-container limit could not.
    let outer = limits.timeout + std::time::Duration::from_secs(30);
    let timed_out = tokio::time::timeout(outer, executor.execute(container, &command))
        .await
        .is_err();

    if timed_out {
        // Nothing was recorded, so say plainly what happened rather than
        // returning an empty observation the model would read as success.
        return Ok(ScriptResult {
            stdout: String::new(),
            stderr: format!(
                "the executor did not return within {}s; the script was abandoned",
                outer.as_secs()
            ),
            exit_code: None,
        });
    }

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

    // `timeout` reports 124 when it killed the script. Left as a bare exit
    // code the model reads it as an ordinary failure and starts debugging
    // the wrong thing, so name it.
    if exit_code == Some(124) {
        let note = format!("\n[killed after {secs}s by the agent script timeout]");
        stderr.push_str(&note);
    }

    Ok(ScriptResult {
        stdout: clamp_output(&entry.output, limits.max_output_bytes),
        stderr: clamp_output(&stderr, limits.max_output_bytes),
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
        let result = run_script(&ex, AGENT_CONTAINER, &block, 0, ScriptLimits::default())
            .await
            .unwrap();
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
        let result = run_script(&ex, AGENT_CONTAINER, &block, 0, ScriptLimits::default())
            .await
            .unwrap();
        assert_eq!(result.exit_code, Some(3));
        assert!(result.stderr.contains("oops"));
    }
}

#[cfg(test)]
mod limit_tests {
    use super::*;
    use hickory_executor::LocalExecutor;

    #[test]
    fn short_output_is_untouched() {
        assert_eq!(clamp_output("hello", 1024), "hello");
    }

    #[test]
    fn truncation_keeps_both_ends_and_says_how_much_it_dropped() {
        // The tail is where the error and the exit line live; a head-only
        // truncation would hide exactly what the model needs.
        let text = format!("{}MIDDLE{}", "A".repeat(5000), "Z".repeat(5000));
        let out = clamp_output(&text, 1000);
        assert!(out.starts_with("AAAA"), "head lost");
        assert!(out.ends_with("ZZZZ"), "tail lost");
        assert!(!out.contains("MIDDLE"), "middle should be dropped");
        assert!(out.contains("bytes omitted"), "truncation must be visible");
        assert!(out.len() < text.len());
    }

    #[test]
    fn truncation_never_splits_a_character() {
        // Byte-slicing a multi-byte character panics; observations are full
        // of them (test output, diffs, non-ASCII prose).
        let text = "é".repeat(4000);
        let out = clamp_output(&text, 1001);
        assert!(out.contains("bytes omitted"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_script_that_never_finishes_is_killed_and_labelled() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started(AGENT_CONTAINER, "host").await.unwrap();
        let block = CodeBlock {
            language: Language::Shell,
            code: "sleep 30".into(),
        };
        let limits = ScriptLimits {
            timeout: std::time::Duration::from_secs(1),
            ..ScriptLimits::default()
        };
        let started = std::time::Instant::now();
        let result = run_script(&ex, AGENT_CONTAINER, &block, 0, limits)
            .await
            .unwrap();
        assert!(
            started.elapsed() < std::time::Duration::from_secs(20),
            "the timeout did not fire; the script ran to completion"
        );
        assert_eq!(
            result.exit_code,
            Some(124),
            "expected the timeout exit code"
        );
        assert!(
            result.stderr.contains("agent script timeout"),
            "a bare 124 reads as an ordinary failure: {}",
            result.stderr
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_flood_of_output_is_clamped_before_it_reaches_the_model() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started(AGENT_CONTAINER, "host").await.unwrap();
        let block = CodeBlock {
            language: Language::Shell,
            // ~2 MB, the shape of one `grep -r` over a large repo.
            code:
                "for i in $(seq 1 40000); do echo 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'; done"
                    .into(),
        };
        let limits = ScriptLimits {
            max_output_bytes: 4096,
            ..ScriptLimits::default()
        };
        let result = run_script(&ex, AGENT_CONTAINER, &block, 0, limits)
            .await
            .unwrap();
        assert!(
            result.stdout.len() < 5000,
            "an unclamped dump reached the observation: {} bytes",
            result.stdout.len()
        );
        assert!(result.stdout.contains("bytes omitted"));
    }
}
