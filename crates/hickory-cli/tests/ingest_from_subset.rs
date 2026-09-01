//! `hick ingest --from` runs only the target cell's dependencies, not the
//! whole document.
//!
//! Guarantee:
//! docs/guarantees/authoring/ingest-does-not-require-the-rest-of-the-document-to-already-pass.md
//!
//! Drives the real binary: the claim is about whether the CLI command
//! succeeds against a real document with a real broken cell elsewhere in
//! it, not about a function in isolation.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

fn repo(dir: &Path) {
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("run git");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "T"]);
}

/// The target cell (to be ingested) comes FIRST — the ordinary authoring
/// order, a cell written before the rest of the document that will depend
/// on it exists. A second, unrelated cell AFTER it fails outright: it has
/// no dependency relationship with the target at all.
fn doc() -> &'static str {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="sdk" />
<hick:volume name="project" output="app" />

<hick:exec container="sdk" mount="project:out">
<hick:copy id="scaffold">
mkdir -p out && printf 'hello\n' > out/main.txt
</hick:copy>
</hick:exec>

<hick:exec container="sdk">
this-command-does-not-exist-and-fails
</hick:exec>
</hick:doc>
"##
}

#[test]
fn ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken() {
    let dir = tempfile::tempdir().expect("tempdir");
    repo(dir.path());
    let doc_path = dir.path().join("app.hick");
    std::fs::write(&doc_path, doc()).expect("write doc");

    // Confirm the premise first: running the WHOLE document really does
    // fail, because the second cell really is broken. If this assertion
    // ever stops holding, the test below proves nothing.
    let whole_run = hick().arg("run").arg(&doc_path).output().expect("run hick");
    assert!(
        !whole_run.status.success(),
        "the premise of this test requires the whole document to fail"
    );

    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc_path)
        .output()
        .expect("run hick ingest");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "ingest must succeed on the target cell's own subgraph, \
         independent of an unrelated cell elsewhere in the document: \
         {stderr}\n{stdout}"
    );

    let source = std::fs::read_to_string(&doc_path).unwrap();
    assert!(
        source.contains("<hick:ingested from=\"#scaffold\""),
        "{source}"
    );
    assert!(source.contains("path=\"app/main.txt\""), "{source}");
}
