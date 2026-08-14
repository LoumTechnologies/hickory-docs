//! Editor wiring for `hick init`: register `hick-lsp` with the editors this
//! repository is already set up for, and teach `hick-lsp` to use the language
//! servers those editors were already configured with.
//!
//! Two halves, and they point in opposite directions:
//!
//! 1. **Adopt** (`discover_servers`) — read the repository's existing editor
//!    configuration and write what it says into `.hick-lsp.json`, so a
//!    `hick:file` block containing Python is checked by the same server that
//!    checks the repository's `.py` files. A project pinned to `pylsp` that
//!    silently got `pyright` inside its documents would produce two different
//!    sets of diagnostics for one piece of code, which is exactly the drift
//!    this product exists to remove.
//! 2. **Register** (`configure_*`) — write the project-local configuration
//!    that points an editor at `hick-lsp` for `*.hick` files, for the editors
//!    where that is a file we can honestly write. Where it is not (Zed needs
//!    an extension; Neovim has no project-local LSP registration), `hick init`
//!    prints the exact snippet instead of pretending.
//!
//! Everything here is best-effort and non-fatal: `hick init`'s job is the
//! pre-commit gate and the agent tools, and a repository with no editor
//! configuration at all must still init cleanly.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context as _, Result};
use serde_json::{Map, Value, json};

/// A child language server adopted from the repository's editor config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptedServer {
    /// LSP language id, as `hick-lsp` names it (`python`, `rust`, …).
    pub language: String,
    /// Command and arguments to spawn.
    pub command: Vec<String>,
    /// Repo-relative file this was read out of, for the report.
    pub source: String,
}

/// An editor `hick init` found, and what it managed to do about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorOutcome {
    /// Display name, e.g. "VS Code".
    pub editor: &'static str,
    /// Repo-relative file written, or `None` when only advice was possible.
    pub wrote: Option<String>,
    /// True if that file was created or changed by this run.
    pub changed: bool,
    /// What the user still has to do by hand, if anything.
    pub manual: Option<String>,
}

/// Everything the editor half of `hick init` did.
#[derive(Debug, Default)]
pub struct EditorSetup {
    /// Child servers taken from existing editor configuration.
    pub adopted: Vec<AdoptedServer>,
    /// True if `.hick-lsp.json` was created or changed.
    pub hick_lsp_json_changed: bool,
    /// Per-editor results, in detection order.
    pub editors: Vec<EditorOutcome>,
    /// False if `hick-lsp` is not on PATH — every registration below is inert
    /// until it is.
    pub hick_lsp_on_path: bool,
}

/// Run the editor half of `hick init` against the work-tree root.
pub fn configure_editors(root: &Path) -> Result<EditorSetup> {
    let mut setup = EditorSetup {
        adopted: discover_servers(root),
        hick_lsp_on_path: on_path("hick-lsp") || on_path("hick-lsp.exe"),
        ..Default::default()
    };

    setup.hick_lsp_json_changed = write_server_overrides(root, &setup.adopted)?;

    if detected(root, ".vscode", &["code", "codium", "code-insiders"]) {
        setup.editors.push(configure_vscode(root)?);
    }
    if detected(root, ".helix", &["hx", "helix"]) {
        setup.editors.push(configure_helix(root)?);
    }
    if detected(root, ".zed", &["zed", "zeditor"]) {
        setup.editors.push(EditorOutcome {
            editor: "Zed",
            wrote: None,
            changed: false,
            // Zed can only get a new language server from an extension, so
            // there is no project file to write. The extension is in this
            // repository; saying where beats a settings block that would not
            // work.
            manual: Some(
                "install the dev extension from the hickory-docs checkout \
                 (`zed: install dev extension` → editors/zed-hick); it starts \
                 `hick-lsp` from your PATH for *.hick files"
                    .into(),
            ),
        });
    }
    if on_path("nvim") {
        setup.editors.push(EditorOutcome {
            editor: "Neovim",
            wrote: None,
            changed: false,
            // Neovim reads no project-local LSP configuration unless the user
            // has opted into `exrc`, and writing to their init.lua from a
            // repository would be a surprising thing for `hick init` to do.
            manual: Some(
                "add the `vim.lsp.start` snippet from docs/users/editor-setup.md \
                 to your own config (Neovim has no project-local LSP registration)"
                    .into(),
            ),
        });
    }

    Ok(setup)
}

