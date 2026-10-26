<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/tests/prediction_verification.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-575d4e4459ff"></a>
## `crates/hickory-agent/tests/prediction_verification.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

[Read the test `request` beside its requirement](../../../../contract.md#f-575d4e4459ff-request).
<a id="f-575d4e4459ff-source"></a>
### `source`

<storycode:copy id="f-575d4e4459ff-source">
fn source(value: u64) -> String {
    format!(
        "<hick:container name=\"c\" image=\"alpine\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\">printf {value}<hick:expect>100</hick:expect></hick:exec></hick:file>"
    )
}
</storycode:copy>
<a id="f-575d4e4459ff-tool"></a>
### `tool`

<storycode:copy id="f-575d4e4459ff-tool">
async fn tool(
    session: &mut EditSession,
    name: &str,
    args: Vec<(String, String)>,
) -> hickory_agent::ToolOutcome {
    execute_tool(
        session,
        Arc::new(LocalExecutor::new().unwrap()),
        &ToolInvocation::synthetic(name, args, None).unwrap(),
    )
    .await
}
</storycode:copy>
<a id="f-575d4e4459ff-shared_verify_preserves_failure_then_repair_without_answers_or_output_publication"></a>
### `shared_verify_preserves_failure_then_repair_without_answers_or_output_publication`

<storycode:copy id="f-575d4e4459ff-shared_verify_preserves_failure_then_repair_without_answers_or_output_publication">
#[tokio::test]
async fn shared_verify_preserves_failure_then_repair_without_answers_or_output_publication() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let ledger = Arc::new(Ledger::open(private.path()).unwrap());
    let path = root.path().join("doc.md");
    let faulty = source(160);
    std::fs::write(&path, &faulty).unwrap();
    std::fs::write(root.path().join("previous.txt"), "PRIVATE-CACHED-ANSWER").unwrap();
    let initial = request(&ledger, &faulty, "fault");
    let mut session =
        EditSession::open_prediction(&path, ledger.clone(), initial, OutputEncoding::Json)
            .await
            .unwrap();
    let failed = tool(&mut session, "verify", vec![]).await;
    assert!(!failed.ok);
    let feedback: serde_json::Value = serde_json::from_str(&failed.text).unwrap();
    assert_eq!(feedback["summary"]["classification"], "execution_rejected");
    assert!(
        !failed.text.contains("stdout")
            && !failed.text.contains("captures")
            && !failed.text.contains("actual")
    );
    assert!(!root.path().join("answer.txt").exists());
    for name in ["read_file", "read_output", "edit_output", "shell", "run"] {
        let result = tool(
            &mut session,
            name,
            vec![("path".into(), "previous.txt".into())],
        )
        .await;
        assert!(!result.ok && !result.text.contains("PRIVATE-CACHED-ANSWER"));
    }
    let preview = session
        .preview_edit(
            &ToolInvocation::synthetic(
                "edit_output",
                vec![("path".into(), "answer.txt".into())],
                None,
            )
            .unwrap(),
        )
        .await;
    assert!(preview.is_err());
    // A source edit through the captured tool still needs a new host seal.
    let corrected = source(100);
    let edit = EditSession::reviewed_invocation(
        &ToolInvocation::synthetic("edit_doc", vec![], None).unwrap(),
        &faulty,
        corrected.clone(),
    )
    .unwrap();
    let edited = execute_tool(&mut session, Arc::new(LocalExecutor::new().unwrap()), &edit).await;
    assert!(edited.ok, "{}", edited.text);

    assert!(!tool(&mut session, "verify", vec![]).await.ok);
    let next = request(&ledger, &corrected, "repair");
    session.replace_prediction(next).unwrap();
    let passed = tool(&mut session, "verify", vec![]).await;
    assert!(passed.ok, "{}", passed.text);
    let feedback: serde_json::Value = serde_json::from_str(&passed.text).unwrap();
    assert_eq!(
        feedback["summary"]["classification"],
        "verified_after_correction"
    );
    assert!(!root.path().join("answer.txt").exists());
    let history = ledger.history(None).unwrap();
    assert_eq!(history.attempts.len(), 2);
    assert!(history.attempts.iter().all(|entry| !entry.promoted));
    // Out-of-band writes cannot inject runtime answers through source reads.
    std::fs::write(&path, "PRIVATE-RUNTIME-ANSWER").unwrap();
    let refused = tool(&mut session, "read_doc", vec![]).await;
    assert!(!refused.ok && !refused.text.contains("PRIVATE-RUNTIME-ANSWER"));
    let snapshots = session.source_snapshots();
    assert_eq!(
        snapshots[0]["sha256"],
        format!("{:x}", Sha256::digest(corrected.as_bytes()))
    );
    assert_eq!(snapshots[0]["current"], false);
    assert!(
        !serde_json::to_string(&snapshots)
            .unwrap()
            .contains(&format!("{:x}", Sha256::digest(b"PRIVATE-RUNTIME-ANSWER")))
    );
}
</storycode:copy>

### Assemble `crates/hickory-agent/tests/prediction_verification.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/tests/prediction_verification.rs"><storycode:paste select="#f-575d4e4459ff-request"/><storycode:paste select="#f-575d4e4459ff-source"/><storycode:paste select="#f-575d4e4459ff-tool"/><storycode:paste select="#f-575d4e4459ff-shared_verify_preserves_failure_then_repair_without_answers_or_output_publication"/></storycode:file>

</storycode:doc>
