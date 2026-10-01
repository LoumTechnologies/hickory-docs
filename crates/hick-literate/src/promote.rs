//! Session → pipeline promotion.
//!
//! Reads an agent session file and extracts a clean, minimal pipeline
//! containing only the surviving side effects.  The result is a `.md`
//! pipeline document that produces identical output to replaying the session.
//!
//! # Algorithm
//!
//! 1. Parse the `hick:session` file.
//! 2. Walk `hick:assistant` action blocks and `hick:command` nodes in order.
//! 3. Detect file write operations (`hick.pipeline_write`, `hick.pipeline_new_file`,
//!    direct Python/shell writes).
//! 4. Deduplicate by path — only the last write per path survives.
//! 5. For each surviving write:
//!    - Pipeline-owned path → `hick:copy target="slot"` element.
//!    - New path → `hick:file path="…"` element with final content.
//! 6. Emit a valid `.md` pipeline document.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use hick_lang::{HickTag, SessionNode, parse_session};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Options for promoting a session to a pipeline.
pub struct PromoteOpts<'a> {
    /// Source text of the session file to promote.
    pub session_source: &'a str,
    /// Project root directory (used to locate `.md` sources and read files).
    pub project_dir: &'a Path,
    /// Display name of the session file (e.g. `agent-20260515-143022.md`).
    pub session_name: &'a str,
}

/// Result of a promotion run.
pub struct PromoteResult {
    /// The promoted `.md` pipeline document source.
    pub promoted_source: String,
    /// Total write operations detected across all action blocks.
    pub total_writes: usize,
    /// Surviving write operations after path-level deduplication.
    pub surviving_writes: usize,
}

// ---------------------------------------------------------------------------
// Internal write-operation representation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum WriteKind {
    /// `hick.pipeline_write(path, content, slot=?)` → `hick:copy`
    PipelineWrite,
    /// `hick.pipeline_new_file(path, content)` → `hick:file`
    PipelineNewFile,
    /// Direct write (`open(path, 'w')`, shell redirect) → determined by pipeline ownership
    Direct,
}

#[derive(Debug, Clone)]
struct WriteOp {
    path: String,
    kind: WriteKind,
    /// Content extracted from the call's string literal argument, if possible.
    content: Option<String>,
    /// Named paste slot (for `PipelineWrite` only).
    slot: Option<String>,
}

// ---------------------------------------------------------------------------
// Promotion entry point
// ---------------------------------------------------------------------------

/// Promote a session file to a clean pipeline document.
pub fn promote(opts: &PromoteOpts<'_>) -> Result<PromoteResult> {
    let session = parse_session(opts.session_source)
        .map_err(|e| anyhow::anyhow!("failed to parse session: {e}"))?;

    // Collect write operations in document order.
    let mut all_writes: Vec<WriteOp> = Vec::new();
    for node in &session.nodes {
        match node {
            SessionNode::Assistant { actions, .. } => {
                for action in actions {
                    all_writes.extend(extract_writes(&action.code, &action.lang));
                }
            }
            SessionNode::Command { text } => {
                all_writes.extend(extract_writes(text, "sh"));
            }
            _ => {}
        }
    }

    let total_writes = all_writes.len();

    // Deduplicate: last write per path wins.
    // `path_order` preserves first-seen insertion order for deterministic output.
    let mut path_order: Vec<String> = Vec::new();
    let mut surviving: HashMap<String, WriteOp> = HashMap::new();
    for write in all_writes {
        if !surviving.contains_key(&write.path) {
            path_order.push(write.path.clone());
        }
        surviving.insert(write.path.clone(), write);
    }
    let surviving_writes = surviving.len();

    // Scan the project for pipeline-owned file declarations.
    let pipeline_owned = scan_pipeline_owned(opts.project_dir);

    // Build output elements.
    let mut elements: Vec<String> = Vec::new();
    for path in &path_order {
        let Some(write) = surviving.get(path) else {
            continue;
        };
        if let Some(elem) = generate_element(write, &pipeline_owned, opts.project_dir) {
            elements.push(elem);
        }
    }

    let body = elements.join("\n");
    let promoted_source = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!-- promoted from {session_name} -->\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
         {body}\n\
         </hick:doc>\n",
        session_name = opts.session_name,
    );

    Ok(PromoteResult {
        promoted_source,
        total_writes,
        surviving_writes,
    })
}

