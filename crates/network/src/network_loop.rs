//! Main network event loop tying codec, interface, DNS, and TCP proxy together.
//!
//! [`run_network`] is the top-level async function that owns the full pipeline:
//! reads QEMU frames from the wire, drives the smoltcp poll loop, processes
//! DNS and TCP, and writes outbound frames.

use std::collections::HashMap;
use std::time::Duration;

use smoltcp::iface::SocketHandle;
use smoltcp::time::Instant;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch};

use log::{debug, trace, warn};

use crate::dns;
use crate::interface::VirtualNetwork;
use crate::tcp_proxy::{ProxyAction, TcpProxy};

/// Channel message from real socket relay tasks back to the network loop.
enum RealSocketMsg {
    /// Real TCP connection established.
    Connected(SocketHandle),
    /// Data received from the real socket.
    Data(SocketHandle, Vec<u8>),
    /// Real socket closed or errored.
    Closed(SocketHandle),
    /// Connection to real host failed.
    ConnectFailed(SocketHandle),
}

/// Run the network stack.
///
/// Reads/writes QEMU-protocol frames from `stream`, drives smoltcp, handles
/// DHCP/DNS/TCP proxy. Runs until `shutdown` signal fires or the stream closes.
pub async fn run_network<S>(stream: S, mut shutdown: watch::Receiver<bool>)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (reader, writer) = tokio::io::split(stream);
    let (mut frame_reader, mut frame_writer) = crate::codec::split(reader, writer);

    let mut net = VirtualNetwork::new();
    let mut tcp_proxy = TcpProxy::new();

    // DNS socket on port 53
    let dns_handle = net
        .add_udp_socket(53, 4096, 4096)
        .expect("failed to bind DNS socket");

    // Channel for real socket events
    let (real_tx, mut real_rx) = mpsc::channel::<RealSocketMsg>(256);

    // Track real socket senders for writing data to them
    let mut real_writers: HashMap<SocketHandle, mpsc::Sender<Vec<u8>>> = HashMap::new();

    let resolver = dns::system_resolver();
    let mut poll_interval = tokio::time::interval(Duration::from_millis(1));
    poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    debug!("[net] Network loop starting, waiting for frames...");
    let mut frame_count = 0u64;
    let mut last_status = std::time::Instant::now();

    loop {
        // Periodic status update
        if last_status.elapsed() > Duration::from_secs(5) {
            debug!("[net] Status: {} frames received so far", frame_count);
            last_status = std::time::Instant::now();
        }

        tokio::select! {
            // Read a frame from the QEMU wire
            frame_result = frame_reader.read() => {
                match frame_result {
                    Ok(Some(frame)) => {
                        frame_count += 1;
                        if frame.len() >= 14 {
                            let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
                            trace!("[net] Received frame #{}: {} bytes, ethertype=0x{ethertype:04x}", frame_count, frame.len());
                        } else {
                            trace!("[net] Received short frame #{}: {} bytes", frame_count, frame.len());
                        }
                        net.inject_frame(frame);
                    }
                    Ok(None) => {
                        debug!("[net] EOF on frame reader");
                        break;
                    }
                    Err(e) => {
                        warn!("[net] Error reading frame: {e}");
                        break;
                    }
                }
            }

            // Receive messages from real socket relay tasks
            Some(msg) = real_rx.recv() => {
                match msg {
                    RealSocketMsg::Connected(handle) => {
                        tcp_proxy.mark_connected(handle);
                    }
                    RealSocketMsg::Data(handle, data) => {
                        tcp_proxy.push_from_real(handle, &data);
                    }
                    RealSocketMsg::Closed(handle) => {
                        tcp_proxy.mark_closed(handle, &mut net);
                        real_writers.remove(&handle);
                    }
                    RealSocketMsg::ConnectFailed(handle) => {
                        tcp_proxy.mark_connect_failed(handle, &mut net);
                        real_writers.remove(&handle);
                    }
                }
            }

            // Poll tick
            _ = poll_interval.tick() => {}

            // Shutdown
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
            }
        }

        // Drive smoltcp
        let now = Instant::from_millis(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        );
        let outbound = net.poll(now);

        // Send outbound frames to the wire
        for frame in outbound {
            if frame_writer.write(&frame).await.is_err() {
                break;
            }
        }

        // Handle DNS queries
        handle_dns(&mut net, dns_handle, resolver).await;

        // Poll again after DNS responses
        let outbound = net.poll(now);
        for frame in outbound {
            if frame_writer.write(&frame).await.is_err() {
                break;
            }
        }

        // Check for new TCP connections (SYN received by smoltcp)
        // We need to add listening sockets for any new connections smoltcp accepted
        check_for_new_tcp_connections(&mut net, &mut tcp_proxy);

        // Process TCP proxy
        let actions = tcp_proxy.poll(&mut net);
        for action in actions {
            match action {
                ProxyAction::Connect { handle, dest } => {
                    let tx = real_tx.clone();
                    let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>(64);
                    real_writers.insert(handle, write_tx);

                    tokio::spawn(async move {
                        relay_real_socket(handle, dest, tx, write_rx).await;
                    });
                }
                ProxyAction::SendToReal { handle, data } => {
                    if let Some(writer) = real_writers.get(&handle) {
                        let _ = writer.try_send(data);
                    }
                }
                ProxyAction::CloseReal { handle } => {
                    real_writers.remove(&handle);
                }
            }
        }

        // Final poll to flush any TCP data
        let outbound = net.poll(now);
        for frame in outbound {
            if frame_writer.write(&frame).await.is_err() {
                break;
            }
        }
    }
}

