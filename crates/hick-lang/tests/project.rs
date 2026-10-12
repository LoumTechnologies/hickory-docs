use hick_lang::*;
use std::collections::BTreeMap;
fn tree(files: &[(&str, &[u8])]) -> ProjectTree {
    let mut tree = ProjectTree::default();
    for (path, bytes) in files {
        tree.insert(path, ProjectEntry::file(bytes.to_vec()))
            .unwrap();
    }
    tree
}
fn generate(source: &str) -> Result<EvaluatedProject, String> {
    evaluate_project(
        "project.md",
        &BTreeMap::from([("project.md".into(), source.into())]),
        &ProjectTree::default(),
    )
}
// Guarantee: docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
#[test]
fn patch_preserves_bom_crlf_unicode_and_untouched_origins() {
    let mut before = tree(&[("x", "\u{feff}é\r\nunchanged\r\nlast".as_bytes())]);
    let mut entry = before.get("x").unwrap().as_ref().clone();
    entry.origins.push(ByteOrigin {
        output: (0, entry.bytes.len()),
        document: "base.md".into(),
        source: (100, 100 + entry.bytes.len()),
    });
    before.insert("x", entry).unwrap();
    let diff = "--- a/x\n+++ b/x\n@@ -1,2 +1,2 @@\n-\u{feff}é\r\n+\u{feff}🦀\r\n unchanged\r\n";
    let patch = apply_project_patch(&before, diff, "patch.md", 40).unwrap();
    assert_eq!(
        &*patch.after.get("x").unwrap().bytes,
        "\u{feff}🦀\r\nunchanged\r\nlast".as_bytes()
    );
    assert_eq!(
        &*before.get("x").unwrap().bytes,
        "\u{feff}é\r\nunchanged\r\nlast".as_bytes()
    );
    for origin in &patch.after.get("x").unwrap().origins {
        let source = if origin.document == "patch.md" {
            &diff.as_bytes()[origin.source.0 - 40..origin.source.1 - 40]
        } else {
            &before.get("x").unwrap().bytes[origin.source.0 - 100..origin.source.1 - 100]
        };
        assert_eq!(
            &patch.after.get("x").unwrap().bytes[origin.output.0..origin.output.1],
            source
        );
    }
}
// Guarantee: docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
#[test]
fn newline_markers_add_delete_rename_and_failure_are_atomic() {
    let before = tree(&[("x", b"old"), ("keep", b"same\n")]);
    let patch = apply_project_patch(&before,"--- a/x\n+++ b/renamed\n@@ -1 +1 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n--- /dev/null\n+++ b/added\n@@ -0,0 +1 @@\n+hello\n","p",0).unwrap();
    assert!(patch.after.get("x").is_none());
    assert_eq!(&*patch.after.get("renamed").unwrap().bytes, b"new");
    assert_eq!(&*patch.after.get("added").unwrap().bytes, b"hello\n");
    let deleted = apply_project_patch(
        &patch.after,
        "--- a/added\n+++ /dev/null\n@@ -1 +0,0 @@\n-hello\n",
        "p",
        0,
    )
    .unwrap();
    assert!(deleted.after.get("added").is_none());
    assert!(
        apply_project_patch(
            &before,
            "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-wrong\n+new\n",
            "p",
            0
        )
        .is_err()
    );
    assert_eq!(&*before.get("x").unwrap().bytes, b"old");
    for bad in [
        "--- a/x\n+++ b/x\n@@ -2 +1 @@\n-old\n+new\n",
        "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n", // missing marker
        "--- a/x\n+++ b/x\n@@ -1,1 +1,2 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n+next\n",
        "--- a/x\n+++ /dev/null\n@@ -0,0 +0,0 @@\n", // partial deletion
    ] {
        assert!(apply_project_patch(&before, bad, "p", 0).is_err(), "{bad}");
    }
}
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn closure_constructs_atomic_endpoints_and_can_be_reorganized() {
    let sources = BTreeMap::from([
        (
            "project.md".into(),
            include_str!("fixtures/literate-project/project.md").into(),
        ),
        (
            "library.md".into(),
            include_str!("fixtures/literate-project/library.md").into(),
        ),
    ]);
    let project = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    assert!(project.old.entries().is_empty());
    assert_eq!(
        &*project.new.get("app.ts").unwrap().bytes,
        b"import { welcome } from './library';\nconsole.log(welcome('Hickory'));\n"
    );
    assert_eq!(
        &*project.new.get("library.ts").unwrap().bytes,
        b"export function welcome(name: string) { return 'Hello ' + name; }\n"
    );
    assert_eq!(
        &*project.local_states["caller"]
            .1
            .get("library.ts")
            .unwrap()
            .bytes,
        b"export function greet() { return 'Hello'; }\n"
    );
    let folded = generate("<hick:project/>\n<hick:file path=\"app.ts\">\nimport { welcome } from './library';\nconsole.log(welcome('Hickory'));\n</hick:file>\n<hick:file path=\"library.ts\">\nexport function welcome(name: string) { return 'Hello ' + name; }\n</hick:file>").unwrap();
    assert!(folded.new.compare(&project.new).is_empty());
    let mut incomplete = sources.clone();
    incomplete.remove("library.md");
    assert!(
        evaluate_project("project.md", &incomplete, &ProjectTree::default())
            .unwrap_err()
            .contains("missing library.md")
    );
    assert_eq!(ProjectTree::default().compare(&project.new).len(), 2);
}
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn refuses_unordered_transforms_hidden_inputs_and_processes() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"x\">one\n</hick:file><hick:patch id=\"p\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-one\n+two\n</hick:patch>";
    assert!(generate(source).unwrap_err().contains("unordered"));
    assert!(generate(&source.replace("id=\"p\"", "id=\"p\" after=\"base\"")).is_ok());
    for extra in [
        "<hick:exec container=\"c\">echo x</hick:exec>",
        "<hick:upstream src=\"secret.md\"/>",
        "<hick:when test=\"unknown\"/>",
    ] {
        assert!(
            generate(&format!("<hick:project/>{extra}"))
                .unwrap_err()
                .contains("required")
        );
    }
    assert!(generate("<hick:project/><hick:file id=\"a\" after=\"b\" path=\"x\">x</hick:file><hick:file id=\"b\" after=\"a\" path=\"y\">y</hick:file>").is_err());
}
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
// Guarantee: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
#[test]
fn permissions_replay_and_destination_identity() {
    for policy in [
        EditPolicy::Neither,
        EditPolicy::Old,
        EditPolicy::New,
        EditPolicy::Both,
    ] {
        let source = |name: &str| ComparisonSource {
            identity: RevisionIdentity::Authored {
                document: name.into(),
            },
            tree: ProjectTree::default(),
            writable_target: Some(name.into()),
        };
        let mut comparison = ProjectComparison::new(
            vec![source("old"), source("new"), source("other")],
            0,
            1,
            policy,
        )
        .unwrap();
        assert!(
            comparison
                .verify_transition(
                    "p.md",
                    &BTreeMap::from([("p.md".into(), "<hick:project/>".into())])
                )
                .unwrap()
                .is_empty()
        );
        assert!(comparison.verified());
        assert_eq!(
            comparison.edit_endpoint(0, ProjectTree::default()).is_ok(),
            policy.old()
        );
        assert_eq!(
            comparison.edit_endpoint(1, ProjectTree::default()).is_ok(),
            policy.new_side()
        );
        if policy != EditPolicy::Neither {
            assert!(!comparison.verified());
        }
        assert!(
            comparison
                .check_destination(1, &ProjectTree::default())
                .is_ok()
        );
        assert!(
            comparison
                .check_destination(1, &tree(&[("concurrent", b"x")]))
                .is_err()
        );
    }
    let immutable = ComparisonSource {
        identity: RevisionIdentity::Git {
            revision: "abc".into(),
        },
        tree: ProjectTree::default(),
        writable_target: None,
    };
    assert!(
        ProjectComparison::new(vec![immutable.clone(), immutable], 0, 1, EditPolicy::Both).is_err()
    );
}
// Guarantee: docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
#[test]
fn raw_patch_grammar_and_ordinary_diff_fence() {
    let source = "<hick:patch id=\"p\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-<hick:file path=\"nested\">\n+<hick:paste select=\"#code\"/>\n</hick:patch>";
    let doc = parse(source).unwrap();
    assert_eq!(doc.all_tags().len(), 1);
    assert_eq!(doc.find_tags("patch")[0].children.len(), 1);
    let source = "<hick:project/>\n```diff\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-x\n+y\n```";
    assert!(generate(source).unwrap().new.entries().is_empty());
}
// Guarantee: docs/guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities.md
#[test]
fn final_response_uses_dirty_start_and_incorporation_rechecks_destinations() {
    let before = tree(&[
        ("app.ts", b"user's existing uncommitted work\n"),
        ("asset", &[0, 255]),
    ]);
    let completed = tree(&[
        ("app.ts", b"successful implementation\n"),
        ("asset", &[0, 255]),
        ("empty", b""),
    ]);
    let mut capture = AgentProjectCapture::new(before.clone());
    capture.concurrent_write("app.ts");
    assert!(
        capture
            .finalize(&completed, "The successful result.")
            .unwrap_err()
            .contains("concurrent")
    );
    capture.resolve_concurrent("app.ts");
    let response = capture
        .finalize(&completed, "The successful result.")
        .unwrap();
    let replay = evaluate_project(
        "response.md",
        &BTreeMap::from([("response.md".into(), response)]),
        &before,
    )
    .unwrap();
    assert!(replay.new.compare(&completed).is_empty());
    capture.incomplete();
    assert!(
        capture
            .finalize(&completed, "The result.")
            .unwrap_err()
            .contains("incomplete")
    );
    let baseline = BTreeMap::from([("project.md".into(), "previous explanation".into())]);
    let documents = BTreeMap::from([("project.md".into(),"<hick:project/><hick:file path=\"app.ts\">successful implementation\n</hick:file><hick:entry path=\"asset\" encoding=\"hex\">00ff</hick:entry><hick:file path=\"empty\"></hick:file>".into())]);
    let candidate = IncorporationCandidate::verify(
        "project.md",
        baseline.clone(),
        documents,
        completed.clone(),
    )
    .unwrap();
    assert!(
        candidate
            .recheck_destinations(&baseline, &completed)
            .is_ok()
    );
    assert!(
        candidate
            .recheck_destinations(&BTreeMap::new(), &completed)
            .is_err()
    );
    assert!(candidate.recheck_destinations(&baseline, &before).is_err());
    assert!(
        IncorporationCandidate::verify(
            "project.md",
            baseline,
            BTreeMap::from([("project.md".into(), "<hick:project/>".into())]),
            completed
        )
        .is_err()
    );
}
// Guarantee: docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md
#[test]
fn patch_regeneration_preserves_order_and_reports_invalid_later_context() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"x\">a\n</hick:file><hick:patch id=\"early\" after=\"base\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n</hick:patch><hick:patch id=\"later\" after=\"early\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-b\n+c\n</hick:patch>";
    let sources = BTreeMap::from([("project.md".into(), source.into())]);
    let input = ProjectTree::default();
    let error = regenerate_patch(
        "project.md",
        &sources,
        &input,
        "early",
        &tree(&[("x", b"different\n")]),
    )
    .unwrap_err();
    assert!(error.contains("wrong patch context"));
    assert_eq!(sources["project.md"], source);
    let candidate = regenerate_patch(
        "project.md",
        &sources,
        &input,
        "later",
        &tree(&[("x", b"final\nwithout final newline")]),
    )
    .unwrap();
    let generated = evaluate_project("project.md", &candidate, &input).unwrap();
    assert_eq!(
        generated.new.get("x").unwrap().bytes.as_ref(),
        b"final\nwithout final newline"
    );
}
// Guarantee: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
#[test]
fn editing_a_proposal_does_not_replace_its_destination_guard() {
    let source = |name: &str| ComparisonSource {
        identity: RevisionIdentity::Authored {
            document: name.into(),
        },
        tree: ProjectTree::default(),
        writable_target: Some(name.into()),
    };
    let mut comparison =
        ProjectComparison::new(vec![source("old"), source("new")], 0, 1, EditPolicy::New).unwrap();
    comparison
        .edit_endpoint(1, tree(&[("proposed", b"new")]))
        .unwrap();
    assert!(
        comparison
            .check_destination(1, &ProjectTree::default())
            .is_ok()
    );
    assert!(
        comparison
            .check_destination(1, &tree(&[("concurrent", b"write")]))
            .is_err()
    );
}
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn contributors_can_route_to_the_completed_project_from_markdown_alone() {
    let sources = BTreeMap::from([
        (
            "project.md".into(),
            include_str!("fixtures/literate-project/project.md").into(),
        ),
        (
            "library.md".into(),
            include_str!("fixtures/literate-project/library.md").into(),
        ),
    ]);
    let root = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    let member = evaluate_project("library.md", &sources, &ProjectTree::default()).unwrap();
    assert!(root.new.compare(&member.new).is_empty());
    let discovered = load_project_sources("library.md", &|path| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| format!("missing {path}"))
    })
    .unwrap();
    assert_eq!(discovered, sources);
    let cycle = BTreeMap::from([
        ("a.md".into(), "<hick:project entry=\"b.md\"/>".into()),
        ("b.md".into(), "<hick:project entry=\"a.md\"/>".into()),
    ]);
    assert!(
        evaluate_project("a.md", &cycle, &ProjectTree::default())
            .unwrap_err()
            .contains("routing cycle")
    );
}

