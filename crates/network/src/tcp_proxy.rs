//! TCP proxy bridging smoltcp TCP connections to real network sockets.
//!
//! When smoltcp accepts a new TCP connection, the proxy opens a real
//! `tokio::net::TcpStream` to the destination and relays data bidirectionally.

use std::collections::HashMap;
use std::net::SocketAddr;

use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp;
use smoltcp::wire::{IpAddress, IpEndpoint};

use crate::interface::VirtualNetwork;

/// Tracks active TCP proxy connections.
pub struct TcpProxy {
    /// Maps smoltcp socket handle → (destination, relay state).
    connections: HashMap<SocketHandle, ProxyConnection>,
    /// Handles that have been registered but not yet connected in smoltcp.
    listening: Vec<SocketHandle>,
}

struct ProxyConnection {
    #[allow(dead_code)]
    dest: SocketAddr,
    state: ConnectionState,
    vm_src_port: u16,
}

enum ConnectionState {
    /// Waiting for tokio TcpStream connect to complete.
    Connecting,
    /// Connected — relay data via channels.
    Connected {
        /// Data received from the real socket, to be sent to smoltcp.
        from_real: Vec<u8>,
        /// Data received from smoltcp, to be sent to the real socket.
        to_real: Vec<u8>,
    },
    /// Real socket closed; flushing remaining data to smoltcp before
    /// graceful close.
    Draining {
        /// Data still to be sent to smoltcp.
        from_real: Vec<u8>,
    },
}

/// Actions the network loop should perform based on proxy state changes.
pub enum ProxyAction {
    /// Open a real TCP connection to this address.
    Connect {
        handle: SocketHandle,
        dest: SocketAddr,
        vm_src_port: u16,
    },
    /// Send this data to the real socket.
    SendToReal { handle: SocketHandle, data: Vec<u8> },
    /// Close the real socket.
    CloseReal {
        handle: SocketHandle,
        vm_src_port: u16,
    },
}

