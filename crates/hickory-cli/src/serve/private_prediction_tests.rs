//! Guarantee: predictions-precede-managed-observation. Queued intent cannot
//! silently acquire a newer seal, including one for identical source bytes.
use super::*;
use crate::{
    ExecutorChoice,
    serve::{ServeOptions, prepare_with_prediction},
};
use hick_literate::prediction::output_target;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

async fn fixture() -> (
    tempfile::TempDir,
    tempfile::TempDir,
    LocalState,
    Arc<Ledger>,
    std::path::PathBuf,
    Identity,
) {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let ledger = Arc::new(Ledger::open(private.path()).unwrap());
    let marker = root.path().join("executed");
    let source = format!(
        "<hick:container name=\"c\" image=\"alpine\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\">touch '{}'; printf 100<hick:expect>100</hick:expect></hick:exec></hick:file>",
        marker.display()
    );
    let path = root.path().join("doc.md");
    std::fs::write(&path, &source).unwrap();
    let actor = Actor {
        kind: "agent".into(),
        id: "queued-native-predictor".into(),
    };
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: hick_observe::prediction::digest(&json!([])).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([(
            "runtime".into(),
            hick_observe::prediction::digest(&"local-printf").unwrap(),
        )]),
    };
    let target = output_target("c", 2, OutputEncoding::Json);
    let sealed = ledger
        .seal(
            0,
            Seal {
                identity: identity.clone(),
                target: target.clone(),
                predicted: Predicate::Equals(json!(100)),
                rationale: "Expected fixed output".into(),
                scenario: Scenario {
                    family: "queued-native".into(),
                    inputs: json!([]),
                    requirement_reference: "fixed 100".into(),
                    requirement: Predicate::Equals(json!(100)),
                    identity_known: true,
                },
            },
            actor.clone(),
        )
        .unwrap();
    let prepared = prepare_with_prediction(
        ServeOptions {
            target: path,
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        },
        Binding {
            ledger: ledger.clone(),
            request: SealedRequest {
                seal: sealed.id,
                request_id: "original".into(),
                actor,
                identity: identity.clone(),
                target,
            },
            encoding: OutputEncoding::Json,
            question: None,
            runtime: None,
            inputs: None,
            retained: None,
        },
    )
    .await
    .unwrap();
    let state = prepared.state;
    (root, private, state, ledger, marker, identity)
}

#[tokio::test]
async fn queued_run_refuses_same_source_reseal_before_dispatch() {
    let (_root, _private, state, ledger, marker, identity) = fixture().await;
    crate::serve::acp::assert_private_startup_refused(&state).await;
    let bound = state.private_prediction.clone().unwrap();
    let guard = bound.session.lock().await;
    let next = bound.clone();
    let hash = identity.source.clone();
    let (ready, waiting) = tokio::sync::oneshot::channel();
    let reseal = tokio::spawn(async move {
        let future = next.seal_prediction(
            &hash,
            Predicate::Equals(json!(100)),
            "Same prediction, later seal".into(),
        );
        tokio::pin!(future);
        assert!(futures::poll!(future.as_mut()).is_pending());
        ready.send(()).unwrap();
        future.await.unwrap()
    });
    waiting.await.unwrap();
    let run = bound
        .start(&state, &bound.doc, Some(identity.source.clone()))
        .unwrap();
    tokio::task::yield_now().await;
    drop(guard);
    reseal.await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if state.runs.lock().unwrap()[&run].status != "running" {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(state.runs.lock().unwrap()[&run].status, "failed");
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    assert!(
        !marker.exists(),
        "A retired native Run intent must never execute"
    );
    let held = bound.session.lock().await;
    assert!(
        bound.retire(&state).await.is_err(),
        "Busy source admission cannot end"
    );
    drop(held);
    bound.retire(&state).await.unwrap();
    assert!(bound.start(&state, &bound.doc, None).is_err());
    assert!(
        bound
            .seal_prediction(
                &identity.source,
                Predicate::Equals(json!(100)),
                "Retired context".into()
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn private_acp_managed_tools_require_review_and_refuse_unscoped_delivery() {
    let (_root, _private, state, ledger, marker, _identity) = fixture().await;
    super::super::private_prediction_acp::test_managed_boundary(state).await;
    assert!(
        !marker.exists(),
        "tool discovery/review must not execute the target"
    );
    assert!(ledger.history(None).unwrap().attempts.is_empty());
}

#[tokio::test]
async fn private_acp_protocol_refuses_filesystem_permissions_and_unverified_completion() {
    let (_root, _private, state, ledger, marker, _identity) = fixture().await;
    super::super::private_prediction_acp::test_protocol_boundary(state).await;
    assert!(!marker.exists());
    assert!(ledger.history(None).unwrap().attempts.is_empty());
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn advertised_private_whole_preview_never_writes_or_executes_and_refuses_stale_source() {
    let (_root, _private, state, ledger, marker, _) = fixture().await;
    let bound = state.private_prediction.clone().unwrap();
    let source = std::fs::read_to_string(&bound.path).unwrap();
    let snapshot = hickory_agent::hashline::LineIndex::new(&source).snapshot;
    let tools = catalogue(false);
    let edit = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "edit_doc")
        .unwrap();
    let whole = edit["inputSchema"]["properties"]["whole"]["enum"][0].clone();
    assert_eq!(whole, "true");
    let proposed = format!("{source}\n\nA proposed explanation.\n");
    let arguments = json!({"whole":whole,"snapshot":snapshot,"input":proposed});
    let Json(result) = preview(
        State(state.clone()),
        Json(json!({"name":"edit_doc","arguments":arguments})),
    )
    .await
    .unwrap();
    assert_eq!(result["before"], source);
    assert_eq!(result["after"], proposed);
    assert_eq!(std::fs::read_to_string(&bound.path).unwrap(), source);
    assert!(!marker.exists());
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    assert!(preview(State(state.clone()), Json(json!({"name":"edit_doc","arguments":{"snapshot":snapshot,"run":"literal source text","input":"replacement"}}))).await.is_err());
    std::fs::write(
        &bound.path,
        format!("{source}\nChanged by another owner.\n"),
    )
    .unwrap();
    assert!(
        preview(
            State(state),
            Json(json!({"name":"edit_doc","arguments":arguments}))
        )
        .await
        .is_err()
    );
    assert!(!marker.exists());
}
