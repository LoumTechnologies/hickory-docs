<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Ingest Does Not Require The Rest Of The Document To Already Pass

For: Authors editing executable documents and the files those documents produce.

Given a document whose target cell (the one named by `hick ingest --from
'#id'`) does not depend on some OTHER cell elsewhere in the document, when
that other cell currently fails, then the ingest still succeeds — it runs
only the target cell and its real dependencies, never the whole document.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/ingest-does-not-require-the-rest-of-the-document-to-already-pass.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-5cc206fde6e4"></a>
## `crates/hickory-cli/tests/ingest_from_subset.rs`

Related contracts: [Ingest Does Not Require The Rest Of The Document To Already Pass](contract.md).

<a id="f-5cc206fde6e4-hick"></a>
### `hick`

`hick ingest --from` runs only the target cell's dependencies, not the
whole document.
Drives the real binary: the claim is about whether the CLI command
succeeds against a real document with a real broken cell elsewhere in
it, not about a function in isolation.

<storycode:copy id="f-5cc206fde6e4-hick">
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

</storycode:copy>
<a id="f-5cc206fde6e4-repo"></a>
### `repo`

<storycode:copy id="f-5cc206fde6e4-repo">
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

</storycode:copy>
<a id="f-5cc206fde6e4-doc"></a>
### `doc`

The target cell (to be ingested) comes FIRST — the ordinary authoring
order, a cell written before the rest of the document that will depend
on it exists. A second, unrelated cell AFTER it fails outright: it has
no dependency relationship with the target at all.

<storycode:copy id="f-5cc206fde6e4-doc">
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

</storycode:copy>
<a id="f-5cc206fde6e4-ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken"></a>
### `ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken`

<storycode:copy id="f-5cc206fde6e4-ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken">
#[test]
fn ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken() {
    let dir = tempfile::tempdir().expect("tempdir");
    repo(dir.path());
    let doc_path = dir.path().join("app.md");
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/ingest_from_subset.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/ingest_from_subset.rs"><storycode:paste select="#f-5cc206fde6e4-hick"/><storycode:paste select="#f-5cc206fde6e4-repo"/><storycode:paste select="#f-5cc206fde6e4-doc"/><storycode:paste select="#f-5cc206fde6e4-ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/ingest_exec.rs](../../../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#output-4e4b92369bcd)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hickory-cli/tests/ingest_from_subset.rs](contract.md#output-5cc206fde6e4)
- [crates/hickory-cli/tests/ingest_naming.rs](../ingest-keeps-the-original-bytes/contract.md#output-44b2a022e6da)
- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#output-6682c6282b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
