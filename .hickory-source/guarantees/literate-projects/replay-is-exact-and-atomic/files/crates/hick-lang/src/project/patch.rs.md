<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/patch.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Reject a patch that does not fit its exact input](../../../../../contract.md)

Captured SHA-256: `445024f328e6da9a340fde8b513f9bb54e782279a4551fd340d3e05c6bf70101`.

<book:entry path="crates/hick-lang/src/project/patch.rs">
use super::tree::copy_origins;
use super::{ByteOrigin, EntryKind, ProjectEntry, ProjectTree, validate_project_path};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub before: ProjectTree,
    pub after: ProjectTree,
    pub paths: BTreeSet<String>,
}

/// Exact, transactional unified-diff replay. Diff payload line endings are data;
/// header line endings are syntax. No offset search, native IO, merge or model.
pub fn apply_project_patch(
    tree: &ProjectTree,
    diff: &str,
    document: &str,
    source_offset: usize,
) -> Result<PatchResult, String> {
    let mut lines = vec![];
    let mut at = 0;
    for line in diff.split_inclusive('\n') {
        lines.push((line, at));
        at += line.len();
    }
    let mut cursor = 0;
    let mut after = tree.clone();
    let mut paths = BTreeSet::new();
    while cursor < lines.len() {
        let old = header(lines[cursor].0, "--- ", "a/")?;
        cursor += 1;
        let new = header(
            lines.get(cursor).ok_or("missing +++ header")?.0,
            "+++ ",
            "b/",
        )?;
        cursor += 1;
        if old.is_none() && new.is_none() {
            return Err("patch cannot have two /dev/null paths".into());
        }
        for path in [old.as_ref(), new.as_ref()].into_iter().flatten() {
            validate_project_path(path)?;
        }
        let touched: BTreeSet<_> = [old.as_ref(), new.as_ref()]
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        if touched.iter().any(|p| paths.contains(p)) {
            return Err("a file occurs twice in one patch act".into());
        }
        paths.extend(touched);
        let original = match &old {
            Some(path) => tree
                .get(path)
                .ok_or_else(|| format!("patch input missing: {path}"))?
                .as_ref()
                .clone(),
            None => ProjectEntry::file(Vec::<u8>::new()),
        };
        if original.kind != EntryKind::File {
            return Err("text patch of a symlink is unsupported".into());
        }
        let text =
            std::str::from_utf8(&original.bytes).map_err(|_| "text patch requires UTF-8 input")?;
        if let Some(path) = &new
            && old.as_ref() != Some(path)
            && tree.get(path).is_some()
        {
            return Err(format!("patch destination already exists: {path}"));
        }
        let old_lines: Vec<&str> = text.split_inclusive('\n').collect();
        let mut offsets = vec![0];
        for line in &old_lines {
            offsets.push(offsets.last().unwrap() + line.len());
        }
        let mut consumed = 0;
        let mut output_lines = 0;
        let mut bytes = Vec::new();
        let mut origins = vec![];
        let mut hunks = 0;
        while cursor < lines.len() && !lines[cursor].0.starts_with("--- ") {
            let (old_start, old_count, new_start, new_count) = hunk_header(lines[cursor].0)?;
            cursor += 1;
            hunks += 1;
            let old_at = line_index(old_start, old_count)?;
            let new_at = line_index(new_start, new_count)?;
            if old_at < consumed || old_at > old_lines.len() {
                return Err("hunk offset is overlapping or outside input".into());
            }
            append_original(
                &original,
                (offsets[consumed], offsets[old_at]),
                &mut bytes,
                &mut origins,
            );
            output_lines += old_at - consumed;
            consumed = old_at;
            if new_at != output_lines {
                return Err("new hunk offset does not match constructed output".into());
            }
            let mut removed = 0;
            let mut added = 0;
            while removed < old_count || added < new_count {
                let (line, offset) = *lines.get(cursor).ok_or("truncated hunk")?;
                let operation = *line.as_bytes().first().ok_or("empty hunk line")?;
                if !matches!(operation, b' ' | b'+' | b'-') {
                    return Err("hunk line must begin with space, + or -".into());
                }
                let mut payload = &line[1..];
                let mut no_newline = false;
                cursor += 1;
                if lines
                    .get(cursor)
                    .is_some_and(|(l, _)| syntax(l) == "\\ No newline at end of file")
                {
                    if !payload.ends_with('\n') {
                        return Err("newline marker requires a terminated diff line".into());
                    }
                    payload = &payload[..payload.len() - 1];
                    no_newline = true;
                    cursor += 1;
                }
                if !no_newline && !payload.ends_with('\n') {
                    return Err("unterminated diff payload needs a no-newline marker".into());
                }
                if operation != b'+' {
                    if removed == old_count {
                        return Err("hunk exceeds old count".into());
                    }
                    if old_lines.get(consumed).copied() != Some(payload) {
                        return Err(format!(
                            "wrong patch context at input line {}",
                            consumed + 1
                        ));
                    }
                    if operation == b' ' {
                        append_original(
                            &original,
                            (offsets[consumed], offsets[consumed + 1]),
                            &mut bytes,
                            &mut origins,
                        );
                    }
                    consumed += 1;
                    removed += 1;
                }
                if operation != b'-' {
                    if added == new_count {
                        return Err("hunk exceeds new count".into());
                    }
                    if operation == b'+' {
                        let start = bytes.len();
                        bytes.extend_from_slice(payload.as_bytes());
                        if !payload.is_empty() {
                            origins.push(ByteOrigin {
                                output: (start, bytes.len()),
                                document: document.into(),
                                source: (
                                    source_offset + offset + 1,
                                    source_offset + offset + 1 + payload.len(),
                                ),
                            });
                        }
                    }
                    output_lines += 1;
                    added += 1;
                }
            }
        }
        if hunks == 0 {
            return Err("file patch requires at least one hunk".into());
        }
        append_original(
            &original,
            (offsets[consumed], original.bytes.len()),
            &mut bytes,
            &mut origins,
        );
        // A no-newline line can occur only at the actual end of the output.
        let resulting = std::str::from_utf8(&bytes).map_err(|_| "patch produced non-UTF-8")?;
        if resulting.split_inclusive('\n').count() != output_lines + old_lines.len() - consumed {
            return Err("no-newline marker occurs before the last output line".into());
        }
        if let Some(path) = &old {
            after.remove(path)?;
        }
        match new {
            Some(path) => after.insert(
                &path,
                ProjectEntry {
                    bytes: bytes.into(),
                    origins,
                    ..original
                },
            )?,
            None if bytes.is_empty() => (),
            None => return Err("deletion patch did not remove the complete file".into()),
        }
    }
    if paths.is_empty() {
        return Err("empty patch".into());
    }
    Ok(PatchResult {
        before: tree.clone(),
        after,
        paths,
    })
}

