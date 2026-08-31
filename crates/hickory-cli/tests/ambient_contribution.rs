//! Many documents contribute to one file, and none of them names the others:
//! a `<hick:copy class=…>` anywhere in the folder is collected by a
//! `<hick:paste select=".class">` in the document that owns the file, and
//! `<hick:private />` withholds a document's fragments.
//! Guarantee: docs/guarantees/authoring/many-documents-contribute-to-one-file.md

fn hick() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
}

const OWNER: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:file path="dist/ignore.txt"><hick:paste select=".ignore" distinct separator="," /></hick:file>
</hick:doc>
"##;

fn folder(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, source) in files {
        std::fs::write(dir.path().join(name), source).unwrap();
    }
    dir
}

fn weave(dir: &tempfile::TempDir, doc: &str) -> std::process::Output {
    hick()
        .args(["weave", dir.path().join(doc).to_str().unwrap()])
        .output()
        .unwrap()
}

#[test]
fn a_sibling_contributes_without_either_document_naming_the_other() {
    let dir = folder(&[
        ("owner.hick", OWNER),
        (
            "bot-a.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-a.md">
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
        (
            "bot-b.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-b.md">
<hick:copy class="ignore">node_modules/</hick:copy>
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.hick").status.success());
    let out = std::fs::read_to_string(dir.path().join("dist/ignore.txt")).unwrap();
    // Sorted by filename, document order within each, and `distinct` drops
    // bot-b's repeat of bot-a's line.
    assert_eq!(out, "bin/,node_modules/");
}

#[test]
fn a_private_document_keeps_its_fragments() {
    let dir = folder(&[
        ("owner.hick", OWNER),
        (
            "bot-a.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-a.md">
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
        (
            "secret.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="secret.md">
<hick:private />
<hick:copy class="ignore">do-not-share/</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.hick").status.success());
    let out = std::fs::read_to_string(dir.path().join("dist/ignore.txt")).unwrap();
    assert_eq!(out, "bin/", "a private document contributed anyway");
}

#[test]
fn lineage_names_the_contributing_document() {
    let dir = folder(&[
        ("owner.hick", OWNER),
        (
            "bot-a.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-a.md">
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.hick").status.success());
    let out = hick()
        .args([
            "lineage",
            dir.path().join("owner.hick").to_str().unwrap(),
            "--output",
            "dist/ignore.txt",
        ])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&out.stdout);
    // The bytes belong to the contributor, not to the document that collected
    // them — otherwise a reverse edit lands in the wrong file.
    assert!(
        report.contains("bot-a.hick"),
        "lineage did not name the contributing document:\n{report}"
    );
}

#[test]
fn a_contributor_does_not_drag_in_its_own_upstreams() {
    // The first build of this synthesised `hick:upstream` edges and ran them
    // back through the resolver, which made every document upstream of every
    // other — and the first explicit chain in the folder was then a cycle.
    let dir = folder(&[
        ("owner.hick", OWNER),
        (
            "middle.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="middle.md">
<hick:upstream file="base.hick" />
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
        (
            "base.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="base.md">
<hick:copy id="base-note">a note</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    for doc in ["owner.hick", "middle.hick", "base.hick"] {
        let out = weave(&dir, doc);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "weaving {doc} failed:\n{stderr}"
        );
        assert!(
            !stderr.contains("circular"),
            "ambient contribution created a cycle weaving {doc}:\n{stderr}"
        );
    }
}

#[test]
fn a_local_fragment_wins_over_a_contributed_one_of_the_same_id() {
    let dir = folder(&[
        (
            "owner.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:copy id="ver">mine</hick:copy>
<hick:file path="dist/v.txt"><hick:paste select="#ver" /></hick:file>
</hick:doc>
"##,
        ),
        (
            "other.hick",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="other.md">
<hick:copy id="ver">theirs</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.hick").status.success());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("dist/v.txt")).unwrap(),
        "mine",
        "a contributed fragment overrode the document's own declaration"
    );
}
