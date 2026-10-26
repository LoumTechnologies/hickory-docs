<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_agent.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d24aa77e0987"></a>
## `crates/hickory-cli/tests/serve_agent.rs`

Related contracts: [A Missing Key Degrades to a Note, Never a Crash](../../../../guarantees/agent/a-missing-key-degrades-to-a-note/contract.md), [A running agent can be stopped](../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [The Dock Reports Spend and Runs the Chosen Model](../../../../guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model/contract.md).

[Read the test `DOC` beside its requirement](../../../../guarantees/agent/a-missing-key-degrades-to-a-note/contract.md#f-d24aa77e0987-DOC).
<a id="f-d24aa77e0987-Session"></a>
### `Session`

<storycode:copy id="f-d24aa77e0987-Session">
struct Session {
    base: String,
    doc_id: String,
    state: hickory_cli::serve::LocalState,
    _dir: tempfile::TempDir,
}

</storycode:copy>
<a id="f-d24aa77e0987-start"></a>
### `start`

<storycode:copy id="f-d24aa77e0987-start">
async fn start() -> Session {
    start_with_folder(false).await
}

</storycode:copy>
<a id="f-d24aa77e0987-start_with_folder"></a>
### `start_with_folder`

<storycode:copy id="f-d24aa77e0987-start_with_folder">
async fn start_with_folder(folder: bool) -> Session {
    start_session(folder, false).await
}

</storycode:copy>
<a id="f-d24aa77e0987-start_session"></a>
### `start_session`

<storycode:copy id="f-d24aa77e0987-start_session">
async fn start_session(folder: bool, editor_only: bool) -> Session {
    let dir = tempfile::tempdir().unwrap();
    if !editor_only {
        std::fs::write(dir.path().join("demo.md"), DOC).unwrap();
    }
    let root = dir.path().canonicalize().unwrap();

    let opts = ServeOptions {
        target: if folder || editor_only {
            root.clone()
        } else {
            root.join("demo.md")
        },
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    };
    let prepared = if editor_only {
        hickory_cli::serve::prepare_without_folder(opts).await
    } else {
        prepare(opts).await
    }
    .expect("session prepares");

    let state = prepared.state.clone();
    let doc_id = if editor_only {
        assert!(state.index.entries().is_empty());
        "workspace".into()
    } else {
        state.index.sole().expect("one document").0
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        doc_id,
        state,
        _dir: dir,
    }
}

</storycode:copy>
<a id="f-d24aa77e0987-get"></a>
### `get`

<storycode:copy id="f-d24aa77e0987-get">
async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.json().await.unwrap_or(Value::Null),
    )
}

</storycode:copy>
<a id="f-d24aa77e0987-post"></a>
### `post`

<storycode:copy id="f-d24aa77e0987-post">
async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.json().await.unwrap_or(Value::Null),
    )
}

</storycode:copy>
<a id="f-d24aa77e0987-no_key_degrades_to_the_note_never_a_crash"></a>
### `no_key_degrades_to_the_note_never_a_crash`

<storycode:copy id="f-d24aa77e0987-no_key_degrades_to_the_note_never_a_crash">
#[tokio::test(flavor = "multi_thread")]
async fn no_key_degrades_to_the_note_never_a_crash() {
    for var in [
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "DEEPSEEK_API_KEY",
        "XAI_API_KEY",
        "OPENROUTER_API_KEY",
        "HICKORY_LLM_PROVIDER",
    ] {
        unsafe { std::env::remove_var(var) };
    }

    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "hello", "parent_id": null }),
    )
    .await;
    assert_eq!(status, 503, "{body}");
    let error = body["error"].as_str().expect("an {{error}} body");
    assert!(
        error.starts_with("agent not available"),
        "the client contract is the leading phrase: {error}"
    );
    // Degraded, not broken: the rest of the session still answers.
    let (status, _) = get(&session, "/api/health").await;
    assert_eq!(status, 200);
}

</storycode:copy>
<a id="f-d24aa77e0987-an_untouched_document_lists_resolved_defaults_and_zero_totals"></a>
### `an_untouched_document_lists_resolved_defaults_and_zero_totals`

<storycode:copy id="f-d24aa77e0987-an_untouched_document_lists_resolved_defaults_and_zero_totals">
#[tokio::test(flavor = "multi_thread")]
async fn an_untouched_document_lists_resolved_defaults_and_zero_totals() {
    let session = start().await;
    session
        .state
        .agent
        .set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new(["unused"])));

    let (status, listing) = get(
        &session,
        &format!("/api/docs/{}/agent/turns", session.doc_id),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listing["provider"], "anthropic");
    assert_eq!(listing["model"], "claude-sonnet-5");
    assert_eq!(listing["totals"]["usd"], 0.0);
    assert_eq!(listing["totals"]["input"], 0);
}

</storycode:copy>
<a id="f-d24aa77e0987-stopping_an_idle_document_names_the_situation"></a>
### `stopping_an_idle_document_names_the_situation`

<storycode:copy id="f-d24aa77e0987-stopping_an_idle_document_names_the_situation">
#[tokio::test(flavor = "multi_thread")]
async fn stopping_an_idle_document_names_the_situation() {
    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent/stop", session.doc_id),
        json!({}),
    )
    .await;
    assert_eq!(status, 409, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("no agent turn is running"),
        "{body}"
    );
}

</storycode:copy>
<a id="f-d24aa77e0987-removed_rust_backend_is_refused"></a>
### `removed_rust_backend_is_refused`

<storycode:copy id="f-d24aa77e0987-removed_rust_backend_is_refused">
// Guarantee: the Rust runtime has no fallback or compatibility mode.
#[tokio::test]
async fn removed_rust_backend_is_refused() {
    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({"prompt":"hello", "backend":"builtin"}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(body["error"].as_str().unwrap().contains("removed"));
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_agent.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_agent.rs"><storycode:paste select="#f-d24aa77e0987-DOC"/><storycode:paste select="#f-d24aa77e0987-Session"/><storycode:paste select="#f-d24aa77e0987-start"/><storycode:paste select="#f-d24aa77e0987-start_with_folder"/><storycode:paste select="#f-d24aa77e0987-start_session"/><storycode:paste select="#f-d24aa77e0987-get"/><storycode:paste select="#f-d24aa77e0987-post"/><storycode:paste select="#f-d24aa77e0987-no_key_degrades_to_the_note_never_a_crash"/><storycode:paste select="#f-d24aa77e0987-an_untouched_document_lists_resolved_defaults_and_zero_totals"/><storycode:paste select="#f-d24aa77e0987-stopping_an_idle_document_names_the_situation"/><storycode:paste select="#f-d24aa77e0987-removed_rust_backend_is_refused"/></storycode:file>

</storycode:doc>
