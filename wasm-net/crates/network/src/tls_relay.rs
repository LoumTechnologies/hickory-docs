//! TLS MITM relay for auth-rule-matched HTTPS connections.
//!
//! Intercepts TLS connections to hosts with auth injection rules, terminates
//! TLS using an ephemeral per-host certificate (signed by [`HickCa`]), applies
//! the [`HttpFilter`] to the decrypted traffic, then re-encrypts and forwards
//! to the real destination.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use log::{debug, warn};
use rustls::pki_types::ServerName;
use smoltcp::iface::SocketHandle;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::sync::mpsc;
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::http::{
    self, HttpFilter, HttpFilterAction, parse_http_request, parse_http_response,
    serialize_http_request, serialize_http_response,
};
use crate::network_loop::RealSocketMsg;
use crate::tls_ca::HickCa;

/// TLS MITM relay for a single connection.
///
/// 1. Bridges the mpsc channel into a DuplexStream
/// 2. Buffers initial data to extract SNI from TLS ClientHello
/// 3. Generates a cert for the hostname using the CA
/// 4. Completes TLS handshake with the container (as server)
/// 5. Establishes real TLS to the destination (as client)
/// 6. Runs HTTP relay loop with the filter applied
pub(crate) async fn tls_mitm_relay(
    handle: SocketHandle,
    dest: std::net::SocketAddr,
    event_tx: mpsc::Sender<RealSocketMsg>,
    data_rx: mpsc::Receiver<Vec<u8>>,
    ca: Arc<HickCa>,
    http_filter: Arc<dyn HttpFilter>,
) {
    // Bridge the mpsc channel into a duplex stream for async I/O.
    let (duplex_write, duplex_read) = tokio::io::duplex(65536);
    let channel_bridge = tokio::spawn(bridge_channel_to_duplex(data_rx, duplex_write));

    let _ = event_tx.send(RealSocketMsg::Connected(handle)).await;

    // Buffer initial data to extract SNI.
    let mut initial_buf = Vec::with_capacity(4096);
    let mut reader = tokio::io::BufReader::new(duplex_read);

    // Read enough data for the ClientHello.
    use tokio::io::AsyncReadExt;
    let mut peek_buf = [0u8; 4096];
    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        reader.read(&mut peek_buf),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => {
            initial_buf.extend_from_slice(&peek_buf[..n]);
        }
        _ => {
            debug!("[tls-mitm] Failed to read ClientHello");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    }

    // Extract SNI hostname.
    let hostname = match crate::tls_sni::extract_sni(&initial_buf) {
        Some(h) => h,
        None => {
            warn!("[tls-mitm] No SNI in ClientHello, cannot intercept");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    debug!("[tls-mitm] Intercepting TLS to {hostname} (dest={dest})");

    // Generate a certificate for this hostname.
    let host_cert = match ca.cert_for_host(&hostname) {
        Ok(c) => c,
        Err(e) => {
            warn!("[tls-mitm] Failed to generate cert for {hostname}: {e}");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    // Build TLS server config for the container-facing side.
    let server_config = match rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![host_cert.cert_der.clone()],
            host_cert.key_der.clone_key(),
        ) {
        Ok(c) => c,
        Err(e) => {
            warn!("[tls-mitm] Failed to build server TLS config: {e}");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    // Create a ReplayStream that first yields the buffered ClientHello,
    // then reads from the duplex stream.
    let replay = ReplayStream::new(initial_buf, reader);

    // Accept TLS from the container.
    let container_tls = match acceptor.accept(replay).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[tls-mitm] TLS accept failed: {e}");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    // Connect to the real destination with TLS.
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().certs {
        let _ = root_store.add(cert);
    }
    let client_config = match rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
    {
        c => c,
    };

    let connector = TlsConnector::from(Arc::new(client_config));
    let server_name = match ServerName::try_from(hostname.clone()) {
        Ok(s) => s,
        Err(e) => {
            warn!("[tls-mitm] Invalid server name '{hostname}': {e}");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    let tcp_stream = match tokio::net::TcpStream::connect(dest).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[tls-mitm] Failed to connect to {dest}: {e}");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    let server_tls = match connector.connect(server_name, tcp_stream).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[tls-mitm] TLS connect to {dest} failed: {e}");
            let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
            channel_bridge.abort();
            return;
        }
    };

    debug!("[tls-mitm] TLS MITM established for {hostname}");

    // Run HTTP relay over the decrypted streams.
    let (container_read, _container_write) = tokio::io::split(container_tls);
    let (server_read, mut server_write) = tokio::io::split(server_tls);

    let mut client_reader = tokio::io::BufReader::new(container_read);
    let mut server_reader = tokio::io::BufReader::new(server_read);

    loop {
        let request = match parse_http_request(&mut client_reader).await {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(e) => {
                debug!("[tls-mitm] Request parse error: {e}");
                break;
            }
        };

        debug!("[tls-mitm] {} {} ({})", request.method, request.uri, hostname);

        let request = match http_filter.filter_request(request).await {
            HttpFilterAction::Forward(r) => r,
            HttpFilterAction::Block => {
                debug!("[tls-mitm] Request blocked by filter");
                let resp_bytes = http::serialize_http_response(&http::HttpResponse {
                    status: 403,
                    reason: "Forbidden".into(),
                    version: 1,
                    headers: vec![
                        ("Content-Length".into(), "22".into()),
                        ("Connection".into(), "close".into()),
                    ],
                    body: b"Blocked by HTTP filter".to_vec(),
                });
                let _ = event_tx.send(RealSocketMsg::Data(handle, resp_bytes)).await;
                break;
            }
        };

        let req_bytes = serialize_http_request(&request);
        if server_write.write_all(&req_bytes).await.is_err() {
            break;
        }
        if server_write.flush().await.is_err() {
            break;
        }

        let response = match parse_http_response(&mut server_reader).await {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(e) => {
                debug!("[tls-mitm] Response parse error: {e}");
                break;
            }
        };

        let response = match http_filter.filter_response(&request, response).await {
            HttpFilterAction::Forward(r) => r,
            HttpFilterAction::Block => break,
        };

        let resp_bytes = serialize_http_response(&response);
        if event_tx
            .send(RealSocketMsg::Data(handle, resp_bytes))
            .await
            .is_err()
        {
            break;
        }

        // Check Connection: close
        let close = response
            .headers
            .iter()
            .any(|(n, v)| n.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close"))
            || response.version == 0;
        if close {
            break;
        }
    }

    let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
    channel_bridge.abort();
}

/// Bridge mpsc channel data into a DuplexStream for async I/O.
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
}

/// A stream that first yields buffered bytes, then delegates to an inner stream.
///
/// Used to replay the TLS ClientHello (consumed during SNI extraction) back
/// to the TLS acceptor.
struct ReplayStream<S> {
    buffer: Vec<u8>,
    buffer_pos: usize,
    inner: S,
}

impl<S> ReplayStream<S> {
    fn new(buffer: Vec<u8>, inner: S) -> Self {
        Self {
            buffer,
            buffer_pos: 0,
            inner,
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for ReplayStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();

        // First, drain the buffer.
        if this.buffer_pos < this.buffer.len() {
            let remaining = &this.buffer[this.buffer_pos..];
            let to_copy = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..to_copy]);
            this.buffer_pos += to_copy;
            return Poll::Ready(Ok(()));
        }

        // Then delegate to the inner stream.
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for ReplayStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