// ---------------------------------------------------------------------------
// Write-op extraction
// ---------------------------------------------------------------------------

fn extract_writes(code: &str, lang: &str) -> Vec<WriteOp> {
    match lang {
        "python" | "py" => extract_python_writes(code),
        "sh" | "bash" | "shell" | "posix-sh" => extract_shell_writes(code),
        _ => Vec::new(),
    }
}

/// Extract `hick.pipeline_write` and `hick.pipeline_new_file` calls from Python code.
///
/// Also detects simple direct file writes (`open(path, 'w')`).
fn extract_python_writes(code: &str) -> Vec<WriteOp> {
    let mut writes = Vec::new();
    let mut search_from = 0;

    loop {
        let pw = find_call(code, search_from, "hick.pipeline_write(");
        let pnf = find_call(code, search_from, "hick.pipeline_new_file(");
        let direct = find_call(code, search_from, "open(");

        let next = smallest_some([
            pw.map(|p| (p, WriteKind::PipelineWrite, "hick.pipeline_write(")),
            pnf.map(|p| (p, WriteKind::PipelineNewFile, "hick.pipeline_new_file(")),
            direct.map(|p| (p, WriteKind::Direct, "open(")),
        ]);

        let Some((call_pos, kind, marker)) = next else {
            break;
        };

        let arg_start = call_pos + marker.len();

        match kind {
            WriteKind::PipelineWrite => {
                if let Some(write) = parse_pipeline_write_call(code, arg_start) {
                    writes.push(write);
                }
            }
            WriteKind::PipelineNewFile => {
                if let Some(write) = parse_pipeline_new_file_call(code, arg_start) {
                    writes.push(write);
                }
            }
            WriteKind::Direct => {
                if let Some(write) = parse_open_call(code, arg_start) {
                    writes.push(write);
                }
            }
        }

        search_from = call_pos + 1;
    }

    writes
}

/// Find the byte position of `needle` starting at `from` in `code`.
fn find_call(code: &str, from: usize, needle: &str) -> Option<usize> {
    code[from..].find(needle).map(|p| from + p)
}

