//! A `<hick:copy>` weaves where it is declared; a `<hick:cut>` stays silent.
//!
//! The guide has said since it was written that the pair differs in exactly
//! this: *copy stores content and keeps it in the output; cut stores content
//! and removes it*. The weave did not, which made the two the same tag with
//! two names and left `docs/todo-app/implementation.md` with headings and
//! nothing beneath them.
//!
//! Guarantee: docs/guarantees/authoring/a-copy-weaves-where-it-is-declared.md
//!
//! Drives the real binary, because the claim is about the bytes that reach
//! the woven `.md`.

use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

/// Write `doc` as `report.hick`, run it, and return the woven markdown.
fn weave(doc: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("report.hick"), doc).expect("write doc");
    let out = hick()
        .arg("run")
        .arg(dir.path().join("report.hick"))
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "hick run failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("report.md")).expect("woven markdown");
    (dir, woven)
}

#[test]
fn a_copy_with_a_language_weaves_as_a_fenced_block_with_no_heading() {
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
## Storage

<hick:copy id="save" language="rust">
fn save(&self) -> Result<()> {
    Ok(())
}
</hick:copy>

That is the whole of it.
</hick:doc>
"##;
    let (_dir, woven) = weave(doc);
    assert!(
        woven.contains("```rust\nfn save(&self) -> Result<()> {\n    Ok(())\n}\n```"),
        "the fragment weaves as a clean rust fence, got:\n{woven}"
    );
    // No `### path` heading: a fragment has no path to name.
    assert!(
        !woven.contains("###"),
        "a copy is not a file and weaves no heading, got:\n{woven}"
    );
    assert!(
        woven.contains("## Storage"),
        "the prose around it is untouched, got:\n{woven}"
    );
}

#[test]
fn a_copy_without_a_language_weaves_its_children_as_prose() {
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
Before.

<hick:copy id="promise">Every order is refundable for thirty days.</hick:copy>

After.
</hick:doc>
"##;
    let (_dir, woven) = weave(doc);
    assert!(
        woven.contains("Every order is refundable for thirty days."),
        "prose in a copy is prose in the weave, got:\n{woven}"
    );
    assert!(
        !woven.contains("```"),
        "no fence without a language attribute, got:\n{woven}"
    );
}

#[test]
fn a_cut_still_weaves_nothing() {
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
Before.

<hick:cut id="hidden" language="rust">fn secret() {}</hick:cut>

After.
</hick:doc>
"##;
    let (_dir, woven) = weave(doc);
    assert!(
        !woven.contains("secret"),
        "a cut removes its content from the output, got:\n{woven}"
    );
    assert!(
        woven.contains("Before.\n\nAfter."),
        "and still leaves exactly one blank line, got:\n{woven}"
    );
}

#[test]
fn a_paste_inside_a_copy_resolves_when_the_copy_weaves() {
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:copy id="inner">let x = 1;</hick:copy>

<hick:copy id="outer" language="rust">
fn main() {
    <hick:paste select="#inner" />
}
</hick:copy>
</hick:doc>
"##;
    let (_dir, woven) = weave(doc);
    assert!(
        woven.contains("```rust\nfn main() {\n    let x = 1;\n}\n```"),
        "a fragment assembled from other fragments weaves assembled, got:\n{woven}"
    );
}

#[test]
fn a_copy_pasted_into_a_file_weaves_twice_and_tangles_once() {
    // The literate shape this exists for: the fragment is shown where the
    // prose explains it, and the file is assembled from pastes. Both appear
    // in the markdown; the file on disk is the assembled bytes only.
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
The saver:

<hick:copy id="save" language="rust">
pub fn save() {}
</hick:copy>

<hick:file path="lib.rs"><hick:paste select="#save" /></hick:file>
</hick:doc>
"##;
    let (dir, woven) = weave(doc);
    assert_eq!(
        woven.matches("pub fn save() {}").count(),
        2,
        "once where it is declared and once in the file's fence, got:\n{woven}"
    );
    // The tangled bytes are the fragment's OWN bytes, unchanged — including
    // the break after the open tag, which a paste has always kept and which
    // only the FENCE drops (`strip_opening_break`, the same rule a
    // `hick:file` body's display follows). Weaving a copy does not touch
    // what a paste of it produces.
    let tangled = std::fs::read_to_string(dir.path().join("lib.rs")).expect("lib.rs");
    assert_eq!(tangled, "\npub fn save() {}\n");
}
