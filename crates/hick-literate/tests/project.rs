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
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn pipeline_uses_the_same_pure_complete_evaluator() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"x\">a\n</hick:file><hick:patch after=\"base\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n</hick:patch>";
    let run = hick_literate::run_pipeline_weave(&[("project.md", source)], &[], None)
        .await
        .unwrap();
    assert_eq!(run.files["x"].as_text(), Some("b\n"));
    assert_eq!(run.evaluated_project.unwrap().new.entries().len(), 1);
    assert!(run.transcripts.is_empty());
    assert!(
        hick_literate::run_pipeline_weave(&[("p.md", "<hick:patch>diff</hick:patch>")], &[], None)
            .await
            .is_err()
    );
}

// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins() {
    let key = hick_lang::exec_cache_key(
        "alpine",
        "",
        "printf hello",
        &[],
        &hick_lang::inputs_digest(&[]),
        &[],
    );
    let source = format!(
        "<hick:project/><hick:container name=\"c\" image=\"alpine\"/><hick:file path=\"out.txt\"><hick:exec container=\"c\" show=\"output\"><hick:cmd>printf hello</hick:cmd><hick:ingested key=\"{key}\">\nhello</hick:ingested><hick:expect match=\"exact\">hello</hick:expect></hick:exec></hick:file>"
    );
    let result = hick_literate::run_pipeline_weave(&[("project.md", &source)], &[], None)
        .await
        .unwrap();
    assert_eq!(result.files["out.txt"].as_text(), Some("hello\n"));
    assert!(result.expectations.iter().all(|e| e.passed));
    assert_eq!(result.expectations.len(), 1);
    assert!(
        result.provenance_maps["out.txt"]
            .spans()
            .iter()
            .any(|s| matches!(s.origin, hick_exec::node::SourceOrigin::Exec { .. }))
    );
    // Literal command wrappers and bare native command text describe the same
    // kept run. A nested paste still needs resolution and must not be ignored.
    let bare = source.replace("<hick:cmd>printf hello</hick:cmd>", "printf hello");
    let bare_result = hick_literate::run_pipeline_weave(&[("project.md", &bare)], &[], None)
        .await
        .unwrap();
    assert!(hick_literate::equiv::compare_outputs(&result.files, &bare_result.files).is_empty());
    let unresolved = source.replace(
        "printf hello</hick:cmd>",
        "<hick:paste select=\"#command\"/></hick:cmd>",
    );
    assert!(
        hick_literate::run_pipeline_weave(&[("project.md", &unresolved)], &[], None)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("requires execution/resolution")
    );
    let bad = source.replace("printf hello", "printf goodbye");
    assert!(
        hick_literate::run_pipeline_weave(&[("project.md", &bad)], &[], None)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("stale")
    );
    let bad = source.replace("match=\"exact\">hello", "match=\"exact\">wrong");
    let result = hick_literate::run_pipeline_weave(&[("project.md", &bad)], &[], None)
        .await
        .unwrap();
    assert!(!result.expectations[0].passed);
    let sources = std::collections::BTreeMap::from([("project.md".into(), source)]);
    let p = hick_lang::evaluate_project("project.md", &sources, &hick_lang::ProjectTree::default())
        .unwrap();
    assert!(
        hick_lang::reverse_project_edit(
            "project.md",
            &sources,
            &hick_lang::ProjectTree::default(),
            &p.new,
            "out.txt",
            (0, 5),
            "other"
        )
        .unwrap_err()
        .contains("synthetic")
    );
}

// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn owned_ingest_is_editable_source_without_claiming_generator_execution() {
    let source = format!(
        "<hick:project/><hick:exec container=\"sdk\"><hick:ingested sha256=\"{}\"><hick:file id=\"owned\" path=\"app.ts\">x\n</hick:file></hick:ingested></hick:exec>",
        "a".repeat(64)
    );
    let result = hick_literate::run_pipeline_weave(&[("project.md", &source)], &[], None)
        .await
        .unwrap();
    assert!(result.transcripts.is_empty());
    assert!(matches!(
        result.provenance_maps["app.ts"].spans()[0].origin,
        hick_exec::node::SourceOrigin::Ingested { .. }
    ));
}
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