fn syntax(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}
fn header(line: &str, marker: &str, prefix: &str) -> Result<Option<String>, String> {
    let path = syntax(line)
        .strip_prefix(marker)
        .ok_or_else(|| format!("expected {marker}header"))?;
    if path == "/dev/null" {
        return Ok(None);
    }
    let path = path
        .strip_prefix(prefix)
        .ok_or_else(|| format!("patch path must start with {prefix}"))?;
    validate_project_path(path)?;
    Ok(Some(path.into()))
}
fn line_index(start: usize, count: usize) -> Result<usize, String> {
    if count == 0 {
        Ok(start)
    } else {
        start
            .checked_sub(1)
            .ok_or_else(|| "nonempty hunk starts at line zero".into())
    }
}
fn range(value: &str, sign: char) -> Result<(usize, usize), String> {
    let value = value.strip_prefix(sign).ok_or("invalid hunk range")?;
    let (start, count) = value.split_once(',').map_or((value, "1"), |pair| pair);
    Ok((
        start.parse().map_err(|_| "invalid hunk line number")?,
        count.parse().map_err(|_| "invalid hunk count")?,
    ))
}
fn hunk_header(line: &str) -> Result<(usize, usize, usize, usize), String> {
    let line = syntax(line)
        .strip_prefix("@@ ")
        .ok_or("expected @@ hunk header")?;
    let (ranges, suffix) = line.split_once(" @@").ok_or("malformed hunk header")?;
    if !suffix.is_empty() && !suffix.starts_with(' ') {
        return Err("malformed hunk suffix".into());
    }
    let (old, new) = ranges.split_once(' ').ok_or("missing new hunk range")?;
    let (a, b) = range(old, '-')?;
    let (c, d) = range(new, '+')?;
    Ok((a, b, c, d))
}
fn append_original(
    entry: &ProjectEntry,
    range: (usize, usize),
    bytes: &mut Vec<u8>,
    origins: &mut Vec<ByteOrigin>,
) {
    copy_origins(entry, range, bytes.len(), origins);
    bytes.extend_from_slice(&entry.bytes[range.0..range.1]);
}
</book:entry>

</book:doc>
