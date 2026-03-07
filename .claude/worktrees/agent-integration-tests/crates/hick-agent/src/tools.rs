//! Code block extraction and execution types for the script-first ReAct loop.
//!
//! The LLM writes scripts in markdown code blocks (Python, TypeScript, or C#).
//! This module extracts those code blocks and provides types for execution results.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Languages supported for code block execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Python,
    TypeScript,
    CSharp,
}

impl Language {
    /// File extension for scripts in this language.
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Python => "py",
            Language::TypeScript => "ts",
            Language::CSharp => "csx",
        }
    }

    /// Human-readable name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Python => "Python",
            Language::TypeScript => "TypeScript",
            Language::CSharp => "C#",
        }
    }

    /// Command and prefix arguments to execute a script file.
    ///
    /// Returns `(command, prefix_args)`. The script path is appended after prefix_args.
    pub fn runtime_command(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            Language::Python => ("python3", &[]),
            Language::TypeScript => ("npx", &["tsx"]),
            Language::CSharp => ("dotnet-script", &[]),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// A code block extracted from an LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Extract code blocks for all supported languages from an LLM response.
///
/// Looks for fenced code blocks tagged with a supported language:
/// ````markdown
/// ```python
/// print("hello")
/// ```
///
/// ```typescript
/// console.log("hello")
/// ```
///
/// ```csharp
/// Console.WriteLine("hello");
/// ```
/// ````
///
/// Returns blocks in the order they appear, with language and code content.
pub fn extract_code_blocks(text: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut remaining = text;

    while let Some((lang, content_start)) = find_code_fence(remaining) {
        let after_fence = &remaining[content_start..];
        if let Some(end) = after_fence.find("\n```") {
            let code = &after_fence[..end];
            if !code.trim().is_empty() {
                blocks.push(CodeBlock {
                    language: lang,
                    code: code.to_string(),
                });
            }
            remaining = &after_fence[end + 4..];
        } else {
            // Unclosed block — take the rest
            let code = after_fence;
            if !code.trim().is_empty() {
                blocks.push(CodeBlock {
                    language: lang,
                    code: code.to_string(),
                });
            }
            break;
        }
    }

    blocks
}

/// Extract only Python code blocks (convenience wrapper for backward compatibility).
pub fn extract_python_blocks(text: &str) -> Vec<String> {
    extract_code_blocks(text)
        .into_iter()
        .filter(|b| b.language == Language::Python)
        .map(|b| b.code)
        .collect()
}

/// Find the earliest code fence for any supported language.
///
/// Returns `(language, content_start_offset)` where content_start_offset is
/// the byte offset in `text` where the code content begins (after the fence line).
fn find_code_fence(text: &str) -> Option<(Language, usize)> {
    let mut best: Option<(Language, usize, usize)> = None; // (lang, fence_pos, content_start)

    let languages: &[(Language, &[&str])] = &[
        (Language::Python, &["python"]),
        (Language::TypeScript, &["typescript", "ts"]),
        (Language::CSharp, &["csharp", "cs"]),
    ];

    for &(lang, tags) in languages {
        for tag in tags {
            for suffix in &["\n", "\r\n"] {
                for pattern in &[
                    format!("```{tag}{suffix}"),
                    format!("``` {tag}{suffix}"),
                ] {
                    if let Some(pos) = text.find(pattern.as_str()) {
                        let content_start = pos + pattern.len();
                        if best.map_or(true, |(_, bp, _)| pos < bp) {
                            best = Some((lang, pos, content_start));
                        }
                    }
                }
            }
        }
    }

    best.map(|(lang, _, cs)| (lang, cs))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_python_blocks (backward compatibility) ──────────────

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

    // ── extract_code_blocks (multi-language) ────────────────────────

    #[test]
    fn extract_typescript_block() {
        let text = "Here's the code:\n```typescript\nconsole.log('hello')\n```\n";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::TypeScript);
        assert_eq!(blocks[0].code, "console.log('hello')");
    }

    #[test]
    fn extract_ts_shorthand() {
        let text = "```ts\nconsole.log('hi')\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::TypeScript);
    }

    #[test]
    fn extract_csharp_block() {
        let text = "```csharp\nConsole.WriteLine(\"hello\");\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::CSharp);
        assert_eq!(blocks[0].code, "Console.WriteLine(\"hello\");");
    }

    #[test]
    fn extract_cs_shorthand() {
        let text = "```cs\nvar x = 42;\nConsole.WriteLine(x);\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::CSharp);
    }

    #[test]
    fn mixed_language_blocks_preserved_in_order() {
        let text = "A:\n```python\nprint(1)\n```\nB:\n```typescript\nconsole.log(2)\n```\nC:\n```csharp\nConsole.WriteLine(3);\n```\n";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].language, Language::Python);
        assert_eq!(blocks[1].language, Language::TypeScript);
        assert_eq!(blocks[2].language, Language::CSharp);
    }

    #[test]
    fn unsupported_language_blocks_skipped() {
        let text = "```rust\nfn main() {}\n```\n```bash\necho hi\n```\n```python\nprint('ok')\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Python);
    }

    #[test]
    fn python_blocks_filtered_from_mixed() {
        let text = "```typescript\nconsole.log(1)\n```\n```python\nprint(2)\n```";
        let py_blocks = extract_python_blocks(text);
        assert_eq!(py_blocks.len(), 1);
        assert_eq!(py_blocks[0], "print(2)");
    }

    #[test]
    fn language_display() {
        assert_eq!(Language::Python.to_string(), "Python");
        assert_eq!(Language::TypeScript.to_string(), "TypeScript");
        assert_eq!(Language::CSharp.to_string(), "C#");
    }

    #[test]
    fn language_extensions() {
        assert_eq!(Language::Python.extension(), "py");
        assert_eq!(Language::TypeScript.extension(), "ts");
        assert_eq!(Language::CSharp.extension(), "csx");
    }

    #[test]
    fn code_fence_with_space() {
        let text = "``` python\nprint('spaced')\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, Language::Python);
        assert_eq!(blocks[0].code, "print('spaced')");
    }
}
