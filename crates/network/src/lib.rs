//! Virtual network stack for hick containers.
//!
//! Provides a software-defined network with DHCP, DNS, TCP proxying,
//! HTTP interception, TLS MITM, and auth injection.

pub mod auth_injector;
pub mod auth_rules;
pub mod bridge;
pub mod codec;
pub mod connection_filter;
pub mod device;
pub mod dhcp;
pub mod dns;
pub mod http;
pub mod interceptor;
pub mod interface;
pub mod net_config;
pub mod network_loop;
pub mod tcp_nat;
pub mod tcp_proxy;
pub mod tls_ca;
pub mod tls_relay;
pub mod tls_sni;

// ---- pub use facade ----

/// HTTP filter that injects secrets into auth headers based on host-matching rules.
pub use auth_injector::AuthInjector;
/// Generate de-duplicated placeholder environment variable pairs from auth config.
pub use auth_injector::placeholder_env_vars;
/// Sentinel value injected into container env vars for managed secrets.
pub use auth_injector::PLACEHOLDER;

/// Top-level auth injection configuration mapping hosts to secrets and headers.
pub use auth_rules::AuthConfig;
/// A single auth injection rule binding a hostname to a secret and header format.
pub use auth_rules::AuthRule;

/// Read a single length-prefixed Ethernet frame from an async reader.
pub use codec::read_frame;
/// Write a single length-prefixed Ethernet frame to an async writer.
pub use codec::write_frame;
/// Bidirectional frame I/O wrapper over an async read/write stream.
pub use codec::FrameIo;
/// Read-half of a split frame I/O stream.
pub use codec::FrameReader;
/// Write-half of a split frame I/O stream.
pub use codec::FrameWriter;
/// A combined stream pairing separate async reader and writer halves.
pub use codec::CombinedStream;
/// Split a reader/writer pair into independent [`FrameReader`] and [`FrameWriter`] halves.
pub use codec::split;

/// Virtual Ethernet device backed by in-memory frame queues, implementing smoltcp's `Device` trait.
pub use device::VirtualDevice;

/// Minimal DHCP server that assigns a single IP to the container guest.
pub use dhcp::DhcpServer;

/// Forward a raw DNS query to a resolver and return the response.
pub use dns::forward_query;
/// Detect the system DNS resolver address from `/etc/resolv.conf`.
pub use dns::system_resolver;
/// Extract the queried domain name from a raw DNS query packet.
pub use dns::parse_query_domain;
/// Build a minimal DNS A-record response for a given query and IPv4 address.
pub use dns::build_a_response;
/// Extract the queried domain and resolved IPv4 address from a DNS A-record response.
pub use dns::parse_a_response;

/// An HTTP request parsed from the wire.
pub use http::HttpRequest;
/// An HTTP response parsed from the wire.
pub use http::HttpResponse;
/// Trait for inspecting and modifying HTTP request/response traffic.
pub use http::HttpFilter;
/// Action returned by [`HttpFilter`] methods: forward or block.
pub use http::HttpFilterAction;
/// Default HTTP filter that forwards everything unmodified.
pub use http::PassthroughHttpFilter;

/// Trait for extensible TCP connection handling based on destination address.
pub use interceptor::Interceptor;
/// Default interceptor that forwards all TCP traffic without modification.
pub use interceptor::PassthroughInterceptor;
/// Ordered registry of interceptors; the first match handles the connection.
pub use interceptor::InterceptorRegistry;

/// Owns the smoltcp interface, virtual device, DHCP server, and socket set.
pub use interface::VirtualNetwork;

/// Run the network stack with default config, handling DHCP/DNS/TCP proxy.
pub use network_loop::run_network;
/// Run the network stack with a specific [`net_config::NetworkConfig`].
pub use network_loop::run_network_with_config;
/// Run the network stack with bridge support for inter-container networking.
pub use network_loop::run_network_bridged;
/// Configuration for TLS interception (MITM) of HTTPS connections.
pub use network_loop::TlsInterceptConfig;

/// Tracks active TCP proxy connections bridging smoltcp to real network sockets.
pub use tcp_proxy::TcpProxy;
/// Actions the network loop should perform based on TCP proxy state changes.
pub use tcp_proxy::ProxyAction;

/// Ephemeral certificate authority for TLS interception, issuing per-host certs on demand.
pub use tls_ca::HickCa;
/// A certified key pair (certificate + private key) ready for rustls.
pub use tls_ca::CertifiedKey;

/// Extract the SNI hostname from a raw TLS ClientHello message.
pub use tls_sni::extract_sni;
