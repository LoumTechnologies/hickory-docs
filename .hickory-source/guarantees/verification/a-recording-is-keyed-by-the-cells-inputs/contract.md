<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Reuse a recording only for the inputs that produced it

For: Reviewers deciding what an output, recording or prediction actually establishes.

Change an authored script without changing the shell command. The old recording must become stale; writing the transcript itself must not invalidate its own key.

## What must happen

A recording key includes the cell's command, relevant environment and features, declared seeded input bytes, and upstream cell identities. Literal document-produced inputs remain part of the digest. The document's own unstable outputs, weave targets and retained recording payloads do not key a cell on its own results. Seeded volume state is hashed before earlier cells mutate it; upstream outputs arrive through the upstream term. Unseeded output-only volumes contribute no input bytes. Cache directories are excluded at every depth. A broad output-path exclusion must not erase real inputs.

## Limits of the evidence

Correct identity coverage depends on declared inputs. A matching cache key does not establish correctness of an undeclared ambient dependency.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Combine the September 2 and 3 input-digest amendments, including the explicitly withdrawn output-volume-prefix shortcut, into one rule.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `a_documents_own_unstable_products_are_not_in_the_key`

This is the maintained test fragment from `crates/hick-literate/src/lib/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-23d11bf7cb82-a_documents_own_unstable_products_are_not_in_the_key"></a>
### `a_documents_own_unstable_products_are_not_in_the_key`

Protects docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md

<storycode:copy id="f-23d11bf7cb82-a_documents_own_unstable_products_are_not_in_the_key">
/// Protects docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md
#[test]
fn a_documents_own_unstable_products_are_not_in_the_key() {
    // A cell mounting `.` mounts its own weave and the file it fills.
    // Those bytes change on every run, so a key that counted them was
    // never findable again; the recorded output of a real run then read
    // `[never run]` on the next weave. They are not inputs.
    let dir = tempfile::tempdir().unwrap();
    let mut store = volume_state::VolumeStore::new();
    let before = vec![
        ("analysis.py".to_string(), b"print(1)".to_vec()),
        ("report.md".to_string(), b"old weave".to_vec()),
        ("chart.svg".to_string(), b"<svg>old</svg>".to_vec()),
    ];
    let after = vec![
        ("analysis.py".to_string(), b"print(1)".to_vec()),
        ("report.md".to_string(), b"new weave".to_vec()),
        ("chart.svg".to_string(), b"<svg>new</svg>".to_vec()),
    ];
    let mounts = vec![("src".to_string(), "/work".to_string())];
    let unstable: HashSet<String> = ["report.md".to_string(), "chart.svg".to_string()]
        .into_iter()
        .collect();
    store.seed_tar("src", volume_state::pack_tar_files(&before).unwrap());
    let policy = DigestPolicy { unstable };
    let key_before = mounted_inputs_digest(&store, &mounts, dir.path(), &policy);
    store.seed_tar("src", volume_state::pack_tar_files(&after).unwrap());
    let key_after = mounted_inputs_digest(&store, &mounts, dir.path(), &policy);
    assert_eq!(
        key_before, key_after,
        "the run's own products moved the key"
    );

    // The script the document assembles IS an input: change it and the
    // key changes, which is the guarantee's whole point.
    let edited = vec![
        ("analysis.py".to_string(), b"print(2)".to_vec()),
        ("report.md".to_string(), b"new weave".to_vec()),
        ("chart.svg".to_string(), b"<svg>new</svg>".to_vec()),
    ];
    store.seed_tar("src", volume_state::pack_tar_files(&edited).unwrap());
    assert_ne!(
        key_after,
        mounted_inputs_digest(&store, &mounts, dir.path(), &policy)
    );
    // And a product under a subdirectory is matched by its tail.
    assert!(is_unstable_output(
        "chart.svg",
        &["project/chart.svg".to_string()].into_iter().collect()
    ));
    assert!(!is_unstable_output(
        "art.svg",
        &["project/chart.svg".to_string()].into_iter().collect()
    ));
}

</storycode:copy>

## Test the behavior: `hick`

This is the maintained test fragment from `crates/hickory-cli/tests/cache_inputs.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-518e60ffbf9d-hick"></a>
### `hick`

A recording is keyed by what the cell READ, not only by what it says.
The failure this protects against is silent and expensive: a document
assembles a file, a cell runs that file, the file changes, the command
does not — and the document reports numbers produced by code it no longer
contains. Nothing about that looks wrong from the outside.

<storycode:copy id="f-518e60ffbf9d-hick">
//! A recording is keyed by what the cell READ, not only by what it says.
//!
//! Guarantee: `docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md`
//!
//! The failure this protects against is silent and expensive: a document
//! assembles a file, a cell runs that file, the file changes, the command
//! does not — and the document reports numbers produced by code it no longer
//! contains. Nothing about that looks wrong from the outside.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-literate/src/cache.rs](../../../mechanisms/crates/hick-literate/src/cache.rs.md#output-4b4919c14d46)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)

### Tests and fixtures

- [crates/hick-literate/src/lib/tests/tests.rs](../../../mechanisms/crates/hick-literate/src/lib/tests/tests.rs.md#output-23d11bf7cb82)
- [crates/hickory-cli/tests/cache_inputs.rs](../../../mechanisms/crates/hickory-cli/tests/cache_inputs.rs.md#output-518e60ffbf9d)
- [crates/hickory-cli/tests/round_trip.rs](../../../mechanisms/crates/hickory-cli/tests/round_trip.rs.md#output-82b3411e044d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
