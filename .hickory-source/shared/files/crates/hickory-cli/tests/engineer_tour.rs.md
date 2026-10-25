<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/engineer_tour.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [The engineer tour distinguishes replay from execution](../../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md)

Captured SHA-256: `2c46f61a24c8889ba7aa5b5edf1801e7ec31af628f847b561c92d1ea51fec914`.

<book:entry path="crates/hickory-cli/tests/engineer_tour.rs">
//! The downloaded tour is an executable specification, not just browser code.
use hickory_cli::{ExecutorChoice, RunMode, run_doc};

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
#[tokio::test]
async fn expected_root_source_rejects_changed_bytes_before_parsing_or_execution() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("guarded.md");
    let original = "# Captured requirement\n";
    let expected = hick_context_protocol::digest(original.as_bytes());
    std::fs::write(&path, "<hick:exec>this must never execute</hick:exec>\n").unwrap();
    let error = hickory_cli::run_doc_expected(
        &path,
        &[],
        RunMode::Execute,
        ExecutorChoice::Local,
        &expected,
    )
    .await
    .err()
    .expect("Changed source must be refused");
    assert!(
        error
            .to_string()
            .contains("Source revision changed before execution"),
        "{error:#}"
    );
    std::fs::write(&path, original).unwrap();
    let result = hickory_cli::run_doc_expected(
        &path,
        &[],
        RunMode::Verify,
        ExecutorChoice::Local,
        &expected,
    )
    .await
    .unwrap();
    assert_eq!(result.source, original);
    assert_eq!(
        hick_context_protocol::digest(result.source.as_bytes()),
        expected
    );
}
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

// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
#[tokio::test]
async fn native_verification_receipt_names_the_exact_source_that_ran() {
    use hickory_cli::serve::{ServeOptions, prepare};
    let temporary = tempfile::tempdir().unwrap();
    let source = include_str!("../../../examples/ai-engineer-deadline.md");
    let path = temporary.path().join("deadline-spec.md");
    std::fs::write(&path, source).unwrap();
    let woven = run_doc(&path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    hickory_cli::write_outputs(&woven, None).unwrap();
    let prepared = prepare(ServeOptions {
        target: temporary.path().to_path_buf(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: Some(temporary.path().join("keys.json")),
        ui_settings_path: Some(temporary.path().join("ui.json")),
    })
    .await
    .unwrap();
    let id = prepared.state.index.add("deadline-spec.md");
    let run = prepared.state.start_run(&id, true).await.unwrap();
    let receipt = tokio::time::timeout(std::time::Duration::from_secs(20), async {
        loop {
            let receipt = hickory_cli::serve::api::get_run(
                axum::extract::State(prepared.state.clone()),
                axum::extract::Path(run.clone()),
            )
            .await
            .unwrap()
            .0;
            if receipt["status"] != "running" {
                break receipt;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(receipt["status"], "ok");
    assert_eq!(
        receipt["source_hash"],
        hick_context_protocol::digest(source.as_bytes())
    );
    assert!(
        receipt["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["kind"] == "exec" && b["status"] == "ok")
    );
}
</book:entry>

</book:doc>
