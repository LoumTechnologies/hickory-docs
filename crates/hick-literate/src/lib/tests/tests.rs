use super::*;

// Protects docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md
// (the cache-digest half — `mounted_inputs_digest` must apply the same
// `.gitignore` filter `seed_from_directory` already applies at seed time).
#[test]
fn a_sibling_documents_kept_recordings_are_not_in_the_key() {
    let with = b"<h:doc>\n<h:exec container=\"c\">cmd<h:ingested key=\"k\" sha256=\"s\" at=\"d\">\nout\n</h:ingested></h:exec>\n</h:doc>\n";
    let without = b"<h:doc>\n<h:exec container=\"c\">cmd</h:exec>\n</h:doc>\n";
    assert_eq!(strip_kept_recordings(with), without.to_vec());
    assert_eq!(strip_kept_recordings(without), without.to_vec());
}

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

// Protects docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md
#[test]
fn strip_leading_bom_removes_the_bom_and_advances_the_span() {
    let span = hick_lang::SourceSpan {
        start: 50,
        end: 90,
        start_line: 3,
        start_col: 0,
        file_id: None,
    };
    let (text, moved) = strip_leading_bom("\u{feff}// real content", Some(&span));
    assert_eq!(text, "// real content");
    let moved = moved.expect("a span in must be a span out");
    // The BOM is 3 UTF-8 bytes (U+FEFF), never 1.
    assert_eq!(moved.start, 53);
    assert_eq!(moved.end, 90);
    assert_eq!(moved.start_line, 3, "no newline was crossed");
    assert_eq!(moved.start_col, 1);
}

#[test]
fn strip_leading_bom_leaves_ordinary_text_untouched() {
    let (text, span) = strip_leading_bom("// real content", None);
    assert_eq!(text, "// real content");
    assert!(span.is_none());
}

#[test]
fn strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file() {
    // A BOM is a file-start marker. One sitting mid-content (however it
    // got there) is real content this function has no business touching.
    let (text, _) = strip_leading_bom("line one\n\u{feff}line two", None);
    assert_eq!(text, "line one\n\u{feff}line two");
}

// Protects the "key does not assume a container" clause of
// docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.
#[test]
fn a_cell_with_no_container_can_be_named_and_keyed() {
    let agentish = CellId::containerless(12);
    assert_eq!(agentish.container(), None);
    assert_eq!(agentish.to_string(), "line 12");

    let exec = CellId::exec("build", 12);
    assert_eq!(exec.container(), Some("build"));
    assert_eq!(exec.to_string(), "line 12 (container 'build')");

    // Same line, different cells: the key distinguishes them.
    let mut never_run = NeverRun::new();
    never_run.insert(agentish.clone(), NoBaseline::NotExecuted);
    never_run.insert(exec.clone(), NoBaseline::NotExecuted);
    assert_eq!(never_run.len(), 2);
    assert!(never_run.contains_key(&agentish));
    assert!(never_run.contains_key(&exec));
}

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
