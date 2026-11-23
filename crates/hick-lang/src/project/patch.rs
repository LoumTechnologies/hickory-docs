use super::tree::copy_origins;
use super::{ByteOrigin, EntryKind, ProjectEntry, ProjectTree, validate_project_path};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub before: ProjectTree,
    pub after: ProjectTree,
    pub paths: BTreeSet<String>,
    /// Validated context row source bytes and zero-based destination lines.
    pub context_lines: Vec<(String, usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchTarget {
    pub before: Option<String>,
    pub after: Option<String>,
}

struct PatchSection<'a> {
    target: PatchTarget,
    hunks: Vec<PatchHunk<'a>>,
}

struct PatchHunk<'a> {
    old_at: usize,
    lines: Vec<PatchLine<'a>>,
}

struct PatchLine<'a> {
    operation: u8,
    payload: &'a str,
    offset: usize,
}

/// Exact, transactional unified-diff replay. Diff payload line endings are data;
/// header line endings are syntax. No offset search, native IO, merge or model.
pub fn apply_project_patch(
    tree: &ProjectTree,
    diff: &str,
    document: &str,
    source_offset: usize,
) -> Result<PatchResult, String> {
    let (sections, paths) = parse_patch(diff)?;
    let mut after = tree.clone();
    let mut context_lines = Vec::new();
    for PatchSection { target: PatchTarget { before: old, after: new }, hunks } in sections {
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
        for PatchHunk { old_at, lines } in hunks {
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
            for PatchLine { operation, payload, offset } in lines {
                if operation != b'+' {
                    if old_lines.get(consumed).copied() != Some(payload) {
                        return Err(format!(
                            "wrong patch context at input line {}",
                            consumed + 1
                        ));
                    }
                    if operation == b' ' {
                        if let Some(path) = &new
                            && !payload.trim().is_empty()
                        {
                            context_lines.push((path.clone(), source_offset + offset, output_lines));
                        }
                        append_original(
                            &original,
                            (offsets[consumed], offsets[consumed + 1]),
                            &mut bytes,
                            &mut origins,
                        );
                    }
                    consumed += 1;
                }
                if operation != b'-' {
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
                }
            }
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
    Ok(PatchResult {
        before: tree.clone(),
        after,
        paths,
        context_lines,
    })
}

/// Enumerate exact header path pairs without opening or changing any source.
pub fn project_patch_targets(diff: &str) -> Result<Vec<PatchTarget>, String> {
    let (sections, _) = parse_patch(diff)?;
    Ok(sections.into_iter().map(|section| section.target).collect())
}

fn parse_patch(diff: &str) -> Result<(Vec<PatchSection<'_>>, BTreeSet<String>), String> {
    let mut lines = diff.split_inclusive('\n').scan(0, |offset, line| {
        let at = *offset;
        *offset += line.len();
        Some((line, at))
    }).peekable();
    let mut sections = Vec::new();
    let mut paths = BTreeSet::new();
    while let Some((line, _)) = lines.next() {
        let before = header(line, "--- ", "a/")?;
        let after = header(lines.next().ok_or("missing +++ header")?.0, "+++ ", "b/")?;
        let touched: BTreeSet<_> = before.iter().chain(&after).cloned().collect();
        if touched.is_empty() {
            return Err("patch cannot have two /dev/null paths".into());
        }
        if !paths.is_disjoint(&touched) {
            return Err("a file occurs twice in one patch act".into());
        }
        paths.extend(touched);

        let mut hunks = Vec::new();
        let (mut previous_old, mut previous_new) = (0usize, 0usize);
        while lines.peek().is_some_and(|(line, _)| !line.starts_with("--- ")) {
            let (old_start, old_count, new_start, new_count) = hunk_header(lines.next().unwrap().0)?;
            let old_at = line_index(old_start, old_count)?;
            let new_at = line_index(new_start, new_count)?;
            let untouched = old_at.checked_sub(previous_old)
                .ok_or("hunk offset is overlapping or outside input")?;
            if previous_new.checked_add(untouched) != Some(new_at) {
                return Err("new hunk offset does not match constructed output".into());
            }
            previous_old = old_at.checked_add(old_count).ok_or("hunk old count overflows")?;
            previous_new = new_at.checked_add(new_count).ok_or("hunk new count overflows")?;
            let (mut removed, mut added) = (0, 0);
            let mut payload = Vec::new();
            while removed < old_count || added < new_count {
                let (line, offset) = lines.next().ok_or("truncated hunk")?;
                let operation = *line.as_bytes().first().ok_or("empty hunk line")?;
                if !matches!(operation, b' ' | b'+' | b'-') {
                    return Err("hunk line must begin with space, + or -".into());
                }
                let text = &line[1..];
                let no_newline = lines.peek().is_some_and(|(next, _)| syntax(next) == "\\ No newline at end of file");
                let text = if no_newline {
                    lines.next();
                    text.strip_suffix('\n').ok_or("newline marker requires a terminated diff line")?
                } else {
                    if !text.ends_with('\n') {
                        return Err("unterminated diff payload needs a no-newline marker".into());
                    }
                    text
                };
                if operation != b'+' {
                    if removed == old_count { return Err("hunk exceeds old count".into()); }
                    removed += 1;
                }
                if operation != b'-' {
                    if added == new_count { return Err("hunk exceeds new count".into()); }
                    added += 1;
                }
                payload.push(PatchLine { operation, payload: text, offset });
            }
            hunks.push(PatchHunk { old_at, lines: payload });
        }
        if hunks.is_empty() {
            return Err("file patch requires at least one hunk".into());
        }
        sections.push(PatchSection { target: PatchTarget { before, after }, hunks });
    }
    if sections.is_empty() { return Err("empty patch".into()); }
    Ok((sections, paths))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_target_pairs_ignore_header_like_payload_and_support_resource_changes() {
        let diff = concat!(
            "--- a/a.txt\r\n+++ b/moved.txt\r\n@@ -1 +1 @@\r\n--- a/not-a-target\r\n+++ b/not-a-target\r\n",
            "--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1 @@\n+new\n\\ No newline at end of file\n",
            "--- a/delete.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n-old\n",
        );
        let targets = project_patch_targets(diff).unwrap();
        assert_eq!(targets, [
            PatchTarget { before: Some("a.txt".into()), after: Some("moved.txt".into()) },
            PatchTarget { before: None, after: Some("new.txt".into()) },
            PatchTarget { before: Some("delete.txt".into()), after: None },
        ]);
        let mut start = ProjectTree::default();
        start.insert("a.txt", ProjectEntry::file(b"-- a/not-a-target\r\n".as_slice())).unwrap();
        start.insert("delete.txt", ProjectEntry::file(b"old\n".as_slice())).unwrap();
        let replay = apply_project_patch(&start, diff, "patch.md", 0).unwrap();
        assert_eq!(replay.after.get("moved.txt").unwrap().bytes.as_ref(), b"++ b/not-a-target\r\n");
        assert_eq!(replay.after.get("new.txt").unwrap().bytes.as_ref(), b"new");
        assert!(replay.after.get("a.txt").is_none() && replay.after.get("delete.txt").is_none());
        let target_paths = targets.into_iter().flat_map(|target| target.before.into_iter().chain(target.after)).collect::<BTreeSet<_>>();
        assert_eq!(replay.paths, target_paths);
    }

    #[test]
    fn target_discovery_refuses_malformed_or_ambiguous_hunks_before_acquisition() {
        let valid = "--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-old\n+new\n";
        for diff in ["".into(), format!("{valid}{valid}"), valid.replace("-1 +1", "-1 +2"),
            valid.replace("-1 +1", "-1,2 +1"), valid.replace("a/a.txt", "a/../outside"),
            "--- /dev/null\n+++ /dev/null\n@@ -0,0 +0,0 @@\n".into(),
        ] {
            assert!(project_patch_targets(&diff).is_err(), "{diff:?}");
        }
    }
}
