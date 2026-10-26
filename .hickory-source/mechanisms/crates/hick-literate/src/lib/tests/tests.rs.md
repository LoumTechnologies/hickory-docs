<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/lib/tests/tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-23d11bf7cb82"></a>
## `crates/hick-literate/src/lib/tests/tests.rs`

Related contracts: [Apply repository exclusions when seeding execution inputs](../../../../../../guarantees/execution/a-volume-carries-what-the-repository-carries/contract.md), [An Output Volume Flushes Only What The Run Changed](../../../../../../guarantees/execution/an-output-volume-flushes-only-what-the-run-changed/contract.md), [Reuse a recording only for the inputs that produced it](../../../../../../guarantees/verification/a-recording-is-keyed-by-the-cells-inputs/contract.md), [A Woven Fenced Block Has No Leading BOM](../../../../../../guarantees/verification/a-woven-fenced-block-has-no-leading-bom/contract.md), [Report whether verification found drift, no baseline or a false expectation](../../../../../../guarantees/verification/test-separates-unverifiable-from-drifted/contract.md).

[Read the test `a_sibling_documents_kept_recordings_are_not_in_the_key` beside its requirement](../../../../../../guarantees/execution/a-volume-carries-what-the-repository-carries/contract.md#f-23d11bf7cb82-a_sibling_documents_kept_recordings_are_not_in_the_key).
[Read the test `a_flush_writes_only_what_the_run_changed` beside its requirement](../../../../../../guarantees/execution/an-output-volume-flushes-only-what-the-run-changed/contract.md#f-23d11bf7cb82-a_flush_writes_only_what_the_run_changed).
[Read the test `a_documents_own_unstable_products_are_not_in_the_key` beside its requirement](../../../../../../guarantees/verification/a-recording-is-keyed-by-the-cells-inputs/contract.md#f-23d11bf7cb82-a_documents_own_unstable_products_are_not_in_the_key).
<a id="f-23d11bf7cb82-mounted_inputs_digest_excludes_gitignored_paths"></a>
### `mounted_inputs_digest_excludes_gitignored_paths`

<storycode:copy id="f-23d11bf7cb82-mounted_inputs_digest_excludes_gitignored_paths">
#[test]
fn mounted_inputs_digest_excludes_gitignored_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(args)
            .output()
            .expect("run git");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "T"]);
    std::fs::write(dir.path().join(".gitignore"), "build/\n").expect("write .gitignore");

    let mounts = vec![("v".to_string(), "proj".to_string())];
    let digest_of = |stable: &[u8], junk: &[u8]| {
        let mut store = volume_state::VolumeStore::new();
        let tar = volume_state::pack_tar_files(&[
            ("stable.txt".to_string(), stable.to_vec()),
            ("build/junk.txt".to_string(), junk.to_vec()),
        ])
        .unwrap();
        store.seed_tar("v", tar);
        mounted_inputs_digest(&store, &mounts, dir.path(), &DigestPolicy::default())
    };

    assert_eq!(
        digest_of(b"stable", b"one"),
        digest_of(b"stable", b"two"),
        "a gitignored path's content changing must not move the digest — \
             this is the exact instability that made a build cell's own \
             bin/obj destabilize every cell sharing its volume"
    );
    assert_ne!(
        digest_of(b"stable", b"one"),
        digest_of(b"changed", b"one"),
        "a real, non-ignored input changing must still move the digest — \
             otherwise this test would pass by digesting nothing at all"
    );
}

</storycode:copy>
<a id="f-23d11bf7cb82-mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions"></a>
### `mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions`

Sibling: outside a repository, `gitignored` returns `None` and nothing
is filtered — matching `seed_from_directory`'s own "no repository, keep
everything" rule (`a-volume-carries-what-the-repository-carries.md`) —
so the digest still moves. Not a regression: `is_run_artifact`'s fixed
`.hick-cache`/`.git` exclusions apply regardless of a repository.

