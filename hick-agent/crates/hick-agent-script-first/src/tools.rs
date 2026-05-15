//! Code block extraction and execution for the script-first ReAct loop.
//!
//! The LLM writes code in fenced markdown blocks. This module extracts those
//! blocks and maps fence tags to a runner (`sh`, `python3`, `node`, …).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Execution backend for an extracted fenced code block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Shell,
    Python,
    CSharp,
    TypeScript,
    JavaScript,
}

impl Language {
    /// File extension used when writing the script to disk.
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Shell => "sh",
            Language::Python => "py",
            Language::CSharp => "csx",
            Language::TypeScript => "ts",
            Language::JavaScript => "js",
        }
    }

    /// Shell command to execute a script file at the given path inside the container.
    pub fn run_command(&self, script_path: &str) -> String {
        match self {
            Language::Shell => format!("sh {script_path}"),
            Language::Python => format!("python3 {script_path}"),
            Language::CSharp => format!("dotnet-script {script_path}"),
            Language::TypeScript => format!("tsx {script_path}"),
            Language::JavaScript => format!("node {script_path}"),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Shell => write!(f, "shell"),
            Language::Python => write!(f, "python"),
            Language::CSharp => write!(f, "csharp"),
            Language::TypeScript => write!(f, "typescript"),
            Language::JavaScript => write!(f, "javascript"),
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

/// Fence tag → Language mapping. Longer tags must appear before shorter ones
/// that are prefixes (e.g. `typescript` before `ts`).
const FENCE_TAGS: &[(&str, Language)] = &[
    ("bash", Language::Shell),
    ("shell", Language::Shell),
    ("posix-sh", Language::Shell),
    ("sh", Language::Shell),
    ("python", Language::Python),
    ("csharp", Language::CSharp),
    ("cs", Language::CSharp),
    ("typescript", Language::TypeScript),
    ("ts", Language::TypeScript),
    ("javascript", Language::JavaScript),
    ("js", Language::JavaScript),
];

/// Extract code blocks from an LLM response.
///
/// Finds fenced code blocks tagged with a supported language:
/// ````markdown
/// ```python
/// print("hello")
/// ```
/// ````
///
/// Supports: `bash`/`sh`/`shell`/`posix-sh`, `python`, `csharp`/`cs`,
/// `typescript`/`ts`, `javascript`/`js`.
/// Returns blocks in document order.
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

/// Backward-compatible wrapper: extract only Python blocks as raw strings.
pub fn extract_python_blocks(text: &str) -> Vec<String> {
    extract_code_blocks(text)
        .into_iter()
        .filter(|b| b.language == Language::Python)
        .map(|b| b.code)
        .collect()
}

/// Find the next fenced code block with a supported language tag.
/// Returns `(content_start_index, Language)` where `content_start_index` is
/// the byte offset into `text` where the code content begins (after the fence line).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_python_block() {
        let text = "Some text\n```python\nprint('hello')\n```\nMore text";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Python);
        assert_eq!(blocks[0].code, "print('hello')");
    }

    #[test]
    fn extract_csharp_block() {
        let text = "Here:\n```csharp\nConsole.WriteLine(\"hi\");\n```\n";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::CSharp);
        assert_eq!(blocks[0].code, "Console.WriteLine(\"hi\");");
    }

    #[test]
    fn extract_cs_alias() {
        let text = "```cs\nvar x = 1;\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::CSharp);
    }

    #[test]
    fn extract_typescript_block() {
        let text = "```typescript\nconst x: number = 42;\nconsole.log(x);\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::TypeScript);
        assert!(blocks[0].code.contains("const x: number = 42;"));
    }

    #[test]
    fn extract_ts_alias() {
        let text = "```ts\nconsole.log('hi')\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::TypeScript);
    }

    #[test]
    fn extract_multiple_languages() {
        let text = "Py:\n```python\nprint(1)\n```\nC#:\n```csharp\nConsole.Write(2);\n```\nTS:\n```typescript\nconsole.log(3)\n```\n";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].language, Language::Python);
        assert_eq!(blocks[1].language, Language::CSharp);
        assert_eq!(blocks[2].language, Language::TypeScript);
    }

    #[test]
    fn no_blocks() {
        let text = "Just plain text with no code blocks.";
        let blocks = extract_code_blocks(text);
        assert!(blocks.is_empty());
    }

    #[test]
    fn unsupported_language_ignored() {
        let text = "```rust\nfn main() {}\n```\n```python\nprint('hi')\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Python);
        assert_eq!(blocks[0].code, "print('hi')");
    }

    #[test]
    fn multiline_block() {
        let text = "```python\nimport os\nfor f in os.listdir('.'):\n    print(f)\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].code.contains("import os"));
        assert!(blocks[0].code.contains("print(f)"));
    }

    #[test]
    fn python_blocks_filters_to_python_only() {
        let text = "```python\nprint(1)\n```\n```csharp\nConsole.Write(2);\n```\n";
        let blocks = extract_python_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "print(1)");
    }

    #[test]
    fn language_extensions() {
        assert_eq!(Language::Shell.extension(), "sh");
        assert_eq!(Language::Python.extension(), "py");
        assert_eq!(Language::CSharp.extension(), "csx");
        assert_eq!(Language::TypeScript.extension(), "ts");
        assert_eq!(Language::JavaScript.extension(), "js");
    }

    #[test]
    fn language_run_commands() {
        assert_eq!(Language::Shell.run_command("/tmp/s.sh"), "sh /tmp/s.sh");
        assert_eq!(Language::Python.run_command("/tmp/s.py"), "python3 /tmp/s.py");
        assert_eq!(Language::CSharp.run_command("/tmp/s.csx"), "dotnet-script /tmp/s.csx");
        assert_eq!(Language::TypeScript.run_command("/tmp/s.ts"), "tsx /tmp/s.ts");
        assert_eq!(Language::JavaScript.run_command("/tmp/s.js"), "node /tmp/s.js");
    }

    #[test]
    fn extract_shell_block_bash() {
        let text = "```bash\nls -la\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Shell);
        assert_eq!(blocks[0].code.trim(), "ls -la");
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
