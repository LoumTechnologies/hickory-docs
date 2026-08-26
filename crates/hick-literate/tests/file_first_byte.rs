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

async fn file_out(body: &str, path: &str) -> String {
    let src = hick_doc(body);
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .expect("pipeline runs");
    result
        .files
        .get(path)
        .unwrap_or_else(|| panic!("{path} was not produced"))
        .to_string()
}

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

#[tokio::test]
async fn content_on_the_tags_own_line_keeps_every_byte() {
    // There is no tag-line break to remove here, and taking a real byte
    // would corrupt the file.
    let out = file_out("<hick:file path=\"a.txt\">hello</hick:file>", "a.txt").await;
    assert_eq!(out, "hello");
}

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

#[tokio::test]
async fn a_windows_line_ending_counts_as_one_break() {
    let src = hick_doc("<hick:file path=\"a.txt\">\r\nvalue\n</hick:file>");
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .expect("pipeline runs");
    assert_eq!(result.files.get("a.txt").unwrap().to_string(), "value\n");
}