// Guarantees:
// - docs/guarantees/embedding/literal-files-use-native-bytes.md
// - docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn the_legacy_literal_subset_cannot_return_a_half_patched_program() {
    let source =
        "<hick:project/><hick:file path=\"x\">old</hick:file><hick:patch>diff</hick:patch>";
    assert!(
        literal_files(source)
            .unwrap_err()
            .contains("complete project evaluation")
    );
}
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
// Guarantees:
// - docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
// - docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn immutable_identity_cannot_be_mislabeled_as_editable_and_invalid_origins_refuse() {
    let immutable = ComparisonSource {
        identity: RevisionIdentity::Git {
            revision: "abc".into(),
        },
        tree: ProjectTree::default(),
        writable_target: Some("fork.md".into()),
    };
    assert!(
        ProjectComparison::new(vec![immutable.clone(), immutable], 0, 1, EditPolicy::Both).is_err()
    );
    let mut t = ProjectTree::default();
    let mut entry = ProjectEntry::file(b"x".to_vec());
    entry.origins.push(ByteOrigin {
        output: (0, 2),
        document: "source.md".into(),
        source: (0, 2),
    });
    assert!(t.insert("x", entry).unwrap_err().contains("provenance"));
}
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn local_predecessor_state_does_not_depend_on_independent_act_names() {
    for independent in ["before", "zzz_after"] {
        let source = format!(
            "<hick:project/><hick:file id=\"base\" path=\"x\">a\n</hick:file><hick:file id=\"{independent}\" path=\"y\">independent</hick:file><hick:patch id=\"patch\" after=\"base\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n</hick:patch>"
        );
        let project = generate(&source).unwrap();
        assert_eq!(project.local_states["patch"].0.entries().len(), 1);
        assert!(project.local_states["patch"].0.get("y").is_none());
        assert_eq!(project.new.entries().len(), 2);
        assert_eq!(project.new.get("x").unwrap().bytes.as_ref(), b"b\n");
    }
}