/// Return the `(pos, kind, marker)` tuple with the smallest `pos` among Some values.
fn smallest_some<T: Ord>(
    options: [Option<(T, WriteKind, &'static str)>; 3],
) -> Option<(T, WriteKind, &'static str)> {
    options.into_iter().flatten().min_by(|a, b| a.0.cmp(&b.0))
}

/// Parse `hick.pipeline_write(path, content [, slot=...])` arguments.
fn parse_pipeline_write_call(code: &str, arg_start: usize) -> Option<WriteOp> {
    let pos = skip_ws(code, arg_start);
    let (path, after_path) = parse_python_string(code, pos)?;
    let after_comma = skip_comma_ws(code, after_path);
    let (content, after_content) = match parse_python_string(code, after_comma) {
        Some((c, p)) => (Some(c), p),
        None => (None, after_comma),
    };
    let slot = extract_slot_kwarg(code, after_content);
    Some(WriteOp {
        path,
        kind: WriteKind::PipelineWrite,
        content,
        slot,
    })
}

/// Parse `hick.pipeline_new_file(path, content)` arguments.
fn parse_pipeline_new_file_call(code: &str, arg_start: usize) -> Option<WriteOp> {
    let pos = skip_ws(code, arg_start);
    let (path, after_path) = parse_python_string(code, pos)?;
    let after_comma = skip_comma_ws(code, after_path);
    let content = parse_python_string(code, after_comma).map(|(c, _)| c);
    Some(WriteOp {
        path,
        kind: WriteKind::PipelineNewFile,
        content,
        slot: None,
    })
}

/// Parse `open(path, 'w')` or `open(path, 'a')` to detect direct file writes.
fn parse_open_call(code: &str, arg_start: usize) -> Option<WriteOp> {
    let pos = skip_ws(code, arg_start);
    let (path, after_path) = parse_python_string(code, pos)?;
    // Second arg must be 'w' or 'a' (write mode)
    let after_comma = skip_comma_ws(code, after_path);
    let (mode, _) = parse_python_string(code, after_comma)?;
    if !mode.starts_with('w') && !mode.starts_with('a') {
        return None;
    }
    Some(WriteOp {
        path,
        kind: WriteKind::Direct,
        content: None,
        slot: None,
    })
}

/// Look for `slot='...'` or `slot="..."` after `pos` in `code`.
fn extract_slot_kwarg(code: &str, pos: usize) -> Option<String> {
    // Search only up to the matching close paren (approximate: find next `slot=`)
    let remaining = &code[pos..];
    let slot_rel = remaining.find("slot=")?;
    let slot_start = pos + slot_rel + "slot=".len();
    let (slot_val, _) = parse_python_string(code, slot_start)?;
    Some(slot_val)
}

/// Extract simple shell file writes (`echo ... > path`, `> path`).
fn extract_shell_writes(code: &str) -> Vec<WriteOp> {
    let mut writes = Vec::new();
    for line in code.lines() {
        let trimmed = line.trim();
        // Look for `> path` or `>> path` redirects
        for redirect in [" > ", " >> "] {
            if let Some(pos) = trimmed.find(redirect) {
                let path_part = trimmed[pos + redirect.len()..].trim();
                // Skip special files and quoted paths for now
                if !path_part.is_empty()
                    && !path_part.starts_with('/')
                    && !path_part.contains(' ')
                    && !path_part.starts_with('$')
                    && !path_part.starts_with('&')
                {
                    writes.push(WriteOp {
                        path: path_part.to_string(),
                        kind: WriteKind::Direct,
                        content: None,
                        slot: None,
                    });
                }
                break;
            }
        }
    }
    writes
}

// ---------------------------------------------------------------------------
// Python string literal parser
// ---------------------------------------------------------------------------

/// Parse a Python string literal starting at byte `pos` in `code`.
///
/// Handles single-quoted, double-quoted, and triple-quoted strings.
/// Returns `(content, end_pos)` where `end_pos` is the byte after the closing
/// delimiter, or `None` if there is no string literal at `pos`.
fn parse_python_string(code: &str, pos: usize) -> Option<(String, usize)> {
    let pos = skip_ws(code, pos);
    if pos >= code.len() {
        return None;
    }

    let ch = code.as_bytes()[pos];
    if ch != b'\'' && ch != b'"' {
        return None;
    }

    // Check for triple quotes.
    let is_triple =
        pos + 2 < code.len() && code.as_bytes()[pos + 1] == ch && code.as_bytes()[pos + 2] == ch;

    if is_triple {
        let delim: &str = if ch == b'\'' { "'''" } else { "\"\"\"" };
        let start = pos + 3;
        let rest = &code[start..];
        let close_rel = rest.find(delim)?;
        let content = rest[..close_rel].to_string();
        Some((content, start + close_rel + 3))
    } else {
        // Single-delimiter string — parse with escape handling.
        let start = pos + 1;
        let mut end = start;
        let mut content = String::new();
        let bytes = code.as_bytes();
        while end < bytes.len() {
            if bytes[end] == b'\\' && end + 1 < bytes.len() {
                end += 1;
                content.push(match bytes[end] {
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'\\' => '\\',
                    b'\'' => '\'',
                    b'"' => '"',
                    b'0' => '\0',
                    other => other as char,
                });
                end += 1;
            } else if bytes[end] == ch {
                break;
            } else {
                // Safety: parse as UTF-8 char rather than byte-at-a-time to avoid splitting.
                let slice = &code[end..];
                let c = slice.chars().next()?;
                content.push(c);
                end += c.len_utf8();
            }
        }
        if end >= bytes.len() {
            return None; // unclosed string
        }
        Some((content, end + 1))
    }
}

// ---------------------------------------------------------------------------
// Whitespace / delimiter helpers
// ---------------------------------------------------------------------------

fn skip_ws(code: &str, pos: usize) -> usize {
    let bytes = code.as_bytes();
    let mut p = pos;
    while p < bytes.len()
        && (bytes[p] == b' ' || bytes[p] == b'\t' || bytes[p] == b'\n' || bytes[p] == b'\r')
    {
        p += 1;
    }
    p
}

fn skip_comma_ws(code: &str, pos: usize) -> usize {
    let pos = skip_ws(code, pos);
    let bytes = code.as_bytes();
    if pos < bytes.len() && bytes[pos] == b',' {
        skip_ws(code, pos + 1)
    } else {
        pos
    }
}

// ---------------------------------------------------------------------------
// Pipeline ownership scan
// ---------------------------------------------------------------------------

/// Records about one pipeline-managed output file.
struct PipelineFile {
    /// Named paste slots declared inside the `hick:file` element.
    paste_slots: Vec<String>,
}

/// Scan project `.md` files and return a map of output_path → [`PipelineFile`].
fn scan_pipeline_owned(project_dir: &Path) -> HashMap<String, PipelineFile> {
    let mut owned: HashMap<String, PipelineFile> = HashMap::new();

    // Look for *.md files in the project root and common subdirectories.
    let search_paths = [project_dir.to_path_buf(), project_dir.join("src")];

    for base in &search_paths {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("md" | "hick")
            ) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(doc) = hick_lang::parse(&content) else {
                continue;
            };
            for file_tag in doc.find_tags("file") {
                let Some(output_path) = file_tag.get_attribute("path") else {
                    continue;
                };
                let paste_slots = collect_paste_slots(file_tag);
                owned.insert(output_path.to_string(), PipelineFile { paste_slots });
            }
        }
    }

    // Also check `_hick.yml` for a `files:` list and scan those.
    let config_path = project_dir.join("_hick.yml");
    if let Ok(yaml) = std::fs::read_to_string(&config_path)
        && let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&yaml)
        && let Some(files) = val.get("files").and_then(|v| v.as_sequence())
    {
        for f in files {
            if let Some(pattern) = f.as_str() {
                let full = project_dir.join(pattern);
                if let Ok(content) = std::fs::read_to_string(&full)
                    && let Ok(doc) = hick_lang::parse(&content)
                {
                    for file_tag in doc.find_tags("file") {
                        let Some(output_path) = file_tag.get_attribute("path") else {
                            continue;
                        };
                        let paste_slots = collect_paste_slots(file_tag);
                        owned.insert(output_path.to_string(), PipelineFile { paste_slots });
                    }
                }
            }
        }
    }

    owned
}