// ---------------------------------------------------------------------------
// Adopting the repository's existing language servers
// ---------------------------------------------------------------------------

/// Language server names an editor config can name, and the command each one
/// actually is. Editors name servers; `hick-lsp` spawns processes.
const SERVER_COMMANDS: &[(&str, &[&str])] = &[
    ("pylsp", &["pylsp"]),
    ("python-lsp-server", &["pylsp"]),
    ("pyright", &["pyright-langserver", "--stdio"]),
    ("pyright-langserver", &["pyright-langserver", "--stdio"]),
    ("basedpyright", &["basedpyright-langserver", "--stdio"]),
    ("jedi", &["jedi-language-server"]),
    ("jedi-language-server", &["jedi-language-server"]),
    ("ruff", &["ruff", "server"]),
    ("rust-analyzer", &["rust-analyzer"]),
    ("gopls", &["gopls"]),
    ("clangd", &["clangd"]),
    ("zls", &["zls"]),
    ("lua-language-server", &["lua-language-server"]),
    (
        "typescript-language-server",
        &["typescript-language-server", "--stdio"],
    ),
    ("vtsls", &["vtsls", "--stdio"]),
    ("nil", &["nil"]),
];

/// Which language a named server belongs to. Used when an editor names a
/// server without saying which language it is for.
const SERVER_LANGUAGES: &[(&str, &str)] = &[
    ("pylsp", "python"),
    ("python-lsp-server", "python"),
    ("pyright", "python"),
    ("pyright-langserver", "python"),
    ("basedpyright", "python"),
    ("jedi", "python"),
    ("jedi-language-server", "python"),
    ("ruff", "python"),
    ("rust-analyzer", "rust"),
    ("gopls", "go"),
    ("clangd", "c"),
    ("zls", "zig"),
    ("lua-language-server", "lua"),
    ("typescript-language-server", "typescript"),
    ("vtsls", "typescript"),
    ("nil", "nix"),
];

fn command_for_server(name: &str) -> Option<Vec<String>> {
    let key = name.trim().to_ascii_lowercase();
    SERVER_COMMANDS
        .iter()
        .find(|(n, _)| *n == key)
        .map(|(_, cmd)| cmd.iter().map(|s| (*s).to_string()).collect())
}

fn language_for_server(name: &str) -> Option<&'static str> {
    let key = name.trim().to_ascii_lowercase();
    SERVER_LANGUAGES
        .iter()
        .find(|(n, _)| *n == key)
        .map(|(_, lang)| *lang)
}

/// Read every editor configuration in the repository and return the child
/// servers they imply, first source winning per language.
pub fn discover_servers(root: &Path) -> Vec<AdoptedServer> {
    let mut found: Vec<AdoptedServer> = Vec::new();
    let mut push = |server: AdoptedServer| {
        if !found.iter().any(|s| s.language == server.language) {
            found.push(server);
        }
    };

    for server in from_vscode(root) {
        push(server);
    }
    for server in from_zed(root) {
        push(server);
    }
    for server in from_helix(root) {
        push(server);
    }
    found
}

