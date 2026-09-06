//! A cell knows where its document is.
//!
//! Guarantee: docs/guarantees/execution/a-cell-knows-where-its-document-is.md
//!
//! A cell's cwd is a scratch directory, which is what makes a run
//! reproducible and what stops a document from writing into the folder it
//! lives in. `HICK_DOC_DIR` and `HICK_DOC_PATH` are how a document reaches
//! the project it belongs to anyway — without an absolute path written into
//! a file people send each other.

use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
<hick:exec container="c" image="alpine">
echo "dir=$HICK_DOC_DIR"
echo "path=$HICK_DOC_PATH"
echo "cwd=$(pwd)"
</hick:exec>
</hick:doc>
"##;

#[test]
fn a_cell_is_told_its_documents_directory_and_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Canonicalised, because a temp dir is often behind a symlink (`/tmp` →
    // `/private/tmp` on macOS) and the variables promise the real path.
    let root = std::fs::canonicalize(dir.path()).expect("canonicalize");
    let doc = root.join("report.hick");
    std::fs::write(&doc, DOC).expect("write doc");

    let out = hick().arg("run").arg(&doc).output().expect("run hick");
    assert!(
        out.status.success(),
        "hick run failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let woven = std::fs::read_to_string(root.join("report.md")).expect("report.md");
    assert!(
        woven.contains(&format!("dir={}", root.display())),
        "HICK_DOC_DIR must be the document's own directory, got:\n{woven}"
    );
    assert!(
        woven.contains(&format!("path={}", doc.display())),
        "HICK_DOC_PATH must be the document file, got:\n{woven}"
    );
    // And the cwd is still the scratch directory: the variables tell a cell
    // where the document is, they do not move it there.
    assert!(
        !woven.contains(&format!("cwd={}", root.display())),
        "a cell still runs in its own scratch directory, got:\n{woven}"
    );
}

#[test]
fn a_bare_document_name_still_gets_an_absolute_directory() {
    // `Path::new("report.hick").parent()` is `Some("")`, not `None` — the
    // shape that has bitten the scratch-directory key before. The variable
    // must be the absolute directory the document is actually in, never an
    // empty string.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = std::fs::canonicalize(dir.path()).expect("canonicalize");
    std::fs::write(root.join("report.hick"), DOC).expect("write doc");

    let out = hick()
        .current_dir(&root)
        .arg("run")
        .arg("report.hick")
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "hick run failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let woven = std::fs::read_to_string(root.join("report.md")).expect("report.md");
    assert!(
        woven.contains(&format!("dir={}", root.display())),
        "a bare document name still names its real directory, got:\n{woven}"
    );
    assert!(
        !woven.contains("dir=\n"),
        "never an empty HICK_DOC_DIR, got:\n{woven}"
    );
}