/// Handle DNS queries received on the DNS UDP socket.
async fn handle_dns(
    net: &mut VirtualNetwork,
    dns_handle: SocketHandle,
    resolver: std::net::SocketAddr,
) {
    let sock = net.udp_socket(dns_handle);
    let mut queries = Vec::new();
    while sock.can_recv() {
        match sock.recv() {
            Ok((data, meta)) => {
                debug!(
                    "[dns] Received query from {:?}, {} bytes",
                    meta.endpoint,
                    data.len()
                );
                queries.push((data.to_vec(), meta.endpoint));
            }
            Err(_) => break,
        }
    }

    for (query, source) in queries {
        debug!("[dns] Forwarding query to system resolver {resolver}");
        match dns::forward_query(&query, resolver, Duration::from_secs(5)).await {
            Ok(response) => {
                debug!(
                    "[dns] Got response, {} bytes, sending to {source:?}",
                    response.len()
                );
                let sock = net.udp_socket(dns_handle);
                let _ = sock.send_slice(&response, source);
            }
            Err(e) => {
                warn!("[dns] Forward failed: {e}");
            }
        }
    }
}

/// Check smoltcp sockets for new incoming TCP connections.
///
/// When the container initiates a TCP connection to any address, smoltcp
/// sees it as a SYN to our gateway. We accept it and register it with the
/// TCP proxy for relay to the real destination.
fn check_for_new_tcp_connections(net: &mut VirtualNetwork, proxy: &mut TcpProxy) {
    // For each incoming TCP connection, we need a fresh smoltcp socket
    // that has already received the SYN. Since smoltcp doesn't have a
    // "listener" concept like a real OS, we pre-allocate sockets that
    // listen on port 0 (wildcard) and let smoltcp match incoming SYNs.
    //
    // We maintain a pool of listening sockets. When one becomes active
    // (connection established), we register it with the proxy and add
    // a new listening socket to replace it.

    // This is handled by the proxy's poll() method checking listening sockets.
    // We just need to ensure there are always some listening sockets available.

    // Count how many listening sockets the proxy currently has
    // (handled internally by TcpProxy)
    let _ = (net, proxy);
}

