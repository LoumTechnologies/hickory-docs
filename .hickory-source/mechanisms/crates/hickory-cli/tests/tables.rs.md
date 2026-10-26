<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/tables.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e0de523c23a6"></a>
## `crates/hickory-cli/tests/tables.rs`

Related contracts: [A Table Is A Dataset And A Paragraph](../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [Cell formatting keeps numbers numeric](../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md).

[Read the test `hick` beside its requirement](../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md#f-e0de523c23a6-hick).
<a id="f-e0de523c23a6-run"></a>
### `run`

<storycode:copy id="f-e0de523c23a6-run">
fn run(source: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("sales.md");
    std::fs::write(&doc, source).unwrap();
    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("sales.md")).unwrap();
    (dir, woven)
}

</storycode:copy>
<a id="f-e0de523c23a6-INLINE"></a>
### `INLINE`

<storycode:copy id="f-e0de523c23a6-INLINE">
const INLINE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="sales.md">
# Sales

<hick:table>
region,units
north,120
south,90
</hick:table>
</hick:doc>
"##;

</storycode:copy>
<a id="f-e0de523c23a6-ENTANGLED"></a>
### `ENTANGLED`

<storycode:copy id="f-e0de523c23a6-ENTANGLED">
const ENTANGLED: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="sales.md">
# Sales

<hick:table path="sales.csv">
region,units
north,120
south,90
</hick:table>
</hick:doc>
"##;

</storycode:copy>
<a id="f-e0de523c23a6-a_table_weaves_to_markdown_not_to_a_fence_full_of_commas"></a>
### `a_table_weaves_to_markdown_not_to_a_fence_full_of_commas`

<storycode:copy id="f-e0de523c23a6-a_table_weaves_to_markdown_not_to_a_fence_full_of_commas">
#[test]
fn a_table_weaves_to_markdown_not_to_a_fence_full_of_commas() {
    // A reader opening the document wants the data, not the delimiters.
    let (_dir, woven) = run(INLINE);
    assert!(
        woven.contains("| region | units |"),
        "no header row:\n{woven}"
    );
    assert!(woven.contains("| --- | --- |"), "no rule:\n{woven}");
    assert!(woven.contains("| north | 120 |"), "no data row:\n{woven}");
    assert!(!woven.contains("```"), "weaved as a fence:\n{woven}");
    assert!(!woven.contains("hick:table"), "the tag leaked:\n{woven}");
}

</storycode:copy>
<a id="f-e0de523c23a6-a_table_with_a_path_writes_its_csv_exactly"></a>
### `a_table_with_a_path_writes_its_csv_exactly`

<storycode:copy id="f-e0de523c23a6-a_table_with_a_path_writes_its_csv_exactly">
#[test]
fn a_table_with_a_path_writes_its_csv_exactly() {
    // The dataset half. A script reading `sales.csv` must find CSV, not a
    // markdown table — that is what makes the document's data usable by
    // anything other than this app.
    let (dir, _woven) = run(ENTANGLED);
    let csv = std::fs::read_to_string(dir.path().join("sales.csv")).unwrap();
    assert!(csv.contains("region,units"), "not CSV:\n{csv}");
    assert!(csv.contains("north,120"), "not CSV:\n{csv}");
    assert!(
        !csv.contains('|'),
        "markdown leaked into the dataset:\n{csv}"
    );
}

</storycode:copy>
<a id="f-e0de523c23a6-a_table_with_a_path_still_weaves_as_a_table"></a>
### `a_table_with_a_path_still_weaves_as_a_table`

<storycode:copy id="f-e0de523c23a6-a_table_with_a_path_still_weaves_as_a_table">
#[test]
fn a_table_with_a_path_still_weaves_as_a_table() {
    // Both halves, from one block. This is the whole feature.
    let (_dir, woven) = run(ENTANGLED);
    assert!(woven.contains("| region | units |"), "{woven}");
}

</storycode:copy>
<a id="f-e0de523c23a6-a_table_without_a_path_writes_no_file"></a>
### `a_table_without_a_path_writes_no_file`

<storycode:copy id="f-e0de523c23a6-a_table_without_a_path_writes_no_file">
#[test]
fn a_table_without_a_path_writes_no_file() {
    // Prose that happens to be tabular. Writing a file nobody asked for
    // would litter the folder.
    let (dir, _woven) = run(INLINE);
    let names: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.ends_with(".csv")),
        "a file appeared: {names:?}"
    );
}

