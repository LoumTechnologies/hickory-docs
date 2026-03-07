//! HTTP request/response interception for the in-process network stack.
//!
//! Provides an [`HttpFilter`] trait that lets callers inspect and modify HTTP/1.x
//! requests and responses flowing between the container and the real network.
//! The [`http_relay_real_socket`] function replaces [`relay_real_socket`](crate::network_loop)
//! for port-80 connections when a filter is configured.

use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, warn};
use smoltcp::iface::SocketHandle;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::network_loop::RealSocketMsg;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// An HTTP request parsed from the wire.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    /// 0 = HTTP/1.0, 1 = HTTP/1.1
    pub version: u8,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// An HTTP response parsed from the wire.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub reason: String,
    /// 0 = HTTP/1.0, 1 = HTTP/1.1
    pub version: u8,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Action returned by [`HttpFilter`] methods.
pub enum HttpFilterAction<T> {
    /// Forward the (possibly modified) value.
    Forward(T),
    /// Block the request/response.
    Block,
}

/// Trait for inspecting and modifying HTTP traffic.
#[async_trait]
pub trait HttpFilter: Send + Sync {
    /// Inspect/modify an HTTP request before forwarding to the real server.
    async fn filter_request(&self, request: HttpRequest) -> HttpFilterAction<HttpRequest>;

    /// Inspect/modify an HTTP response before returning to the container.
    async fn filter_response(
        &self,
        request: &HttpRequest,
        response: HttpResponse,
    ) -> HttpFilterAction<HttpResponse>;
}

/// Default filter that forwards everything unmodified.
pub struct PassthroughHttpFilter;

#[async_trait]
impl HttpFilter for PassthroughHttpFilter {
    async fn filter_request(&self, request: HttpRequest) -> HttpFilterAction<HttpRequest> {
        HttpFilterAction::Forward(request)
    }

