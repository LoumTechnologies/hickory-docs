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

/// Addendum to [`SYSTEM_PROMPT`] enabling the document edit tool set. Only
/// appended when the run has a primary document (`--doc`).
pub const TOOLS_SYSTEM_PROMPT: &str = r#"

## Document tools (REQUIRED for document work)

This session has a primary hick document. Besides `code` and `done`, you may
respond with:

  <hick:next>tool</hick:next>

followed by exactly ONE tool invocation:

  <hick:tool name="TOOL">
  <hick:arg name="ARG">value</hick:arg>
  <hick:input>
  multi-line payload (replacement text) goes here, raw
  </hick:input>
  </hick:tool>

The system executes the tool and returns a <hick:tool-result> observation.

Tools:
- read_doc — the document source, hashline-rendered: every line is prefixed
  `hhhh|` where hhhh is a 4-hex content hash of that line. Args: doc
  (optional — an UPSTREAM document by name or path; omit for the primary).
  The result lists the upstream documents available to this session.
- read_output — a woven output file, hashline-rendered. Args: path (the
  output path); with_lineage (optional, "true") adds per-range lineage
  annotations showing which output lines are editable and where they come
  from in the document.
- edit_output — edit CODE through the output; the edit is mapped back to the
  document byte-exactly via lineage. Args: path; run (a contiguous line run,
  `firsthash..lasthash`, or one hash for a single line); or after (a single
  line hash, or `^` for the top of the file) to INSERT below that line;
  occurrence (optional 1-based index when the anchor is ambiguous). The
  replacement text goes in <hick:input> (omit it to delete the run).
- edit_doc — same edit shape (run/after/occurrence + <hick:input>) applied
  directly to the DOCUMENT source, using read_doc hashes. Args: doc
  (optional — edit an UPSTREAM document instead; the primary is re-woven
  afterwards, and if that re-weave fails the upstream edit is rolled back).
- verify — execute the document for real: runs every exec block, evaluates
  every expectation, writes the output files. Args: none.

Doctrine — follow this order of operations:
1. Read BOTH surfaces first: read_doc and read_output (with_lineage) before
   editing anything.
2. Code changes go through the OUTPUT: use edit_output so the document is
   updated byte-exactly through lineage.
3. Structural and prose work goes through the DOCUMENT: use edit_doc for
   headings, prose, copy blocks, pipeline structure, and anything lineage
   cannot map.
4. When edit_output refuses (synthetic range, duplicated paste), the result
   names the document location to edit — follow that pointer with edit_doc.
   The refusal is routing, not failure.
5. Fix a disagreement WHERE IT IS RECORDED, not where you found it. These
   documents form a chain: a requirement pastes fragments from a domain
   model, which pastes decisions from a meeting note. If the requirement
   contradicts a decision, the decision is the thing to change (or the
   requirement is wrong) — read upstream with read_doc doc=..., and edit
   upstream with edit_doc doc=.... Restating an upstream fact locally
   produces two copies that can disagree, which is the exact failure the
   chain exists to prevent. Never paste an upstream fragment's text inline;
   reference it.
6. Run verify before <hick:next>done</hick:next>. Scripts (<hick:next>code)
   remain available for everything else (exploring the repo, running other
   commands).

Payloads are raw text (no escaping): everything between <hick:input> and the
next literal </hick:input> is taken verbatim — hick tags inside a payload,
balanced or not, are fine. The only thing a payload cannot contain is the
literal string </hick:input> itself."#;

/// One tool invocation parsed from a `<hick:next>tool</hick:next>` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolInvocation {
    /// Tool name (`read_doc`, `read_output`, `edit_output`, `edit_doc`,
    /// `verify`).
    pub name: String,
    /// `<hick:arg name="...">value</hick:arg>` children, in order.
    pub args: Vec<(String, String)>,
    /// The `<hick:input>` payload (multi-line, raw), if present.
    pub input: Option<String>,
    /// The verbatim `<hick:tool>...</hick:tool>` XML, for session logging.
    pub raw_xml: String,
}

