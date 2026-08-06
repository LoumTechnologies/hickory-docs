//! The script-first response protocol.
//!
//! Every LLM response must begin with `<hick:next>code</hick:next>` (run the
//! fenced code block that follows) or `<hick:next>done</hick:next>` (final
//! answer, no code). Malformed responses produce [`Turn::Invalid`] and the
//! loop re-prompts with [`correction_message`].

use crate::script::{CodeBlock, extract_code_blocks};

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

## Executable fenced blocks

- Shell: ```bash, ```sh, ```shell — saved as a script and run with POSIX `sh`; use for `ls`, pipelines, installers, compilers, etc.
- Python: ```python — run with `python3`.

Scripts run in a workspace directory; state accumulates across scripts in files (each script is a fresh process). Print your results — your script's stdout is what you see as the observation."#;

/// One parsed turn of the protocol.
#[derive(Debug)]
pub enum Turn {
    /// Valid `<hick:next>code</hick:next>` response.
    Code {
        /// Reasoning text before the first code block, if any.
        thought: Option<String>,
        /// The (single) code block to execute.
        block: CodeBlock,
    },
    /// Valid `<hick:next>done</hick:next>` response with the final answer.
    Done { summary: String },
    /// The response violated the protocol; re-prompt with
    /// [`correction_message`].
    Invalid { reason: String },
}

/// Parse an LLM response against the `<hick:next>` protocol.
pub fn parse_response(response: &str) -> Turn {
    let trimmed = response.trim_start();

    if trimmed.starts_with("<hick:next>done</hick:next>") {
        if !extract_code_blocks(response).is_empty() {
            return Turn::Invalid {
                reason: "declared <hick:next>done</hick:next> but included a code block".into(),
            };
        }
        return Turn::Done {
            summary: extract_thought(response).unwrap_or_default(),
        };
    }

    if trimmed.starts_with("<hick:next>code</hick:next>") {
        let mut blocks = extract_code_blocks(response);
        if blocks.is_empty() {
            return Turn::Invalid {
                reason: "declared <hick:next>code</hick:next> but did not include a code block"
                    .into(),
            };
        }
        let block = blocks.remove(0);
        return Turn::Code {
            thought: extract_thought(response),
            block,
        };
    }

    Turn::Invalid {
        reason:
            "response did not start with <hick:next>code</hick:next> or <hick:next>done</hick:next>"
                .into(),
    }
}

/// Build the re-prompt correction message to inject into history.
pub fn correction_message(reason: &str) -> String {
    format!(
        "Your last response was invalid: {reason}.\n\n\
         Please try again. Start your response with exactly one of:\n\
         - `<hick:next>code</hick:next>` followed immediately by a fenced code block \
           (shell: bash/sh/shell — or python)\n\
         - `<hick:next>done</hick:next>` followed by your final plain-text answer with no code blocks"
    )
}

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
    use crate::script::Language;

    #[test]
    fn valid_code_turn() {
        let turn =
            parse_response("<hick:next>code</hick:next>\nLet me look.\n```python\nprint(1)\n```");
        match turn {
            Turn::Code { thought, block } => {
                assert_eq!(thought.as_deref(), Some("Let me look."));
                assert_eq!(block.language, Language::Python);
            }
            other => panic!("expected Code, got {other:?}"),
        }
    }

    #[test]
    fn valid_done_turn() {
        let turn = parse_response("<hick:next>done</hick:next>\nAll finished!");
        match turn {
            Turn::Done { summary } => assert_eq!(summary, "All finished!"),
            other => panic!("expected Done, got {other:?}"),
        }
    }

    #[test]
    fn code_without_block_is_invalid() {
        assert!(matches!(
            parse_response("<hick:next>code</hick:next>\nNo block."),
            Turn::Invalid { .. }
        ));
    }

    #[test]
    fn done_with_block_is_invalid() {
        assert!(matches!(
            parse_response("<hick:next>done</hick:next>\n```python\nprint(1)\n```"),
            Turn::Invalid { .. }
        ));
    }

    #[test]
    fn missing_tag_is_invalid() {
        assert!(matches!(
            parse_response("Just prose."),
            Turn::Invalid { .. }
        ));
    }
}