/// `.vscode/settings.json`: `python.languageServer`, `rust-analyzer.server.path`,
/// `clangd.path`, `go.alternateTools.gopls`.
fn from_vscode(root: &Path) -> Vec<AdoptedServer> {
    let path = root.join(".vscode/settings.json");
    let Some(settings) = read_jsonc_object(&path) else {
        return Vec::new();
    };
    let source = ".vscode/settings.json".to_string();
    let mut out = Vec::new();

    // VS Code names the Python server by product name, not by binary.
    if let Some(named) = settings
        .get("python.languageServer")
        .and_then(Value::as_str)
    {
        let command = match named.to_ascii_lowercase().as_str() {
            // Pylance is closed-source and only runs inside VS Code, so the
            // honest fallback is the server it is built on.
            "pylance" | "default" => Some(vec!["pyright-langserver".into(), "--stdio".into()]),
            other => command_for_server(other),
        };
        if let Some(command) = command {
            out.push(AdoptedServer {
                language: "python".into(),
                command,
                source: source.clone(),
            });
        }
    }
    for (key, language) in [
        ("rust-analyzer.server.path", "rust"),
        ("clangd.path", "c"),
        ("zig.zls.path", "zig"),
    ] {
        if let Some(bin) = settings.get(key).and_then(Value::as_str)
            && !bin.trim().is_empty()
        {
            out.push(AdoptedServer {
                language: language.into(),
                command: vec![bin.trim().to_string()],
                source: source.clone(),
            });
        }
    }
    if let Some(bin) = settings
        .get("go.alternateTools")
        .and_then(Value::as_object)
        .and_then(|t| t.get("gopls"))
        .and_then(Value::as_str)
    {
        out.push(AdoptedServer {
            language: "go".into(),
            command: vec![bin.to_string()],
            source,
        });
    }
    out
}

