//! Virtual network interface combining smoltcp [`Interface`], [`VirtualDevice`],
//! [`DhcpServer`], and [`SocketSet`] into a single poll-driven abstraction.

use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::socket::{tcp, udp};
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, IpCidr, IpEndpoint};

use crate::device::VirtualDevice;
use crate::dhcp::DhcpServer;
use crate::net_config::{GATEWAY_CIDR, GATEWAY_MAC};

/// Owns the smoltcp interface, device, DHCP server, and socket set.
pub struct VirtualNetwork {
    iface: Interface,
    device: VirtualDevice,
    sockets: SocketSet<'static>,
    dhcp_handle: SocketHandle,
    dhcp_server: DhcpServer,
}

impl Default for VirtualNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualNetwork {
    pub fn new() -> Self {
        let mut device = VirtualDevice::new();

        let config = Config::new(GATEWAY_MAC.into());
        let mut iface = Interface::new(config, &mut device, Instant::ZERO);

        // Set gateway IP
        iface.update_ip_addrs(|addrs| {
            addrs
                .push(IpCidr::Ipv4(GATEWAY_CIDR))
                .expect("failed to add gateway IP");
        });

        // No default route needed — we ARE the gateway

        let mut sockets = SocketSet::new(vec![]);

        // Create UDP socket for DHCP server (port 67)
        let dhcp_rx_meta = vec![udp::PacketMetadata::EMPTY; 4];
        let dhcp_rx_buf = vec![0u8; 1500];
        let dhcp_tx_meta = vec![udp::PacketMetadata::EMPTY; 4];
        let dhcp_tx_buf = vec![0u8; 1500];
        let dhcp_socket = udp::Socket::new(
            udp::PacketBuffer::new(dhcp_rx_meta, dhcp_rx_buf),
            udp::PacketBuffer::new(dhcp_tx_meta, dhcp_tx_buf),
        );
        let dhcp_handle = sockets.add(dhcp_socket);

        // Bind DHCP socket to port 67
        let sock = sockets.get_mut::<udp::Socket>(dhcp_handle);
        sock.bind(67).expect("failed to bind DHCP socket");

        Self {
            iface,
            device,
            sockets,
            dhcp_handle,
            dhcp_server: DhcpServer::new(),
        }
    }

    /// Push a raw Ethernet frame from the wire side into the virtual device.
    pub fn inject_frame(&mut self, frame: Vec<u8>) {
        self.device.inject_frame(frame);
    }

    /// Run a single poll cycle: process ingress, handle DHCP, produce egress.
    /// Returns outbound Ethernet frames to send back to the wire.
    pub fn poll(&mut self, now: Instant) -> Vec<Vec<u8>> {
        // Run smoltcp poll to process ingress and generate responses
        self.iface.poll(now, &mut self.device, &mut self.sockets);

        // Process DHCP requests
        self.handle_dhcp();

        // Poll again to transmit any DHCP responses
        self.iface.poll(now, &mut self.device, &mut self.sockets);

        // Collect all outbound frames
        self.device.drain_all_tx()
    }

    /// Add a TCP socket and return its handle.
    pub fn add_tcp_socket(&mut self, rx_size: usize, tx_size: usize) -> SocketHandle {
        let rx_buf = tcp::SocketBuffer::new(vec![0u8; rx_size]);
        let tx_buf = tcp::SocketBuffer::new(vec![0u8; tx_size]);
        let socket = tcp::Socket::new(rx_buf, tx_buf);
        self.sockets.add(socket)
    }

    /// Add a UDP socket bound to a specific port and return its handle.
    pub fn add_udp_socket(
        &mut self,
        port: u16,
        rx_size: usize,
        tx_size: usize,
    ) -> anyhow::Result<SocketHandle> {
        let rx_meta = vec![udp::PacketMetadata::EMPTY; 16];
        let rx_buf = vec![0u8; rx_size];
        let tx_meta = vec![udp::PacketMetadata::EMPTY; 16];
        let tx_buf = vec![0u8; tx_size];
        let socket = udp::Socket::new(
            udp::PacketBuffer::new(rx_meta, rx_buf),
            udp::PacketBuffer::new(tx_meta, tx_buf),
        );
        let handle = self.sockets.add(socket);
        let sock = self.sockets.get_mut::<udp::Socket>(handle);
        sock.bind(port)?;
        Ok(handle)
    }

