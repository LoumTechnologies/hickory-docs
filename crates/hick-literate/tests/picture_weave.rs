//! A file block that writes a picture weaves as the picture.
//!
//! Guarantee: docs/guarantees/authoring/a-generated-picture-shows-as-a-picture.md
//!
//! Found by dogfooding: showing the block as its chart in the editor made the
//! document show each chart twice — once for the block, once for the
//! hand-written `![…](chart.svg)` beneath it. That line existed because the
//! weave had no other way to put a picture in the markdown, and fencing an
//! SVG puts forty kilobytes of markup in the middle of a document.

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
{body}
</hick:doc>"#
    )
}

async fn woven(body: &str) -> String {
    let src = hick_doc(body);
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .expect("pipeline runs");
    result
        .files
        .get("out.md")
        .expect("the weave target was produced")
        .to_string()
}

#[tokio::test]
async fn a_picture_file_weaves_as_a_markdown_image() {
    let out = woven("<hick:file path=\"chart.svg\">\n<svg/>\n</hick:file>").await;
    assert!(
        out.contains("![chart.svg](chart.svg)"),
        "the woven markdown must reference the picture: {out}"
    );
    // And NOT as a fence: the bytes of an SVG are not something a reader
    // reads, and dumping them is what forced the hand-written image line.
    assert!(
        !out.contains("```"),
        "a picture must not weave as a code fence: {out}"
    );
    assert!(
        !out.contains("### `chart.svg`"),
        "a picture needs no source heading: {out}"
    );
}

#[tokio::test]
async fn every_picture_extension_is_recognised() {
    for ext in ["svg", "png", "jpg", "jpeg", "gif", "webp", "avif"] {
        let out = woven(&format!(
            "<hick:file path=\"a.{ext}\">\nbytes\n</hick:file>"
        ))
        .await;
        assert!(
            out.contains(&format!("![a.{ext}](a.{ext})")),
            ".{ext} must weave as a picture: {out}"
        );
    }
}

#[tokio::test]
async fn a_text_file_still_weaves_as_its_source() {
    // The fence exists to show a file's source, and a `.py` has one worth
    // reading. Only a picture loses it.
    let out = woven("<hick:file path=\"app.py\">\nprint(1)\n</hick:file>").await;
    assert!(out.contains("### `app.py`"), "{out}");
    assert!(out.contains("```python"), "{out}");
    assert!(out.contains("print(1)"), "{out}");
}

#[tokio::test]
async fn the_fence_shows_the_files_real_first_line() {
    // The fence and the file must agree about the file's first byte, or the
    // fence is lying about the one thing it exists to show. See
    // docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md.
    let src =
        hick_doc("<hick:file path=\"app.py\">\n#!/usr/bin/env python\nprint(1)\n</hick:file>");
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .expect("pipeline runs");
    let file = result.files.get("app.py").unwrap().to_string();
    let md = result.files.get("out.md").unwrap().to_string();
    assert!(file.starts_with("#!/usr/bin/env python"), "{file:?}");
    assert!(
        md.contains("```python\n#!/usr/bin/env python"),
        "the fence must open on the file's own first line: {md}"
    );
}

#[tokio::test]
async fn a_hidden_picture_still_weaves_nothing() {
    // `doc-hidden` is unchanged and still wins: a document that wants the
    // file written but not shown keeps saying so.
    let out =
        woven("<hick:file path=\"chart.svg\" doc-hidden=\"true\">\n<svg/>\n</hick:file>").await;
    assert!(!out.contains("chart.svg"), "{out}");
}
