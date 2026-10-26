<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/engine/client_prediction_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-014c7e3acc59"></a>
## `crates/hickory-cli/src/engine/client_prediction_tests.rs`

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

[Read the test `options` beside its requirement](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md#f-014c7e3acc59-options).
<a id="f-014c7e3acc59-server"></a>
### `server`

<storycode:copy id="f-014c7e3acc59-server">
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
</storycode:copy>
<a id="f-014c7e3acc59-unsupported_engine_refuses_before_attach_instead_of_silently_ignoring_private_options"></a>
### `unsupported_engine_refuses_before_attach_instead_of_silently_ignoring_private_options`

<storycode:copy id="f-014c7e3acc59-unsupported_engine_refuses_before_attach_instead_of_silently_ignoring_private_options">
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
</storycode:copy>
<a id="f-014c7e3acc59-capability_without_exact_binding_acknowledgement_is_not_a_success_or_fallback"></a>
### `capability_without_exact_binding_acknowledgement_is_not_a_success_or_fallback`

<storycode:copy id="f-014c7e3acc59-capability_without_exact_binding_acknowledgement_is_not_a_success_or_fallback">
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

</storycode:copy>
<a id="f-014c7e3acc59-native_window_close_requires_end_and_survives_retired_engine"></a>
### `native_window_close_requires_end_and_survives_retired_engine`

<storycode:copy id="f-014c7e3acc59-native_window_close_requires_end_and_survives_retired_engine">
#[tokio::test]
async fn native_window_close_requires_end_and_survives_retired_engine() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let backend = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(|| async { StatusCode::UNPROCESSABLE_ENTITY }),
        )
        .await
        .unwrap();
    });
    let ended = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let connection = Connection {
        endpoint: Arc::new(tokio::sync::RwLock::new(Endpoint {
            url: format!("http://{address}"),
            token: "fixture".into(),
            protocol: PROTOCOL,
            pid: 0,
        })),
        ended: ended.clone(),
        private_ended: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        attach: options(),
        slot: "0".into(),
        log_offset: 0,
        http: reqwest::Client::new(),
    };
    let closed = Arc::new(AtomicUsize::new(0));
    let observed = closed.clone();
    let cancelled = Arc::new(AtomicUsize::new(0));
    let cancel_observed = cancelled.clone();
    let shell = crate::serve::Shell {
        window: Arc::new(|operation| {
            Ok(crate::serve::WindowStatus {
                native: true,
                platform: "linux".into(),
                minimized: matches!(operation, crate::serve::WindowOperation::Minimize),
            })
        }),
        close_window: Arc::new(move || {
            observed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
        cancel_quit: Arc::new(move || { cancel_observed.fetch_add(1, Ordering::SeqCst); }),
        open_folder: Arc::new(|_, _| anyhow::bail!("unexpected filesystem callback")),
        pick_folder: Arc::new(|_| anyhow::bail!("unexpected filesystem callback")),
        save_file: Arc::new(|_, _| anyhow::bail!("unexpected filesystem callback")),
    };
    let router =
        super::super::proxy::client_router(connection.clone(), Some(shell), "fixture".into(), None);
    let request = |method: &str, path: &str, body: &str, origin: Option<&str>| {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("host", "127.0.0.1:9000");
        if let Some(origin) = origin {
            builder = builder.header("origin", origin);
        }
        builder.body(Body::from(body.to_owned())).unwrap()
    };
    assert_eq!(
        router
            .clone()
            .oneshot(request("POST", "/api/window/close", "{}", None))
            .await
            .unwrap()
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(closed.load(Ordering::SeqCst), 0);
    ended.store(true, Ordering::SeqCst);
    backend.abort();
    let _ = backend.await;
    for (body, origin, expected) in [
        ("{\"window\":\"other\"}", None, StatusCode::BAD_REQUEST),
        ("{}", Some("https://foreign.example"), StatusCode::FORBIDDEN),
    ] {
        assert_eq!(
            router
                .clone()
                .oneshot(request("POST", "/api/window/close", body, origin))
                .await
                .unwrap()
                .status(),
            expected
        );
    }
    assert_eq!(closed.load(Ordering::SeqCst), 0);
    for path in [
        "/api/window/status",
        "/api/window/minimize",
        "/api/window/restore",
    ] {
        let method = if path.ends_with("status") {
            "GET"
        } else {
            "POST"
        };
        let response = router
            .clone()
            .oneshot(request(method, path, "{}", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["minimized"], path.ends_with("minimize"));
    }
    assert_eq!(
        router
            .clone()
            .oneshot(request("POST", "/api/window/close", "{}", None))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(closed.load(Ordering::SeqCst), 1);
    assert_eq!(
        router
            .clone()
            .oneshot(request("POST", "/api/window/cancel-quit", "{}", None))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(cancelled.load(Ordering::SeqCst), 1);
    let browser = super::super::proxy::client_router(connection, None, "fixture".into(), None);
    assert_eq!(
        browser
            .oneshot(request("POST", "/api/window/close", "{}", None))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_IMPLEMENTED
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/engine/client_prediction_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/engine/client_prediction_tests.rs"><storycode:paste select="#f-014c7e3acc59-options"/><storycode:paste select="#f-014c7e3acc59-server"/><storycode:paste select="#f-014c7e3acc59-unsupported_engine_refuses_before_attach_instead_of_silently_ignoring_private_options"/><storycode:paste select="#f-014c7e3acc59-capability_without_exact_binding_acknowledgement_is_not_a_success_or_fallback"/><storycode:paste select="#f-014c7e3acc59-native_window_close_requires_end_and_survives_retired_engine"/></storycode:file>

</storycode:doc>