/// Collect `<hick:paste name="…"/>` slot names from a `hick:file` tag (non-recursive).
fn collect_paste_slots(file_tag: &HickTag) -> Vec<String> {
    file_tag
        .child_tags()
        .filter(|t| t.name == "paste")
        .filter_map(|t| t.get_attribute("name"))
        .map(|s| s.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Element generation
// ---------------------------------------------------------------------------

fn generate_element(
    write: &WriteOp,
    pipeline_owned: &HashMap<String, PipelineFile>,
    project_dir: &Path,
) -> Option<String> {
    let path = &write.path;

    match &write.kind {
        WriteKind::PipelineWrite => {
            let content = write.content.as_deref().unwrap_or("");
            let slot = write
                .slot
                .as_deref()
                .or_else(|| {
                    pipeline_owned
                        .get(path)
                        .and_then(|pf| pf.paste_slots.first())
                        .map(|s| s.as_str())
                })
                .unwrap_or("body");
            Some(make_copy_element(slot, content))
        }
        WriteKind::PipelineNewFile => {
            let content = write
                .content
                .as_deref()
                .map(|c| c.to_string())
                .unwrap_or_else(|| read_disk_content(project_dir, path));
            Some(make_file_element(path, &content))
        }
        WriteKind::Direct => {
            if let Some(pf) = pipeline_owned.get(path) {
                // Pipeline-owned file written directly — use first slot, content from disk.
                let slot = pf.paste_slots.first().map(|s| s.as_str()).unwrap_or("body");
                let content = write
                    .content
                    .as_deref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| read_disk_content(project_dir, path));
                Some(make_copy_element(slot, &content))
            } else {
                let content = write
                    .content
                    .as_deref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| read_disk_content(project_dir, path));
                Some(make_file_element(path, &content))
            }
        }
    }
}