impl ToolInvocation {
    /// First value of the named argument, if present.
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

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
    /// Valid `<hick:next>tool</hick:next>` response with one tool
    /// invocation.
    Tool {
        /// Reasoning text before the tool element, if any.
        thought: Option<String>,
        /// The parsed tool invocation.
        invocation: ToolInvocation,
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

    if trimmed.starts_with("<hick:next>tool</hick:next>") {
        return match parse_tool_invocation(response) {
            Ok(invocation) => Turn::Tool {
                thought: extract_tool_thought(response),
                invocation,
            },
            Err(reason) => Turn::Invalid { reason },
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

/// Parse the (single) `<hick:tool>` element out of a
/// `<hick:next>tool</hick:next>` response.
///
/// The tool XML uses the historic session vocabulary — `hick:tool` with
/// `hick:arg` children and an optional multi-line `hick:input` payload — so
/// the logged session stays parseable by `hick_lang::parse_session` (unknown
/// tags are gracefully skipped there). Parsing reuses the hick parser (no
/// escaping: payloads are raw text) by wrapping the element in a doc root.
pub fn parse_tool_invocation(response: &str) -> Result<ToolInvocation, String> {
    let start = response.find("<hick:tool").ok_or_else(|| {
        "declared <hick:next>tool</hick:next> but included no <hick:tool> element".to_string()
    })?;
    let close = "</hick:tool>";
    let end = response
        .rfind(close)
        .ok_or_else(|| "the <hick:tool> element is not closed with </hick:tool>".to_string())?;
    if end < start {
        return Err("malformed <hick:tool> element".into());
    }
    let raw_xml = response[start..end + close.len()].to_string();

    let wrapped = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{raw_xml}\n</hick:doc>\n"
    );
    let doc = hick_lang::parse(&wrapped)
        .map_err(|e| format!("could not parse the <hick:tool> element: {e}"))?;

    let tool = doc
        .nodes
        .iter()
        .find_map(|n| match n {
            hick_lang::HickNode::Tag(t) if t.name == "tool" => Some(t),
            _ => None,
        })
        .ok_or_else(|| "no <hick:tool> element found".to_string())?;

    let name = tool
        .get_attribute("name")
        .ok_or_else(|| "the <hick:tool> element has no name attribute".to_string())?
        .to_string();

    let mut args = Vec::new();
    let mut input_count = 0usize;
    for child in &tool.children {
        if let hick_lang::HickNode::Tag(t) = child {
            match t.name.as_str() {
                "arg" => {
                    let arg_name = t
                        .get_attribute("name")
                        .ok_or_else(|| "a <hick:arg> element has no name attribute".to_string())?
                        .to_string();
                    args.push((arg_name, t.text_content().trim().to_string()));
                }
                "input" => input_count += 1,
                other => {
                    return Err(format!(
                        "unexpected <hick:{other}> inside <hick:tool> (only hick:arg and hick:input are allowed)"
                    ));
                }
            }
        }
    }
    if input_count > 1 {
        return Err("a <hick:tool> element may carry at most one <hick:input> payload".into());
    }

    // Extract the payload RAW from the verbatim XML (not via text_content),
    // so balanced hick: tags inside it survive byte-for-byte.
    let input = if input_count == 1 {
        let open_at = raw_xml
            .find("<hick:input")
            .ok_or_else(|| "malformed <hick:input> payload".to_string())?;
        let open_end = raw_xml[open_at..]
            .find('>')
            .map(|i| open_at + i + 1)
            .ok_or_else(|| "malformed <hick:input> payload".to_string())?;
        let close_at = raw_xml
            .rfind("</hick:input>")
            .ok_or_else(|| "the <hick:input> payload is not closed".to_string())?;
        if close_at < open_end {
            return Err("malformed <hick:input> payload".into());
        }
        Some(trim_payload(&raw_xml[open_end..close_at]))
    } else {
        None
    };

    Ok(ToolInvocation {
        name,
        args,
        input,
        raw_xml,
    })
}

/// Strip exactly one leading and one trailing newline from an
/// `<hick:input>` payload (the newlines separating the payload from its
/// tags), preserving all interior whitespace.
fn trim_payload(text: &str) -> String {
    let text = text
        .strip_prefix("\r\n")
        .or_else(|| text.strip_prefix('\n'))
        .unwrap_or(text);
    let text = text.strip_suffix('\n').unwrap_or(text);
    let text = text.strip_suffix('\r').unwrap_or(text);
    text.to_string()
}

/// Thought text of a tool turn: everything between the intent tag and the
/// `<hick:tool>` element.
fn extract_tool_thought(response: &str) -> Option<String> {
    let stripped = response
        .trim_start()
        .trim_start_matches("<hick:next>tool</hick:next>")
        .trim_start();
    let text = match stripped.find("<hick:tool") {
        Some(pos) => stripped[..pos].trim(),
        None => stripped.trim(),
    };
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Build the re-prompt correction message to inject into history.
pub fn correction_message(reason: &str) -> String {
    format!(
        "Your last response was invalid: {reason}.\n\n\
         Please try again. Start your response with exactly one of:\n\
         - `<hick:next>code</hick:next>` followed immediately by a fenced code block \
           (shell: bash/sh/shell — or python)\n\
         - `<hick:next>tool</hick:next>` followed by exactly one <hick:tool name=\"...\"> \
           element (document sessions only)\n\
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
    fn valid_tool_turn_with_args_and_input() {
        let response = "<hick:next>tool</hick:next>\nFixing the greeting.\n\
<hick:tool name=\"edit_output\">\n\
<hick:arg name=\"path\">src/gen.rs</hick:arg>\n\
<hick:arg name=\"run\">a1b2..c3d4</hick:arg>\n\
<hick:input>\n    println!(\"hi\");\n</hick:input>\n\
</hick:tool>";
        match parse_response(response) {
            Turn::Tool {
                thought,
                invocation,
            } => {
                assert_eq!(thought.as_deref(), Some("Fixing the greeting."));
                assert_eq!(invocation.name, "edit_output");
                assert_eq!(invocation.arg("path"), Some("src/gen.rs"));
                assert_eq!(invocation.arg("run"), Some("a1b2..c3d4"));
                assert_eq!(invocation.input.as_deref(), Some("    println!(\"hi\");"));
                assert!(invocation.raw_xml.starts_with("<hick:tool"));
                assert!(invocation.raw_xml.ends_with("</hick:tool>"));
            }
            other => panic!("expected Tool, got {other:?}"),
        }
    }

    #[test]
    fn tool_without_element_is_invalid() {
        assert!(matches!(
            parse_response("<hick:next>tool</hick:next>\nNo element."),
            Turn::Invalid { .. }
        ));
    }

    #[test]
    fn tool_without_name_is_invalid() {
        assert!(matches!(
            parse_response("<hick:next>tool</hick:next>\n<hick:tool></hick:tool>"),
            Turn::Invalid { .. }
        ));
    }

    #[test]
    fn tool_input_keeps_balanced_hick_tags_raw() {
        let response = "<hick:next>tool</hick:next>\n<hick:tool name=\"edit_doc\">\n\
<hick:arg name=\"run\">aaaa..bbbb</hick:arg>\n\
<hick:input>\n<hick:copy id=\"x\">new content\n</hick:copy>\n</hick:input>\n</hick:tool>";
        match parse_response(response) {
            Turn::Tool { invocation, .. } => {
                assert_eq!(
                    invocation.input.as_deref(),
                    Some("<hick:copy id=\"x\">new content\n</hick:copy>")
                );
            }
            other => panic!("expected Tool, got {other:?}"),
        }
    }

    #[test]
    fn tool_input_with_unbalanced_hick_tag_is_preserved_raw() {
        // Payloads are verbatim: an unbalanced hick fragment (e.g. replacing
        // just a copy-open line) survives byte-for-byte.
        let response = "<hick:next>tool</hick:next>\n<hick:tool name=\"edit_doc\">\n\
<hick:arg name=\"run\">aaaa</hick:arg>\n\
<hick:input>\n<hick:copy id=\"x\">no close\n</hick:input>\n</hick:tool>";
        match parse_response(response) {
            Turn::Tool { invocation, .. } => {
                assert_eq!(
                    invocation.input.as_deref(),
                    Some("<hick:copy id=\"x\">no close")
                );
            }
            other => panic!("expected Tool, got {other:?}"),
        }
    }

    #[test]
    fn tool_input_preserves_interior_blank_lines() {
        let response = "<hick:next>tool</hick:next>\n<hick:tool name=\"edit_doc\">\n\
<hick:arg name=\"run\">ffff</hick:arg>\n\
<hick:input>\nline one\n\nline three\n</hick:input>\n</hick:tool>";
        match parse_response(response) {
            Turn::Tool { invocation, .. } => {
                assert_eq!(invocation.input.as_deref(), Some("line one\n\nline three"));
            }
            other => panic!("expected Tool, got {other:?}"),
        }
    }

    #[test]
    fn missing_tag_is_invalid() {
        assert!(matches!(
            parse_response("Just prose."),
            Turn::Invalid { .. }
        ));
    }
}
