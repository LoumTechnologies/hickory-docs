<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Many Documents Contribute To One File, And None Of Them Names The Others

For: Authors editing executable documents and the files those documents produce.

Given `<hick:paste select=".ignore" />` in the document that owns a generated
file, when any other `.md` document **in the same folder** declares
`<hick:copy class="ignore">`, then that fragment is collected into the file —
and the owning document names no contributor, while the contributor names no
collector. They meet on the class.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/many-documents-contribute-to-one-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-480e62e44632"></a>
## `crates/hickory-cli/tests/ambient_contribution.rs`

Related contracts: [Many Documents Contribute To One File, And None Of Them Names The Others](contract.md).

<a id="f-480e62e44632-hick"></a>
### `hick`

Many documents contribute to one file, and none of them names the others:
a `<hick:copy class=…>` anywhere in the folder is collected by a
`<hick:paste select=".class">` in the document that owns the file, and
`<hick:private />` withholds a document's fragments.

<storycode:copy id="f-480e62e44632-hick">
//! Many documents contribute to one file, and none of them names the others:
//! a `<hick:copy class=…>` anywhere in the folder is collected by a
//! `<hick:paste select=".class">` in the document that owns the file, and
//! `<hick:private />` withholds a document's fragments.
//! Guarantee: docs/guarantees/authoring/many-documents-contribute-to-one-file.md

fn hick() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-480e62e44632-OWNER"></a>
### `OWNER`

<storycode:copy id="f-480e62e44632-OWNER">
const OWNER: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:file path="dist/ignore.txt"><hick:paste select=".ignore" distinct separator="," /></hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-480e62e44632-folder"></a>
### `folder`

<storycode:copy id="f-480e62e44632-folder">
fn folder(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, source) in files {
        std::fs::write(dir.path().join(name), source).unwrap();
    }
    dir
}

</storycode:copy>
<a id="f-480e62e44632-weave"></a>
### `weave`

<storycode:copy id="f-480e62e44632-weave">
fn weave(dir: &tempfile::TempDir, doc: &str) -> std::process::Output {
    hick()
        .args(["weave", dir.path().join(doc).to_str().unwrap()])
        .output()
        .unwrap()
}

</storycode:copy>
<a id="f-480e62e44632-a_sibling_contributes_without_either_document_naming_the_other"></a>
### `a_sibling_contributes_without_either_document_naming_the_other`

