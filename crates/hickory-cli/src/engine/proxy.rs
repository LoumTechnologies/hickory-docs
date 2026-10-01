//! Each desktop serves its UI and proxies engine requests on one origin.
//! Only native shell actions live here; no documents or watchers are created.
use super::Connection;
use crate::serve::{OpenWhere, Shell};
use anyhow::{Result, bail};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{
        Request, State,
        ws::{Message, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
struct Proxy {
    connection: Connection,
    shell: Option<Arc<Shell>>,
    token: String,
    ui_origin: Option<String>,
}

pub fn client_router(
    connection: Connection,
    shell: Option<Shell>,
    token: String,
    ui_origin: Option<String>,
) -> Router {
    Router::new()
        .route("/api", axum::routing::any(forward))
        .route("/api/{*path}", axum::routing::any(forward))
        .route("/__shell/{action}", axum::routing::post(callback))
        .with_state(Proxy {
            connection,
            shell: shell.map(Arc::new),
            token,
            ui_origin,
        })
}

async fn callback(State(proxy): State<Proxy>, req: Request) -> Response {
    if req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        != Some(format!("Bearer {}", proxy.token).as_str())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(shell) = proxy.shell else {
        return StatusCode::NOT_IMPLEMENTED.into_response();
    };
    let action = req
        .uri()
        .path()
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string();
    let body = match to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let value: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let result = tokio::task::spawn_blocking(move || -> Result<Value> {
        let path = PathBuf::from(value["path"].as_str().unwrap_or_default());
        match action.as_str() {
            "pick" => Ok(json!({"path": (shell.pick_folder)(&path)?})),
            "save" => Ok(json!({"path": (shell.save_file)(&path, value["name"].as_str().unwrap_or_default())?})),
            "open" => { (shell.open_folder)(&path, serde_json::from_value(value["where"].clone())?)?; Ok(json!({})) }
            "close" => { (shell.close_window)()?; Ok(json!({})) }
            _ => bail!("unknown native window action"),
        }
    }).await;
    match result {
        Ok(Ok(value)) => Json(value).into_response(),
        other => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error":format!("{other:?}")})),
        )
            .into_response(),
    }
}

async fn forward(State(proxy): State<Proxy>, req: Request) -> Response {
    // A website cannot ask a local window to edit files or run a command.
    if let Some(origin) = req.headers().get("origin").and_then(|v| v.to_str().ok()) {
        let own = format!(
            "http://{}",
            req.headers()
                .get("host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
        );
        if origin != own && proxy.ui_origin.as_deref() != Some(origin) {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    let path = req.uri().to_string();
    if req.headers().contains_key("upgrade") {
        let (mut parts, body) = req.into_parts();
        return match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
            Ok(upgrade) => {
                let connection = proxy.connection;
                upgrade
                    .on_upgrade(move |socket| bridge(connection, path, socket))
                    .into_response()
            }
            Err(error) => {
                let _ = body;
                error.into_response()
            }
        };
    }
    let (parts, body) = req.into_parts();
    let body = match to_bytes(body, 64 * 1024 * 1024).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let (url, token) = proxy.connection.api_url(&path).await;
    let mut headers = parts.headers;
    for name in ["host", "authorization", "connection", "transfer-encoding"] {
        headers.remove(name);
    }
    let response = proxy
        .connection
        .http
        .request(parts.method, url)
        .headers(headers)
        .bearer_auth(token)
        .body(body)
        .send()
        .await;
    match response {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let mut result = Response::new(Body::from_stream(response.bytes_stream()));
            *result.status_mut() = status;
            *result.headers_mut() = headers;
            result.headers_mut().remove("transfer-encoding");
            result
        }
        Err(_) => {
            // Never replay a mutation after a lost response: it might already
            // have committed. The heartbeat restores subsequent requests.
            let _ = proxy.connection.reconnect().await;
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error":"The local engine disconnected. Your buffer is still here; reconnect before saving or running again."}))).into_response()
        }
    }
}

use axum::extract::FromRequestParts;
async fn bridge(connection: Connection, path: String, socket: axum::extract::ws::WebSocket) {
    use tokio_tungstenite::tungstenite::{Message as Peer, client::IntoClientRequest};
    let (url, token) = connection.api_url(&path).await;
    let Ok(mut request) = url.replacen("http:", "ws:", 1).into_client_request() else {
        return;
    };
    request
        .headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let Ok((upstream, _)) = tokio_tungstenite::connect_async(request).await else {
        return;
    };
    let (mut browser_out, mut browser_in) = socket.split();
    let (mut engine_out, mut engine_in) = upstream.split();
    let to_engine = async {
        while let Some(Ok(message)) = browser_in.next().await {
            let message = match message {
                Message::Binary(b) => Peer::Binary(b.to_vec()),
                Message::Text(t) => Peer::Text(t.to_string()),
                Message::Ping(b) => Peer::Ping(b.to_vec()),
                Message::Pong(b) => Peer::Pong(b.to_vec()),
                Message::Close(_) => break,
            };
            if engine_out.send(message).await.is_err() {
                break;
            }
        }
        let _ = engine_out.close().await;
    };
    let to_browser = async {
        while let Some(Ok(message)) = engine_in.next().await {
            let message = match message {
                Peer::Binary(b) => Message::Binary(b.into()),
                Peer::Text(t) => Message::Text(t.into()),
                Peer::Ping(b) => Message::Ping(b.into()),
                Peer::Pong(b) => Message::Pong(b.into()),
                Peer::Close(_) => break,
                _ => continue,
            };
            if browser_out.send(message).await.is_err() {
                break;
            }
        }
        let _ = browser_out.close().await;
    };
    tokio::select! { _ = to_engine => {}, _ = to_browser => {} }
}

pub(super) fn remote_shell(url: String, token: String) -> Shell {
    let call = Arc::new(move |action: &str, value: Value| -> Result<Value> {
        let url = format!("{url}/__shell/{action}");
        let token = token.clone();
        // Shell's facade is synchronous; callbacks must not block a runtime
        // worker that is needed to service the native dialog's HTTP request.
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async {
                let response = reqwest::Client::builder()
                    .no_proxy()
                    .build()?
                    .post(url)
                    .bearer_auth(token)
                    .json(&value)
                    .send()
                    .await?;
                if !response.status().is_success() {
                    bail!(
                        "the requesting window could not complete its native action: {}",
                        response.text().await?
                    );
                }
                Ok(response.json().await?)
            })
        })
        .join()
        .map_err(|_| anyhow::anyhow!("native window callback stopped"))?
    });
    let pick = call.clone();
    let save = call.clone();
    let open = call.clone();
    Shell {
        pick_folder: Arc::new(move |path: &Path| {
            let v = pick("pick", json!({"path":path}))?;
            Ok(v["path"].as_str().map(PathBuf::from))
        }),
        save_file: Arc::new(move |path: &Path, name: &str| {
            let v = save("save", json!({"path":path,"name":name}))?;
            Ok(v["path"].as_str().map(PathBuf::from))
        }),
        open_folder: Arc::new(move |path: &Path, where_: OpenWhere| {
            open("open", json!({"path":path,"where":where_}))?;
            Ok(())
        }),
        close_window: Arc::new(move || {
            call("close", json!({}))?;
            Ok(())
        }),
    }
}