    async fn filter_response(
        &self,
        _request: &HttpRequest,
        response: HttpResponse,
    ) -> HttpFilterAction<HttpResponse> {
        HttpFilterAction::Forward(response)
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse an HTTP/1.x request from an async buffered reader.
///
/// Reads headers with `httparse`, then reads the body according to
/// `Content-Length` (or chunked transfer-encoding). Returns `None` on EOF
/// before any data is read.
pub async fn parse_http_request<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> anyhow::Result<Option<HttpRequest>> {
    // Read header block (terminated by \r\n\r\n).
    let header_bytes = match read_until_double_crlf(reader).await? {
        Some(b) => b,
        None => return Ok(None),
    };

    let mut headers_buf = [httparse::EMPTY_HEADER; 64];
    let mut parsed = httparse::Request::new(&mut headers_buf);
    parsed
        .parse(&header_bytes)
        .map_err(|e| anyhow::anyhow!("HTTP request parse error: {e}"))?;

    let method = parsed
        .method
        .ok_or_else(|| anyhow::anyhow!("missing method"))?
        .to_string();
    let uri = parsed
        .path
        .ok_or_else(|| anyhow::anyhow!("missing path"))?
        .to_string();
    let version = parsed.version.unwrap_or(1);

    let headers: Vec<(String, String)> = parsed
        .headers
        .iter()
        .map(|h| {
            (
                h.name.to_string(),
                String::from_utf8_lossy(h.value).to_string(),
            )
        })
        .collect();

    let body = read_body(reader, &headers).await?;

    Ok(Some(HttpRequest {
        method,
        uri,
        version,
        headers,
        body,
    }))
}

/// Parse an HTTP/1.x response from an async buffered reader.
///
/// Returns `None` on EOF before any data is read.
pub async fn parse_http_response<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> anyhow::Result<Option<HttpResponse>> {
    let header_bytes = match read_until_double_crlf(reader).await? {
        Some(b) => b,
        None => return Ok(None),
    };

    let mut headers_buf = [httparse::EMPTY_HEADER; 64];
    let mut parsed = httparse::Response::new(&mut headers_buf);
    parsed
        .parse(&header_bytes)
        .map_err(|e| anyhow::anyhow!("HTTP response parse error: {e}"))?;

    let status = parsed
        .code
        .ok_or_else(|| anyhow::anyhow!("missing status code"))?;
    let reason = parsed.reason.unwrap_or("").to_string();
    let version = parsed.version.unwrap_or(1);

    let headers: Vec<(String, String)> = parsed
        .headers
        .iter()
        .map(|h| {
            (
                h.name.to_string(),
                String::from_utf8_lossy(h.value).to_string(),
            )
        })
        .collect();

    let body = read_body(reader, &headers).await?;

    Ok(Some(HttpResponse {
        status,
        reason,
        version,
        headers,
        body,
    }))
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/// Serialize an HTTP request to wire bytes.
///
/// Overwrites `Content-Length` to match `body.len()` and removes any
/// `Transfer-Encoding` header (chunked bodies are re-serialized with a
/// Content-Length).
pub fn serialize_http_request(req: &HttpRequest) -> Vec<u8> {
    let version = if req.version == 0 { "1.0" } else { "1.1" };
    let mut out = format!("{} {} HTTP/{}\r\n", req.method, req.uri, version);
    write_headers(&mut out, &req.headers, req.body.len());
    let mut bytes = out.into_bytes();
    bytes.extend_from_slice(&req.body);
    bytes
}

/// Serialize an HTTP response to wire bytes.
pub fn serialize_http_response(resp: &HttpResponse) -> Vec<u8> {
    let version = if resp.version == 0 { "1.0" } else { "1.1" };
    let mut out = format!("HTTP/{} {} {}\r\n", version, resp.status, resp.reason);
    write_headers(&mut out, &resp.headers, resp.body.len());
    let mut bytes = out.into_bytes();
    bytes.extend_from_slice(&resp.body);
    bytes
}

/// Generate a minimal 403 Forbidden response.
fn blocked_response() -> Vec<u8> {
    let body = b"Blocked by HTTP filter";
    let mut out = String::from("HTTP/1.1 403 Forbidden\r\n");
    out.push_str(&format!("Content-Length: {}\r\n", body.len()));
    out.push_str("Connection: close\r\n");
    out.push_str("\r\n");
    let mut bytes = out.into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

// ---------------------------------------------------------------------------
// HTTP-aware relay
// ---------------------------------------------------------------------------

/// HTTP-aware relay between the smoltcp side (channels) and a real TCP socket.
///
/// This is a drop-in replacement for `relay_real_socket` that parses HTTP
/// request/response pairs and calls the filter between them. It uses the
/// **same channel interface** so `tcp_proxy.rs` and the polling model are
/// unchanged.
pub(crate) async fn http_relay_real_socket(
    handle: SocketHandle,
    dest: std::net::SocketAddr,
    event_tx: mpsc::Sender<RealSocketMsg>,
    data_rx: mpsc::Receiver<Vec<u8>>,
    filter: Arc<dyn HttpFilter>,
) {
    // Connect to the real host.
    let stream = match TcpStream::connect(dest).await {
        Ok(s) => s,
        Err(_) => {
            let _ = event_tx.send(RealSocketMsg::ConnectFailed(handle)).await;
            return;
        }
    };

    let _ = event_tx.send(RealSocketMsg::Connected(handle)).await;
    debug!("[http] Connected to {dest}, starting HTTP relay");

    let (real_read, real_write) = tokio::io::split(stream);

    // Bridge the mpsc channel into an AsyncBufRead via a DuplexStream.
    let (duplex_write, duplex_read) = tokio::io::duplex(65536);
    let channel_bridge = tokio::spawn(bridge_channel_to_duplex(data_rx, duplex_write));

    let mut client_reader = tokio::io::BufReader::new(duplex_read);
    let mut server_reader = tokio::io::BufReader::new(real_read);
    let mut server_writer = real_write;

    // Process HTTP request/response pairs in a loop.
    loop {
        // 1. Parse request from the container (smoltcp side).
        let request = match parse_http_request(&mut client_reader).await {
            Ok(Some(r)) => r,
            Ok(None) => {
                debug!("[http] Client closed (EOF)");
                break;
            }
            Err(e) => {
                warn!("[http] Failed to parse request: {e}");
                break;
            }
        };

        debug!("[http] Request: {} {}", request.method, request.uri);

        // 2. Run filter.
        let request = match filter.filter_request(request).await {
            HttpFilterAction::Forward(r) => r,
            HttpFilterAction::Block => {
                debug!("[http] Request blocked by filter");
                let resp_bytes = blocked_response();
                let _ = event_tx.send(RealSocketMsg::Data(handle, resp_bytes)).await;
                let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
                channel_bridge.abort();
                return;
            }
        };

        // 3. Forward request to the real server.
        let req_bytes = serialize_http_request(&request);
        if server_writer.write_all(&req_bytes).await.is_err() {
            warn!("[http] Failed to write request to server");
            break;
        }
        if server_writer.flush().await.is_err() {
            warn!("[http] Failed to flush request to server");
            break;
        }

        // 4. Parse response from the real server.
        let response = match parse_http_response(&mut server_reader).await {
            Ok(Some(r)) => r,
            Ok(None) => {
                debug!("[http] Server closed (EOF)");
                break;
            }
            Err(e) => {
                warn!("[http] Failed to parse response: {e}");
                break;
            }
        };

        debug!("[http] Response: {} {}", response.status, response.reason);

        // 5. Run response filter.
        let response = match filter.filter_response(&request, response).await {
            HttpFilterAction::Forward(r) => r,
            HttpFilterAction::Block => {
                debug!("[http] Response blocked by filter");
                let resp_bytes = blocked_response();
                let _ = event_tx.send(RealSocketMsg::Data(handle, resp_bytes)).await;
                let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
                channel_bridge.abort();
                return;
            }
        };

        // 6. Send response back to the container.
        let resp_bytes = serialize_http_response(&response);
        if event_tx
            .send(RealSocketMsg::Data(handle, resp_bytes))
            .await
            .is_err()
        {
            break;
        }

        // Check Connection: close
        let close = has_connection_close(&response.headers) || response.version == 0;
        if close {
            debug!("[http] Connection: close — ending relay");
            break;
        }
    }

    let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
    channel_bridge.abort();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read from an mpsc channel and write into the DuplexStream so that the
/// HTTP parser can use `AsyncBufRead`.
async fn bridge_channel_to_duplex(
    mut rx: mpsc::Receiver<Vec<u8>>,
    mut writer: tokio::io::DuplexStream,
) {
    while let Some(data) = rx.recv().await {
        if writer.write_all(&data).await.is_err() {
            break;
        }
        if writer.flush().await.is_err() {
            break;
        }
    }
    // Drop writer to signal EOF to the reader side.
}

/// Read header bytes line-by-line until an empty line (`\r\n`) is found,
/// which marks the end of the HTTP header section. Returns the complete
/// header block including the trailing `\r\n\r\n`. Returns `None` on EOF
/// before any data is read.
async fn read_until_double_crlf<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> anyhow::Result<Option<Vec<u8>>> {
    let mut buf = Vec::with_capacity(4096);
    loop {
        let mut line = Vec::new();
        let n = reader.read_until(b'\n', &mut line).await?;
        if n == 0 {
            return if buf.is_empty() {
                Ok(None)
            } else {
                Err(anyhow::anyhow!("unexpected EOF in HTTP headers"))
            };
        }
        buf.extend_from_slice(&line);
        // An empty line (just \r\n) signals end of headers.
        if line == b"\r\n" {
            break;
        }
        if buf.len() > 64 * 1024 {
            return Err(anyhow::anyhow!("HTTP headers too large"));
        }
    }
    Ok(Some(buf))
}

/// Read the body based on Content-Length or Transfer-Encoding headers.
async fn read_body<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    headers: &[(String, String)],
) -> anyhow::Result<Vec<u8>> {
    // Check for chunked transfer-encoding.
    let is_chunked = headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("transfer-encoding")
            && value.to_ascii_lowercase().contains("chunked")
    });

    if is_chunked {
        return read_chunked_body(reader).await;
    }

    // Check Content-Length.
    let content_length: Option<usize> = headers.iter().find_map(|(name, value)| {
        if name.eq_ignore_ascii_case("content-length") {
            value.trim().parse().ok()
        } else {
            None
        }
    });

    match content_length {
        Some(len) => {
            let mut body = vec![0u8; len];
            reader.read_exact(&mut body).await?;
            Ok(body)
        }
        None => Ok(Vec::new()),
    }
}

/// Read a chunked transfer-encoded body and return the concatenated bytes.
async fn read_chunked_body<R: AsyncBufRead + Unpin>(reader: &mut R) -> anyhow::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let size_str = line.trim();
        let size = usize::from_str_radix(size_str.split(';').next().unwrap_or("0"), 16)?;
        if size == 0 {
            // Read trailing \r\n after the last chunk.
            let mut trailer = String::new();
            reader.read_line(&mut trailer).await?;
            break;
        }
        let mut chunk = vec![0u8; size];
        reader.read_exact(&mut chunk).await?;
        body.extend_from_slice(&chunk);
        // Read trailing \r\n after chunk data.
        let mut crlf = [0u8; 2];
        reader.read_exact(&mut crlf).await?;
    }
    Ok(body)
}

