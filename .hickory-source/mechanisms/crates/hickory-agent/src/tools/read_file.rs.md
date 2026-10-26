<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/read_file.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7845fffcfd2e"></a>
## `crates/hickory-agent/src/tools/read_file.rs`

Related contracts: [Context Provenance Is Derived From The Session, Never From The Model](../../../../../guarantees/agent/context-provenance-is-derived-from-the-session/contract.md), [Host context observations are immutable and changes are source-backed](../../../../../guarantees/agent/immutable-context-lineage/contract.md), [Select model context without saving open buffers](../../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md).

<a id="f-7845fffcfd2e-read_file"></a>
### `read_file`

<storycode:copy id="f-7845fffcfd2e-read_file">
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
        (Ok(a), Ok(b)) if a <= total && b <= total => (a, b),
        (Ok(_), Ok(_)) => {
            return ToolOutcome::err(
                NAME,
                format!(
                    "Requested lines are outside the file ({total} lines); no read receipt was recorded"
                ),
            );
        }
        (Err(e), _) | (_, Err(e)) => return ToolOutcome::err(NAME, e),
    };
    if last < first {
        return ToolOutcome::err(NAME, format!("to={last} is before from={first}"));
    }
    let shown = index.render_range(first - 1, last - 1);
    let excerpt: String = text
        .split_inclusive('\n')
        .skip(first - 1)
        .take(last - first + 1)
        .collect();
    let unique = !excerpt.is_empty()
        && text.find(&excerpt).is_some_and(|first| {
            let next = first + text[first..].chars().next().unwrap().len_utf8();
            !text[next..].contains(&excerpt)
        });
    let read = ContextRead {
        path: canonical
            .strip_prefix(&root)
            .unwrap_or(&canonical)
            .to_string_lossy()
            .replace('\\', "/"),
        source_path: Some(canonical.clone()),
        commit: head_commit_for(&canonical),
        sha256: sha256_hex(&bytes),
        first_line: first,
        last_line: last,
        byte_span: (
            text.split_inclusive('\n')
                .take(first - 1)
                .map(str::len)
                .sum(),
            text.split_inclusive('\n').take(last).map(str::len).sum(),
        ),
        content: excerpt,
        unique,
    };
    let header = if first == 1 && last == total {
        format!("file: {rel} ({total} lines)\n")
    } else {
        format!("file: {rel} lines {first}-{last} of {total}\n")
    };
    ToolOutcome::ok(NAME, format!("{header}{shown}")).with_read(read)
}

</storycode:copy>
<a id="f-7845fffcfd2e-tests"></a>
### `tests`

<storycode:copy id="f-7845fffcfd2e-tests">
#[cfg(test)]
mod tests {
    use super::*;

    // Guarantee: docs/guarantees/agent/context-provenance-is-derived-from-the-session.md
    #[test]
    fn a_relative_read_receipt_resolves_from_the_document_directory() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let base = root.join("notes");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(root.join("source.txt"), "first\nsecond\n").unwrap();
        let inv = ToolInvocation::synthetic(
            "read_file",
            vec![
                ("path".into(), "../source.txt".into()),
                ("from".into(), "2".into()),
                ("to".into(), "2".into()),
            ],
            None,
        )
        .unwrap();
        let outcome = read_file(root, &base, &inv);
        assert!(outcome.ok, "{}", outcome.text);
        assert_eq!(outcome.reads[0].path, "source.txt");
        // Guarantee: docs/guarantees/agent/immutable-context-lineage.md
        assert_eq!(outcome.reads[0].byte_span, (6, 13));
        assert_eq!(outcome.reads[0].content, "second\n");
        assert_eq!(
            (outcome.reads[0].first_line, outcome.reads[0].last_line),
            (2, 2)
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/read_file.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/read_file.rs"><storycode:paste select="#f-7845fffcfd2e-read_file"/><storycode:paste select="#f-7845fffcfd2e-tests"/></storycode:file>

</storycode:doc>
