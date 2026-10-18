//! The downloaded tour is an executable specification, not just browser code.
use hickory_cli::{ExecutorChoice, RunMode, run_doc};
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
#[tokio::test]
async fn deadline_witness_passes_and_exposes_the_original_bug() {
    let source = include_str!("../../../examples/ai-engineer-deadline.md");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("deadline.md");
    std::fs::write(&path, source).unwrap();
    let woven = run_doc(&path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    assert_eq!(
        woven
            .result
            .files
            .get("deadline.js")
            .and_then(|content| content.as_text()),
        Some(include_str!("../../../examples/deadline.js"))
    );
    hickory_cli::write_outputs(&woven, None).unwrap();
    let corrected = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        run_doc(&path, &[], RunMode::Verify, ExecutorChoice::Local),
    )
    .await
    .expect("tour check must finish")
    .unwrap();
    assert_eq!(corrected.result.expectations.len(), 1);
    assert!(
        corrected.result.expectations[0].passed,
        "{:?}",
        corrected.result.expectations
    );
    let broken = source.replace(
        "var allowed = Math.max(0, budget - total);",
        "var allowed = budget;",
    );
    assert_ne!(broken, source);
    std::fs::write(&path, broken).unwrap();
    let woven = run_doc(&path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    hickory_cli::write_outputs(&woven, None).unwrap();
    let broken = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        run_doc(&path, &[], RunMode::Verify, ExecutorChoice::Local),
    )
    .await
    .expect("counterexample must finish");
    match broken {
        Err(error) => assert!(
            error
                .to_string()
                .contains("R1 failed: expected 100, got 160"),
            "{error:#}"
        ),
        Ok(run) => assert!(run.result.expectations.iter().any(|check| !check.passed)),
    }
}
