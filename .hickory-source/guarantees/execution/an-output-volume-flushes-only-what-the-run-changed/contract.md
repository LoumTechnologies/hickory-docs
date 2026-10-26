<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Output Volume Flushes Only What The Run Changed

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a volume declared with an `output=` path and seeded from a directory,
when the run finishes and the volume is flushed to disk, then only the files
a cell created or changed since the volume was seeded are written; a file
whose bytes are identical to its seeded bytes is left alone.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/an-output-volume-flushes-only-what-the-run-changed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `a_flush_writes_only_what_the_run_changed`

This is the maintained test fragment from `crates/hick-literate/src/lib/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-23d11bf7cb82-a_flush_writes_only_what_the_run_changed"></a>
### `a_flush_writes_only_what_the_run_changed`

Protects docs/guarantees/execution/an-output-volume-flushes-only-what-the-run-changed.md

<storycode:copy id="f-23d11bf7cb82-a_flush_writes_only_what_the_run_changed">
/// Protects docs/guarantees/execution/an-output-volume-flushes-only-what-the-run-changed.md
#[test]
fn a_flush_writes_only_what_the_run_changed() {
    let decl = hick_exec::volume::VolumeDeclaration {
        name: "work".to_string(),
        kind: hick_exec::volume::VolumeKind::InputOutput {
            input: ".".to_string(),
            output: ".".to_string(),
        },
        access_rules: Vec::new(),
    };
    let seeded = volume_state::pack_tar_files(&[
        ("doc.md".to_string(), b"<doc/>".to_vec()),
        ("out/a.txt".to_string(), b"[never run]".to_vec()),
    ])
    .unwrap();
    let after = volume_state::pack_tar_files(&[
        ("doc.md".to_string(), b"<doc/>".to_vec()),
        ("out/a.txt".to_string(), b"[never run]".to_vec()),
        ("out/b.txt".to_string(), b"made by a cell".to_vec()),
    ])
    .unwrap();
    let files = flushable_volume_files(&decl, Some(&after), Some(&seeded)).unwrap();
    // The document and the staged placeholder came along for the ride;
    // only the cell's own product is an output.
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files.contains_key("out/b.txt"));
    // With no record of the seed, everything flushes, as before.
    let files = flushable_volume_files(&decl, Some(&after), None).unwrap();
    assert_eq!(files.len(), 3);
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/volume_state.rs](../../../mechanisms/crates/hick-literate/src/volume_state.rs.md#output-d1d876a59c4b)

### Tests and fixtures

- [crates/hick-literate/src/lib/tests/tests.rs](../../../mechanisms/crates/hick-literate/src/lib/tests/tests.rs.md#output-23d11bf7cb82)
- [crates/hickory-cli/tests/round_trip.rs](../../../mechanisms/crates/hickory-cli/tests/round_trip.rs.md#output-82b3411e044d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
