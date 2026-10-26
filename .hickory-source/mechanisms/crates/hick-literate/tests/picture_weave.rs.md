<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/picture_weave.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e26fcf8c737f"></a>
## `crates/hick-literate/tests/picture_weave.rs`

Related contracts: [A Generated Picture Shows As A Picture](../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md), [A Generated File Starts At Its First Byte](../../../../guarantees/language/a-generated-file-starts-at-its-first-byte/contract.md).

[Read the test `hick_doc` beside its requirement](../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md#f-e26fcf8c737f-hick_doc).
<a id="f-e26fcf8c737f-woven"></a>
### `woven`

<storycode:copy id="f-e26fcf8c737f-woven">
async fn woven(body: &str) -> String {
    let src = hick_doc(body);
    let result = hick_literate::run_pipeline(&[("test.md", &src)], &[])
        .await
        .expect("pipeline runs");
    result
        .files
        .get("out.md")
        .expect("the weave target was produced")
        .to_string()
}

</storycode:copy>
<a id="f-e26fcf8c737f-a_picture_file_weaves_as_a_markdown_image"></a>
### `a_picture_file_weaves_as_a_markdown_image`

<storycode:copy id="f-e26fcf8c737f-a_picture_file_weaves_as_a_markdown_image">
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

</storycode:copy>
<a id="f-e26fcf8c737f-every_picture_extension_is_recognised"></a>
### `every_picture_extension_is_recognised`

<storycode:copy id="f-e26fcf8c737f-every_picture_extension_is_recognised">
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

</storycode:copy>
<a id="f-e26fcf8c737f-a_text_file_still_weaves_as_its_source"></a>
### `a_text_file_still_weaves_as_its_source`

<storycode:copy id="f-e26fcf8c737f-a_text_file_still_weaves_as_its_source">
#[tokio::test]
async fn a_text_file_still_weaves_as_its_source() {
    // The fence exists to show a file's source, and a `.py` has one worth
    // reading. Only a picture loses it.
    let out = woven("<hick:file path=\"app.py\">\nprint(1)\n</hick:file>").await;
    assert!(out.contains("### `app.py`"), "{out}");
    assert!(out.contains("```python"), "{out}");
    assert!(out.contains("print(1)"), "{out}");
}

</storycode:copy>
<a id="f-e26fcf8c737f-the_fence_shows_the_files_real_first_line"></a>
### `the_fence_shows_the_files_real_first_line`

<storycode:copy id="f-e26fcf8c737f-the_fence_shows_the_files_real_first_line">
#[tokio::test]
async fn the_fence_shows_the_files_real_first_line() {
    // The fence and the file must agree about the file's first byte, or the
    // fence is lying about the one thing it exists to show. See
    // docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md.
    let src =
        hick_doc("<hick:file path=\"app.py\">\n#!/usr/bin/env python\nprint(1)\n</hick:file>");
    let result = hick_literate::run_pipeline(&[("test.md", &src)], &[])
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

</storycode:copy>
<a id="f-e26fcf8c737f-a_hidden_picture_still_weaves_nothing"></a>
### `a_hidden_picture_still_weaves_nothing`

<storycode:copy id="f-e26fcf8c737f-a_hidden_picture_still_weaves_nothing">
#[tokio::test]
async fn a_hidden_picture_still_weaves_nothing() {
    // `doc-hidden` is unchanged and still wins: a document that wants the
    // file written but not shown keeps saying so.
    let out =
        woven("<hick:file path=\"chart.svg\" doc-hidden=\"true\">\n<svg/>\n</hick:file>").await;
    assert!(!out.contains("chart.svg"), "{out}");
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/picture_weave.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/picture_weave.rs"><storycode:paste select="#f-e26fcf8c737f-hick_doc"/><storycode:paste select="#f-e26fcf8c737f-woven"/><storycode:paste select="#f-e26fcf8c737f-a_picture_file_weaves_as_a_markdown_image"/><storycode:paste select="#f-e26fcf8c737f-every_picture_extension_is_recognised"/><storycode:paste select="#f-e26fcf8c737f-a_text_file_still_weaves_as_its_source"/><storycode:paste select="#f-e26fcf8c737f-the_fence_shows_the_files_real_first_line"/><storycode:paste select="#f-e26fcf8c737f-a_hidden_picture_still_weaves_nothing"/></storycode:file>

</storycode:doc>
