//! Connection filtering trait for selectively allowing or blocking network
//! requests made by the WASM container.

use std::fmt;
use std::net::SocketAddr;

use async_trait::async_trait;

/// A network request that can be filtered.
pub enum NetworkRequest {
    /// Outbound TCP connection attempt.
    Tcp { dest: SocketAddr },
    /// DNS query for a domain name.
    Dns { domain: String },
}

impl fmt::Display for NetworkRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkRequest::Tcp { dest } => write!(f, "TCP connection to {dest}"),
            NetworkRequest::Dns { domain } => write!(f, "DNS query for {domain}"),
        }
    }
}

/// The verdict for a filtered network request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionVerdict {
    Allow,
    Block,
}

/// Trait for filtering outbound network requests.
///
/// Implementations receive each TCP connect and DNS query before it is
/// processed, and return a verdict that determines whether the request
/// proceeds or is silently dropped.
#[async_trait]
pub trait ConnectionFilter: Send + Sync {
    async fn filter(&self, request: &NetworkRequest) -> ConnectionVerdict;
}

/// Default filter that allows all connections.
pub struct AllowAll;

#[async_trait]
impl ConnectionFilter for AllowAll {
    async fn filter(&self, _request: &NetworkRequest) -> ConnectionVerdict {
        ConnectionVerdict::Allow
    }
}
