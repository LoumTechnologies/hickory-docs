<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/workspace_fs/host.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a9f697a11c80"></a>
## `crates/hickory-cli/src/serve/workspace_fs/host.rs`

<a id="f-a9f697a11c80-Host"></a>
### `Host`

<storycode:copy id="f-a9f697a11c80-Host">
use super::Engine;
use anyhow::Result;
use axum::{Json, Router, extract::State, routing::post};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Host {
    pub url: String,
    task: tokio::task::JoinHandle<()>,
}
</storycode:copy>
<a id="f-a9f697a11c80-impl-Host"></a>
### `impl-Host`

<storycode:copy id="f-a9f697a11c80-impl-Host">
impl Host {
    pub async fn start(engine: Engine) -> Result<Self> {
        let token = format!(
            "{:016x}{:016x}",
            super::super::rand_id(),
            super::super::rand_id()
        );
        let router = Router::new()
            .route(&format!("/{token}"), post(exchange))
            .layer(axum::extract::DefaultBodyLimit::max(90 * 1024 * 1024))
            .with_state(Arc::new(Mutex::new(engine)));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/{token}", listener.local_addr()?);
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        Ok(Self { url, task })
    }
}
</storycode:copy>
<a id="f-a9f697a11c80-impl-Drop-for-Host"></a>
### `impl-Drop for Host`

<storycode:copy id="f-a9f697a11c80-impl-Drop-for-Host">
impl Drop for Host {
    fn drop(&mut self) {
        self.task.abort();
    }
}
</storycode:copy>
<a id="f-a9f697a11c80-exchange"></a>
### `exchange`

<storycode:copy id="f-a9f697a11c80-exchange">
async fn exchange(
    State(engine): State<Arc<Mutex<Engine>>>,
    Json(request): Json<Value>,
) -> Json<Value> {
    let gate = engine.lock().await.gate.clone();
    let _gate = gate.lock().await;
    let result = engine.lock().await.request(&request).await;
    Json(match result {
        Ok(value) => json!({"result":value}),
        Err(e) => {
            let errno = e
                .chain()
                .find_map(|e| {
                    e.downcast_ref::<std::io::Error>()
                        .and_then(|e| e.raw_os_error())
                })
                .unwrap_or(1);
            json!({"error":format!("{e:#}"),"errno":errno})
        }
    })
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/workspace_fs/host.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/workspace_fs/host.rs"><storycode:paste select="#f-a9f697a11c80-Host"/><storycode:paste select="#f-a9f697a11c80-impl-Host"/><storycode:paste select="#f-a9f697a11c80-impl-Drop-for-Host"/><storycode:paste select="#f-a9f697a11c80-exchange"/></storycode:file>

</storycode:doc>