</storycode:copy>
<a id="f-e0de523c23a6-a_named_delimiter_is_honoured"></a>
### `a_named_delimiter_is_honoured`

<storycode:copy id="f-e0de523c23a6-a_named_delimiter_is_honoured">
#[test]
fn a_named_delimiter_is_honoured() {
    let source = INLINE
        .replace("<hick:table>", r##"<hick:table delimiter="tab">"##)
        .replace("region,units", "region\tunits")
        .replace("north,120", "north\t120")
        .replace("south,90", "south\t90");
    let (_dir, woven) = run(&source);
    assert!(woven.contains("| region | units |"), "{woven}");
}

</storycode:copy>
<a id="f-e0de523c23a6-a_headerless_table_keeps_its_first_row_as_data"></a>
### `a_headerless_table_keeps_its_first_row_as_data`

<storycode:copy id="f-e0de523c23a6-a_headerless_table_keeps_its_first_row_as_data">
#[test]
fn a_headerless_table_keeps_its_first_row_as_data() {
    let source = INLINE.replace("<hick:table>", r##"<hick:table header="false">"##);
    let (_dir, woven) = run(&source);
    assert!(
        woven.contains("| region | units |"),
        "the first row must still appear as DATA:\n{woven}"
    );
    // ...under an empty header rather than under invented column names.
    assert!(woven.contains("|  |  |"), "{woven}");
}

</storycode:copy>
<a id="f-e0de523c23a6-the_prose_around_a_table_is_untouched"></a>
### `the_prose_around_a_table_is_untouched`

<storycode:copy id="f-e0de523c23a6-the_prose_around_a_table_is_untouched">
#[test]
fn the_prose_around_a_table_is_untouched() {
    let (_dir, woven) = run(INLINE);
    assert!(woven.contains("# Sales"), "{woven}");
}

</storycode:copy>
[Read the test `number_formats_stay_in_the_document_and_out_of_the_csv_dataset` beside its requirement](../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md#f-e0de523c23a6-number_formats_stay_in_the_document_and_out_of_the_csv_dataset).

### Assemble `crates/hickory-cli/tests/tables.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/tables.rs"><storycode:paste select="#f-e0de523c23a6-hick"/><storycode:paste select="#f-e0de523c23a6-run"/><storycode:paste select="#f-e0de523c23a6-INLINE"/><storycode:paste select="#f-e0de523c23a6-ENTANGLED"/><storycode:paste select="#f-e0de523c23a6-a_table_weaves_to_markdown_not_to_a_fence_full_of_commas"/><storycode:paste select="#f-e0de523c23a6-a_table_with_a_path_writes_its_csv_exactly"/><storycode:paste select="#f-e0de523c23a6-a_table_with_a_path_still_weaves_as_a_table"/><storycode:paste select="#f-e0de523c23a6-a_table_without_a_path_writes_no_file"/><storycode:paste select="#f-e0de523c23a6-a_named_delimiter_is_honoured"/><storycode:paste select="#f-e0de523c23a6-a_headerless_table_keeps_its_first_row_as_data"/><storycode:paste select="#f-e0de523c23a6-the_prose_around_a_table_is_untouched"/><storycode:paste select="#f-e0de523c23a6-number_formats_stay_in_the_document_and_out_of_the_csv_dataset"/></storycode:file>

</storycode:doc>
