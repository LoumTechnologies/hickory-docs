//! Strongly-typed tool definitions for the ReAct loop.

use serde::{Deserialize, Serialize};

/// A tool invocation from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolCall {
    RunPipeline {
        #[serde(default)]
        params: Vec<String>,
    },
    Exec {
        command: String,
        #[serde(default)]
        container: Option<String>,
        #[serde(default)]
        image: Option<String>,
        #[serde(default)]
        timeout: Option<u64>,
    },
    Query {
        selector: String,
        #[serde(default)]
        file: Option<String>,
    },
    Structure {
        file: String,
    },
    Provenance {
        file: String,
        start_line: usize,
        end_line: usize,
    },
    ComputeEdits {
        file: String,
        selector: String,
        new_content: String,
    },
    ApplyEdits {
        /// JSON array of edits: [{"file": "...", "start": 0, "end": 0, "content": "..."}]
        edits_json: String,
    },
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    ListDir {
        path: String,
    },
    /// Create a backing .hick file when a generated file was edited outside the pipeline.
    /// Use when provenance is lost; creates a .hick that wraps the current content.
    CreateBacking {
        output_path: String,
        backing_path: String,
    },
    /// Signal that the agent is done (final answer).
    Done {
        #[serde(default)]
        message: Option<String>,
    },
}

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub output: String,
}

impl ToolResult {
    pub fn ok(tool: &str, output: impl Into<String>) -> Self {
        Self {
            tool: tool.to_string(),
            success: true,
            output: output.into(),
        }
    }

    pub fn err(tool: &str, output: impl Into<String>) -> Self {
        Self {
            tool: tool.to_string(),
            success: false,
            output: output.into(),
        }
    }
}

/// Parse a tool call from LLM response text.
/// Looks for JSON objects with a "tool" field.
pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut results = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find('{') {
        let rest = &remaining[start..];
        if let Some(end) = find_matching_brace(rest) {
            let json_str = &rest[..=end];
            if let Ok(call) = serde_json::from_str::<ToolCall>(json_str) {
                results.push(call);
            }
            remaining = &rest[end + 1..];
        } else {
            break;
        }
    }

    results
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

impl ToolCall {
    /// Whether this tool only performs filesystem I/O and can run in parallel
    /// without needing agent state.
    pub fn is_filesystem_only(&self) -> bool {
        matches!(self, ToolCall::ReadFile { .. } | ToolCall::ListDir { .. })
    }
}

/// Check if the response indicates the agent is done (no tool calls, or explicit Done).
pub fn extract_final_message(text: &str) -> Option<String> {
    let calls = parse_tool_calls(text);
    if calls.is_empty() {
        return Some(text.trim().to_string());
    }
    for call in &calls {
        if let ToolCall::Done { message } = call {
            return Some(message.clone().unwrap_or_default());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exec_tool() {
        let json = r#"{"tool": "exec", "command": "cargo test"}"#;
        let calls = parse_tool_calls(json);
        assert_eq!(calls.len(), 1);
        if let ToolCall::Exec { command, .. } = &calls[0] {
            assert_eq!(command, "cargo test");
        } else {
            panic!("expected Exec");
        }
    }

    #[test]
    fn parse_multiple_tools() {
        let text = r#"Before {"tool": "exec", "command": "ls"} middle {"tool": "query", "selector": "fn main"} after"#;
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 2);
    }
}
