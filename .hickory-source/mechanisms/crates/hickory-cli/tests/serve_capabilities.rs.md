<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_capabilities.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e130154a51ce"></a>
## `crates/hickory-cli/tests/serve_capabilities.rs`

<a id="f-e130154a51ce-project_detection_exposes_every_provider_and_keeps_readiness_honest"></a>
### `project_detection_exposes_every_provider_and_keeps_readiness_honest`

Capability discovery describes providers without running project code.

<storycode:copy id="f-e130154a51ce-project_detection_exposes_every_provider_and_keeps_readiness_honest">
//! Capability discovery describes providers without running project code.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::Value;

#[tokio::test]
async fn project_detection_exposes_every_provider_and_keeps_readiness_honest() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("App.csproj"), "<Project />").unwrap();
    let prepared = prepare(ServeOptions {
        target: root.path().to_path_buf(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let response = reqwest::get(format!("{url}/api/capabilities?path=App.csproj"))
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body: Value = response.json().await.unwrap();
    let capabilities = body["capabilities"].as_array().unwrap();
    for interface in hick_plugins::registry()
        .plugins()
        .find(|p| p.id == "csharp")
        .unwrap()
        .interfaces
    {
        assert!(
            capabilities
                .iter()
                .any(|c| c["plugin"] == "csharp" && c["kind"] == format!("{:?}", interface.kind()))
        );
    }
    assert!(capabilities.iter().any(|c| c["language"] == "xml"));
    assert!(
        capabilities
            .iter()
            .all(|c| c["availability"] == "unchecked")
    );
    assert!(!root.path().join(".hick-cache/servers").exists());
    assert!(!root.path().join("bin").exists());
    let response = reqwest::get(format!("{url}/api/capabilities?path=..%2Foutside.py"))
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    task.abort();
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_capabilities.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_capabilities.rs"><storycode:paste select="#f-e130154a51ce-project_detection_exposes_every_provider_and_keeps_readiness_honest"/></storycode:file>

</storycode:doc>