/// `.zed/settings.json`: `languages.<Name>.language_servers`, plus any
/// explicit binary path under `lsp.<name>.binary.path`.
fn from_zed(root: &Path) -> Vec<AdoptedServer> {
    let path = root.join(".zed/settings.json");
    let Some(settings) = read_jsonc_object(&path) else {
        return Vec::new();
    };
    let source = ".zed/settings.json".to_string();
    let mut out = Vec::new();

    let explicit_paths: BTreeMap<String, String> = settings
        .get("lsp")
        .and_then(Value::as_object)
        .map(|servers| {
            servers
                .iter()
                .filter_map(|(name, cfg)| {
                    let bin = cfg
                        .get("binary")
                        .and_then(|b| b.get("path"))
                        .and_then(Value::as_str)?;
                    Some((name.to_ascii_lowercase(), bin.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    if let Some(languages) = settings.get("languages").and_then(Value::as_object) {
        for config in languages.values() {
            let Some(names) = config.get("language_servers").and_then(Value::as_array) else {
                continue;
            };
            // Zed writes disabled servers as "!name"; the first plain entry is
            // the one it actually runs.
            let Some(name) = names
                .iter()
                .filter_map(Value::as_str)
                .find(|n| !n.starts_with('!') && *n != "...")
            else {
                continue;
            };
            let Some(language) = language_for_server(name) else {
                continue;
            };
            let command = explicit_paths
                .get(&name.to_ascii_lowercase())
                .map(|bin| vec![bin.clone()])
                .or_else(|| command_for_server(name));
            if let Some(command) = command {
                out.push(AdoptedServer {
                    language: language.into(),
                    command,
                    source: source.clone(),
                });
            }
        }
    }
    out
}

/// `.helix/languages.toml`: `[language-server.<name>] command/args`, tied to
/// languages by `[[language]] language-servers`.
///
/// Parsed with a small line scanner rather than a TOML crate: the two shapes
/// read here are flat and always written the same way, and the failure mode of
/// missing one is "keep the default server", not a broken init.
fn from_helix(root: &Path) -> Vec<AdoptedServer> {
    let path = root.join(".helix/languages.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let source = ".helix/languages.toml".to_string();

    let mut commands: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut language_servers: Vec<(String, String)> = Vec::new();

    let mut section: Option<String> = None; // "server:<name>" or "language"
    let mut current_language: Option<String> = None;
    let mut pending_server: Option<(String, Option<String>, Vec<String>)> = None;

    let flush = |pending: &mut Option<(String, Option<String>, Vec<String>)>,
                 commands: &mut BTreeMap<String, Vec<String>>| {
        if let Some((name, Some(program), args)) = pending.take() {
            let mut command = vec![program];
            command.extend(args);
            commands.insert(name.to_ascii_lowercase(), command);
        }
    };

    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            flush(&mut pending_server, &mut commands);
            let header = header.trim_matches('[').trim_matches(']').trim();
            if let Some(name) = header.strip_prefix("language-server.") {
                let name = name.trim_matches('"').to_string();
                pending_server = Some((name.clone(), None, Vec::new()));
                section = Some(format!("server:{name}"));
            } else if header == "language" {
                current_language = None;
                section = Some("language".into());
            } else {
                section = None;
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match section.as_deref() {
            Some(s) if s.starts_with("server:") => {
                if let Some((_, program, args)) = pending_server.as_mut() {
                    if key == "command" {
                        *program = Some(unquote(value));
                    } else if key == "args" {
                        *args = toml_string_array(value);
                    }
                }
            }
            Some("language") => {
                if key == "name" {
                    current_language = Some(unquote(value));
                } else if key == "language-servers"
                    && let Some(language) = current_language.clone()
                    && let Some(first) = toml_string_array(value).into_iter().next()
                {
                    language_servers.push((language, first));
                }
            }
            _ => {}
        }
    }
    flush(&mut pending_server, &mut commands);

    let mut out = Vec::new();
    for (language, server) in language_servers {
        // Helix language names match LSP language ids for everything hick-lsp
        // can route, so the name is used as-is when a server is bound to it.
        let command = commands
            .get(&server.to_ascii_lowercase())
            .cloned()
            .or_else(|| command_for_server(&server));
        if let Some(command) = command {
            out.push(AdoptedServer {
                language,
                command,
                source: source.clone(),
            });
        }
    }
    out
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn toml_string_array(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(unquote)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Read a JSON file that may carry `//` comments and trailing commas, as
/// every editor's settings file in practice does.
fn read_jsonc_object(path: &Path) -> Option<Map<String, Value>> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&strip_jsonc(&text)).ok()?;
    value.as_object().cloned()
}

/// Strip `//` and `/* */` comments and trailing commas, leaving byte offsets
/// otherwise untouched. String literals are respected, so a `//` inside a path
/// survives.
fn strip_jsonc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = '\0';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
            }
            _ => out.push(c),
        }
    }
    // Trailing commas, now that comments are gone.
    let mut cleaned = String::with_capacity(out.len());
    let bytes: Vec<char> = out.chars().collect();
    let mut i = 0;
    let mut in_string = false;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            cleaned.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            cleaned.push(c);
            i += 1;
            continue;
        }
        if c == ',' {
            let next = bytes[i + 1..].iter().find(|c| !c.is_whitespace());
            if matches!(next, Some('}') | Some(']')) {
                i += 1;
                continue;
            }
        }
        cleaned.push(c);
        i += 1;
    }
    cleaned
}

// ---------------------------------------------------------------------------
// Writing what we learned
// ---------------------------------------------------------------------------

/// Write the adopted servers into `.hick-lsp.json`, which `hick-lsp` reads.
///
/// Existing entries are never overwritten: the file is a person's to edit once
/// it exists, and `hick init` re-running must not undo a hand-tuned command.
/// Returns true if the file changed.
fn write_server_overrides(root: &Path, adopted: &[AdoptedServer]) -> Result<bool> {
    let path = root.join(hick_lsp::server_config::CONFIG_FILE);
    let existing = match std::fs::read_to_string(&path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    if existing.is_none() && adopted.is_empty() {
        return Ok(false);
    }

    let mut root_value: Value = match &existing {
        None => json!({
            "//": "Which language server hick-lsp spawns for code inside hick:file \
                   blocks. Written by `hick init` from this repo's editor config; \
                   yours to edit. Re-running init only adds languages it newly finds.",
            "servers": {},
        }),
        Some(content) => serde_json::from_str(content).with_context(|| {
            format!(
                "{} is not valid JSON — fix or remove it, then re-run `hick init` \
                 (it will not overwrite a file it cannot understand)",
                path.display()
            )
        })?,
    };
    let Some(obj) = root_value.as_object_mut() else {
        anyhow::bail!(
            "{} does not contain a JSON object; refusing to overwrite it",
            path.display()
        );
    };
    let servers = obj
        .entry("servers")
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(servers) = servers.as_object_mut() else {
        anyhow::bail!(
            "{}: \"servers\" is not an object; refusing to overwrite it",
            path.display()
        );
    };
    for server in adopted {
        if servers.contains_key(&server.language) {
            continue;
        }
        servers.insert(
            server.language.clone(),
            json!({ "command": server.command, "discoveredFrom": server.source }),
        );
    }

    let rendered = format!("{}\n", serde_json::to_string_pretty(&root_value)?);
    let changed = existing.as_deref() != Some(rendered.as_str());
    if changed {
        std::fs::write(&path, &rendered)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(changed)
}

/// True if the repo has this editor's directory, or the editor is on PATH.
fn detected(root: &Path, dir: &str, binaries: &[&str]) -> bool {
    root.join(dir).is_dir() || binaries.iter().any(|b| on_path(b))
}

fn on_path(bin: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| dir.join(bin).is_file())
}

/// `.vscode/settings.json`: associate `*.hick` and point a generic LSP client
/// at `hick-lsp`. VS Code has no built-in way to register a language server
/// from settings, so this is only half the job and says so.
fn configure_vscode(root: &Path) -> Result<EditorOutcome> {
    let path = root.join(".vscode/settings.json");
    let mut settings = read_jsonc_object(&path).unwrap_or_default();
    let before = settings.clone();

    settings
        .entry("files.associations")
        .or_insert_with(|| json!({}));
    if let Some(assoc) = settings
        .get_mut("files.associations")
        .and_then(Value::as_object_mut)
    {
        assoc.entry("*.hick").or_insert_with(|| json!("hick"));
    }
    settings
        .entry("glspc.languageId")
        .or_insert_with(|| json!("hick"));
    settings
        .entry("glspc.serverCommand")
        .or_insert_with(|| json!("hick-lsp"));

    let changed = settings != before;
    if changed {
        std::fs::create_dir_all(path.parent().expect("has a parent"))?;
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&settings)?),
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(EditorOutcome {
        editor: "VS Code",
        wrote: Some(".vscode/settings.json".into()),
        changed,
        manual: Some(
            "VS Code cannot start a language server from settings alone — install a \
             generic LSP client extension (e.g. glspc); the settings it reads are \
             already written"
                .into(),
        ),
    })
}

/// Start sentinel of the managed block in `.helix/languages.toml`.
pub const HELIX_BLOCK_START: &str = "# --- HICKORY ---";
/// End sentinel of the managed block in `.helix/languages.toml`.
pub const HELIX_BLOCK_END: &str = "# --- END HICKORY ---";

/// The Helix block. Helix registers language servers from project config with
/// no extension, so this one is complete on its own.
const HELIX_BODY: &str = r##"# Managed by `hick init` — do not edit inside this block.
[language-server.hick-lsp]
command = "hick-lsp"

[[language]]
name = "hick"
scope = "source.hick"
file-types = ["hick"]
roots = [".git"]
comment-token = "#"
indent = { tab-width = 2, unit = "  " }
language-servers = ["hick-lsp"]"##;

fn configure_helix(root: &Path) -> Result<EditorOutcome> {
    let path = root.join(".helix/languages.toml");
    let block = format!("{HELIX_BLOCK_START}\n{HELIX_BODY}\n{HELIX_BLOCK_END}\n");
    let existing = match std::fs::read_to_string(&path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    let new_content = match &existing {
        None => block,
        Some(content) => {
            match crate::init::replace_between(content, HELIX_BLOCK_START, HELIX_BLOCK_END, &block)?
            {
                Some(replaced) => replaced,
                None => {
                    let mut s = content.clone();
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                    s.push('\n');
                    s.push_str(&block);
                    s
                }
            }
        }
    };
    let changed = existing.as_deref() != Some(new_content.as_str());
    if changed {
        std::fs::create_dir_all(path.parent().expect("has a parent"))?;
        std::fs::write(&path, &new_content)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(EditorOutcome {
        editor: "Helix",
        wrote: Some(".helix/languages.toml".into()),
        changed,
        manual: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn adopts_the_python_server_vscode_already_chose() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".vscode/settings.json",
            r#"{
                 // the repo pinned this deliberately
                 "python.languageServer": "Jedi",
                 "rust-analyzer.server.path": "/opt/ra/rust-analyzer",
               }"#,
        );
        let found = discover_servers(dir.path());
        assert_eq!(
            found,
            vec![
                AdoptedServer {
                    language: "python".into(),
                    command: vec!["jedi-language-server".into()],
                    source: ".vscode/settings.json".into(),
                },
                AdoptedServer {
                    language: "rust".into(),
                    command: vec!["/opt/ra/rust-analyzer".into()],
                    source: ".vscode/settings.json".into(),
                },
            ]
        );
    }

    #[test]
    fn adopts_from_zed_language_server_lists() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".zed/settings.json",
            r#"{
                 "languages": { "Python": { "language_servers": ["!pyright", "pylsp"] } },
                 "lsp": { "pylsp": { "binary": { "path": "/venv/bin/pylsp" } } }
               }"#,
        );
        let found = discover_servers(dir.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].language, "python");
        assert_eq!(found[0].command, vec!["/venv/bin/pylsp"]);
    }

    #[test]
    fn adopts_from_helix_language_config() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".helix/languages.toml",
            r#"
