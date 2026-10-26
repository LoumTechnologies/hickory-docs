<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/prediction_environment.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b078fc2a53bf"></a>
## `crates/hick-literate/tests/prediction_environment.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

[Read the test `seal` beside its requirement](../../../../contract.md#f-b078fc2a53bf-seal).
<a id="f-b078fc2a53bf-runtime_is_revalidated_before_dispatch_and_before_settlement"></a>
### `runtime_is_revalidated_before_dispatch_and_before_settlement`

<storycode:copy id="f-b078fc2a53bf-runtime_is_revalidated_before_dispatch_and_before_settlement">
#[tokio::test]
#[ignore = "requires installed Python/debugpy; explicitly run the focused runtime witness"]
async fn runtime_is_revalidated_before_dispatch_and_before_settlement() {
    let project = tempfile::tempdir().unwrap();
    let parent = project.path().join(".hick-cache/adapters");
    std::fs::create_dir_all(&parent).unwrap();
    let adapter = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/adapters/python")
        .canonicalize()
        .unwrap();
    let link = parent.join("python");
    std::os::unix::fs::symlink(adapter, &link).unwrap();
    let profile = Profile::PythonDebugpy {
        project: project.path().into(),
    };
    let captured = profile.capture().await.unwrap();
    let source = "<hick:container name=\"c\" image=\"python\" />\n<hick:exec container=\"c\">python3 -c 'print(100)'</hick:exec>";
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let mut wrong = captured.clone();
    wrong.insert("output_python".into(), "0".repeat(64));
    let request = seal(&ledger, source, wrong);
    assert!(
        run_output_with_environment(
            &ledger,
            &request,
            source,
            Arc::new(LocalExecutor::new().unwrap()),
            Some(&profile)
        )
        .await
        .is_err()
    );
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    let stable = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(stable.path()).unwrap();
    let request = seal(&ledger, source, captured.clone());
    let result = run_output_with_environment(
        &ledger,
        &request,
        source,
        Arc::new(LocalExecutor::new().unwrap()),
        Some(&profile),
    )
    .await
    .unwrap();
    assert_eq!(
        result.summary.classification,
        Classification::FirstAttemptVerified
    );
    let source = format!(
        "<hick:container name=\"c\" image=\"python\" />\n<hick:exec container=\"c\">python3 -c \"import os; os.unlink('{}'); print(100)\"</hick:exec>",
        link.display()
    );
    let another = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(another.path()).unwrap();
    let request = seal(&ledger, &source, captured);
    let result = run_output_with_environment(
        &ledger,
        &request,
        &source,
        Arc::new(LocalExecutor::new().unwrap()),
        Some(&profile),
    )
    .await
    .unwrap();
    assert_eq!(result.summary.classification, Classification::Unknown);
    assert!(
        ledger
            .promote(&result.summary.attempt, &request.identity, request.actor)
            .is_err()
    );
    assert!(!link.exists());
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/prediction_environment.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/prediction_environment.rs"><storycode:paste select="#f-b078fc2a53bf-seal"/><storycode:paste select="#f-b078fc2a53bf-runtime_is_revalidated_before_dispatch_and_before_settlement"/></storycode:file>

</storycode:doc>
