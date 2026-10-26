<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Reject a patch that does not fit its exact input

For: Reviewers applying an agent explanation or maintaining a patch-based construction.

A patch expecting old text must fail if that text has changed. It must leave the input tree intact, including when a later hunk fails.

## What must happen

Given a virtual input tree and an explicitly declared UTF-8 unified patch, every hunk applies at its stated offsets with exact context. Invalid counts, overlaps, incorrect context, path collisions, unsupported headers, and incomplete deletions reject the whole act. The input tree is unchanged. BOM, CRLF, Unicode and final-newline differences remain data. Untouched bytes retain their origins; inserted bytes refer to the patch payload. Ordinary diff fences are display text.


## Limits of the evidence

Binary, Git extended-header and mode-only patch operations are unsupported. Explicit entries can construct binary content and metadata.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/replay-is-exact-and-atomic.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: State atomicity for the in-memory patch act; do not imply a filesystem transaction across all generated files.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-e9e76260c9bd"></a>
## `crates/hick-lang/src/project/patch.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Reject a patch that does not fit its exact input](contract.md).

<a id="f-e9e76260c9bd-PatchResult"></a>
### `PatchResult`

<storycode:copy id="f-e9e76260c9bd-PatchResult">
use super::tree::copy_origins;
use super::{ByteOrigin, EntryKind, ProjectEntry, ProjectTree, validate_project_path};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub before: ProjectTree,
    pub after: ProjectTree,
    pub paths: BTreeSet<String>,
}

</storycode:copy>
<a id="f-e9e76260c9bd-apply_project_patch"></a>
### Index and stage

Index diff lines with byte offsets, then clone the input tree as a candidate. Offset bookkeeping later gives inserted text a source location; the caller still owns an untouched input if replay fails.

<storycode:copy id="f-e9e76260c9bd-apply_project_patch">
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
</storycode:copy>
<a id="f-e9e76260c9bd-apply_project_patch--validate-file-endpoints"></a>
### Validate file endpoints

Validate old and new paths before reading a file. A file may occur only once in this act, symlinks cannot receive a text patch, and a rename cannot overwrite an existing destination. Build byte offsets for the exact old lines before applying hunks.

<storycode:copy id="f-e9e76260c9bd-apply_project_patch--validate-file-endpoints">
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
</storycode:copy>
<a id="f-e9e76260c9bd-apply_project_patch--check-hunk-positions"></a>
### Check hunk positions

Require each old range to begin after the preceding consumed input and each new range to agree with the output already built. Copy untouched prefixes with their origins; do not search for a nearby matching line.

<storycode:copy id="f-e9e76260c9bd-apply_project_patch--check-hunk-positions">
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
</storycode:copy>
<a id="f-e9e76260c9bd-apply_project_patch--replay-counted-payload"></a>
### Replay counted payload

Count old and new lines separately. Context and removal lines must match the input bytes exactly. A no-newline marker removes only the diff line terminator; inserted bytes retain a range pointing into the patch payload.

<storycode:copy id="f-e9e76260c9bd-apply_project_patch--replay-counted-payload">
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
</storycode:copy>
<a id="f-e9e76260c9bd-apply_project_patch--finish-and-publish"></a>
### Finish and publish

Refuse a file with no hunks or a no-newline marker before the final output line. A deletion must remove the complete file. Changes remain in the local candidate until the complete act returns its before/after pair; a later error never mutates the input tree.

<storycode:copy id="f-e9e76260c9bd-apply_project_patch--finish-and-publish">
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

</storycode:copy>

<a id="f-e9e76260c9bd-syntax"></a>
### `syntax`

Strip line endings only while reading diff syntax. Hunk payload line endings remain data, so CRLF and final-newline differences can still be checked exactly.

<storycode:copy id="f-e9e76260c9bd-syntax">
fn syntax(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}
</storycode:copy>
<a id="f-e9e76260c9bd-header"></a>
### `header`

Treat `/dev/null` as an absent endpoint. Otherwise require the expected `a/` or `b/` prefix and validate the remaining project path before accessing the tree.

<storycode:copy id="f-e9e76260c9bd-header">
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
</storycode:copy>
<a id="f-e9e76260c9bd-line_index"></a>
### `line_index`

Unified diff uses one-based positions for nonempty ranges, but an empty range names an insertion boundary. Keep those cases separate and reject a nonempty hunk starting at zero.