[language-server.ruff]
command = "ruff"
args = ["server", "--preview"]

[[language]]
name = "python"
language-servers = ["ruff", "pylsp"]
"#,
        );
        let found = discover_servers(dir.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].language, "python");
        assert_eq!(found[0].command, vec!["ruff", "server", "--preview"]);
    }

    #[test]
    fn writing_overrides_never_clobbers_a_hand_edited_entry() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".vscode/settings.json",
            r#"{"python.languageServer": "Pylsp"}"#,
        );
        let adopted = discover_servers(dir.path());
        assert!(write_server_overrides(dir.path(), &adopted).unwrap());

        // Hand-tune it, then re-run: the edit survives and nothing changes.
        let path = dir.path().join(".hick-lsp.json");
        let tuned = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\"pylsp\"", "\"my-pylsp-wrapper\"");
        std::fs::write(&path, &tuned).unwrap();
        assert!(!write_server_overrides(dir.path(), &adopted).unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), tuned);
    }

    #[test]
    fn no_editor_config_writes_no_override_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!write_server_overrides(dir.path(), &[]).unwrap());
        assert!(!dir.path().join(".hick-lsp.json").exists());
    }

    #[test]
    fn helix_block_is_idempotent_and_preserves_user_content() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".helix/languages.toml",
            "[[language]]\nname = \"python\"\n",
        );
        let first = configure_helix(dir.path()).unwrap();
        assert!(first.changed);
        let content = std::fs::read_to_string(dir.path().join(".helix/languages.toml")).unwrap();
        assert!(content.starts_with("[[language]]\nname = \"python\"\n"));
        assert!(content.contains("[language-server.hick-lsp]"));

        let second = configure_helix(dir.path()).unwrap();
        assert!(!second.changed);
        assert_eq!(
            content,
            std::fs::read_to_string(dir.path().join(".helix/languages.toml")).unwrap()
        );
    }

    #[test]
    fn vscode_settings_are_added_without_disturbing_existing_keys() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".vscode/settings.json",
            r#"{"editor.tabSize": 2, "glspc.serverCommand": "my-own-server"}"#,
        );
        let outcome = configure_vscode(dir.path()).unwrap();
        assert!(outcome.changed);
        let settings = read_jsonc_object(&dir.path().join(".vscode/settings.json")).unwrap();
        assert_eq!(settings["editor.tabSize"], json!(2));
        // An existing choice is left alone rather than replaced.
        assert_eq!(settings["glspc.serverCommand"], json!("my-own-server"));
        assert_eq!(settings["files.associations"]["*.hick"], json!("hick"));

        assert!(!configure_vscode(dir.path()).unwrap().changed);
    }

    #[test]
    fn jsonc_comments_and_trailing_commas_are_tolerated() {
        let stripped = strip_jsonc(
            r#"{
                 /* block */
                 "a": "http://x//y", // line
                 "b": [1, 2,],
               }"#,
        );
        let value: Value = serde_json::from_str(&stripped).unwrap();
        assert_eq!(value["a"], json!("http://x//y"));
        assert_eq!(value["b"], json!([1, 2]));
    }
}
