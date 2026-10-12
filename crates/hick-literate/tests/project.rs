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
