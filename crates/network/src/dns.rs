//! DNS forwarder that relays raw DNS queries to the system resolver.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::UdpSocket;

/// Forward a raw DNS query to a resolver and return the response.
///
/// Returns `Err` on timeout or network failure.
pub async fn forward_query(
    query: &[u8],
    resolver: SocketAddr,
    timeout: Duration,
) -> anyhow::Result<Vec<u8>> {
    let bind_addr: SocketAddr = if resolver.is_ipv4() {
        "0.0.0.0:0".parse().unwrap()
    } else {
        "[::]:0".parse().unwrap()
    };

    let sock = UdpSocket::bind(bind_addr).await?;
    sock.send_to(query, resolver).await?;

    let mut buf = vec![0u8; 4096];
    let len = tokio::time::timeout(timeout, sock.recv(&mut buf)).await??;
    buf.truncate(len);
    Ok(buf)
}

/// Detect the system DNS resolver address.
///
/// On macOS/Linux, reads `/etc/resolv.conf` and returns the first
/// `nameserver` entry. Falls back to `8.8.8.8:53`.
pub fn system_resolver() -> SocketAddr {
    if let Ok(contents) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in contents.lines() {
            let line = line.trim();
            if let Some(addr_str) = line.strip_prefix("nameserver") {
                let addr_str = addr_str.trim();
                // Try parsing as IP address, add port 53
                if let Ok(ip) = addr_str.parse::<std::net::IpAddr>() {
                    return SocketAddr::new(ip, 53);
                }
            }
        }
    }
    // Fallback
    SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(8, 8, 8, 8)),
        53,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_forward_query_to_mock() {
        // Start a local UDP "echo" server as a mock DNS resolver
        let mock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mock_addr = mock.local_addr().unwrap();

        let query = b"fake dns query payload";

        let mock_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            let (len, src) = mock.recv_from(&mut buf).await.unwrap();
            // Echo it back
            mock.send_to(&buf[..len], src).await.unwrap();
        });

        let response = forward_query(query, mock_addr, Duration::from_secs(5))
            .await
            .unwrap();

        assert_eq!(response, query);
        mock_task.await.unwrap();
    }

    #[test]
    fn test_system_resolver_parsing() {
        let resolver = system_resolver();
        // Should return a valid address with port 53
        assert_eq!(resolver.port(), 53);
        assert!(!resolver.ip().is_unspecified());
    }

    #[tokio::test]
    async fn test_timeout_on_no_response() {
        // Bind a socket but never respond
        let mock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mock_addr = mock.local_addr().unwrap();

        let result = forward_query(b"query", mock_addr, Duration::from_millis(100)).await;

        assert!(result.is_err(), "Expected timeout error");
        drop(mock);
    }
}