<storycode:copy id="f-480e62e44632-a_sibling_contributes_without_either_document_naming_the_other">
#[test]
fn a_sibling_contributes_without_either_document_naming_the_other() {
    let dir = folder(&[
        ("owner.md", OWNER),
        (
            "bot-a.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-a.md">
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
        (
            "bot-b.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-b.md">
<hick:copy class="ignore">node_modules/</hick:copy>
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.md").status.success());
    let out = std::fs::read_to_string(dir.path().join("dist/ignore.txt")).unwrap();
    // Sorted by filename, document order within each, and `distinct` drops
    // bot-b's repeat of bot-a's line.
    assert_eq!(out, "bin/,node_modules/");
}

</storycode:copy>
<a id="f-480e62e44632-a_private_document_keeps_its_fragments"></a>
### `a_private_document_keeps_its_fragments`

<storycode:copy id="f-480e62e44632-a_private_document_keeps_its_fragments">
#[test]
fn a_private_document_keeps_its_fragments() {
    let dir = folder(&[
        ("owner.md", OWNER),
        (
            "bot-a.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-a.md">
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
        (
            "secret.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="secret.md">
<hick:private />
<hick:copy class="ignore">do-not-share/</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.md").status.success());
    let out = std::fs::read_to_string(dir.path().join("dist/ignore.txt")).unwrap();
    assert_eq!(out, "bin/", "a private document contributed anyway");
}

</storycode:copy>
<a id="f-480e62e44632-lineage_names_the_contributing_document"></a>
### `lineage_names_the_contributing_document`

<storycode:copy id="f-480e62e44632-lineage_names_the_contributing_document">
#[test]
fn lineage_names_the_contributing_document() {
    let dir = folder(&[
        ("owner.md", OWNER),
        (
            "bot-a.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="bot-a.md">
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.md").status.success());
    let out = hick()
        .args([
            "lineage",
            dir.path().join("owner.md").to_str().unwrap(),
            "--output",
            "dist/ignore.txt",
        ])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&out.stdout);
    // The bytes belong to the contributor, not to the document that collected
    // them — otherwise a reverse edit lands in the wrong file.
    assert!(
        report.contains("bot-a.md"),
        "lineage did not name the contributing document:\n{report}"
    );
}

</storycode:copy>
<a id="f-480e62e44632-a_contributor_does_not_drag_in_its_own_upstreams"></a>
### `a_contributor_does_not_drag_in_its_own_upstreams`

<storycode:copy id="f-480e62e44632-a_contributor_does_not_drag_in_its_own_upstreams">
#[test]
fn a_contributor_does_not_drag_in_its_own_upstreams() {
    // The first build of this synthesised `hick:upstream` edges and ran them
    // back through the resolver, which made every document upstream of every
    // other — and the first explicit chain in the folder was then a cycle.
    let dir = folder(&[
        ("owner.md", OWNER),
        (
            "middle.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="middle.md">
<hick:upstream file="base.md" />
<hick:copy class="ignore">bin/</hick:copy>
</hick:doc>
"##,
        ),
        (
            "base.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="base.md">
<hick:copy id="base-note">a note</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    for doc in ["owner.md", "middle.md", "base.md"] {
        let out = weave(&dir, doc);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "weaving {doc} failed:\n{stderr}");
        assert!(
            !stderr.contains("circular"),
            "ambient contribution created a cycle weaving {doc}:\n{stderr}"
        );
    }
}

</storycode:copy>
<a id="f-480e62e44632-a_local_fragment_wins_over_a_contributed_one_of_the_same_id"></a>
### `a_local_fragment_wins_over_a_contributed_one_of_the_same_id`

<storycode:copy id="f-480e62e44632-a_local_fragment_wins_over_a_contributed_one_of_the_same_id">
#[test]
fn a_local_fragment_wins_over_a_contributed_one_of_the_same_id() {
    let dir = folder(&[
        (
            "owner.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="owner.md">
<hick:copy id="ver">mine</hick:copy>
<hick:file path="dist/v.txt"><hick:paste select="#ver" /></hick:file>
</hick:doc>
"##,
        ),
        (
            "other.md",
            r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="other.md">
<hick:copy id="ver">theirs</hick:copy>
</hick:doc>
"##,
        ),
    ]);
    assert!(weave(&dir, "owner.md").status.success());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("dist/v.txt")).unwrap(),
        "mine",
        "a contributed fragment overrode the document's own declaration"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/ambient_contribution.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/ambient_contribution.rs"><storycode:paste select="#f-480e62e44632-hick"/><storycode:paste select="#f-480e62e44632-OWNER"/><storycode:paste select="#f-480e62e44632-folder"/><storycode:paste select="#f-480e62e44632-weave"/><storycode:paste select="#f-480e62e44632-a_sibling_contributes_without_either_document_naming_the_other"/><storycode:paste select="#f-480e62e44632-a_private_document_keeps_its_fragments"/><storycode:paste select="#f-480e62e44632-lineage_names_the_contributing_document"/><storycode:paste select="#f-480e62e44632-a_contributor_does_not_drag_in_its_own_upstreams"/><storycode:paste select="#f-480e62e44632-a_local_fragment_wins_over_a_contributed_one_of_the_same_id"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)

### Tests and fixtures

- [crates/hickory-cli/tests/ambient_contribution.rs](contract.md#output-480e62e44632)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
