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
        assert!(out.status.success(), "weaving {doc} failed:\n{stderr}");
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

/// The same mechanism read the other way round.
///
/// Folder-wide discovery is the feature; an `#id` that binds to a sibling
/// nobody named is its hazard. When a document's own `#imports` is renamed or
/// deleted and a sibling happens to declare one too, the paste keeps
/// resolving — to somebody else's bytes — and the tangled file is quietly
/// wrong. Twice in one repository, both times on a generic id.
// Protects docs/guarantees/authoring/an-ambient-id-paste-is-named.md
mod an_ambient_id_paste_is_named {
    use super::{folder, weave};

    const SIBLING: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="other.md">
<hick:copy id="imports" class="imports">import os</hick:copy>
</hick:doc>
"##;

    fn stderr(out: &std::process::Output) -> String {
        assert!(
            out.status.success(),
            "the warning must not stop the weave: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stderr).to_string()
    }

    #[test]
    fn an_id_that_resolved_from_a_sibling_is_named() {
        let dir = folder(&[
            (
                "owner.hick",
                r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:file path="dist/out.txt"><hick:paste select="#imports" /></hick:file>
</hick:doc>
"##,
            ),
            ("other.hick", SIBLING),
        ]);
        let text = stderr(&weave(&dir, "owner.hick"));
        assert!(text.contains("paste \"#imports\""), "{text}");
        assert!(text.contains("resolved from sibling other.hick"), "{text}");
        assert!(
            text.contains("<hick:upstream file=\"other.hick\" />"),
            "the warning says how to mean it: {text}"
        );
        // Still a warning: the bytes are written, because the binding may
        // well be what the author wants.
        let tangled = std::fs::read_to_string(dir.path().join("dist/out.txt")).unwrap();
        assert_eq!(tangled, "import os");
    }

    #[test]
    fn a_class_selector_is_never_warned_about() {
        // Collecting a sibling's contribution is what a `.class` is FOR.
        let dir = folder(&[
            (
                "owner.hick",
                r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:file path="dist/out.txt"><hick:paste select=".imports" /></hick:file>
</hick:doc>
"##,
            ),
            ("other.hick", SIBLING),
        ]);
        assert!(
            !stderr(&weave(&dir, "owner.hick")).contains("resolved from sibling"),
            "a class paste must not warn"
        );
    }

    #[test]
    fn a_declared_upstream_is_the_document_naming_it() {
        let dir = folder(&[
            (
                "owner.hick",
                r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:upstream file="other.hick" />
<hick:file path="dist/out.txt"><hick:paste select="#imports" /></hick:file>
</hick:doc>
"##,
            ),
            ("other.hick", SIBLING),
        ]);
        assert!(
            !stderr(&weave(&dir, "owner.hick")).contains("resolved from sibling"),
            "an upstream the author wrote IS naming it"
        );
    }

    #[test]
    fn a_documents_own_fragment_is_not_a_sibling_binding() {
        let dir = folder(&[
            (
                "owner.hick",
                r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:copy id="imports">import mine</hick:copy>
<hick:file path="dist/out.txt"><hick:paste select="#imports" /></hick:file>
</hick:doc>
"##,
            ),
            ("other.hick", SIBLING),
        ]);
        let dir_out = weave(&dir, "owner.hick");
        assert!(
            !stderr(&dir_out).contains("resolved from sibling"),
            "the document declares it itself"
        );
        // And the local declaration is what it got.
        let tangled = std::fs::read_to_string(dir.path().join("dist/out.txt")).unwrap();
        assert_eq!(tangled, "import mine");
    }
}
