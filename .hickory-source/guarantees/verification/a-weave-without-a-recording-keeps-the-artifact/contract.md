<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep an existing artifact when no recording can construct it

For: Reviewers deciding what an output, recording or prediction actually establishes.

Open a captured project whose transcript is absent locally. Weaving must not replace a retained output with a never-run marker.

## What must happen

When a required cell has no recording, weaving preserves existing outputs it cannot derive, including the woven report, and reports that preservation. A stale recording is different: weaving may display its last recorded bytes with stale status and write that rendering. Only an absent baseline is unrecorded. Explicit execution can refresh the source outputs; an unrecorded weave must not silently destroy retained evidence.

## Limits of the evidence

Retaining an artifact does not verify its current inputs. A newly absent output cannot be recovered from a recording that does not exist.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the September 3 stale-versus-unrecorded amendment to the September 2 preservation rule. Keep missing-baseline and changed-input outcomes separate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `a_cell_whose_inputs_moved_is_stale_and_shows_its_last_recording`

This is the maintained test fragment from `crates/hickory-cli/tests/round_trip.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-82b3411e044d-a_cell_whose_inputs_moved_is_stale_and_shows_its_last_recording"></a>
### `a_cell_whose_inputs_moved_is_stale_and_shows_its_last_recording`

Protects docs/specs/freeform/three-axes.md (axis 1) and

<storycode:copy id="f-82b3411e044d-a_cell_whose_inputs_moved_is_stale_and_shows_its_last_recording">
/// Protects docs/specs/freeform/three-axes.md (axis 1) and
/// docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md
#[test]
fn a_cell_whose_inputs_moved_is_stale_and_shows_its_last_recording() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("service.md"), CHAIN).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    let out = hick()
        .arg("run")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let recorded = std::fs::read_to_string(root.join("service.md")).unwrap();
    assert!(recorded.contains("/pets"), "{recorded}");

    // A new file in the mounted folder changes both cells' inputs, so their
    // recordings no longer match. They are STALE, not unrecorded: the weave
    // keeps showing what was recorded and says so, and nothing turns into a
    // marker.
    std::fs::write(root.join("extra.txt"), "changes the digest\n").unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(
        stderr.contains("stale"),
        "the weave did not report the cells as stale: {stderr}"
    );
    assert!(
        !stderr.contains("unrecorded"),
        "a stale cell was reported as unrecorded: {stderr}"
    );
    let woven = std::fs::read_to_string(root.join("service.md")).unwrap();
    assert!(
        !woven.contains("[never run]"),
        "a marker was written over recorded output: {woven}"
    );
    assert!(
        woven.contains("/pets"),
        "the last recording was not shown: {woven}"
    );

    // `hick test` names it as the drift it is, with the run that fixes it.
    let out = hick()
        .arg("test")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // test re-executes, so the cells match again there; what matters is that
    // the run afterwards leaves nothing stale.
    let out = hick()
        .arg("run")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{stderr}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = hick()
        .arg("weave")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("stale") && !stderr.contains("unrecorded"),
        "{stderr}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)

### Tests and fixtures

- [crates/hickory-cli/tests/round_trip.rs](../../../mechanisms/crates/hickory-cli/tests/round_trip.rs.md#output-82b3411e044d)
- [crates/hickory-cli/tests/weave_keeps_artifacts.rs](../../../mechanisms/crates/hickory-cli/tests/weave_keeps_artifacts.rs.md#output-b0471d375879)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