fn read_disk_content(project_dir: &Path, path: &str) -> String {
    std::fs::read_to_string(project_dir.join(path)).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// XML helpers
// ---------------------------------------------------------------------------

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn make_file_element(path: &str, content: &str) -> String {
    format!(
        "<hick:file path=\"{path}\">{}</hick:file>",
        escape_xml(content)
    )
}

fn make_copy_element(slot: &str, content: &str) -> String {
    format!(
        "<hick:copy target=\"{slot}\">{}</hick:copy>",
        escape_xml(content)
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn session(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-01-01T00:00:00Z">
{body}
</hick:session>"#
        )
    }

    fn promote_str(session_src: &str) -> PromoteResult {
        let dir = tempfile::tempdir().unwrap();
        promote(&PromoteOpts {
            session_source: session_src,
            project_dir: dir.path(),
            session_name: "test.hick",
        })
        .unwrap()
    }

    // --- string parser ---

    #[test]
    fn parse_single_quoted() {
        let code = "'hello world'";
        let (s, end) = parse_python_string(code, 0).unwrap();
        assert_eq!(s, "hello world");
        assert_eq!(end, code.len());
    }

    #[test]
    fn parse_double_quoted() {
        let (s, _) = parse_python_string("\"hello\"", 0).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn parse_triple_double_quoted() {
        let code = "\"\"\"line1\nline2\"\"\"";
        let (s, end) = parse_python_string(code, 0).unwrap();
        assert_eq!(s, "line1\nline2");
        assert_eq!(end, code.len());
    }

    #[test]
    fn parse_triple_single_quoted() {
        let code = "'''abc'''";
        let (s, _) = parse_python_string(code, 0).unwrap();
        assert_eq!(s, "abc");
    }

    #[test]
    fn parse_string_with_escape() {
        let (s, _) = parse_python_string("'foo\\nbar'", 0).unwrap();
        assert_eq!(s, "foo\nbar");
    }

    #[test]
    fn parse_non_string_returns_none() {
        assert!(parse_python_string("variable_name", 0).is_none());
    }

    // --- write extraction ---

    #[test]
    fn extracts_pipeline_new_file_call() {
        let code = "hick.pipeline_new_file('src/auth.rs', 'pub fn main() {}')";
        let writes = extract_python_writes(code);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].path, "src/auth.rs");
        assert_eq!(writes[0].content.as_deref(), Some("pub fn main() {}"));
        assert!(matches!(writes[0].kind, WriteKind::PipelineNewFile));
    }

    #[test]
    fn extracts_pipeline_write_with_slot() {
        let code = "hick.pipeline_write('src/auth.rs', 'fn foo() {}', slot='module-footer')";
        let writes = extract_python_writes(code);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].path, "src/auth.rs");
        assert_eq!(writes[0].content.as_deref(), Some("fn foo() {}"));
        assert_eq!(writes[0].slot.as_deref(), Some("module-footer"));
    }

    #[test]
    fn extracts_multiple_calls() {
        let code = "\
hick.pipeline_new_file('a.rs', 'v1')
hick.pipeline_new_file('b.rs', 'v2')";
        let writes = extract_python_writes(code);
        assert_eq!(writes.len(), 2);
        assert_eq!(writes[0].path, "a.rs");
        assert_eq!(writes[1].path, "b.rs");
    }

    #[test]
    fn extracts_open_write() {
        let code = "open('config.toml', 'w').write('data')";
        let writes = extract_python_writes(code);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].path, "config.toml");
        assert!(matches!(writes[0].kind, WriteKind::Direct));
    }

    #[test]
    fn ignores_open_read() {
        let code = "open('config.toml', 'r').read()";
        let writes = extract_python_writes(code);
        assert_eq!(writes.len(), 0);
    }

    #[test]
    fn shell_redirect_detected() {
        let code = "echo 'hello' > output.txt";
        let writes = extract_shell_writes(code);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].path, "output.txt");
    }

    // --- session promotion ---

    #[test]
    fn promotes_single_new_file() {
        let src = session(
            r#"<hick:assistant>
<hick:action lang="python">
hick.pipeline_new_file('src/lib.rs', 'pub fn lib() {}')
</hick:action>
</hick:assistant>"#,
        );
        let result = promote_str(&src);
        assert_eq!(result.total_writes, 1);
        assert_eq!(result.surviving_writes, 1);
        assert!(result.promoted_source.contains("hick:file"));
        assert!(result.promoted_source.contains(r#"path="src/lib.rs""#));
        assert!(result.promoted_source.contains("pub fn lib() {}"));
    }

    #[test]
    fn deduplicates_same_path_keeps_last() {
        let src = session(
            r#"<hick:assistant>
<hick:action lang="python">
hick.pipeline_new_file('out.py', 'v1')
</hick:action>
</hick:assistant>
<hick:observation source="action-0" exit="0">ok</hick:observation>
<hick:assistant>
<hick:action lang="python">
hick.pipeline_new_file('out.py', 'v2')
</hick:action>
</hick:assistant>"#,
        );
        let result = promote_str(&src);
        assert_eq!(result.total_writes, 2);
        assert_eq!(result.surviving_writes, 1);
        // Only 'v2' survives
        assert!(result.promoted_source.contains("v2"));
        assert!(!result.promoted_source.contains("v1"));
    }

    #[test]
    fn pipeline_write_becomes_hick_copy() {
        let src = session(
            r#"<hick:assistant>
<hick:action lang="python">
hick.pipeline_write('src/main.rs', 'fn foo() {}', slot='after-fn-main')
</hick:action>
</hick:assistant>"#,
        );
        let result = promote_str(&src);
        assert!(result.promoted_source.contains("hick:copy"));
        assert!(result.promoted_source.contains(r#"target="after-fn-main""#));
        assert!(result.promoted_source.contains("fn foo() {}"));
    }

    #[test]
    fn empty_session_produces_empty_pipeline() {
        let src = session("");
        let result = promote_str(&src);
        assert_eq!(result.total_writes, 0);
        assert_eq!(result.surviving_writes, 0);
        assert!(result.promoted_source.contains("<hick:doc"));
        assert!(result.promoted_source.contains("</hick:doc>"));
    }

    #[test]
    fn xml_special_chars_escaped_in_content() {
        let src = session(
            r#"<hick:assistant>
<hick:action lang="python">
hick.pipeline_new_file('out.rs', 'if x < y && z > 0 {}')
</hick:action>
</hick:assistant>"#,
        );
        let result = promote_str(&src);
        assert!(result.promoted_source.contains("&lt;"));
        assert!(result.promoted_source.contains("&amp;"));
        assert!(result.promoted_source.contains("&gt;"));
    }

    #[test]
    fn multiline_triple_quoted_content() {
        let src = session(
            r#"<hick:assistant>
<hick:action lang="python">
content = """
pub struct Foo {
    x: i32,
}
"""
hick.pipeline_new_file('src/foo.rs', """
pub struct Foo {
    x: i32,
}
""")
</hick:action>
</hick:assistant>"#,
        );
        let result = promote_str(&src);
        assert_eq!(result.surviving_writes, 1);
        assert!(result.promoted_source.contains("pub struct Foo"));
    }

    #[test]
    fn command_shell_writes_detected() {
        let src = session(r#"<hick:command>echo hello > greeting.txt</hick:command>"#);
        let result = promote_str(&src);
        assert_eq!(result.total_writes, 1);
        assert_eq!(writes_for_path(&result.promoted_source, "greeting.txt"), 1);
    }

    fn writes_for_path(promoted: &str, path: &str) -> usize {
        promoted.matches(path).count()
    }

    #[test]
    fn output_is_valid_hick_doc() {
        let src = session(
            r#"<hick:assistant>
<hick:action lang="python">
hick.pipeline_new_file('x.txt', 'hello')
</hick:action>
</hick:assistant>"#,
        );
        let result = promote_str(&src);
        // Should be parseable as a hick pipeline doc
        let doc = hick_lang::parse(&result.promoted_source).unwrap();
        assert_eq!(doc.find_tags("file").len(), 1);
    }
}