    /// Access a TCP socket by handle.
    pub fn tcp_socket(&mut self, handle: SocketHandle) -> &mut tcp::Socket<'static> {
        self.sockets.get_mut::<tcp::Socket>(handle)
    }

    /// Access a UDP socket by handle.
    pub fn udp_socket(&mut self, handle: SocketHandle) -> &mut udp::Socket<'static> {
        self.sockets.get_mut::<udp::Socket>(handle)
    }

    /// Access the smoltcp interface (for connect context, etc.).
    pub fn iface(&mut self) -> &mut Interface {
        &mut self.iface
    }

    /// Access the raw device (for draining TX, etc.).
    pub fn device(&mut self) -> &mut VirtualDevice {
        &mut self.device
    }

    /// Access the socket set.
    pub fn sockets(&mut self) -> &mut SocketSet<'static> {
        &mut self.sockets
    }

    fn handle_dhcp(&mut self) {
        let sock = self.sockets.get_mut::<udp::Socket>(self.dhcp_handle);

        // Collect incoming DHCP requests
        let mut requests = Vec::new();
        while sock.can_recv() {
            match sock.recv() {
                Ok((data, meta)) => {
                    requests.push((data.to_vec(), meta.endpoint));
                }
                Err(_) => break,
            }
        }

        // Process each request and send responses
        for (data, _source) in requests {
            if let Some(response) = self.dhcp_server.process(&data) {
                let sock = self.sockets.get_mut::<udp::Socket>(self.dhcp_handle);
                // DHCP responses go to broadcast on port 68
                let dest = IpEndpoint::new(IpAddress::v4(255, 255, 255, 255), 68);
                let _ = sock.send_slice(&response, dest);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_config::{GATEWAY_IP, GATEWAY_MAC, VM_MAC};
    use smoltcp::wire::{
        ArpOperation, ArpPacket, ArpRepr, EthernetAddress, EthernetFrame, EthernetProtocol,
        EthernetRepr, Ipv4Address,
    };

    /// Helper: build an ARP request frame.
    fn build_arp_request(
        src_mac: EthernetAddress,
        src_ip: Ipv4Address,
        target_ip: Ipv4Address,
    ) -> Vec<u8> {
        let arp_repr = ArpRepr::EthernetIpv4 {
            operation: ArpOperation::Request,
            source_hardware_addr: src_mac,
            source_protocol_addr: src_ip,
            target_hardware_addr: EthernetAddress::BROADCAST,
            target_protocol_addr: target_ip,
        };

        let eth_repr = EthernetRepr {
            src_addr: src_mac,
            dst_addr: EthernetAddress::BROADCAST,
            ethertype: EthernetProtocol::Arp,
        };

        let arp_len = arp_repr.buffer_len();
        let total_len = eth_repr.buffer_len() + arp_len;
        let mut buf = vec![0u8; total_len];

        let mut frame = EthernetFrame::new_unchecked(&mut buf);
        eth_repr.emit(&mut frame);

        let mut arp_packet = ArpPacket::new_unchecked(frame.payload_mut());
        arp_repr.emit(&mut arp_packet);

        buf
    }

    /// Helper: build a raw Ethernet/IP/UDP frame.
    fn build_udp_frame(
        src_mac: EthernetAddress,
        dst_mac: EthernetAddress,
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        // Ethernet header (14) + IP header (20) + UDP header (8) + payload
        let total = 14 + 20 + 8 + payload.len();
        let mut buf = vec![0u8; total];

        // Ethernet header
        buf[0..6].copy_from_slice(&dst_mac.0);
        buf[6..12].copy_from_slice(&src_mac.0);
        buf[12..14].copy_from_slice(&u16::from(EthernetProtocol::Ipv4).to_be_bytes());

        // IP header
        let ip_start = 14;
        let ip_total_len = (20 + 8 + payload.len()) as u16;
        buf[ip_start] = 0x45; // version=4, IHL=5
        buf[ip_start + 2..ip_start + 4].copy_from_slice(&ip_total_len.to_be_bytes());
        buf[ip_start + 8] = 64; // TTL
        buf[ip_start + 9] = 17; // Protocol: UDP
        buf[ip_start + 12..ip_start + 16].copy_from_slice(&src_ip);
        buf[ip_start + 16..ip_start + 20].copy_from_slice(&dst_ip);
        // IP checksum
        let checksum = ip_checksum(&buf[ip_start..ip_start + 20]);
        buf[ip_start + 10..ip_start + 12].copy_from_slice(&checksum.to_be_bytes());

        // UDP header
        let udp_start = 34;
        let udp_len = (8 + payload.len()) as u16;
        buf[udp_start..udp_start + 2].copy_from_slice(&src_port.to_be_bytes());
        buf[udp_start + 2..udp_start + 4].copy_from_slice(&dst_port.to_be_bytes());
        buf[udp_start + 4..udp_start + 6].copy_from_slice(&udp_len.to_be_bytes());
        // UDP checksum = 0 (optional for IPv4)
        buf[udp_start + 6..udp_start + 8].copy_from_slice(&[0, 0]);

        // Payload
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

    #[test]
    fn test_arp_request_response() {
        let mut net = VirtualNetwork::new();

        // VM sends ARP: who has GATEWAY_IP?
        let arp_frame = build_arp_request(VM_MAC, Ipv4Address::new(192, 168, 127, 3), GATEWAY_IP);
        net.inject_frame(arp_frame);

        let outbound = net.poll(Instant::ZERO);

        // Should get an ARP reply
        assert!(
            !outbound.is_empty(),
            "Expected ARP reply but got no outbound frames"
        );

        // Parse the first outbound frame as Ethernet
        let frame = EthernetFrame::new_checked(&outbound[0]).expect("valid ethernet frame");
        assert_eq!(frame.ethertype(), EthernetProtocol::Arp);

        let arp = ArpPacket::new_checked(frame.payload()).expect("valid ARP packet");
        let repr = ArpRepr::parse(&arp).expect("valid ARP repr");

        match repr {
            ArpRepr::EthernetIpv4 {
                operation,
                source_hardware_addr,
                source_protocol_addr,
                target_hardware_addr,
                target_protocol_addr,
            } => {
                assert_eq!(operation, ArpOperation::Reply);
                assert_eq!(source_hardware_addr, GATEWAY_MAC);
                assert_eq!(source_protocol_addr, GATEWAY_IP);
                assert_eq!(target_hardware_addr, VM_MAC);
                assert_eq!(target_protocol_addr, Ipv4Address::new(192, 168, 127, 3));
            }
            _ => panic!("unexpected ARP repr"),
        }
    }

    #[test]
    fn test_dhcp_via_interface() {
        use crate::dhcp;

        let mut net = VirtualNetwork::new();

        // Build a DHCP DISCOVER inside a UDP frame
        // DHCP: src=0.0.0.0:68, dst=255.255.255.255:67
        let discover = build_dhcp_discover();
        let frame = build_udp_frame(
            VM_MAC,
            EthernetAddress::BROADCAST,
            [0, 0, 0, 0],
            [255, 255, 255, 255],
            68,
            67,
            &discover,
        );

        net.inject_frame(frame);
        let outbound = net.poll(Instant::from_millis(100));

        // Should have at least one outbound frame (the DHCP OFFER)
        // The response may be preceded by ARP frames
        let mut found_offer = false;
        for frame_bytes in &outbound {
            if let Ok(frame) = EthernetFrame::new_checked(frame_bytes.as_slice()) {
                if frame.ethertype() == EthernetProtocol::Ipv4 {
                    // Parse IP header to check for UDP port 68
                    let ip_payload = frame.payload();
                    if ip_payload.len() >= 20 && ip_payload[9] == 17 {
                        // UDP
                        let ihl = (ip_payload[0] & 0x0F) as usize * 4;
                        if ip_payload.len() >= ihl + 4 {
                            let dst_port =
                                u16::from_be_bytes([ip_payload[ihl + 2], ip_payload[ihl + 3]]);
                            if dst_port == 68 {
                                // Extract DHCP payload
                                let udp_payload = &ip_payload[ihl + 8..];
                                let msg_type = dhcp::message_type(udp_payload);
                                if msg_type == Some(2) {
                                    // OFFER
                                    found_offer = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(found_offer, "Expected DHCP OFFER in outbound frames");
    }

    #[test]
    fn test_poll_empty() {
        let mut net = VirtualNetwork::new();
        let outbound = net.poll(Instant::ZERO);
        assert!(
            outbound.is_empty(),
            "Expected no outbound frames on empty poll"
        );
    }

    /// Build a minimal DHCP DISCOVER payload.
    fn build_dhcp_discover() -> Vec<u8> {
        let magic: [u8; 4] = [99, 130, 83, 99];
        let mut msg = vec![0u8; 240 + 3];
        msg[0] = 1; // BOOTREQUEST
        msg[1] = 1; // Ethernet
        msg[2] = 6; // MAC length
        // XID
        msg[4..8].copy_from_slice(&0xABCD1234u32.to_be_bytes());
        // Client MAC
        msg[28..34].copy_from_slice(&VM_MAC.0);
        // Magic cookie
        msg[236..240].copy_from_slice(&magic);
        // Option: DHCP message type = DISCOVER
        msg[240] = 53; // Option 53
        msg[241] = 1; // Length
        msg[242] = 1; // DISCOVER
        msg
    }
}
