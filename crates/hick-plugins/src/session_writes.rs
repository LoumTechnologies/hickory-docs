//! Textual write extraction providers; never executes source code.
#[derive(Debug, Clone)]
pub enum WriteKind {
    /// `hick.pipeline_write(path, content, slot=?)` → `hick:copy`
    PipelineWrite,
    /// `hick.pipeline_new_file(path, content)` → `hick:file`
    PipelineNewFile,
    /// Direct write (`open(path, 'w')`, shell redirect) → determined by pipeline ownership
    Direct,
}

#[derive(Debug, Clone)]
pub struct WriteOp {
    pub path: String,
    pub kind: WriteKind,
    /// Content extracted from the call's string literal argument, if possible.
    pub content: Option<String>,
    /// Named paste slot (for `PipelineWrite` only).
    pub slot: Option<String>,
}

pub fn extract_writes(code: &str, lang: &str) -> Vec<WriteOp> {
    crate::registry()
        .plugins()
        .flat_map(|p| p.interfaces)
        .find_map(|i| match i {
            crate::Interface::SessionWrites { aliases, extract } if aliases.contains(&lang) => {
                Some(extract(code))
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// Extract `hick.pipeline_write` and `hick.pipeline_new_file` calls from Python code.
///
/// Also detects simple direct file writes (`open(path, 'w')`).
pub fn extract_python_writes(code: &str) -> Vec<WriteOp> {
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
pub fn find_call(code: &str, from: usize, needle: &str) -> Option<usize> {
    code[from..].find(needle).map(|p| from + p)
}

/// Return the `(pos, kind, marker)` tuple with the smallest `pos` among Some values.
pub fn smallest_some<T: Ord>(
    options: [Option<(T, WriteKind, &'static str)>; 3],
) -> Option<(T, WriteKind, &'static str)> {
    options.into_iter().flatten().min_by(|a, b| a.0.cmp(&b.0))
}

/// Parse `hick.pipeline_write(path, content [, slot=...])` arguments.
pub fn parse_pipeline_write_call(code: &str, arg_start: usize) -> Option<WriteOp> {
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
pub fn parse_pipeline_new_file_call(code: &str, arg_start: usize) -> Option<WriteOp> {
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
pub fn parse_open_call(code: &str, arg_start: usize) -> Option<WriteOp> {
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
pub fn extract_slot_kwarg(code: &str, pos: usize) -> Option<String> {
    // Search only up to the matching close paren (approximate: find next `slot=`)
    let remaining = &code[pos..];
    let slot_rel = remaining.find("slot=")?;
    let slot_start = pos + slot_rel + "slot=".len();
    let (slot_val, _) = parse_python_string(code, slot_start)?;
    Some(slot_val)
}

/// Extract simple shell file writes (`echo ... > path`, `> path`).
pub fn extract_shell_writes(code: &str) -> Vec<WriteOp> {
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
pub fn parse_python_string(code: &str, pos: usize) -> Option<(String, usize)> {
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

pub fn skip_ws(code: &str, pos: usize) -> usize {
    let bytes = code.as_bytes();
    let mut p = pos;
    while p < bytes.len()
        && (bytes[p] == b' ' || bytes[p] == b'\t' || bytes[p] == b'\n' || bytes[p] == b'\r')
    {
        p += 1;
    }
    p
}

pub fn skip_comma_ws(code: &str, pos: usize) -> usize {
    let pos = skip_ws(code, pos);
    let bytes = code.as_bytes();
    if pos < bytes.len() && bytes[pos] == b',' {
        skip_ws(code, pos + 1)
    } else {
        pos
    }
}