<storycode:copy id="f-e9e76260c9bd-line_index">
fn line_index(start: usize, count: usize) -> Result<usize, String> {
    if count == 0 {
        Ok(start)
    } else {
        start
            .checked_sub(1)
            .ok_or_else(|| "nonempty hunk starts at line zero".into())
    }
}
</storycode:copy>
<a id="f-e9e76260c9bd-range"></a>
### `range`

A missing count means one line. Require the old/new sign before parsing the position and count, so malformed headers cannot silently become an insertion or deletion.

<storycode:copy id="f-e9e76260c9bd-range">
fn range(value: &str, sign: char) -> Result<(usize, usize), String> {
    let value = value.strip_prefix(sign).ok_or("invalid hunk range")?;
    let (start, count) = value.split_once(',').map_or((value, "1"), |pair| pair);
    Ok((
        start.parse().map_err(|_| "invalid hunk line number")?,
        count.parse().map_err(|_| "invalid hunk count")?,
    ))
}
</storycode:copy>
<a id="f-e9e76260c9bd-hunk_header"></a>
### `hunk_header`

Read both endpoint ranges from the `@@` header before replay. A section label may follow the closing marker, but malformed suffixes and missing new ranges must fail.

<storycode:copy id="f-e9e76260c9bd-hunk_header">
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
</storycode:copy>
<a id="f-e9e76260c9bd-append_original"></a>
### `append_original`

Append the selected input bytes together with their translated origins. Copying only the text would preserve the visible result while losing the reverse-edit destination.

