<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Generated File Starts At Its First Byte

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given a `<hick:file path="…">` whose content begins on the line *after* the
open tag — which is how every readable document writes one — when the file is
produced, then its first byte is the first byte of the content, not the line
break that ended the tag's line.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-37b23a31581c"></a>
## `crates/hick-literate/tests/file_first_byte.rs`

Related contracts: [A Generated File Starts At Its First Byte](contract.md).

<a id="f-37b23a31581c-hick_doc"></a>
### `hick_doc`

A generated file starts at its first byte, not at the line break that
ended its open tag's line.
Found by dogfooding: Firefox refused an SVG this product generated with
"XML or text declaration not at start of entity", because the declaration
sat on line 2. The app drew the same file happily, which is the shape of
bug worth a test — right in the tool, broken everywhere else.

<storycode:copy id="f-37b23a31581c-hick_doc">
//! A generated file starts at its first byte, not at the line break that
//! ended its open tag's line.
//!
//! Guarantee: docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md
//!
//! Found by dogfooding: Firefox refused an SVG this product generated with
//! "XML or text declaration not at start of entity", because the declaration
//! sat on line 2. The app drew the same file happily, which is the shape of
//! bug worth a test — right in the tool, broken everywhere else.

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

</storycode:copy>
<a id="f-37b23a31581c-file_out"></a>
### `file_out`

<storycode:copy id="f-37b23a31581c-file_out">
async fn file_out(body: &str, path: &str) -> String {
    let src = hick_doc(body);
    let result = hick_literate::run_pipeline(&[("test.md", &src)], &[])
        .await
        .expect("pipeline runs");
    result
        .files
        .get(path)
        .unwrap_or_else(|| panic!("{path} was not produced"))
        .to_string()
}

</storycode:copy>
<a id="f-37b23a31581c-an_xml_declaration_lands_at_byte_zero"></a>
### `an_xml_declaration_lands_at_byte_zero`

<storycode:copy id="f-37b23a31581c-an_xml_declaration_lands_at_byte_zero">
#[tokio::test]
async fn an_xml_declaration_lands_at_byte_zero() {
    // The exact failure Firefox reported: a declaration on line 2 is not a
    // declaration at all.
    let out = file_out(
        "<hick:file path=\"chart.svg\">\n<?xml version=\"1.0\"?>\n<svg/>\n</hick:file>",
        "chart.svg",
    )
    .await;
    assert!(
        out.starts_with("<?xml"),
        "the declaration must be the file's first byte, got: {:?}",
        &out[..out.len().min(30)]
    );
}

</storycode:copy>
<a id="f-37b23a31581c-a_shebang_lands_on_line_one"></a>
### `a_shebang_lands_on_line_one`

<storycode:copy id="f-37b23a31581c-a_shebang_lands_on_line_one">
#[tokio::test]
async fn a_shebang_lands_on_line_one() {
    // A `#!` is only a shebang on the first line; one blank line above it and
    // the kernel will not run the script.
    let out = file_out(
        "<hick:file path=\"run.sh\">\n#!/bin/sh\necho hi\n</hick:file>",
        "run.sh",
    )
    .await;
    assert!(out.starts_with("#!/bin/sh"), "got: {out:?}");
}

</storycode:copy>
<a id="f-37b23a31581c-the_files_final_newline_survives"></a>
### `the_files_final_newline_survives`

<storycode:copy id="f-37b23a31581c-the_files_final_newline_survives">
#[tokio::test]
async fn the_files_final_newline_survives() {
    // Only the OPENING break is the tag's. The one before `</hick:file>` is
    // the file's own final newline, which a text file should have.
    let out = file_out(
        "<hick:file path=\"a.txt\">\nfirst\nlast\n</hick:file>",
        "a.txt",
    )
    .await;
    assert_eq!(out, "first\nlast\n");
}

</storycode:copy>
<a id="f-37b23a31581c-content_on_the_tags_own_line_keeps_every_byte"></a>
### `content_on_the_tags_own_line_keeps_every_byte`

<storycode:copy id="f-37b23a31581c-content_on_the_tags_own_line_keeps_every_byte">
#[tokio::test]
async fn content_on_the_tags_own_line_keeps_every_byte() {
    // There is no tag-line break to remove here, and taking a real byte
    // would corrupt the file.
    let out = file_out("<hick:file path=\"a.txt\">hello</hick:file>", "a.txt").await;
    assert_eq!(out, "hello");
}

</storycode:copy>
<a id="f-37b23a31581c-a_leading_blank_line_the_author_wrote_is_kept"></a>
### `a_leading_blank_line_the_author_wrote_is_kept`

<storycode:copy id="f-37b23a31581c-a_leading_blank_line_the_author_wrote_is_kept">
#[tokio::test]
async fn a_leading_blank_line_the_author_wrote_is_kept() {
    // Two breaks: one ends the tag's line, one is a blank line the author
    // typed. Only the first belongs to the tag.
    let out = file_out(
        "<hick:file path=\"a.txt\">\n\nafter a blank line\n</hick:file>",
        "a.txt",
    )
    .await;
    assert_eq!(out, "\nafter a blank line\n");
}

</storycode:copy>
<a id="f-37b23a31581c-a_windows_line_ending_counts_as_one_break"></a>
### `a_windows_line_ending_counts_as_one_break`

<storycode:copy id="f-37b23a31581c-a_windows_line_ending_counts_as_one_break">
#[tokio::test]
async fn a_windows_line_ending_counts_as_one_break() {
    let src = hick_doc("<hick:file path=\"a.txt\">\r\nvalue\n</hick:file>");
    let result = hick_literate::run_pipeline(&[("test.md", &src)], &[])
        .await
        .expect("pipeline runs");
    assert_eq!(result.files.get("a.txt").unwrap().to_string(), "value\n");
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/file_first_byte.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/file_first_byte.rs"><storycode:paste select="#f-37b23a31581c-hick_doc"/><storycode:paste select="#f-37b23a31581c-file_out"/><storycode:paste select="#f-37b23a31581c-an_xml_declaration_lands_at_byte_zero"/><storycode:paste select="#f-37b23a31581c-a_shebang_lands_on_line_one"/><storycode:paste select="#f-37b23a31581c-the_files_final_newline_survives"/><storycode:paste select="#f-37b23a31581c-content_on_the_tags_own_line_keeps_every_byte"/><storycode:paste select="#f-37b23a31581c-a_leading_blank_line_the_author_wrote_is_kept"/><storycode:paste select="#f-37b23a31581c-a_windows_line_ending_counts_as_one_break"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/adopt.rs](../../../mechanisms/crates/hickory-cli/src/adopt.rs.md#output-b820c141d77c)
- [crates/hickory-cli/src/ingest_exec.rs](../../../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#output-4e4b92369bcd)

### Tests and fixtures

- [crates/hick-literate/tests/file_first_byte.rs](contract.md#output-37b23a31581c)
- [crates/hick-literate/tests/picture_weave.rs](../../../mechanisms/crates/hick-literate/tests/picture_weave.rs.md#output-e26fcf8c737f)
- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#output-6682c6282b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
