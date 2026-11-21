use super::interrupted;
use anyhow::{Context, Result, ensure};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use hickory_cli::{ExecutorChoice, engine};
use std::{io::Write, path::Path};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

fn admits(headers: &HeaderMap, authority: &str, origin: &str) -> bool {
    headers
        .get(header::HOST)
        .is_some_and(|host| host == authority)
        && headers
            .get(header::ORIGIN)
            .is_none_or(|found| found == origin)
        && headers
            .get("sec-fetch-site")
            .is_none_or(|site| site == "same-origin" || site == "none")
}

async fn local_request(request: Request, next: Next, authority: &str, origin: &str) -> Response {
    if !admits(request.headers(), authority, origin) {
        return StatusCode::FORBIDDEN.into_response();
    }
    next.run(request).await
}

pub(super) async fn run(args: &[String]) -> Result<()> {
    let [workspace, assets, port] = args else {
        anyhow::bail!(
            "Usage: hickory-simple-host --linux-host WORKSPACE COMPILED_ASSETS LOOPBACK_PORT"
        );
    };
    let root = Path::new(workspace)
        .canonicalize()
        .with_context(|| format!("Open workspace {workspace}. Choose an existing folder."))?;
    ensure!(
        root.is_dir(),
        "Choose a workspace folder: {}",
        root.display()
    );
    let assets = Path::new(assets)
        .canonicalize()
        .context("Editor assets are missing. Rebuild or reinstall Hickory Simple.")?;
    ensure!(
        assets.join("simple.html").is_file(),
        "Editor assets are incomplete in {}. Rebuild or reinstall Hickory Simple.",
        assets.display()
    );
    let port: u16 = port
        .parse()
        .context("Use a loopback port between 0 and 65535.")?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .with_context(|| {
            format!(
                "Cannot bind 127.0.0.1:{port}. Close the other Hickory window or choose --port."
            )
        })?;
    let address = listener.local_addr()?;
    let authority = address.to_string();
    let origin = format!("http://{authority}");

    let mut attach = engine::Attach::new(root, ExecutorChoice::Local);
    attach.callback = Some(origin.clone());
    attach.callback_token = engine::window_token();
    attach.key_store_path = hickory_agent::KeyStore::desktop_path();
    let token = attach.callback_token.clone();
    let (closed, owner) = tokio::sync::oneshot::channel();
    // A dedicated thread can wait for pipe EOF without blocking Tokio shutdown
    // on an open terminal after a signal or server failure.
    std::thread::spawn(move || {
        let _ = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
        let _ = closed.send(());
    });
    tokio::pin!(owner);
    let connection = tokio::select! {
        result = engine::connect(attach) => result?,
        _ = &mut owner => return Ok(()),
        result = interrupted() => return result,
    };
    let guard = connection.guard();
    let router = engine::client_router(connection, None, token, None)
        .fallback_service(ServeDir::new(assets))
        .layer(middleware::from_fn(move |request, next| {
            let authority = authority.clone();
            let origin = origin.clone();
            async move { local_request(request, next, &authority, &origin).await }
        }));
    let result = async {
        println!("HICKORY_SIMPLE_PORT {}", address.port());
        std::io::stdout().flush()?;
        tokio::select! {
            result = axum::serve(listener, router) => Ok(result?),
            _ = &mut owner => Ok(()),
            result = interrupted() => result,
        }
    }
    .await;
    let detached = guard.detach().await;
    result.and(detached)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_host_admits_only_its_own_origin() {
        let authority = "127.0.0.1:38174";
        let origin = "http://127.0.0.1:38174";
        let mut headers = HeaderMap::new();
        assert!(!admits(&headers, authority, origin));
        headers.insert(header::HOST, authority.parse().unwrap());
        assert!(admits(&headers, authority, origin));
        headers.insert(header::ORIGIN, origin.parse().unwrap());
        headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
        assert!(admits(&headers, authority, origin));
        headers.insert(header::ORIGIN, "https://another.example".parse().unwrap());
        assert!(!admits(&headers, authority, origin));
        headers.remove(header::ORIGIN);
        headers.insert("sec-fetch-site", "same-site".parse().unwrap());
        assert!(!admits(&headers, authority, origin));
        headers.remove("sec-fetch-site");
        headers.insert(header::HOST, "other.example:38174".parse().unwrap());
        assert!(!admits(&headers, authority, origin));
    }
}