<storycode:copy id="f-23d11bf7cb82-mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions">
// Sibling: outside a repository, `gitignored` returns `None` and nothing
// is filtered — matching `seed_from_directory`'s own "no repository, keep
// everything" rule (`a-volume-carries-what-the-repository-carries.md`) —
// so the digest still moves. Not a regression: `is_run_artifact`'s fixed
// `.hick-cache`/`.git` exclusions apply regardless of a repository.
#[test]
fn mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions() {
    let dir = tempfile::tempdir().expect("tempdir"); // deliberately NOT a git repo
    let mounts = vec![("v".to_string(), "proj".to_string())];
    let digest_of = |junk: &[u8]| {
        let mut store = volume_state::VolumeStore::new();
        let tar =
            volume_state::pack_tar_files(&[("build/junk.txt".to_string(), junk.to_vec())]).unwrap();
        store.seed_tar("v", tar);
        mounted_inputs_digest(&store, &mounts, dir.path(), &DigestPolicy::default())
    };
    assert_ne!(
        digest_of(b"one"),
        digest_of(b"two"),
        "without a repository nothing is filtered beyond .hick-cache/.git"
    );
}

</storycode:copy>
[Read the test `strip_leading_bom_removes_the_bom_and_advances_the_span` beside its requirement](../../../../../../guarantees/verification/a-woven-fenced-block-has-no-leading-bom/contract.md#f-23d11bf7cb82-strip_leading_bom_removes_the_bom_and_advances_the_span).
<a id="f-23d11bf7cb82-strip_leading_bom_leaves_ordinary_text_untouched"></a>
### `strip_leading_bom_leaves_ordinary_text_untouched`

<storycode:copy id="f-23d11bf7cb82-strip_leading_bom_leaves_ordinary_text_untouched">
#[test]
fn strip_leading_bom_leaves_ordinary_text_untouched() {
    let (text, span) = strip_leading_bom("// real content", None);
    assert_eq!(text, "// real content");
    assert!(span.is_none());
}

</storycode:copy>
<a id="f-23d11bf7cb82-strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file"></a>
### `strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file`

<storycode:copy id="f-23d11bf7cb82-strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file">
#[test]
fn strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file() {
    // A BOM is a file-start marker. One sitting mid-content (however it
    // got there) is real content this function has no business touching.
    let (text, _) = strip_leading_bom("line one\n\u{feff}line two", None);
    assert_eq!(text, "line one\n\u{feff}line two");
}

</storycode:copy>
[Read the test `a_cell_with_no_container_can_be_named_and_keyed` beside its requirement](../../../../../../guarantees/verification/test-separates-unverifiable-from-drifted/contract.md#f-23d11bf7cb82-a_cell_with_no_container_can_be_named_and_keyed).
<a id="f-23d11bf7cb82-extract_single_verify_command"></a>
### `extract_single_verify_command`

<storycode:copy id="f-23d11bf7cb82-extract_single_verify_command">
#[test]
fn extract_single_verify_command() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="cargo check" />
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "cargo check");
    assert!(cmds[0].description.is_none());
    assert_eq!(cmds[0].source_file, "test.md");
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_verify_with_condition_and_description"></a>
### `extract_verify_with_condition_and_description`

<storycode:copy id="f-23d11bf7cb82-extract_verify_with_condition_and_description">
#[test]
fn extract_verify_with_condition_and_description() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="[backend] cargo test" description="Run backend tests" />
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "[backend] cargo test");
    assert_eq!(cmds[0].description.as_deref(), Some("Run backend tests"));
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_multiple_verify_commands"></a>
### `extract_multiple_verify_commands`

<storycode:copy id="f-23d11bf7cb82-extract_multiple_verify_commands">
#[test]
fn extract_multiple_verify_commands() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="cargo check" description="Compile check" />
<hick:verify command="[spa] npm test" description="SPA tests" />
<hick:verify command="[spa+backend] npm run e2e" description="E2E tests" />
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0].command, "cargo check");
    assert_eq!(cmds[1].command, "[spa] npm test");
    assert_eq!(cmds[2].command, "[spa+backend] npm run e2e");
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_verify_from_multiple_sources"></a>
### `extract_verify_from_multiple_sources`

