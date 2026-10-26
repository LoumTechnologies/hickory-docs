<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A frame stays in document coordinates

For: Authors debugging generated code while navigating its document source.

Given a woven file with document lineage, when a debug adapter reports a frame
through a filesystem alias or symlink, then the frame still resolves to the
owning document and source line. Canonical spellings such as macOS `/private/var`
and `/var` name the same file. Files without document lineage keep their own file
coordinates. This lookup does not change breakpoint placement or file bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-frame-stays-in-document-coordinates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `adapter_paths_through_a_symlink_keep_the_document_coordinates`

This is the maintained test fragment from `crates/hick-dap/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-b360a6d46b8d-adapter_paths_through_a_symlink_keep_the_document_coordinates"></a>
### `adapter_paths_through_a_symlink_keep_the_document_coordinates`

<storycode:copy id="f-b360a6d46b8d-adapter_paths_through_a_symlink_keep_the_document_coordinates">
use hick_lang::ProjectTree;
use std::{collections::BTreeMap, path::Path};
// Guarantee: docs/guarantees/debugging/a-frame-stays-in-document-coordinates.md
#[cfg(unix)]
#[test]
fn adapter_paths_through_a_symlink_keep_the_document_coordinates() {
    let dir = tempfile::tempdir().unwrap();
    let actual = dir.path().join("actual");
    std::fs::create_dir(&actual).unwrap();
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&actual, &alias).unwrap();
    let source = "# Explanation\n\n<hick:file path=\"program.py\">\nprint('hello')\n</hick:file>\n";
    let mapping = hick_dap::Mapping::for_document(Path::new("note.md"), source, &actual).unwrap();
    hick_dap::weave_into(source, &actual).unwrap();
    let path = alias.join("program.py");
    assert_eq!(mapping.to_document(&path, 0), Some(3));
    assert_eq!(
        mapping.source_location(&path, 0),
        Some((Path::new("note.md").to_path_buf(), 3))
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-dap/src/mapping.rs](../../../mechanisms/crates/hick-dap/src/mapping.rs.md#output-32f353d4cad8)

### Tests and fixtures

- [crates/hick-dap/tests/live_session.rs](../../../mechanisms/crates/hick-dap/tests/live_session.rs.md#output-7cb62902a4d7)
- [crates/hick-dap/tests/project.rs](../../../mechanisms/crates/hick-dap/tests/project.rs.md#output-b360a6d46b8d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
