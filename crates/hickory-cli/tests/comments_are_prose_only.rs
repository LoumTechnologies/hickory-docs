//! `<!--` starts a comment in PROSE. Inside the body of an element that
//! holds bytes — `copy`, `cut`, `file`, `exec`, `expect`, `diagram`,
//! `table`, `math` — it is four characters of somebody's source file.
//!
//! Guarantee: docs/guarantees/language/prefix-only-parsing-no-escaping.md
//!
//! The promise there is that nothing but a prefixed tag is structure and
//! that there is no escaping to reach for. The comment scanner was the one
//! place the parser broke it, which meant a codebase could not be turned
//! into a literate document if any file in it contained `<!--` — an HTML
//! page, a Markdown fixture, a Rust string.
//!
//! Drives the real binary end to end, because the claim is about the bytes
//! that come out the other side of weave and tangle.

use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

fn run(dir: &std::path::Path, doc: &str) {
    std::fs::write(dir.join("report.hick"), doc).expect("write doc");
    let out = hick()
        .arg("run")
        .arg(dir.join("report.hick"))
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "hick run failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_comment_marker_in_a_copy_round_trips_byte_exact_into_a_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
The marker is data here:

<hick:copy id="frag" language="rust">const OPEN: &str = "<!-- x -->";
</hick:copy>

<hick:file path="lib.rs"><hick:paste select="#frag" /></hick:file>
</hick:doc>
"##;
    run(dir.path(), doc);

    // Tangled: exactly the bytes the fragment declared.
    let tangled = std::fs::read_to_string(dir.path().join("lib.rs")).expect("lib.rs");
    assert_eq!(tangled, "const OPEN: &str = \"<!-- x -->\";\n");

    // Woven: the same bytes, twice — once where the copy stands and once in
    // the file's fence.
    let woven = std::fs::read_to_string(dir.path().join("report.md")).expect("report.md");
    assert_eq!(
        woven.matches("const OPEN: &str = \"<!-- x -->\";").count(),
        2,
        "the marker must survive the weave too, got:\n{woven}"
    );
}

#[test]
fn an_html_file_may_contain_a_comment() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:file path="index.html"><body>
<!-- the nav lives here -->
</body>
</hick:file>
</hick:doc>
"##;
    run(dir.path(), doc);
    let page = std::fs::read_to_string(dir.path().join("index.html")).expect("index.html");
    assert_eq!(page, "<body>\n<!-- the nav lives here -->\n</body>\n");
}

#[test]
fn an_unclosed_marker_in_a_file_is_not_an_unclosed_comment() {
    // Before this, a file holding the four characters `<!--` — with no
    // `-->` anywhere after it — failed the whole document with
    // `unclosed comment`.
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:file path="a.rs">const OPEN: &str = "<!--";
</hick:file>
</hick:doc>
"##;
    run(dir.path(), doc);
    let file = std::fs::read_to_string(dir.path().join("a.rs")).expect("a.rs");
    assert_eq!(file, "const OPEN: &str = \"<!--\";\n");
}

#[test]
fn a_hick_file_block_can_still_be_commented_out_from_outside() {
    // The comment starts in PROSE, so it is a comment — including the
    // `<hick:file>` inside it, which must not be written.
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:file path="keep.rs">kept
</hick:file>
<!--
<hick:file path="removed.rs">this should not appear</hick:file>
-->
</hick:doc>
"##;
    run(dir.path(), doc);
    assert!(dir.path().join("keep.rs").exists());
    assert!(
        !dir.path().join("removed.rs").exists(),
        "a commented-out file block writes nothing"
    );
}