/// Write headers to a string, replacing Content-Length and removing
/// Transfer-Encoding.
fn write_headers(out: &mut String, headers: &[(String, String)], body_len: usize) {
    let mut wrote_content_length = false;
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("content-length") {
            out.push_str(&format!("Content-Length: {body_len}\r\n"));
            wrote_content_length = true;
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            // Drop chunked encoding — we re-serialize with Content-Length.
            continue;
        } else {
            out.push_str(&format!("{name}: {value}\r\n"));
        }
    }
    if !wrote_content_length && body_len > 0 {
        out.push_str(&format!("Content-Length: {body_len}\r\n"));
    }
    out.push_str("\r\n");
}

/// Check if any header indicates `Connection: close`.
fn has_connection_close(headers: &[(String, String)]) -> bool {
    headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("connection") && value.eq_ignore_ascii_case("close")
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Parsing tests --

    #[tokio::test]
    async fn test_parse_request_get() {
        let raw = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let mut reader = tokio::io::BufReader::new(&raw[..]);
        let req = parse_http_request(&mut reader).await.unwrap().unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.uri, "/index.html");
        assert_eq!(req.version, 1);
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].0, "Host");
        assert_eq!(req.headers[0].1, "example.com");
        assert!(req.body.is_empty());
    }

    #[tokio::test]
    async fn test_parse_request_post_with_body() {
        let raw =
            b"POST /api HTTP/1.1\r\nHost: example.com\r\nContent-Length: 13\r\n\r\nHello, world!";
        let mut reader = tokio::io::BufReader::new(&raw[..]);
        let req = parse_http_request(&mut reader).await.unwrap().unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.uri, "/api");
        assert_eq!(req.body, b"Hello, world!");
    }

    #[tokio::test]
    async fn test_parse_response_200() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        let mut reader = tokio::io::BufReader::new(&raw[..]);
        let resp = parse_http_response(&mut reader).await.unwrap().unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.reason, "OK");
        assert_eq!(resp.version, 1);
        assert_eq!(resp.body, b"hello");
    }

    #[tokio::test]
    async fn test_parse_response_no_body() {
        let raw = b"HTTP/1.1 304 Not Modified\r\n\r\n";
        let mut reader = tokio::io::BufReader::new(&raw[..]);
        let resp = parse_http_response(&mut reader).await.unwrap().unwrap();
        assert_eq!(resp.status, 304);
        assert_eq!(resp.reason, "Not Modified");
        assert!(resp.body.is_empty());
    }

    // -- Serialization tests --

    #[test]
    fn test_serialize_request() {
        let req = HttpRequest {
            method: "GET".into(),
            uri: "/test".into(),
            version: 1,
            headers: vec![("Host".into(), "example.com".into())],
            body: Vec::new(),
        };
        let bytes = serialize_http_request(&req);
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.starts_with("GET /test HTTP/1.1\r\n"));
        assert!(s.contains("Host: example.com\r\n"));
        assert!(s.ends_with("\r\n"));
    }

    #[test]
    fn test_serialize_response() {
        let resp = HttpResponse {
            status: 200,
            reason: "OK".into(),
            version: 1,
            headers: vec![("X-Custom".into(), "yes".into())],
            body: b"body".to_vec(),
        };
        let bytes = serialize_http_response(&resp);
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("X-Custom: yes\r\n"));
        assert!(s.contains("Content-Length: 4\r\n"));
        assert!(s.ends_with("body"));
    }

    #[tokio::test]
    async fn test_serialize_roundtrip_request() {
        let req = HttpRequest {
            method: "POST".into(),
            uri: "/data".into(),
            version: 1,
            headers: vec![
                ("Host".into(), "example.com".into()),
                ("Content-Length".into(), "3".into()),
            ],
            body: b"abc".to_vec(),
        };
        let bytes = serialize_http_request(&req);
        let mut reader = tokio::io::BufReader::new(&bytes[..]);
        let parsed = parse_http_request(&mut reader).await.unwrap().unwrap();
        assert_eq!(parsed.method, "POST");
        assert_eq!(parsed.uri, "/data");
        assert_eq!(parsed.body, b"abc");
    }

    #[tokio::test]
    async fn test_serialize_roundtrip_response() {
        let resp = HttpResponse {
            status: 404,
            reason: "Not Found".into(),
            version: 1,
            headers: vec![("Content-Length".into(), "9".into())],
            body: b"not found".to_vec(),
        };
        let bytes = serialize_http_response(&resp);
        let mut reader = tokio::io::BufReader::new(&bytes[..]);
        let parsed = parse_http_response(&mut reader).await.unwrap().unwrap();
        assert_eq!(parsed.status, 404);
        assert_eq!(parsed.reason, "Not Found");
        assert_eq!(parsed.body, b"not found");
    }

    // -- Integration tests using real TCP --

    #[tokio::test]
    async fn test_http_relay_passthrough() {
        // Start a mock HTTP server.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            // Read request.
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.unwrap();
            assert!(n > 0);
            // Send response.
            let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
            stream.write_all(resp).await.unwrap();
            stream.shutdown().await.unwrap();
        });

        // Set up channels simulating the smoltcp side.
        let (event_tx, mut event_rx) = mpsc::channel::<RealSocketMsg>(64);
        let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>(64);

        let handle = dummy_socket_handle();
        let filter: Arc<dyn HttpFilter> = Arc::new(PassthroughHttpFilter);

        let relay_task = tokio::spawn(http_relay_real_socket(
            handle,
            server_addr,
            event_tx,
            data_rx,
            filter,
        ));

        // Wait for Connected.
        let msg = event_rx.recv().await.unwrap();
        assert!(matches!(msg, RealSocketMsg::Connected(_)));

        // Send a GET request from the "container".
        let req = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        data_tx.send(req.to_vec()).await.unwrap();

        // Collect response data.
        let mut response_data = Vec::new();
        loop {
            match event_rx.recv().await {
                Some(RealSocketMsg::Data(_, data)) => response_data.extend_from_slice(&data),
                Some(RealSocketMsg::Closed(_)) => break,
                other => panic!("unexpected message: {other:?}"),
            }
        }

        let response_str = String::from_utf8(response_data).unwrap();
        assert!(response_str.contains("200 OK"));
        assert!(response_str.contains("ok"));

        relay_task.await.unwrap();
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_relay_modify_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await.unwrap();
            let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest";
            stream.write_all(resp).await.unwrap();
            stream.shutdown().await.unwrap();
        });

        // Filter that adds an X-Intercepted header.
        struct AddHeaderFilter;
        #[async_trait]
        impl HttpFilter for AddHeaderFilter {
            async fn filter_request(&self, request: HttpRequest) -> HttpFilterAction<HttpRequest> {
                HttpFilterAction::Forward(request)
            }
            async fn filter_response(
                &self,
                _request: &HttpRequest,
                mut response: HttpResponse,
            ) -> HttpFilterAction<HttpResponse> {
                response
                    .headers
                    .push(("X-Intercepted".into(), "true".into()));
                HttpFilterAction::Forward(response)
            }
        }

        let (event_tx, mut event_rx) = mpsc::channel::<RealSocketMsg>(64);
        let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>(64);

        let handle = dummy_socket_handle();
        let filter: Arc<dyn HttpFilter> = Arc::new(AddHeaderFilter);

        let relay_task = tokio::spawn(http_relay_real_socket(
            handle,
            server_addr,
            event_tx,
            data_rx,
            filter,
        ));

        let msg = event_rx.recv().await.unwrap();
        assert!(matches!(msg, RealSocketMsg::Connected(_)));

        let req = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        data_tx.send(req.to_vec()).await.unwrap();

        let mut response_data = Vec::new();
        loop {
            match event_rx.recv().await {
                Some(RealSocketMsg::Data(_, data)) => response_data.extend_from_slice(&data),
                Some(RealSocketMsg::Closed(_)) => break,
                other => panic!("unexpected message: {other:?}"),
            }
        }

        let response_str = String::from_utf8(response_data).unwrap();
        assert!(response_str.contains("X-Intercepted: true"));
        assert!(response_str.contains("test"));

        relay_task.await.unwrap();
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_relay_modify_body() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await.unwrap();
            let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\noriginal";
            stream.write_all(resp).await.unwrap();
            stream.shutdown().await.unwrap();
        });

        struct ReplaceBodyFilter;
        #[async_trait]
        impl HttpFilter for ReplaceBodyFilter {
            async fn filter_request(&self, request: HttpRequest) -> HttpFilterAction<HttpRequest> {
                HttpFilterAction::Forward(request)
            }
            async fn filter_response(
                &self,
                _request: &HttpRequest,
                mut response: HttpResponse,
            ) -> HttpFilterAction<HttpResponse> {
                response.body = b"replaced".to_vec();
                HttpFilterAction::Forward(response)
            }
        }

        let (event_tx, mut event_rx) = mpsc::channel::<RealSocketMsg>(64);
        let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>(64);

        let handle = dummy_socket_handle();
        let filter: Arc<dyn HttpFilter> = Arc::new(ReplaceBodyFilter);

        let relay_task = tokio::spawn(http_relay_real_socket(
            handle,
            server_addr,
            event_tx,
            data_rx,
            filter,
        ));

        let msg = event_rx.recv().await.unwrap();
        assert!(matches!(msg, RealSocketMsg::Connected(_)));

        let req = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        data_tx.send(req.to_vec()).await.unwrap();

        let mut response_data = Vec::new();
        loop {
            match event_rx.recv().await {
                Some(RealSocketMsg::Data(_, data)) => response_data.extend_from_slice(&data),
                Some(RealSocketMsg::Closed(_)) => break,
                other => panic!("unexpected message: {other:?}"),
            }
        }

        let response_str = String::from_utf8(response_data).unwrap();
        assert!(response_str.contains("replaced"));
        assert!(!response_str.contains("original"));
        // Content-Length should match the new body.
        assert!(response_str.contains("Content-Length: 8"));

        relay_task.await.unwrap();
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_relay_block_request() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        // Server should NOT receive any request (we'll detect via timeout).
        let server_task = tokio::spawn(async move {
            let result =
                tokio::time::timeout(std::time::Duration::from_millis(500), listener.accept())
                    .await;
            // The relay connects first, then blocks — so server may accept
            // but should not receive a full request.
            drop(result);
        });

        struct BlockFilter;
        #[async_trait]
        impl HttpFilter for BlockFilter {
            async fn filter_request(&self, _request: HttpRequest) -> HttpFilterAction<HttpRequest> {
                HttpFilterAction::Block
            }
            async fn filter_response(
                &self,
                _request: &HttpRequest,
                response: HttpResponse,
            ) -> HttpFilterAction<HttpResponse> {
                HttpFilterAction::Forward(response)
            }
        }

        let (event_tx, mut event_rx) = mpsc::channel::<RealSocketMsg>(64);
        let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>(64);

        let handle = dummy_socket_handle();
        let filter: Arc<dyn HttpFilter> = Arc::new(BlockFilter);

        let relay_task = tokio::spawn(http_relay_real_socket(
            handle,
            server_addr,
            event_tx,
            data_rx,
            filter,
        ));

        let msg = event_rx.recv().await.unwrap();
        assert!(matches!(msg, RealSocketMsg::Connected(_)));

        let req = b"GET /secret HTTP/1.1\r\nHost: localhost\r\n\r\n";
        data_tx.send(req.to_vec()).await.unwrap();

        // Should receive 403 response.
        let mut response_data = Vec::new();
        loop {
            match event_rx.recv().await {
                Some(RealSocketMsg::Data(_, data)) => response_data.extend_from_slice(&data),
                Some(RealSocketMsg::Closed(_)) => break,
                other => panic!("unexpected message: {other:?}"),
            }
        }

        let response_str = String::from_utf8(response_data).unwrap();
        assert!(response_str.contains("403 Forbidden"));
        assert!(response_str.contains("Blocked by HTTP filter"));

        relay_task.await.unwrap();
        server_task.await.unwrap();
    }

    /// Create a dummy SocketHandle for testing. The handle value doesn't
    /// matter since we don't interact with smoltcp in relay tests.
    fn dummy_socket_handle() -> SocketHandle {
        // SocketHandle is just a newtype around usize internally.
        // We create a real one via a VirtualNetwork to avoid relying on
        // internal representation.
        let mut net = crate::interface::VirtualNetwork::new();
        net.add_tcp_socket(1024, 1024)
    }
}
