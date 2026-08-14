//! Project-level overrides for which child language server to spawn.
//!
//! `hick-lsp` ships a default command per language (`pyright-langserver` for
//! Python, `rust-analyzer` for Rust, …). That default is wrong whenever the
//! repository has already decided otherwise — a Python project pinned to
//! `pylsp` in `.vscode/settings.json` wants *its* server inside `hick:file`
//! blocks too, or the diagnostics in the document disagree with the
//! diagnostics in the generated file, which is the one thing this whole tool
//! exists to prevent.
//!
//! So a repository may carry a `.hick-lsp.json` naming a command per
//! language. `hick init` writes it from whatever the repo's editor
//! configuration already says (see `editor_lsp.rs` in `hickory-cli`), and it
//! is a plain checked-in file a person can edit by hand afterwards.
//!
//! ```json
//! { "servers": { "python": { "command": ["pylsp"] } } }
//! ```
//!
//! Loading is process-global and first-call-wins: one `hick-lsp` process
//! serves one workspace, so the first document opened decides which
//! `.hick-lsp.json` applies. A missing, unreadable, or malformed file leaves
//! the built-in defaults in place — an editor integration must degrade, never
//! fail (docs/guarantees/editor-intelligence/lsp-channel-degrades-never-errors.md).

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

/// The file a project puts at its root to override child server commands.
pub const CONFIG_FILE: &str = ".hick-lsp.json";

static OVERRIDES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
/// The project directory, remembered when the first document is opened.
///
/// Discovery needs it to prefer a server the repository pins over one the
/// machine happens to have.
static ROOT: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Where the project is, for anything that has to look inside it.
pub fn project_root() -> &'static std::path::Path {
    ROOT.get()
        .map(|p| p.as_path())
        .unwrap_or_else(|| Path::new("."))
}

/// Load overrides from the nearest `.hick-lsp.json` at or above `doc_path`.
///
/// Idempotent and first-call-wins; later calls are no-ops.
pub fn load_from_document(doc_path: &Path) {
    OVERRIDES.get_or_init(|| find_and_parse(doc_path).unwrap_or_default());
    let _ = ROOT.set(
        config_dir(doc_path)
            .or_else(|| doc_path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from(".")),
    );
}

/// The directory holding the nearest `.hick-lsp.json`, or the nearest `.git`.
///
/// Either marks the project root well enough for discovery; the document's
/// own directory is the fallback.
fn config_dir(doc_path: &Path) -> Option<std::path::PathBuf> {
    let mut dir = if doc_path.is_dir() {
        Some(doc_path)
    } else {
        doc_path.parent()
    };
    while let Some(current) = dir {
        if current.join(CONFIG_FILE).is_file() || current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        dir = current.parent();
    }
    None
}

/// The overriding command for `language_id`, if the project named one.
pub fn command_for(language_id: &str) -> Option<Vec<String>> {
    OVERRIDES.get()?.get(language_id).cloned()
}

/// Walk up from `doc_path` looking for a readable, parseable config.
fn find_and_parse(doc_path: &Path) -> Option<HashMap<String, Vec<String>>> {
    let mut dir = if doc_path.is_dir() {
        Some(doc_path)
    } else {
        doc_path.parent()
    };
    while let Some(current) = dir {
        let candidate = current.join(CONFIG_FILE);
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate).ok()?;
            let parsed = parse(&text);
            if parsed.is_none() {
                // Stderr is the editor's LSP log; a bad file is worth saying
                // once rather than silently ignoring, because the symptom
                // otherwise is "my server setting did nothing".
                tracing::warn!(
                    path = %candidate.display(),
                    "could not read server overrides; using built-in defaults"
                );
            }
            return parsed;
        }
        dir = current.parent();
    }
    None
}

/// Parse the `servers` table into language id -> command.
///
/// `command` may be an array of words or a single string; anything else for a
/// given language is skipped rather than failing the whole file.
pub fn parse(text: &str) -> Option<HashMap<String, Vec<String>>> {
    let root: serde_json::Value = serde_json::from_str(text).ok()?;
    let servers = root.get("servers")?.as_object()?;
    let mut out = HashMap::new();
    for (language, entry) in servers {
        let Some(command) = entry.get("command") else {
            continue;
        };
        let words: Vec<String> = match command {
            serde_json::Value::Array(items) => items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            serde_json::Value::String(s) => s.split_whitespace().map(str::to_string).collect(),
            _ => continue,
        };
        if !words.is_empty() {
            out.insert(language.clone(), words);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_and_string_commands() {
        let parsed = parse(
            r#"{"servers": {
                 "python": {"command": ["pylsp"]},
                 "rust":   {"command": "rust-analyzer --log-file /tmp/ra.log"}
               }}"#,
        )
        .unwrap();
        assert_eq!(parsed["python"], vec!["pylsp"]);
        assert_eq!(
            parsed["rust"],
            vec!["rust-analyzer", "--log-file", "/tmp/ra.log"]
        );
    }

    #[test]
    fn a_malformed_entry_does_not_lose_the_good_ones() {
        let parsed =
            parse(r#"{"servers": {"python": {"command": 7}, "go": {"command": ["gopls"]}}}"#)
                .unwrap();
        assert!(!parsed.contains_key("python"));
        assert_eq!(parsed["go"], vec!["gopls"]);
    }

    #[test]
    fn a_file_that_is_not_json_yields_nothing_rather_than_an_error() {
        assert!(parse("not json at all").is_none());
        assert!(parse(r#"{"no servers key": true}"#).is_none());
    }

    #[test]
    fn nearest_config_above_the_document_wins() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("docs/deep");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            dir.path().join(CONFIG_FILE),
            r#"{"servers": {"python": {"command": ["outer"]}}}"#,
        )
        .unwrap();
        std::fs::write(
            nested.join(CONFIG_FILE),
            r#"{"servers": {"python": {"command": ["inner"]}}}"#,
        )
        .unwrap();
        let found = find_and_parse(&nested.join("doc.hick")).unwrap();
        assert_eq!(found["python"], vec!["inner"]);
    }
}
