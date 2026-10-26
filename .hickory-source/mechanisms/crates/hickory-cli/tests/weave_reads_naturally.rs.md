<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/weave_reads_naturally.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a118c221b58a"></a>
## `crates/hickory-cli/tests/weave_reads_naturally.rs`

Related contracts: [A Silent Tag Does Not Double A Blank Line](../../../../guarantees/authoring/a-silent-tag-does-not-double-a-blank-line/contract.md), [A Woven Fenced Block Has No Leading BOM](../../../../guarantees/verification/a-woven-fenced-block-has-no-leading-bom/contract.md).

<a id="f-a118c221b58a-hick"></a>
### `hick`

Two small "reads more naturally" fixes to woven markdown, found comparing
a real `.md` tutorial against a plain-Markdown control:
- A silent declaration tag (`hick:copy`, `hick:cut`, `hick:container`,
`hick:volume`, `hick:feature`, `hick:needs`, `hick:allow`) contributes
zero bytes, but the prose on either side still carries its OWN
paragraph-break newlines — three or more blank lines where a human who
left the tag out would have written one.
- `dotnet new` (and other .NET tooling) writes a UTF-8 BOM at the start
of generated source files; ingesting it correctly keeps the document's
own bytes byte-exact, but the WOVEN fenced code block showing that file
should not show three invisible-but-copy-pasteable bytes before the
first real character.
Drives the real binary because the claim is about what actually reaches
the woven `.md` file, not about a function in isolation.

<storycode:copy id="f-a118c221b58a-hick">
//! Two small "reads more naturally" fixes to woven markdown, found comparing
//! a real `.md` tutorial against a plain-Markdown control:
//!
//! - A silent declaration tag (`hick:copy`, `hick:cut`, `hick:container`,
//!   `hick:volume`, `hick:feature`, `hick:needs`, `hick:allow`) contributes
//!   zero bytes, but the prose on either side still carries its OWN
//!   paragraph-break newlines — three or more blank lines where a human who
//!   left the tag out would have written one.
//!   Guarantee: docs/guarantees/authoring/a-silent-tag-does-not-double-a-blank-line.md
//! - `dotnet new` (and other .NET tooling) writes a UTF-8 BOM at the start
//!   of generated source files; ingesting it correctly keeps the document's
//!   own bytes byte-exact, but the WOVEN fenced code block showing that file
//!   should not show three invisible-but-copy-pasteable bytes before the
//!   first real character.
//!   Guarantee: docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md
//!
//! Drives the real binary because the claim is about what actually reaches
//! the woven `.md` file, not about a function in isolation.

use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

</storycode:copy>
<a id="f-a118c221b58a-a_silent_tag_between_two_paragraphs_leaves_exactly_one_blank_line"></a>
### `a_silent_tag_between_two_paragraphs_leaves_exactly_one_blank_line`

<storycode:copy id="f-a118c221b58a-a_silent_tag_between_two_paragraphs_leaves_exactly_one_blank_line">
#[test]
fn a_silent_tag_between_two_paragraphs_leaves_exactly_one_blank_line() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
First paragraph.

<hick:copy id="unused">some text nobody pastes anywhere</hick:copy>

Second paragraph.
</hick:doc>
"##;
    std::fs::write(dir.path().join("report.md"), doc).expect("write doc");
    assert!(
        hick()
            .arg("run")
            .arg(dir.path().join("report.md"))
            .output()
            .expect("run hick")
            .status
            .success()
    );
    let woven = std::fs::read_to_string(dir.path().join("report.md")).unwrap();
    assert!(
        woven.contains("First paragraph.\n\nSecond paragraph."),
        "expected exactly one blank line between the two paragraphs, got:\n{woven}"
    );
    assert!(
        !woven.contains("\n\n\n"),
        "no run of 3+ newlines (2+ blank lines) should remain anywhere:\n{woven}"
    );
}

</storycode:copy>
<a id="f-a118c221b58a-two_consecutive_silent_tags_still_leave_exactly_one_blank_line"></a>
### `two_consecutive_silent_tags_still_leave_exactly_one_blank_line`

<storycode:copy id="f-a118c221b58a-two_consecutive_silent_tags_still_leave_exactly_one_blank_line">
#[test]
fn two_consecutive_silent_tags_still_leave_exactly_one_blank_line() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
Before both.

