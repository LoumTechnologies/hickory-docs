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
async fn owned_scaffold_accepts_patches_without_generator_execution() {
    // The command is retained as provenance, never executed during replay.
    let source = format!(
        r#"<hick:project/>
<hick:exec id="scaffold" container="sdk">
<hick:copy>cargo init --name seed --vcs none out</hick:copy>
<hick:ingested sha256="{}">
<hick:file id="owned" path="src/main.rs">
fn main() {{}}
</hick:file>
<hick:file path="Cargo.toml">
[package]
name = "seed"
version = "0.1.0"
</hick:file>
</hick:ingested>
</hick:exec>

Give the first executable an observable greeting.
<hick:patch id="greeting" after="scaffold">
--- a/src/main.rs
+++ b/src/main.rs
@@ -1 +1 @@
-fn main() {{}}
+fn main() {{ println!("Hello, Hickory!"); }}
</hick:patch>
"#,
        hick_lang::scaffold_fingerprint(&std::collections::BTreeMap::from([
            ("src/main.rs".into(), "fn main() {}\n".into()),
            (
                "Cargo.toml".into(),
                "[package]\nname = \"seed\"\nversion = \"0.1.0\"\n".into()
            ),
        ]))
    );
    let result = hick_literate::run_pipeline_weave(&[("project.md", &source)], &[], None)
        .await
        .unwrap();
    assert!(result.transcripts.is_empty());
    assert_eq!(
        result.files["src/main.rs"].as_text(),
        Some("fn main() { println!(\"Hello, Hickory!\"); }\n")
    );
    assert!(matches!(
        result.provenance_maps["Cargo.toml"].spans()[0].origin,
        hick_exec::node::SourceOrigin::Ingested { .. }
    ));
    // Compare complete trees, not just the changed entry point. Reconstruction
    // before and after reorganizing the source must agree on all outputs.
    let plain = "<hick:project/><hick:file path=\"src/main.rs\">fn main() { println!(\"Hello, Hickory!\"); }\n</hick:file><hick:file path=\"Cargo.toml\">[package]\nname = \"seed\"\nversion = \"0.1.0\"\n</hick:file>";
    let before = hick_literate::run_pipeline_weave(&[("project.md", plain)], &[], None)
        .await
        .unwrap();
    assert!(
        before
            .evaluated_project
            .unwrap()
            .new
            .compare(&result.evaluated_project.unwrap().new)
            .is_empty()
    );
    let invalid = source.replace("-fn main() {}", "-fn main() { changed(); }");
    assert!(
        hick_literate::run_pipeline_weave(&[("project.md", &invalid)], &[], None)
            .await
            .is_err()
    );
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
#[tokio::test]
async fn standalone_scaffold_replay_keeps_literal_paths_and_bytes() {
    let path = "{{name}}.txt";
    let content = "\u{feff}FavoriteApp λ\r\n";
    let hash = hick_lang::scaffold_fingerprint(&std::collections::BTreeMap::from([(
        path.into(),
        content.into(),
    )]));
    let source = format!(
        r#"<hick:var name="name">changed</hick:var>
<hick:substitute name="app" pattern="FavoriteApp" value="Modified" />
<hick:container name="sdk" />
<hick:exec container="sdk">
<hick:copy>echo retained</hick:copy>
<hick:ingested sha256="{hash}">
<hick:file path="{path}">
{content}</hick:file>
</hick:ingested>
</hick:exec>
"#
    );
    let result = hick_literate::run_pipeline_weave(&[("standalone.md", &source)], &[], None)
        .await
        .unwrap();
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[path].as_text(), Some(content));
    let tampered = source.replace("FavoriteApp", "WrongBase");
    let error = hick_literate::run_pipeline_weave(&[("standalone.md", &tampered)], &[], None)
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("hash mismatch"));
}
