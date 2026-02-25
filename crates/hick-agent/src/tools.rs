//! Python script extraction and execution for the script-first ReAct loop.
//!
//! The LLM writes Python scripts in markdown code blocks. This module
//! extracts those code blocks and provides types for script execution results.

use serde::{Deserialize, Serialize};

/// Result of executing a Python script.
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

/// Extract Python code blocks from an LLM response.
///
/// Looks for fenced code blocks tagged as `python`:
/// ````markdown
/// ```python
/// print("hello")
/// ```
/// ````
///
/// Returns the content of each block (without the fence markers).
pub fn extract_python_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut remaining = text;

    while let Some(start) = find_python_fence(remaining) {
        let after_fence = &remaining[start..];
        // Find the closing ```
        if let Some(end) = after_fence.find("\n```") {
            let code = &after_fence[..end];
            if !code.trim().is_empty() {
                blocks.push(code.to_string());
            }
            remaining = &after_fence[end + 4..];
        } else {
            // Unclosed block — take the rest
            let code = after_fence;
            if !code.trim().is_empty() {
                blocks.push(code.to_string());
            }
            break;
        }
    }

    blocks
}

/// Find the start of a ```python code block, returning the index of the first
/// character after the opening fence line (i.e. the code content start).
fn find_python_fence(text: &str) -> Option<usize> {
    // Match ```python or ``` python (with optional whitespace)
    let patterns = [
        "```python\n",
        "```python\r\n",
        "``` python\n",
        "``` python\r\n",
    ];
    for pattern in patterns {
        if let Some(pos) = text.find(pattern) {
            return Some(pos + pattern.len());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_block() {
        let text = "Some text\n```python\nprint('hello')\n```\nMore text";
        let blocks = extract_python_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "print('hello')");
    }

    #[test]
    fn extract_multiple_blocks() {
        let text = "First:\n```python\nprint(1)\n```\nSecond:\n```python\nprint(2)\n```\n";
        let blocks = extract_python_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], "print(1)");
        assert_eq!(blocks[1], "print(2)");
    }

    #[test]
    fn no_blocks() {
        let text = "Just plain text with no code blocks.";
        let blocks = extract_python_blocks(text);
        assert!(blocks.is_empty());
    }

    #[test]
    fn non_python_blocks_ignored() {
        let text = "```rust\nfn main() {}\n```\n```python\nprint('hi')\n```";
        let blocks = extract_python_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "print('hi')");
    }

    #[test]
    fn multiline_block() {
        let text = "```python\nimport os\nfor f in os.listdir('.'):\n    print(f)\n```";
        let blocks = extract_python_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("import os"));
        assert!(blocks[0].contains("print(f)"));
    }

    #[test]
    fn script_result_observation_format() {
        let result = ScriptResult {
            stdout: "hello world".into(),
            stderr: String::new(),
            exit_code: Some(0),
        };
        let obs = result.as_observation();
        assert!(obs.contains("hello world"));
        assert!(obs.contains("success"));
    }

    #[test]
    fn script_result_error_format() {
        let result = ScriptResult {
            stdout: String::new(),
            stderr: "NameError: name 'x' is not defined".into(),
            exit_code: Some(1),
        };
        let obs = result.as_observation();
        assert!(obs.contains("NameError"));
        assert!(obs.contains("error"));
    }
}
