<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/serve_capabilities.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `19114b7e7409e87993bbc0baa74395dd0486546113bf5e3fe69a7bf7ce4108da`.

<book:entry path="crates/hickory-cli/tests/serve_capabilities.rs">
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
</book:entry>

</book:doc>