/// Relay task for a single real TCP connection.
async fn relay_real_socket(
    handle: SocketHandle,
    dest: std::net::SocketAddr,
    event_tx: mpsc::Sender<RealSocketMsg>,
    mut data_rx: mpsc::Receiver<Vec<u8>>,
) {
    // Connect to the real host
    let stream = match TcpStream::connect(dest).await {
        Ok(s) => s,
        Err(_) => {
            let _ = event_tx.send(RealSocketMsg::ConnectFailed(handle)).await;
            return;
        }
    };

    let _ = event_tx.send(RealSocketMsg::Connected(handle)).await;

    let (mut read_half, mut write_half) = tokio::io::split(stream);

    // Spawn writer task
    let write_event_tx = event_tx.clone();
    let write_handle = handle;
    let write_task = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        while let Some(data) = data_rx.recv().await {
            if write_half.write_all(&data).await.is_err() {
                let _ = write_event_tx
                    .send(RealSocketMsg::Closed(write_handle))
                    .await;
                return;
            }
        }
    });

    // Read from real socket
    use tokio::io::AsyncReadExt;
    let mut buf = vec![0u8; 65536];
    loop {
        match read_half.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                if event_tx
                    .send(RealSocketMsg::Data(handle, buf[..n].to_vec()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let _ = event_tx.send(RealSocketMsg::Closed(handle)).await;
    write_task.abort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_network_loop_shutdown() {
        let (client, server) = duplex(65536);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            run_network(server, shutdown_rx).await;
        });

        // Signal shutdown
        shutdown_tx.send(true).unwrap();

        // Should complete without hanging
        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("network loop should shut down")
            .expect("task should not panic");

        drop(client);
    }

    #[tokio::test]
    async fn test_network_loop_eof() {
        let (client, server) = duplex(65536);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            run_network(server, shutdown_rx).await;
        });

        // Drop client to signal EOF
        drop(client);

        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("network loop should exit on EOF")
            .expect("task should not panic");
    }

    #[tokio::test]
    async fn test_full_stack_with_duplex_dhcp() {
        // Test DHCP through the full network stack
        let (mut client, server) = duplex(65536);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            run_network(server, shutdown_rx).await;
        });

        // Build and send a DHCP DISCOVER frame
        let discover_payload = build_dhcp_discover();
        let frame = build_udp_frame(
            &crate::net_config::VM_MAC.0,
            &[0xFF; 6],
            [0, 0, 0, 0],
            [255, 255, 255, 255],
            68,
            67,
            &discover_payload,
        );

        // Write via codec
        codec::write_frame(&mut client, &frame).await.unwrap();

        // Read response — should be DHCP OFFER
        let response = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Ok(Some(frame)) = codec::read_frame(&mut client).await {
                    // Check if this is a UDP frame to port 68
                    if frame.len() > 42 {
                        let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
                        if ethertype == 0x0800 {
                            // IPv4
                            let ihl = (frame[14] & 0x0F) as usize * 4;
                            let proto = frame[14 + 9];
                            if proto == 17 {
                                // UDP
                                let udp_start = 14 + ihl;
                                if frame.len() > udp_start + 4 {
                                    let dst_port = u16::from_be_bytes([
                                        frame[udp_start + 2],
                                        frame[udp_start + 3],
                                    ]);
                                    if dst_port == 68 {
                                        return frame;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        })
        .await;

        assert!(response.is_ok(), "Should receive DHCP response");

        drop(client);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    fn build_dhcp_discover() -> Vec<u8> {
        let magic: [u8; 4] = [99, 130, 83, 99];
        let mut msg = vec![0u8; 240 + 3];
        msg[0] = 1;
        msg[1] = 1;
        msg[2] = 6;
        msg[4..8].copy_from_slice(&0xABCD1234u32.to_be_bytes());
        msg[28..34].copy_from_slice(&crate::net_config::VM_MAC.0);
        msg[236..240].copy_from_slice(&magic);
        msg[240] = 53;
        msg[241] = 1;
        msg[242] = 1;
        msg
    }

    fn build_udp_frame(
        src_mac: &[u8; 6],
        dst_mac: &[u8; 6],
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let total = 14 + 20 + 8 + payload.len();
        let mut buf = vec![0u8; total];

        buf[0..6].copy_from_slice(dst_mac);
        buf[6..12].copy_from_slice(src_mac);
        buf[12..14].copy_from_slice(&0x0800u16.to_be_bytes());

        let ip_start = 14;
        let ip_total_len = (20 + 8 + payload.len()) as u16;
        buf[ip_start] = 0x45;
        buf[ip_start + 2..ip_start + 4].copy_from_slice(&ip_total_len.to_be_bytes());
        buf[ip_start + 8] = 64;
        buf[ip_start + 9] = 17;
        buf[ip_start + 12..ip_start + 16].copy_from_slice(&src_ip);
        buf[ip_start + 16..ip_start + 20].copy_from_slice(&dst_ip);
        let checksum = ip_checksum(&buf[ip_start..ip_start + 20]);
        buf[ip_start + 10..ip_start + 12].copy_from_slice(&checksum.to_be_bytes());

        let udp_start = 34;
        let udp_len = (8 + payload.len()) as u16;
        buf[udp_start..udp_start + 2].copy_from_slice(&src_port.to_be_bytes());
        buf[udp_start + 2..udp_start + 4].copy_from_slice(&dst_port.to_be_bytes());
        buf[udp_start + 4..udp_start + 6].copy_from_slice(&udp_len.to_be_bytes());

        buf[udp_start + 8..].copy_from_slice(payload);
        buf
    }

    fn ip_checksum(header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        for i in (0..header.len()).step_by(2) {
            let word = if i + 1 < header.len() {
                (header[i] as u32) << 8 | header[i + 1] as u32
            } else {
                (header[i] as u32) << 8
            };
            sum += word;
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        !(sum as u16)
    }
}
