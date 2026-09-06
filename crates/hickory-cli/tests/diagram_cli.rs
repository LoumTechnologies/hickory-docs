//! `hick diagram`: the deterministic, AI-free way a diagram gets authored.
//!
//! Protects docs/guarantees/authoring/a-diagram-the-code-can-regenerate.md.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

/// A two-directory fixture with one cross-directory call.
fn fixture(dir: &Path) {
    std::fs::create_dir_all(dir.join("app")).unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("lib/util.py"), "def load():\n    pass\n").unwrap();
    std::fs::write(dir.join("app/main.py"), "load()\n").unwrap();
}

#[test]
fn emits_a_scene_topology_with_no_layout() {
    let dir = tempfile::tempdir().unwrap();
    fixture(dir.path());
    // Run FROM the folder, the way a cell mounting it does: a node's `path`
    // is spelled relative to the working directory, because that is where
    // the document pasting this topology lives.
    let out = hick()
        .current_dir(dir.path())
        .arg("diagram")
        .arg(".")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Nodes and the cross-directory edge; NEVER a layout — positions are the
    // person's half, and a generator that writes them destroys arrangements.
    assert!(stdout.contains("\"app\""), "{stdout}");
    assert!(stdout.contains("\"lib\""), "{stdout}");
    assert!(stdout.contains("\"edges\""), "{stdout}");
    assert!(!stdout.contains("layout"), "{stdout}");
    // Every node says where it lives, so a box in the picture is a way in.
    assert!(stdout.contains("\"path\": \"app\""), "{stdout}");
    assert!(stdout.contains("\"path\": \"lib\""), "{stdout}");
}

/// Protects docs/guarantees/authoring/a-diagram-node-opens-what-it-stands-for.md
#[test]
fn a_scanned_folder_is_named_from_the_working_directory() {
    let parent = tempfile::tempdir().unwrap();
    let dir = parent.path().join("demo");
    std::fs::create_dir_all(&dir).unwrap();
    fixture(&dir);
    // `hick diagram demo` beside the document: the paths read `demo/app`,
    // which is exactly how the document names the same folder.
    let out = hick()
        .current_dir(parent.path())
        .arg("diagram")
        .arg("demo")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"path\": \"demo/app\""), "{stdout}");
    assert!(stdout.contains("\"path\": \"demo/lib\""), "{stdout}");

    // `--group file` names the file itself, same rule.
    let out = hick()
        .current_dir(parent.path())
        .arg("diagram")
        .arg("demo")
        .arg("--group")
        .arg("file")
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("\"path\": \"demo/app/main.py\""),
        "{stdout}"
    );
}

#[test]
fn mermaid_output_is_the_same_downgrade_the_weave_uses() {
    let dir = tempfile::tempdir().unwrap();
    fixture(dir.path());
    let out = hick()
        .current_dir(dir.path())
        .arg("diagram")
        .arg(".")
        .arg("--format")
        .arg("mermaid")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("flowchart TD"), "{stdout}");
    assert!(stdout.contains("app --> lib"), "{stdout}");
    // The downgrade drops where a node lives, as it drops every other thing
    // markdown has nowhere honest to keep.
    assert!(!stdout.contains("path"), "{stdout}");
}

#[test]
fn refresh_rewrites_the_fragment_and_leaves_the_layout_alone() {
    let dir = tempfile::tempdir().unwrap();
    fixture(dir.path());
    let doc = dir.path().join("arch.hick");
    std::fs::write(
        &doc,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"arch.md\">\n\
         <hick:copy id=\"arch-topology\">{\"nodes\":[],\"edges\":[]}</hick:copy>\n\n\
         <hick:diagram renderer=\"graph\">\n\
         {\n  \"topology\": <hick:paste select=\"#arch-topology\" />,\n  \"layout\": {\"app\": {\"x\": 7, \"y\": 9}}\n}\n\
         </hick:diagram>\n\
         </hick:doc>\n",
    )
    .unwrap();

    let out = hick()
        .current_dir(dir.path())
        .arg("diagram")
        .arg(".")
        .arg("--refresh")
        .arg("arch.hick")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let rewritten = std::fs::read_to_string(&doc).unwrap();
    // The fragment now holds the real topology…
    assert!(rewritten.contains(r#""id":"app""#), "{rewritten}");
    // …and the person's layout was not touched.
    assert!(
        rewritten.contains(r#""app": {"x": 7, "y": 9}"#),
        "{rewritten}"
    );

    // The refreshed document weaves: the derived diagram draws the new
    // nodes into its SVG.
    let weave = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        weave.status.success(),
        "{}",
        String::from_utf8_lossy(&weave.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(woven.contains("![diagram](diagram-"), "{woven}");
    let name = woven
        .split("![diagram](")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("an image reference");
    let svg = std::fs::read_to_string(dir.path().join(name)).expect("the SVG exists");
    assert!(
        svg.contains(">app</text>") && svg.contains(">lib</text>"),
        "{svg}"
    );
    // …and each node is a LINK in the woven picture, so the markdown
    // navigates on GitHub too. The href is relative to the woven file's own
    // directory, which is where the weave puts both the `.md` and this SVG.
    assert!(svg.contains(r#"<a href="app">"#), "{svg}");
    assert!(svg.contains(r#"<a href="lib">"#), "{svg}");
}

#[test]
fn a_folder_with_nothing_to_draw_says_so_and_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not source\n").unwrap();
    let out = hick().arg("diagram").arg(dir.path()).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("nothing to draw"), "{stderr}");
}
