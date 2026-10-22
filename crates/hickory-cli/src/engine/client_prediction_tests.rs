//! Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
use super::*;
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn options() -> Attach {
    let mut opts = Attach::new("/host-captured/doc.md".into(), crate::ExecutorChoice::Local);
    opts.prediction = Some(crate::serve::PrivatePredictionStartup {
        ledger_root: "/host-private/ledger".into(),
        seal: "sealed-question".into(),
        request_id: "request".into(),
        identity: hick_observe::prediction::Identity {
            source: "a".repeat(64),
            inputs: "b".repeat(64),
            dependencies: BTreeMap::new(),
            environment: BTreeMap::from([("runtime".into(), "c".repeat(64))]),
        },
        target: hick_observe::prediction::Target::Output {
            execution: "captured-cell".into(),
        },
        actor: hick_observe::prediction::Actor {
            kind: "agent".into(),
            id: "host-bound-predictor".into(),
        },
        encoding: hick_literate::prediction::OutputEncoding::Json,
        question: None,
        runtime: None,
        inputs: None,
        retained: None,
    });
    opts
}
async fn server(capable: bool) -> (Endpoint, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let received = calls.clone();
    let router=Router::new().route("/health",get(move||async move{Json(json!({"protocol":PROTOCOL,"capabilities":if capable {vec![super::super::PREDICTION_PROTOCOL]}else{vec![]}}))}))
        .route("/attach",post(move||{let received=received.clone();async move{received.fetch_add(1,Ordering::SeqCst);Json(json!({"slot":"0","log_offset":0}))}}));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let job = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (
        Endpoint {
            url: format!("http://{address}"),
            token: "fixture-token".into(),
            protocol: PROTOCOL,
            pid: 0,
        },
        calls,
        job,
    )
}
#[tokio::test]
async fn unsupported_engine_refuses_before_attach_instead_of_silently_ignoring_private_options() {
    let (endpoint, calls, job) = server(false).await;
    assert!(
        attach(&reqwest::Client::new(), &endpoint, &options())
            .await
            .is_err()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    job.abort();
}
#[tokio::test]
async fn capability_without_exact_binding_acknowledgement_is_not_a_success_or_fallback() {
    let (endpoint, calls, job) = server(true).await;
    assert!(
        attach(&reqwest::Client::new(), &endpoint, &options())
            .await
            .is_err()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    job.abort();
}
