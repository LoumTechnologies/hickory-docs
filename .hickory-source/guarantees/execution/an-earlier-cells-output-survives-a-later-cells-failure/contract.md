<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Earlier Cell's Output Survives A Later Cell's Failure

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a `hick run` where a cell mounting an output volume genuinely
succeeds, and a LATER cell in the same run then fails, then that earlier
cell's real output is inspectable on disk under `.hick-cache/last-run/` —
never silently gone just because the run, as a whole, did not finish.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/an-earlier-cells-output-survives-a-later-cells-failure.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-a626dfc8d598"></a>
## `crates/hickory-cli/tests/volume_flush_survives_failure.rs`

Related contracts: [An Earlier Cell's Output Survives A Later Cell's Failure](contract.md).

<a id="f-a626dfc8d598-hick"></a>
### `hick`

An earlier cell's real output volume survives a LATER cell's failure.
Before this existed, output-volume files were flushed into
`PipelineResult::files` only once, after the WHOLE exec loop finished —
so a cell that failed after an earlier cell had already, genuinely
succeeded left that earlier cell's real output invisible on disk
anywhere at all. Drives the real binary because the claim is about what
actually reaches disk, not about a function in isolation.

<storycode:copy id="f-a626dfc8d598-hick">
//! An earlier cell's real output volume survives a LATER cell's failure.
//!
//! Guarantee:
//! docs/guarantees/execution/an-earlier-cells-output-survives-a-later-cells-failure.md
//!
//! Before this existed, output-volume files were flushed into
//! `PipelineResult::files` only once, after the WHOLE exec loop finished —
//! so a cell that failed after an earlier cell had already, genuinely
//! succeeded left that earlier cell's real output invisible on disk
//! anywhere at all. Drives the real binary because the claim is about what
//! actually reaches disk, not about a function in isolation.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-a626dfc8d598-doc"></a>
### `doc`

<storycode:copy id="f-a626dfc8d598-doc">
fn doc() -> &'static str {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="c" />
<hick:volume name="out" output="app" />

<hick:exec container="c" mount="out:proj">
mkdir -p proj && printf 'real output\n' > proj/file.txt
</hick:exec>

<hick:exec container="c">
exit 1
</hick:exec>
</hick:doc>
"##
}

</storycode:copy>
<a id="f-a626dfc8d598-an_earlier_cells_real_output_is_mirrored_even_though_the_run_fails"></a>
### `an_earlier_cells_real_output_is_mirrored_even_though_the_run_fails`

<storycode:copy id="f-a626dfc8d598-an_earlier_cells_real_output_is_mirrored_even_though_the_run_fails">
#[test]
fn an_earlier_cells_real_output_is_mirrored_even_though_the_run_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("report.md"), doc()).expect("write doc");

    let out = hick()
        .arg("run")
        .arg(dir.path().join("report.md"))
        .env("HICKORY_EXECUTOR", "local")
        .output()
        .expect("run hick");
    assert!(
        !out.status.success(),
        "the second cell's exit 1 must fail the run"
    );

    // The real output tree never gets cell 1's file: the whole run failed,
    // so `write_outputs_detailed` — the authoritative write — never ran.
    assert!(
        !dir.path().join("app/file.txt").exists(),
        "the real output tree must not gain a file from a run that failed"
    );

    // But the incremental mirror does — cell 1 genuinely succeeded, and its
    // real output was known before cell 2 ever ran.
    let mirrored = dir.path().join(".hick-cache/last-run/app/file.txt");
    let content = std::fs::read_to_string(&mirrored).unwrap_or_else(|e| {
        panic!(
            "cell 1's real output should be mirrored at {}: {e}",
            mirrored.display()
        )
    });
    assert_eq!(content, "real output\n");
}

</storycode:copy>
<a id="f-a626dfc8d598-a_fully_successful_run_writes_the_real_tree_not_only_the_mirror"></a>
### `a_fully_successful_run_writes_the_real_tree_not_only_the_mirror`

The mirror is best-effort visibility, never a second source of truth: a
FULLY successful run still writes the real output tree the normal way,
and the mirror must not be mistaken for it.

<storycode:copy id="f-a626dfc8d598-a_fully_successful_run_writes_the_real_tree_not_only_the_mirror">
/// The mirror is best-effort visibility, never a second source of truth: a
/// FULLY successful run still writes the real output tree the normal way,
/// and the mirror must not be mistaken for it.
#[test]
fn a_fully_successful_run_writes_the_real_tree_not_only_the_mirror() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ok_doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="c" />
<hick:volume name="out" output="app" />

<hick:exec container="c" mount="out:proj">
mkdir -p proj && printf 'real output\n' > proj/file.txt
</hick:exec>
</hick:doc>
"##;
    std::fs::write(dir.path().join("report.md"), ok_doc).expect("write doc");

    let out = hick()
        .arg("run")
        .arg(dir.path().join("report.md"))
        .env("HICKORY_EXECUTOR", "local")
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(
        std::fs::read_to_string(dir.path().join("app/file.txt")).unwrap(),
        "real output\n",
        "the real output tree must still be written on success"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/volume_flush_survives_failure.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/volume_flush_survives_failure.rs"><storycode:paste select="#f-a626dfc8d598-hick"/><storycode:paste select="#f-a626dfc8d598-doc"/><storycode:paste select="#f-a626dfc8d598-an_earlier_cells_real_output_is_mirrored_even_though_the_run_fails"/><storycode:paste select="#f-a626dfc8d598-a_fully_successful_run_writes_the_real_tree_not_only_the_mirror"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [crates/hickory-cli/tests/volume_flush_survives_failure.rs](contract.md#output-a626dfc8d598)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
