<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Recording A Document Keeps Lives In The Document

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document whose cells are recorded, when `hick ingest --from
recording <doc>` runs, then each recorded cell's output is written into the
document as `<hick:ingested key="…" sha256="…" at="…">` inside the cell,
verbatim; the weave and `hick run` look there first and in the cache
second, match it by the same key the cache uses, and report it *stale* the
same way when the cell's inputs move; `hick run` brings a kept recording
forward when it re-executes the cell, and never adds one the document did
not keep; a clone with no cache weaves the same bytes; and nothing is
written unless the document with its recordings inside weaves, against an
empty cache, every byte the cache-backed weave did. `.hick-cache/` is a
cache again, whole, in `.gitignore`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-recording-a-document-keeps-lives-in-the-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run`

This is the maintained test fragment from `crates/hickory-cli/tests/round_trip.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-82b3411e044d-a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run"></a>
### `a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run`

Protects docs/guarantees/verification/a-recording-a-document-keeps-lives-in-the-document.md

<storycode:copy id="f-82b3411e044d-a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run">
/// Protects docs/guarantees/verification/a-recording-a-document-keeps-lives-in-the-document.md
#[test]
fn a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run() {
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
    let md = std::fs::read_to_string(root.join("service.md")).unwrap();
    let client = std::fs::read_to_string(root.join("client/client.py")).unwrap();

    // Keep the recordings in the document.
    let out = hick()
        .args(["ingest", "--from", "recording"])
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{said}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(said.contains("2 recording(s) now kept"), "{said}");
    let doc = std::fs::read_to_string(root.join("service.md")).unwrap();
    assert_eq!(doc.matches("<hick:ingested key=").count(), 2, "{doc}");
    assert!(
        doc.contains("/pets"),
        "the recorded output is in the document verbatim: {doc}"
    );

    // A clone has no cache. The weave finds everything in the document and
    // produces the same bytes.
    std::fs::remove_dir_all(root.join(".hick-cache")).unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unrecorded") && !stderr.contains("stale"),
        "{stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("service.md")).unwrap(),
        md
    );
    assert_eq!(
        std::fs::read_to_string(root.join("client/client.py")).unwrap(),
        client
    );

    // The inputs move: the document's recording is stale, and says so —
    // the last output still shows.
    std::fs::write(root.join("extra.txt"), "moves the key\n").unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("stale"), "{stderr}");
    assert!(
        std::fs::read_to_string(root.join("service.md"))
            .unwrap()
            .contains("/pets")
    );

    // A run brings the document's recordings forward; a clone is whole again.
    let out = hick()
        .arg("run")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("refreshed 2 recording(s)"), "{stderr}");
    std::fs::remove_dir_all(root.join(".hick-cache")).unwrap();
    let out = hick()
        .arg("weave")
        .arg(root.join("service.md"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unrecorded") && !stderr.contains("stale"),
        "{stderr}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/ingest_recording.rs](../../../mechanisms/crates/hickory-cli/src/ingest_recording.rs.md#output-bc9bc50518aa)
- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/tests/round_trip.rs](../../../mechanisms/crates/hickory-cli/tests/round_trip.rs.md#output-82b3411e044d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
