//! Interceptor trait for extensible TCP connection handling.
//!
//! The [`Interceptor`] trait allows custom handling of TCP connections based
//! on destination address/port. The default [`PassthroughInterceptor`] forwards
//! all traffic unmodified. Custom interceptors can block, log, or modify
//! connections.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};

/// Result type for interceptor operations.
pub type InterceptResult = anyhow::Result<()>;

/// A handler for intercepted TCP connections.
///
/// Implementations decide whether to handle a connection based on its
/// destination, and if so, process the bidirectional data stream.
pub trait Interceptor: Send + Sync {
    /// Whether this interceptor should handle a connection to `dest`.
    fn matches(&self, dest: SocketAddr) -> bool;

    /// Handle the connection. The interceptor owns the bidirectional stream
    /// to the real destination and receives data from the smoltcp side via
    /// the provided reader/writer.
    fn handle(
        &self,
        dest: SocketAddr,
        reader: Pin<Box<dyn AsyncRead + Send>>,
        writer: Pin<Box<dyn AsyncWrite + Send>>,
    ) -> Pin<Box<dyn Future<Output = InterceptResult> + Send>>;
}

/// Default interceptor that forwards all traffic without modification.
pub struct PassthroughInterceptor;

impl Interceptor for PassthroughInterceptor {
    fn matches(&self, _dest: SocketAddr) -> bool {
        true
    }

    fn handle(
        &self,
        dest: SocketAddr,
        reader: Pin<Box<dyn AsyncRead + Send>>,
        writer: Pin<Box<dyn AsyncWrite + Send>>,
    ) -> Pin<Box<dyn Future<Output = InterceptResult> + Send>> {
        Box::pin(async move {
            let stream = tokio::net::TcpStream::connect(dest).await?;
            let (mut real_read, mut real_write) = tokio::io::split(stream);

            let mut reader = reader;
            let mut writer = writer;

            let client_to_server = tokio::io::copy(&mut reader, &mut real_write);
            let server_to_client = tokio::io::copy(&mut real_read, &mut writer);

            tokio::select! {
                r = client_to_server => { r?; }
                r = server_to_client => { r?; }
            }

            Ok(())
        })
    }
}

/// Registry of interceptors, checked in order. The first matching interceptor
/// handles the connection.
pub struct InterceptorRegistry {
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl InterceptorRegistry {
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Create a registry with only the passthrough interceptor.
    pub fn with_passthrough() -> Self {
        let mut reg = Self::new();
        reg.add(Box::new(PassthroughInterceptor));
        reg
    }

    /// Add an interceptor. Earlier interceptors have higher priority.
    pub fn add(&mut self, interceptor: Box<dyn Interceptor>) {
        self.interceptors.push(interceptor);
    }

    /// Find the first interceptor that matches the destination.
    pub fn find(&self, dest: SocketAddr) -> Option<&dyn Interceptor> {
        self.interceptors
            .iter()
            .find(|i| i.matches(dest))
            .map(|i| i.as_ref())
    }
}

impl Default for InterceptorRegistry {
    fn default() -> Self {
        Self::with_passthrough()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[test]
    fn test_passthrough_matches_all() {
        let p = PassthroughInterceptor;
        assert!(p.matches("1.2.3.4:80".parse().unwrap()));
        assert!(p.matches("10.0.0.1:443".parse().unwrap()));
        assert!(p.matches("[::1]:8080".parse().unwrap()));
    }

    #[test]
    fn test_registry_find_passthrough() {
        let reg = InterceptorRegistry::with_passthrough();
        let dest: SocketAddr = "93.184.216.34:80".parse().unwrap();
        assert!(reg.find(dest).is_some());
    }

    #[test]
    fn test_registry_empty_find_none() {
        let reg = InterceptorRegistry::new();
        let dest: SocketAddr = "93.184.216.34:80".parse().unwrap();
        assert!(reg.find(dest).is_none());
    }

    /// A test interceptor that blocks connections to port 9999.
    struct BlockingInterceptor;

    impl Interceptor for BlockingInterceptor {
        fn matches(&self, dest: SocketAddr) -> bool {
            dest.port() == 9999
        }

        fn handle(
            &self,
            _dest: SocketAddr,
            _reader: Pin<Box<dyn AsyncRead + Send>>,
            _writer: Pin<Box<dyn AsyncWrite + Send>>,
        ) -> Pin<Box<dyn Future<Output = InterceptResult> + Send>> {
            Box::pin(async { Err(anyhow::anyhow!("connection blocked")) })
        }
    }

    #[test]
    fn test_custom_interceptor_matches() {
        let mut reg = InterceptorRegistry::new();
        reg.add(Box::new(BlockingInterceptor));
        reg.add(Box::new(PassthroughInterceptor));

        // Port 9999 should match the blocking interceptor
        assert!(reg.find("1.2.3.4:9999".parse().unwrap()).is_some());
        // Port 80 should match the passthrough
        assert!(reg.find("1.2.3.4:80".parse().unwrap()).is_some());
    }

    #[tokio::test]
    async fn test_custom_interceptor_blocks() {
        let blocker = BlockingInterceptor;

        let (client, _server) = duplex(1024);
        let (reader, writer) = tokio::io::split(client);

        let result = blocker
            .handle(
                "1.2.3.4:9999".parse().unwrap(),
                Box::pin(reader),
                Box::pin(writer),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked"));
    }
}
