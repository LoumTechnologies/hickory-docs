use super::*;

pub(crate) fn read_file(root: &Path, base: &Path, inv: &ToolInvocation) -> ToolOutcome {
    const NAME: &str = "read_file";
    const MAX_BYTES: usize = 1 << 20;
    let Some(rel) = inv.arg("path") else {
        return ToolOutcome::err(
            NAME,
            "read_file needs a path argument, relative to the document's directory".into(),
        );
    };
    let candidate = base.join(rel);
    let canonical = match candidate.canonicalize() {
        Ok(c) => c,
        Err(e) => {
            return ToolOutcome::err(
                NAME,
                format!(
                    "cannot read '{rel}' (resolved against {}): {e}",
                    base.display()
                ),
            );
        }
    };
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if !canonical.starts_with(&root) {
        return ToolOutcome::err(
            NAME,
            format!(
                "'{rel}' is outside the project ({}); read_file reads project files only",
                root.display()
            ),
        );
    }
    if canonical.is_dir() {
        let mut names: Vec<String> = std::fs::read_dir(&canonical)
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .map(|e| {
                        let n = e.file_name().to_string_lossy().to_string();
                        if e.path().is_dir() {
                            format!("{n}/")
                        } else {
                            n
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        return ToolOutcome::ok(
            NAME,
            format!("'{rel}' is a directory; it holds:\n{}", names.join("\n")),
        );
    }
    let bytes = match std::fs::read(&canonical) {
        Ok(b) => b,
        Err(e) => return ToolOutcome::err(NAME, format!("cannot read '{rel}': {e}")),
    };
    if bytes.len() > MAX_BYTES {
        return ToolOutcome::err(
            NAME,
            format!(
                "'{rel}' is {} bytes; read_file shows at most {MAX_BYTES}. Read a range \
                     with from/to, or let a cell process the file",
                bytes.len()
            ),
        );
    }
    let Ok(text) = String::from_utf8(bytes.clone()) else {
        return ToolOutcome::err(
            NAME,
            format!("'{rel}' is not UTF-8 text; a cell can process it, read_file cannot show it"),
        );
    };
    let index = LineIndex::new(&text);
    let total = index.lines.len();
    let parse_line = |key: &str, default: usize| -> Result<usize, String> {
        match inv.arg(key) {
            None => Ok(default),
            Some(v) => v
                .parse::<usize>()
                .ok()
                .filter(|n| *n >= 1)
                .ok_or_else(|| format!("{key}='{v}' is not a positive line number")),
        }
    };
    let (first, last) = match (parse_line("from", 1), parse_line("to", total)) {
        (Ok(a), Ok(b)) => (a.min(total.max(1)), b.min(total.max(1))),
        (Err(e), _) | (_, Err(e)) => return ToolOutcome::err(NAME, e),
    };
    if last < first {
        return ToolOutcome::err(NAME, format!("to={last} is before from={first}"));
    }
    let shown = index.render_range(first - 1, last - 1);
    let read = ContextRead {
        path: rel.to_string(),
        commit: head_commit_for(&canonical),
        sha256: sha256_hex(&bytes),
        first_line: first,
        last_line: last,
    };
    let header = if first == 1 && last == total {
        format!("file: {rel} ({total} lines)\n")
    } else {
        format!("file: {rel} lines {first}-{last} of {total}\n")
    };
    ToolOutcome::ok(NAME, format!("{header}{shown}")).with_read(read)
}