<storycode:copy id="f-23d11bf7cb82-extract_verify_from_multiple_sources">
#[test]
fn extract_verify_from_multiple_sources() {
    let source1 = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="cargo check" />
</hick:doc>"#;
    let source2 = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="npm test" />
</hick:doc>"#;
    let sources = vec![("a.md", source1), ("b.md", source2)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].source_file, "a.md");
    assert_eq!(cmds[1].source_file, "b.md");
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_verify_skips_empty_command"></a>
### `extract_verify_skips_empty_command`

<storycode:copy id="f-23d11bf7cb82-extract_verify_skips_empty_command">
#[test]
fn extract_verify_skips_empty_command() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="" />
<hick:verify command="  " />
<hick:verify command="cargo check" />
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "cargo check");
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_verify_skips_missing_command_attr"></a>
### `extract_verify_skips_missing_command_attr`

<storycode:copy id="f-23d11bf7cb82-extract_verify_skips_missing_command_attr">
#[test]
fn extract_verify_skips_missing_command_attr() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify description="No command attribute" />
<hick:verify command="cargo check" />
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 1);
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_verify_nested_inside_other_elements"></a>
### `extract_verify_nested_inside_other_elements`

<storycode:copy id="f-23d11bf7cb82-extract_verify_nested_inside_other_elements">
#[test]
fn extract_verify_nested_inside_other_elements() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:when test="backend">
<hick:verify command="cargo test" description="Nested verify" />
</hick:when>
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "cargo test");
    assert_eq!(cmds[0].description.as_deref(), Some("Nested verify"));
}

</storycode:copy>
<a id="f-23d11bf7cb82-extract_verify_preserves_source_line"></a>
### `extract_verify_preserves_source_line`

<storycode:copy id="f-23d11bf7cb82-extract_verify_preserves_source_line">
#[test]
fn extract_verify_preserves_source_line() {
    let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:feature name="backend" description="Backend" />

<hick:verify command="cargo check" />
</hick:doc>"#;
    let sources = vec![("test.md", source)];
    let cmds = extract_verify_commands(&sources).unwrap();
    assert_eq!(cmds.len(), 1);
    // The verify tag is on line 5
    assert_eq!(cmds[0].source_line, 5);
}
</storycode:copy>

### Assemble `crates/hick-literate/src/lib/tests/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/lib/tests/tests.rs"><storycode:paste select="#f-23d11bf7cb82-a_sibling_documents_kept_recordings_are_not_in_the_key"/><storycode:paste select="#f-23d11bf7cb82-a_flush_writes_only_what_the_run_changed"/><storycode:paste select="#f-23d11bf7cb82-a_documents_own_unstable_products_are_not_in_the_key"/><storycode:paste select="#f-23d11bf7cb82-mounted_inputs_digest_excludes_gitignored_paths"/><storycode:paste select="#f-23d11bf7cb82-mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions"/><storycode:paste select="#f-23d11bf7cb82-strip_leading_bom_removes_the_bom_and_advances_the_span"/><storycode:paste select="#f-23d11bf7cb82-strip_leading_bom_leaves_ordinary_text_untouched"/><storycode:paste select="#f-23d11bf7cb82-strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file"/><storycode:paste select="#f-23d11bf7cb82-a_cell_with_no_container_can_be_named_and_keyed"/><storycode:paste select="#f-23d11bf7cb82-extract_single_verify_command"/><storycode:paste select="#f-23d11bf7cb82-extract_verify_with_condition_and_description"/><storycode:paste select="#f-23d11bf7cb82-extract_multiple_verify_commands"/><storycode:paste select="#f-23d11bf7cb82-extract_verify_from_multiple_sources"/><storycode:paste select="#f-23d11bf7cb82-extract_verify_skips_empty_command"/><storycode:paste select="#f-23d11bf7cb82-extract_verify_skips_missing_command_attr"/><storycode:paste select="#f-23d11bf7cb82-extract_verify_nested_inside_other_elements"/><storycode:paste select="#f-23d11bf7cb82-extract_verify_preserves_source_line"/></storycode:file>

</storycode:doc>