<storycode:copy id="f-e9e76260c9bd-append_original">
fn append_original(
    entry: &ProjectEntry,
    range: (usize, usize),
    bytes: &mut Vec<u8>,
    origins: &mut Vec<ByteOrigin>,
) {
    copy_origins(entry, range, bytes.len(), origins);
    bytes.extend_from_slice(&entry.bytes[range.0..range.1]);
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/patch.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/patch.rs"><storycode:paste select="#f-e9e76260c9bd-PatchResult"/><storycode:paste select="#f-e9e76260c9bd-apply_project_patch"/><storycode:paste select="#f-e9e76260c9bd-apply_project_patch--validate-file-endpoints"/><storycode:paste select="#f-e9e76260c9bd-apply_project_patch--check-hunk-positions"/><storycode:paste select="#f-e9e76260c9bd-apply_project_patch--replay-counted-payload"/><storycode:paste select="#f-e9e76260c9bd-apply_project_patch--finish-and-publish"/><storycode:paste select="#f-e9e76260c9bd-syntax"/><storycode:paste select="#f-e9e76260c9bd-header"/><storycode:paste select="#f-e9e76260c9bd-line_index"/><storycode:paste select="#f-e9e76260c9bd-range"/><storycode:paste select="#f-e9e76260c9bd-hunk_header"/><storycode:paste select="#f-e9e76260c9bd-append_original"/></storycode:file>

## Test the behavior: `patch_preserves_bom_crlf_unicode_and_untouched_origins`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-patch_preserves_bom_crlf_unicode_and_untouched_origins"></a>
### `patch_preserves_bom_crlf_unicode_and_untouched_origins`

Replace one Unicode line in a BOM/CRLF input and assert the complete expected bytes, including the untouched final line without a newline. Then check each origin range against the actual source bytes. Visible text equality alone would miss an incorrect reverse-edit mapping.

<storycode:copy id="f-005c1e9344c7-patch_preserves_bom_crlf_unicode_and_untouched_origins">
// Guarantee: docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
#[test]
fn patch_preserves_bom_crlf_unicode_and_untouched_origins() {
    let mut before = tree(&[("x", "\u{feff}é\r\nunchanged\r\nlast".as_bytes())]);
    let mut entry = before.get("x").unwrap().as_ref().clone();
    entry.origins.push(ByteOrigin {
        output: (0, entry.bytes.len()),
        document: "base.md".into(),
        source: (100, 100 + entry.bytes.len()),
    });
    before.insert("x", entry).unwrap();
    let diff = "--- a/x\n+++ b/x\n@@ -1,2 +1,2 @@\n-\u{feff}é\r\n+\u{feff}🦀\r\n unchanged\r\n";
    let patch = apply_project_patch(&before, diff, "patch.md", 40).unwrap();
    assert_eq!(
        &*patch.after.get("x").unwrap().bytes,
        "\u{feff}🦀\r\nunchanged\r\nlast".as_bytes()
    );
    assert_eq!(
        &*before.get("x").unwrap().bytes,
        "\u{feff}é\r\nunchanged\r\nlast".as_bytes()
    );
    for origin in &patch.after.get("x").unwrap().origins {
        let source = if origin.document == "patch.md" {
            &diff.as_bytes()[origin.source.0 - 40..origin.source.1 - 40]
        } else {
            &before.get("x").unwrap().bytes[origin.source.0 - 100..origin.source.1 - 100]
        };
        assert_eq!(
            &patch.after.get("x").unwrap().bytes[origin.output.0..origin.output.1],
            source
        );
    }
}
</storycode:copy>

## Test the behavior: `newline_markers_add_delete_rename_and_failure_are_atomic`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-newline_markers_add_delete_rename_and_failure_are_atomic"></a>
### `newline_markers_add_delete_rename_and_failure_are_atomic`

Exercise a rename, creation and complete deletion, then challenge incorrect context, offsets and newline markers. The unchanged input assertion checks the refusal boundary; these are in-memory tree operations, not a filesystem transaction.

<storycode:copy id="f-005c1e9344c7-newline_markers_add_delete_rename_and_failure_are_atomic">
// Guarantee: docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
#[test]
fn newline_markers_add_delete_rename_and_failure_are_atomic() {
    let before = tree(&[("x", b"old"), ("keep", b"same\n")]);
    let patch = apply_project_patch(&before,"--- a/x\n+++ b/renamed\n@@ -1 +1 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n--- /dev/null\n+++ b/added\n@@ -0,0 +1 @@\n+hello\n","p",0).unwrap();
    assert!(patch.after.get("x").is_none());
    assert_eq!(&*patch.after.get("renamed").unwrap().bytes, b"new");
    assert_eq!(&*patch.after.get("added").unwrap().bytes, b"hello\n");
    let deleted = apply_project_patch(
        &patch.after,
        "--- a/added\n+++ /dev/null\n@@ -1 +0,0 @@\n-hello\n",
        "p",
        0,
    )
    .unwrap();
    assert!(deleted.after.get("added").is_none());
    assert!(
        apply_project_patch(
            &before,
            "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-wrong\n+new\n",
            "p",
            0
        )
        .is_err()
    );
    assert_eq!(&*before.get("x").unwrap().bytes, b"old");
    for bad in [
        "--- a/x\n+++ b/x\n@@ -2 +1 @@\n-old\n+new\n",
        "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n", // missing marker
        "--- a/x\n+++ b/x\n@@ -1,1 +1,2 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n+next\n",
        "--- a/x\n+++ /dev/null\n@@ -0,0 +0,0 @@\n", // partial deletion
    ] {
        assert!(apply_project_patch(&before, bad, "p", 0).is_err(), "{bad}");
    }
}
</storycode:copy>

## Test the behavior: `raw_patch_grammar_and_ordinary_diff_fence`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-raw_patch_grammar_and_ordinary_diff_fence"></a>
### `raw_patch_grammar_and_ordinary_diff_fence`

<storycode:copy id="f-005c1e9344c7-raw_patch_grammar_and_ordinary_diff_fence">
// Guarantee: docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
#[test]
fn raw_patch_grammar_and_ordinary_diff_fence() {
    let source = "<hick:patch id=\"p\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-<hick:file path=\"nested\">\n+<hick:paste select=\"#code\"/>\n</hick:patch>";
    let doc = parse(source).unwrap();
    assert_eq!(doc.all_tags().len(), 1);
    assert_eq!(doc.find_tags("patch")[0].children.len(), 1);
    let source = "<hick:project/>\n```diff\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-x\n+y\n```";
    assert!(generate(source).unwrap().new.entries().is_empty());
}
</storycode:copy>

## Implementation review

The patch evaluator constructs a candidate tree in memory. It publishes that candidate only after the complete act succeeds. It never searches fuzzily for a similar place to apply a hunk.

### Implementation

- [crates/hick-lang/src/parser.rs](../../language/prefix-only-parsing-no-escaping/contract.md#output-0a2e1b69d493)
- [crates/hick-lang/src/project/patch.rs](contract.md#output-e9e76260c9bd)
- [crates/hick-lang/src/project/tree.rs](../../../mechanisms/crates/hick-lang/src/project/tree.rs.md#output-a485394d6759)

### Tests and fixtures

- [apps/web/src/elements/patch/view.test.tsx](../../../mechanisms/apps/web/src/elements/patch/view.tsx.md#output-059795295090)
- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)

### Focused witnesses

- `patch_preserves_bom_crlf_unicode_and_untouched_origins` in `crates/hick-lang/tests/project.rs`.
- `newline_markers_add_delete_rename_and_failure_are_atomic` in `crates/hick-lang/tests/project.rs`.

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
