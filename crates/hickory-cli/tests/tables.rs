//! Tables: a dataset that is also prose.
//!
//! Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
//!
//! The two halves are the whole point. The CONTENT is CSV, so a script, a
//! spreadsheet, and a query can all read it; the WEAVE is a markdown table,
//! so a reader opening the document gets the data rather than the delimiters.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

fn run(source: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("sales.hick");
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

#[test]
fn a_table_with_a_path_still_weaves_as_a_table() {
    // Both halves, from one block. This is the whole feature.
    let (_dir, woven) = run(ENTANGLED);
    assert!(woven.contains("| region | units |"), "{woven}");
}

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

#[test]
fn the_prose_around_a_table_is_untouched() {
    let (_dir, woven) = run(INLINE);
    assert!(woven.contains("# Sales"), "{woven}");
}
