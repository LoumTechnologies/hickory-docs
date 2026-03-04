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

pub use auth_injector::{AuthInjector, placeholder_env_vars, PLACEHOLDER};
pub use auth_rules::{AuthConfig, AuthRule};
pub use codec::{read_frame, write_frame, FrameIo, FrameReader, FrameWriter, CombinedStream, split};
pub use device::VirtualDevice;
pub use dhcp::DhcpServer;
pub use dns::{forward_query, system_resolver, parse_query_domain, build_a_response, parse_a_response};
pub use http::{HttpRequest, HttpResponse, HttpFilter, HttpFilterAction, PassthroughHttpFilter};
pub use interceptor::{Interceptor, PassthroughInterceptor, InterceptorRegistry};
pub use interface::VirtualNetwork;
pub use network_loop::{run_network, run_network_with_config, run_network_bridged, TlsInterceptConfig};
pub use tcp_proxy::{TcpProxy, ProxyAction};
pub use tls_ca::{HickCa, CertifiedKey};
pub use tls_sni::extract_sni;
