<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Check every source file, including assets and permissions

For: Reviewers deciding whether a reorganized source still constructs the same program.

Changing one byte of an image, removing a migration, or adding an unexplained lockfile must fail equality even when every Rust file matches.

## What must happen

Given generated and expected trees under the declared inventory policy, equality compares the complete path inventory, exact content bytes, entry kinds, executable status, and symlink targets. Missing, extra, or differing entries fail. Only the source closure, declared reading targets, fixed metadata directories, and explicit exclusions are omitted. Unknown kinds and ambiguous portable paths fail; Git ignore rules and filename extensions do not decide source membership.


## Limits of the evidence

Executable versus ordinary is the portable permission model. Ownership, timestamps, empty directories and all POSIX permission bits are outside it. The captured repository has no symlink entries; the focused inventory test exercises virtual symlink metadata.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep complete source equality separate from test success and source-edit reversibility.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `inventory_checks_binary_modes_symlinks_and_portable_paths`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-inventory_checks_binary_modes_symlinks_and_portable_paths"></a>
### `inventory_checks_binary_modes_symlinks_and_portable_paths`

<storycode:copy id="f-005c1e9344c7-inventory_checks_binary_modes_symlinks_and_portable_paths">
// Guarantee: docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn inventory_checks_binary_modes_symlinks_and_portable_paths() {
    let project = generate("<hick:project exclude=\".git/ node_modules/\"/><hick:entry path=\"asset.bin\" encoding=\"hex\">00 ff 01</hick:entry><hick:entry path=\"link\" kind=\"symlink\">asset.bin</hick:entry><hick:file path=\"run.sh\" executable=\"true\">echo ok\n</hick:file>").unwrap();
    let mut expected = project.new.clone();
    expected
        .insert("extra.lock", ProjectEntry::file(b"lock".to_vec()))
        .unwrap();
    let mut asset = expected.get("asset.bin").unwrap().as_ref().clone();
    asset.bytes = vec![0, 254, 1].into();
    expected.insert("asset.bin", asset).unwrap();
    let mut script = expected.get("run.sh").unwrap().as_ref().clone();
    script.executable = false;
    expected.insert("run.sh", script).unwrap();
    let differences = project.new.compare(&expected);
    assert_eq!(
        differences
            .iter()
            .map(|d| d.path.as_str())
            .collect::<Vec<_>>(),
        vec!["asset.bin", "extra.lock", "run.sh"]
    );
    assert!(!project.inventory.contains("project.md"));
    assert!(!project.inventory.contains("node_modules/x"));
    assert!(project.inventory.contains("migration.sql"));
    for path in [
        "../x", "/x", "x//y", "x\\y", "C:x", "NUL.txt", "x.", "x/../y", "é.rs",
    ] {
        assert!(validate_project_path(path).is_err());
    }
    let mut t = tree(&[("A", b"x")]);
    assert!(t.insert("a", ProjectEntry::file(b"y".to_vec())).is_err());
    assert!(
        t.insert("A/child", ProjectEntry::file(b"y".to_vec()))
            .is_err()
    );
}
</storycode:copy>

## Test the behavior: `generated_paths_cannot_shadow_the_markdown_closure_on_another_platform`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-generated_paths_cannot_shadow_the_markdown_closure_on_another_platform"></a>
### `generated_paths_cannot_shadow_the_markdown_closure_on_another_platform`

<storycode:copy id="f-005c1e9344c7-generated_paths_cannot_shadow_the_markdown_closure_on_another_platform">
// Guarantee: docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn generated_paths_cannot_shadow_the_markdown_closure_on_another_platform() {
    assert!(
        generate("<hick:project/><hick:file path=\"Project.md\">overwrite</hick:file>").is_err()
    );
    assert!(
        generate("<hick:project/><hick:file path=\"project.md/child\">overwrite</hick:file>")
            .is_err()
    );
    let sources = BTreeMap::from([
        (
            "project.md".into(),
            "<hick:project sources=\"notes/a.md\"/><hick:file path=\"notes\">overwrite</hick:file>"
                .into(),
        ),
        ("notes/a.md".into(), "source".into()),
    ]);
    assert!(evaluate_project("project.md", &sources, &ProjectTree::default()).is_err());
}
</storycode:copy>

## Test the behavior: `binary_comparison_does_not_equal_same_length_placeholders`

This is the maintained test fragment from `crates/hick-literate/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d5caa71df0d7-binary_comparison_does_not_equal_same_length_placeholders"></a>
### `binary_comparison_does_not_equal_same_length_placeholders`

<storycode:copy id="f-d5caa71df0d7-binary_comparison_does_not_equal_same_length_placeholders">
use hick_exec::node::{BinaryData, FileContent};
use std::collections::HashMap;
// Guarantee: docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn binary_comparison_does_not_equal_same_length_placeholders() {
    let a = HashMap::from([(
        "asset".into(),
        FileContent::Binary(BinaryData::Inline(vec![0, 255])),
    )]);
    let b = HashMap::from([(
        "asset".into(),
        FileContent::Binary(BinaryData::Inline(vec![0, 254])),
    )]);
    assert_eq!(hick_literate::equiv::compare_outputs(&a, &b).len(), 1);
    assert!(hick_literate::equiv::compare_outputs(&a, &a).is_empty());
}
</storycode:copy>

## Test the behavior: `a_refactor_baseline_cannot_follow_mutating_binary_temp_files`

This is the maintained test fragment from `crates/hick-literate/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d5caa71df0d7-a_refactor_baseline_cannot_follow_mutating_binary_temp_files"></a>
### `a_refactor_baseline_cannot_follow_mutating_binary_temp_files`

<storycode:copy id="f-d5caa71df0d7-a_refactor_baseline_cannot_follow_mutating_binary_temp_files">
// Guarantee: docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn a_refactor_baseline_cannot_follow_mutating_binary_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("asset");
    std::fs::write(&path, [0, 255]).unwrap();
    let outputs = HashMap::from([(
        "asset".into(),
        FileContent::Binary(BinaryData::TempFile(path.clone())),
    )]);
    let baseline = hick_literate::freeze_outputs(&outputs).unwrap();
    std::fs::write(&path, [0, 254]).unwrap();
    assert_eq!(
        hick_literate::equiv::compare_outputs(&baseline, &outputs).len(),
        1
    );
    std::fs::remove_file(path).unwrap();
    assert!(hick_literate::equiv::compare_outputs(&baseline, &baseline).is_empty());
}
</storycode:copy>

## Implementation review

ProjectEntry retains portable metadata beside its bytes. ProjectTree compares entries in both directions. Revision and directory adapters apply the same inventory policy before comparison.

### Implementation

- [crates/hick-lang/src/project/evaluate.rs](../../../mechanisms/crates/hick-lang/src/project/evaluate.rs.md#output-4957ad4b2f9a)
- [crates/hick-lang/src/project/tree.rs](../../../mechanisms/crates/hick-lang/src/project/tree.rs.md#output-a485394d6759)
- [crates/hickory-cli/src/project_revision.rs](../../../mechanisms/crates/hickory-cli/src/project_revision.rs.md#output-fb969c2bf635)

### Tests and fixtures

- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)
- [crates/hick-literate/tests/project.rs](../../../mechanisms/crates/hick-literate/tests/project.rs.md#output-d5caa71df0d7)

### Focused witnesses

- `inventory_checks_binary_modes_symlinks_and_portable_paths` in `crates/hick-lang/tests/project.rs`.

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