impl Default for TcpProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpProxy {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            listening: Vec::new(),
        }
    }

    /// Register a new listening TCP socket in smoltcp.
    pub fn add_listener(&mut self, handle: SocketHandle) {
        self.listening.push(handle);
    }

    /// Check for new connections and data transfers. Returns actions for the
    /// network loop to execute.
    pub fn poll(&mut self, net: &mut VirtualNetwork) -> Vec<ProxyAction> {
        let mut actions = Vec::new();

        // Check listening sockets for new connections
        let mut new_connections = Vec::new();
        self.listening.retain(|&handle| {
            let sock = net.tcp_socket(handle);
            if sock.is_active()
                && let Some(local) = sock.local_endpoint()
            {
                let vm_src_port = sock.remote_endpoint().map(|ep| ep.port).unwrap_or(0);
                let dest = endpoint_to_addr(local);
                new_connections.push((handle, dest, vm_src_port));
                return false; // Remove from listening
            }
            true
        });

        for (handle, dest, vm_src_port) in new_connections {
            self.connections.insert(
                handle,
                ProxyConnection {
                    dest,
                    state: ConnectionState::Connecting,
                    vm_src_port,
                },
            );
            actions.push(ProxyAction::Connect {
                handle,
                dest,
                vm_src_port,
            });
        }

        // Process existing connections
        let mut to_remove = Vec::new();
        for (&handle, conn) in &mut self.connections {
            match &mut conn.state {
                ConnectionState::Connected {
                    from_real, to_real, ..
                } => {
                    let sock = net.tcp_socket(handle);

                    // Send data from real socket → smoltcp
                    if !from_real.is_empty() && sock.can_send() {
                        match sock.send_slice(from_real) {
                            Ok(sent) => {
                                from_real.drain(..sent);
                            }
                            Err(_) => {
                                to_remove.push(handle);
                                continue;
                            }
                        }
                    }

                    // Receive data from smoltcp → real socket
                    if sock.can_recv() {
                        let mut buf = vec![0u8; 65536];
                        match sock.recv_slice(&mut buf) {
                            Ok(n) if n > 0 => {
                                buf.truncate(n);
                                to_real.extend_from_slice(&buf);
                            }
                            Ok(_) => {}
                            Err(_) => {
                                to_remove.push(handle);
                                continue;
                            }
                        }
                    }

                    // Flush data to real socket
                    if !to_real.is_empty() {
                        let data = std::mem::take(to_real);
                        actions.push(ProxyAction::SendToReal { handle, data });
                    }

                    // Check if smoltcp side closed
                    if sock.state() == tcp::State::CloseWait
                        || sock.state() == tcp::State::Closed
                        || sock.state() == tcp::State::TimeWait
                    {
                        if sock.state() == tcp::State::CloseWait {
                            sock.close();
                        }
                        to_remove.push(handle);
                        actions.push(ProxyAction::CloseReal {
                            handle,
                            vm_src_port: conn.vm_src_port,
                        });
                    }
                }
                ConnectionState::Draining { from_real } => {
                    let sock = net.tcp_socket(handle);

                    // Flush remaining data from real socket → smoltcp
                    if !from_real.is_empty() && sock.can_send() {
                        match sock.send_slice(from_real) {
                            Ok(sent) => {
                                from_real.drain(..sent);
                            }
                            Err(_) => {
                                to_remove.push(handle);
                                actions.push(ProxyAction::CloseReal {
                                    handle,
                                    vm_src_port: conn.vm_src_port,
                                });
                                continue;
                            }
                        }
                    }

                    // All data flushed — initiate graceful close
                    if from_real.is_empty() {
                        sock.close();
                        to_remove.push(handle);
                        actions.push(ProxyAction::CloseReal {
                            handle,
                            vm_src_port: conn.vm_src_port,
                        });
                        continue;
                    }

                    // VM side closed before drain completed
                    if !sock.is_open() {
                        to_remove.push(handle);
                        actions.push(ProxyAction::CloseReal {
                            handle,
                            vm_src_port: conn.vm_src_port,
                        });
                    }
                }
                ConnectionState::Connecting => {
                    // Check if smoltcp side closed before real connect completed
                    let sock = net.tcp_socket(handle);
                    if !sock.is_open() {
                        to_remove.push(handle);
                    }
                }
            }
        }

        for handle in to_remove {
            self.connections.remove(&handle);
        }

        actions
    }

    /// Mark a connection as established (real socket connected).
    pub fn mark_connected(&mut self, handle: SocketHandle) {
        if let Some(conn) = self.connections.get_mut(&handle) {
            conn.state = ConnectionState::Connected {
                from_real: Vec::new(),
                to_real: Vec::new(),
            };
        }
    }

    /// Push data received from the real socket into the proxy buffer.
    pub fn push_from_real(&mut self, handle: SocketHandle, data: &[u8]) {
        if let Some(conn) = self.connections.get_mut(&handle) {
            match &mut conn.state {
                ConnectionState::Connected { from_real, .. }
                | ConnectionState::Draining { from_real } => {
                    from_real.extend_from_slice(data);
                }
                _ => {}
            }
        }
    }

    /// Mark a connection as closed (real socket disconnected or errored).
    ///
    /// Instead of aborting immediately, transitions to `Draining` so any
    /// buffered data is flushed to smoltcp before a graceful close.
    pub fn mark_closed(&mut self, handle: SocketHandle, net: &mut VirtualNetwork) {
        if let Some(conn) = self.connections.get_mut(&handle)
            && let ConnectionState::Connected { from_real, .. } = &mut conn.state
        {
            let remaining = std::mem::take(from_real);
            conn.state = ConnectionState::Draining {
                from_real: remaining,
            };
            return;
        }
        // Not in Connected state or not found — abort
        self.connections.remove(&handle);
        let sock = net.tcp_socket(handle);
        sock.abort();
    }

    /// Mark a connection as failed to connect.
    pub fn mark_connect_failed(&mut self, handle: SocketHandle, net: &mut VirtualNetwork) {
        self.connections.remove(&handle);
        let sock = net.tcp_socket(handle);
        sock.abort();
    }

    /// Number of active connections.
    pub fn active_count(&self) -> usize {
        self.connections.len()
    }
}

fn endpoint_to_addr(ep: IpEndpoint) -> SocketAddr {
    let IpAddress::Ipv4(v4) = ep.addr;
    SocketAddr::new(v4.into(), ep.port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::VirtualNetwork;

    #[test]
    fn test_tcp_proxy_new() {
        let proxy = TcpProxy::new();
        assert_eq!(proxy.active_count(), 0);
    }

    #[test]
    fn test_tcp_proxy_add_listener() {
        let mut net = VirtualNetwork::new();
        let mut proxy = TcpProxy::new();

        let handle = net.add_tcp_socket(65535, 65535);
        let sock = net.tcp_socket(handle);
        sock.listen(8080).unwrap();

        proxy.add_listener(handle);
        assert_eq!(proxy.listening.len(), 1);
    }

    #[test]
    fn test_endpoint_to_addr() {
        let ep = IpEndpoint::new(IpAddress::v4(10, 0, 0, 1), 80);
        let addr = endpoint_to_addr(ep);
        assert_eq!(addr, "10.0.0.1:80".parse::<SocketAddr>().unwrap());
    }
}
