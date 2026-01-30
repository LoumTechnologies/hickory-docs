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

// ---------------------------------------------------------------------------
// CapabilityFilter (requires "capability" feature)
// ---------------------------------------------------------------------------

/// Filter that validates network requests against macaroon-based capabilities.
///
/// For DNS queries, checks if any network rule allows the domain.
/// For TCP connections, checks the destination IP/port against allowed hosts.
/// Since TCP connections use IP addresses, a DNS-to-IP mapping must be
/// maintained by the caller (or the filter can be configured with known hosts).
#[cfg(feature = "capability")]
pub struct CapabilityFilter {
    capabilities: hick_token::ContainerCapabilities,
    /// Optional mapping from IP addresses to hostnames for TCP filtering.
    /// Populated as DNS queries are resolved.
    dns_cache: std::sync::Mutex<std::collections::HashMap<std::net::IpAddr, String>>,
}

#[cfg(feature = "capability")]
impl CapabilityFilter {
    /// Create a new capability filter from container capabilities.
    pub fn new(capabilities: hick_token::ContainerCapabilities) -> Self {
        Self {
            capabilities,
            dns_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Register a DNS resolution result so TCP connections to this IP
    /// can be checked against hostname-based rules.
    pub fn register_dns_result(&self, domain: &str, addr: std::net::IpAddr) {
        self.dns_cache
            .lock()
            .unwrap()
            .insert(addr, domain.to_string());
    }

    /// Check if a domain is allowed by any network rule.
    fn check_domain(&self, domain: &str) -> bool {
        // If no deny-all rule, allow everything
        let has_deny_all = self
            .capabilities
            .network_rules
            .iter()
            .any(|r| matches!(r, hick_token::NetworkRule::DenyAll));

        if !has_deny_all {
            return true;
        }

        // Check if any allow rule matches this domain (any port)
        self.capabilities.network_rules.iter().any(|r| match r {
            hick_token::NetworkRule::Allow { host, .. } => host == domain || host == "*",
            hick_token::NetworkRule::DenyAll => false,
        })
    }
}

#[cfg(feature = "capability")]
#[async_trait]
impl ConnectionFilter for CapabilityFilter {
    async fn filter(&self, request: &NetworkRequest) -> ConnectionVerdict {
        match request {
            NetworkRequest::Dns { domain } => {
                if self.check_domain(domain) {
                    log::debug!("DNS allowed: {domain}");
                    ConnectionVerdict::Allow
                } else {
                    log::warn!("DNS blocked by capability filter: {domain}");
                    ConnectionVerdict::Block
                }
            }
            NetworkRequest::Tcp { dest } => {
                // Look up the hostname from our DNS cache
                let hostname = self.dns_cache.lock().unwrap().get(&dest.ip()).cloned();

                let allowed = if let Some(ref host) = hostname {
                    self.capabilities.check_network(host, dest.port())
                } else {
                    // No cached hostname — check by IP string
                    self.capabilities
                        .check_network(&dest.ip().to_string(), dest.port())
                };

                if allowed {
                    log::debug!("TCP allowed: {dest}");
                    ConnectionVerdict::Allow
                } else {
                    log::warn!(
                        "TCP blocked by capability filter: {dest} (host: {})",
                        hostname.as_deref().unwrap_or("unknown")
                    );
                    ConnectionVerdict::Block
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allow_all_permits_everything() {
        let filter = AllowAll;
        let tcp = NetworkRequest::Tcp {
            dest: "1.2.3.4:80".parse().unwrap(),
        };
        let dns = NetworkRequest::Dns {
            domain: "example.com".to_string(),
        };
        assert_eq!(filter.filter(&tcp).await, ConnectionVerdict::Allow);
        assert_eq!(filter.filter(&dns).await, ConnectionVerdict::Allow);
    }
}

#[cfg(all(test, feature = "capability"))]
mod capability_tests {
    use super::*;
    use hick_token::ContainerCapabilities;

    #[tokio::test]
    async fn capability_filter_allows_whitelisted_dns() {
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let filter = CapabilityFilter::new(caps);

        let dns = NetworkRequest::Dns {
            domain: "github.com".to_string(),
        };
        assert_eq!(filter.filter(&dns).await, ConnectionVerdict::Allow);
    }

    #[tokio::test]
    async fn capability_filter_blocks_non_whitelisted_dns() {
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let filter = CapabilityFilter::new(caps);

        let dns = NetworkRequest::Dns {
            domain: "evil.com".to_string(),
        };
        assert_eq!(filter.filter(&dns).await, ConnectionVerdict::Block);
    }

    #[tokio::test]
    async fn capability_filter_tcp_with_dns_cache() {
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let filter = CapabilityFilter::new(caps);

        // Register a DNS resolution
        let ip: std::net::IpAddr = "140.82.121.4".parse().unwrap();
        filter.register_dns_result("github.com", ip);

        // TCP to the resolved IP on allowed port
        let tcp_allowed = NetworkRequest::Tcp {
            dest: "140.82.121.4:443".parse().unwrap(),
        };
        assert_eq!(filter.filter(&tcp_allowed).await, ConnectionVerdict::Allow);

        // TCP to the resolved IP on disallowed port
        let tcp_blocked = NetworkRequest::Tcp {
            dest: "140.82.121.4:80".parse().unwrap(),
        };
        assert_eq!(filter.filter(&tcp_blocked).await, ConnectionVerdict::Block);
    }

    #[tokio::test]
    async fn capability_filter_no_deny_allows_all() {
        let caps = ContainerCapabilities::new().allow_network("github.com", "443");

        let filter = CapabilityFilter::new(caps);

        let dns = NetworkRequest::Dns {
            domain: "anything.com".to_string(),
        };
        assert_eq!(filter.filter(&dns).await, ConnectionVerdict::Allow);

        let tcp = NetworkRequest::Tcp {
            dest: "1.2.3.4:9999".parse().unwrap(),
        };
        assert_eq!(filter.filter(&tcp).await, ConnectionVerdict::Allow);
    }
}