<hick:copy id="a">fragment a</hick:copy>

<hick:copy id="b">fragment b</hick:copy>

After both.
</hick:doc>
"##;
    std::fs::write(dir.path().join("report.md"), doc).expect("write doc");
    assert!(
        hick()
            .arg("run")
            .arg(dir.path().join("report.md"))
            .output()
            .expect("run hick")
            .status
            .success()
    );
    let woven = std::fs::read_to_string(dir.path().join("report.md")).unwrap();
    assert!(
        woven.contains("Before both.\n\nAfter both."),
        "two adjacent silent tags must still collapse to one blank line, got:\n{woven}"
    );
}

</storycode:copy>
<a id="f-a118c221b58a-a_real_paragraph_break_around_a_tag_that_does_render_is_left_alone"></a>
### `a_real_paragraph_break_around_a_tag_that_does_render_is_left_alone`

<storycode:copy id="f-a118c221b58a-a_real_paragraph_break_around_a_tag_that_does_render_is_left_alone">
#[test]
fn a_real_paragraph_break_around_a_tag_that_does_render_is_left_alone() {
    // The fix must never touch spacing around a tag that actually emits
    // something — only ones confirmed to contribute nothing.
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
Before the file.

<hick:file path="out.txt">hello
</hick:file>

After the file.
</hick:doc>
"##;
    std::fs::write(dir.path().join("report.md"), doc).expect("write doc");
    assert!(
        hick()
            .arg("run")
            .arg(dir.path().join("report.md"))
            .output()
            .expect("run hick")
            .status
            .success()
    );
    let woven = std::fs::read_to_string(dir.path().join("report.md")).unwrap();
    assert!(
        woven.contains("### `out.txt`"),
        "a hick:file still weaves its usual heading and fence:\n{woven}"
    );
}

</storycode:copy>
<a id="f-a118c221b58a-a_file_starting_with_a_bom_weaves_its_fenced_block_without_one"></a>
### `a_file_starting_with_a_bom_weaves_its_fenced_block_without_one`

<storycode:copy id="f-a118c221b58a-a_file_starting_with_a_bom_weaves_its_fenced_block_without_one">
#[test]
fn a_file_starting_with_a_bom_weaves_its_fenced_block_without_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"report.md\">\n\
        <hick:file path=\"out.cs\">\n\u{feff}// generated by a .NET tool\n</hick:file>\n\
        </hick:doc>\n";
    std::fs::write(dir.path().join("report.md"), doc).expect("write doc");
    assert!(
        hick()
            .arg("run")
            .arg(dir.path().join("report.md"))
            .output()
            .expect("run hick")
            .status
            .success()
    );

    // The document's OWN bytes, and the real output file, must both keep
    // the BOM — only the woven markdown's DISPLAY strips it.
    let source = std::fs::read_to_string(dir.path().join("report.md")).unwrap();
    assert!(
        source.contains("\u{feff}// generated"),
        "the document's own hick:file bytes must stay byte-exact"
    );
    let real_file = std::fs::read_to_string(dir.path().join("out.cs")).unwrap();
    assert!(
        real_file.starts_with('\u{feff}'),
        "the real output file must keep the BOM its content actually has"
    );

    let woven = std::fs::read_to_string(dir.path().join("report.md")).unwrap();
    assert!(
        woven.contains("```csharp\n// generated by a .NET tool\n```"),
        "the woven fenced block must not show a leading BOM, got:\n{woven}"
    );
    assert!(
        !woven.contains('\u{feff}'),
        "no BOM anywhere in the woven markdown"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/weave_reads_naturally.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/weave_reads_naturally.rs"><storycode:paste select="#f-a118c221b58a-hick"/><storycode:paste select="#f-a118c221b58a-a_silent_tag_between_two_paragraphs_leaves_exactly_one_blank_line"/><storycode:paste select="#f-a118c221b58a-two_consecutive_silent_tags_still_leave_exactly_one_blank_line"/><storycode:paste select="#f-a118c221b58a-a_real_paragraph_break_around_a_tag_that_does_render_is_left_alone"/><storycode:paste select="#f-a118c221b58a-a_file_starting_with_a_bom_weaves_its_fenced_block_without_one"/></storycode:file>

</storycode:doc>
