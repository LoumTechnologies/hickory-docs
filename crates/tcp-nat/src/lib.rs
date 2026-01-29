//! TCP NAT for transparent proxying of external connections.
//!
//! Rewrites TCP frame headers so smoltcp always sees connections to
//! `GATEWAY_IP`, eliminating the need for external IPs in the smoltcp
//! address list. This solves the 2-slot limitation in smoltcp 0.12's
//! `proto-ipv4` address list.
//!
//! - **Ingress** (VM -> smoltcp): rewrites `dst_ip` from external -> `GATEWAY_IP`
//! - **Egress** (smoltcp -> VM): rewrites `src_ip` from `GATEWAY_IP` -> real external IP

use std::collections::HashMap;
use std::net::Ipv4Addr;

use net_config::{self, NetworkConfig};

/// Result of ingress frame rewriting.
pub enum IngressResult {
    /// Frame was not TCP or not to an external IP — no rewrite performed.
    PassThrough,
    /// TCP frame was rewritten. Not a SYN.
    Rewritten,
    /// TCP SYN was rewritten. Network loop should create a listener.
    Syn { dest_port: u16 },
}

/// NAT table mapping VM source ports to real external destination IPs.
pub struct TcpNat {
    /// Maps VM source port -> real external destination IP.
    nat_table: HashMap<u16, Ipv4Addr>,
    /// The gateway IP that smoltcp is configured with.
    gateway_ip: Ipv4Addr,
    /// The VM IP (passthrough — don't NAT traffic to itself).
    vm_ip: Ipv4Addr,
}

impl Default for TcpNat {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpNat {
    pub fn new() -> Self {
        Self {
            nat_table: HashMap::new(),
            gateway_ip: net_config::GATEWAY_IP_STD,
            vm_ip: net_config::VM_IP_STD,
        }
    }

    /// Create a NAT configured for a specific container's network.
    pub fn with_config(config: &NetworkConfig) -> Self {
        Self {
            nat_table: HashMap::new(),
            gateway_ip: config.gateway_ip_std(),
            vm_ip: config.vm_ip_std(),
        }
    }

    /// Rewrite an ingress frame (VM -> smoltcp).
    ///
    /// If the frame is TCP destined for an external IP, rewrites `dst_ip` to
    /// `GATEWAY_IP` and records the mapping for egress rewriting.
    pub fn rewrite_ingress(&mut self, frame: &mut [u8]) -> IngressResult {
        // Minimum: Ethernet (14) + IP (20) + TCP (20)
        if frame.len() < 54 {
            return IngressResult::PassThrough;
        }

        let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
        if ethertype != 0x0800 {
            return IngressResult::PassThrough;
        }

        let ip_start = 14;
        let ihl = (frame[ip_start] & 0x0F) as usize * 4;
        if ihl < 20 || frame.len() < ip_start + ihl {
            return IngressResult::PassThrough;
        }

        let proto = frame[ip_start + 9];
        if proto != 6 {
            return IngressResult::PassThrough; // Not TCP
        }

        let tcp_start = ip_start + ihl;
        if frame.len() < tcp_start + 20 {
            return IngressResult::PassThrough;
        }

        let dst_ip = Ipv4Addr::new(
            frame[ip_start + 16],
            frame[ip_start + 17],
            frame[ip_start + 18],
            frame[ip_start + 19],
        );

        // Only rewrite external IPs
        if dst_ip == self.gateway_ip || dst_ip == self.vm_ip {
            return IngressResult::PassThrough;
        }

        let src_port = u16::from_be_bytes([frame[tcp_start], frame[tcp_start + 1]]);
        let dest_port = u16::from_be_bytes([frame[tcp_start + 2], frame[tcp_start + 3]]);

        // Record the mapping
        self.nat_table.insert(src_port, dst_ip);

        // Rewrite dst_ip -> gateway IP
        let gw = self.gateway_ip.octets();
        frame[ip_start + 16] = gw[0];
        frame[ip_start + 17] = gw[1];
        frame[ip_start + 18] = gw[2];
        frame[ip_start + 19] = gw[3];

        // Recalculate checksums
        recompute_ip_checksum(frame);
        recompute_tcp_checksum(frame);

        // Check for SYN (but not SYN+ACK)
        let tcp_flags = frame[tcp_start + 13];
        let syn = tcp_flags & 0x02 != 0;
        let ack = tcp_flags & 0x10 != 0;
        if syn && !ack {
            IngressResult::Syn { dest_port }
        } else {
            IngressResult::Rewritten
        }
    }

    /// Rewrite an egress frame (smoltcp -> VM).
    ///
    /// If the frame is TCP from `GATEWAY_IP`, looks up the real destination IP
    /// from the NAT table and rewrites `src_ip`.
    pub fn rewrite_egress(&mut self, frame: &mut [u8]) {
        if frame.len() < 54 {
            return;
        }

        let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
        if ethertype != 0x0800 {
            return;
        }

        let ip_start = 14;
        let ihl = (frame[ip_start] & 0x0F) as usize * 4;
        if ihl < 20 || frame.len() < ip_start + ihl {
            return;
        }

        let proto = frame[ip_start + 9];
        if proto != 6 {
            return; // Not TCP
        }

        let tcp_start = ip_start + ihl;
        if frame.len() < tcp_start + 20 {
            return;
        }

        let src_ip = Ipv4Addr::new(
            frame[ip_start + 12],
            frame[ip_start + 13],
            frame[ip_start + 14],
            frame[ip_start + 15],
        );

        if src_ip != self.gateway_ip {
            return;
        }

        // dst_port in egress is the VM's source port
        let dst_port = u16::from_be_bytes([frame[tcp_start + 2], frame[tcp_start + 3]]);

        if let Some(&real_ip) = self.nat_table.get(&dst_port) {
            let octets = real_ip.octets();
            frame[ip_start + 12] = octets[0];
            frame[ip_start + 13] = octets[1];
            frame[ip_start + 14] = octets[2];
            frame[ip_start + 15] = octets[3];

            recompute_ip_checksum(frame);
            recompute_tcp_checksum(frame);
        }
    }

    /// Look up the real destination IP for a VM source port.
    pub fn lookup_real_dest(&self, vm_src_port: u16) -> Option<Ipv4Addr> {
        self.nat_table.get(&vm_src_port).copied()
    }

    /// Remove a NAT table entry when the connection closes.
    pub fn remove(&mut self, vm_src_port: u16) {
        self.nat_table.remove(&vm_src_port);
    }
}

/// Recompute the IPv4 header checksum in place.
fn recompute_ip_checksum(frame: &mut [u8]) {
    let ip_start = 14;
    let ihl = (frame[ip_start] & 0x0F) as usize * 4;

    // Zero the checksum field
    frame[ip_start + 10] = 0;
    frame[ip_start + 11] = 0;

    let mut sum: u32 = 0;
    for i in (0..ihl).step_by(2) {
        let word = if i + 1 < ihl {
            (frame[ip_start + i] as u32) << 8 | frame[ip_start + i + 1] as u32
        } else {
            (frame[ip_start + i] as u32) << 8
        };
        sum += word;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let checksum = !(sum as u16);
    frame[ip_start + 10..ip_start + 12].copy_from_slice(&checksum.to_be_bytes());
}

/// Recompute the TCP checksum in place (pseudo-header + TCP segment).
fn recompute_tcp_checksum(frame: &mut [u8]) {
    let ip_start = 14;
    let ihl = (frame[ip_start] & 0x0F) as usize * 4;
    let ip_total_len = u16::from_be_bytes([frame[ip_start + 2], frame[ip_start + 3]]) as usize;
    let tcp_start = ip_start + ihl;
    let tcp_len = ip_total_len.saturating_sub(ihl);

    if frame.len() < tcp_start + tcp_len || tcp_len < 20 {
        return;
    }

    // Zero the TCP checksum field
    frame[tcp_start + 16] = 0;
    frame[tcp_start + 17] = 0;

    let mut sum: u32 = 0;

    // Pseudo-header
    let src_ip = &frame[ip_start + 12..ip_start + 16];
    let dst_ip = &frame[ip_start + 16..ip_start + 20];
    sum += (src_ip[0] as u32) << 8 | src_ip[1] as u32;
    sum += (src_ip[2] as u32) << 8 | src_ip[3] as u32;
    sum += (dst_ip[0] as u32) << 8 | dst_ip[1] as u32;
    sum += (dst_ip[2] as u32) << 8 | dst_ip[3] as u32;
    sum += 6u32; // Protocol: TCP
    sum += tcp_len as u32;

    // TCP segment
    for i in (0..tcp_len).step_by(2) {
        let word = if i + 1 < tcp_len {
            (frame[tcp_start + i] as u32) << 8 | frame[tcp_start + i + 1] as u32
        } else {
            (frame[tcp_start + i] as u32) << 8
        };
        sum += word;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let checksum = !(sum as u16);
    frame[tcp_start + 16..tcp_start + 18].copy_from_slice(&checksum.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use net_config::{GATEWAY_IP_STD, VM_IP_STD};

    /// Build a raw Ethernet/IP/TCP frame for testing.
    fn build_tcp_frame(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        flags: u8,
    ) -> Vec<u8> {
        let tcp_header_len = 20;
        let ip_total_len = (20 + tcp_header_len) as u16;
        let total = 14 + 20 + tcp_header_len;
        let mut buf = vec![0u8; total];

        // Ethernet header
        buf[0..6].copy_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x01]); // dst MAC
        buf[6..12].copy_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x02]); // src MAC
        buf[12..14].copy_from_slice(&0x0800u16.to_be_bytes());

        // IP header
        let ip_start = 14;
        buf[ip_start] = 0x45; // version=4, IHL=5
        buf[ip_start + 2..ip_start + 4].copy_from_slice(&ip_total_len.to_be_bytes());
        buf[ip_start + 8] = 64; // TTL
        buf[ip_start + 9] = 6; // Protocol: TCP
        buf[ip_start + 12..ip_start + 16].copy_from_slice(&src_ip);
        buf[ip_start + 16..ip_start + 20].copy_from_slice(&dst_ip);

        // IP checksum
        recompute_ip_checksum(&mut buf);

        // TCP header
        let tcp_start = 34;
        buf[tcp_start..tcp_start + 2].copy_from_slice(&src_port.to_be_bytes());
        buf[tcp_start + 2..tcp_start + 4].copy_from_slice(&dst_port.to_be_bytes());
        buf[tcp_start + 4..tcp_start + 8].copy_from_slice(&1000u32.to_be_bytes()); // seq
        buf[tcp_start + 12] = 5 << 4; // data offset = 5
        buf[tcp_start + 13] = flags;
        buf[tcp_start + 14..tcp_start + 16].copy_from_slice(&65535u16.to_be_bytes()); // window

        // TCP checksum
        recompute_tcp_checksum(&mut buf);

        buf
    }

    #[test]
    fn test_ingress_syn_to_external_ip() {
        let mut nat = TcpNat::new();

        let external_ip = [93, 184, 216, 34]; // example.com
        let vm_ip = VM_IP_STD.octets();
        let mut frame = build_tcp_frame(vm_ip, external_ip, 12345, 80, 0x02); // SYN

        let result = nat.rewrite_ingress(&mut frame);
        assert!(matches!(result, IngressResult::Syn { dest_port: 80 }));

        // dst_ip should now be GATEWAY_IP
        let ip_start = 14;
        let rewritten_dst = Ipv4Addr::new(
            frame[ip_start + 16],
            frame[ip_start + 17],
            frame[ip_start + 18],
            frame[ip_start + 19],
        );
        assert_eq!(rewritten_dst, GATEWAY_IP_STD);

        // NAT table should have the mapping
        assert_eq!(
            nat.lookup_real_dest(12345),
            Some(Ipv4Addr::new(93, 184, 216, 34))
        );
    }

    #[test]
    fn test_ingress_passthrough_to_gateway() {
        let mut nat = TcpNat::new();

        let vm_ip = VM_IP_STD.octets();
        let gw_ip = GATEWAY_IP_STD.octets();
        let mut frame = build_tcp_frame(vm_ip, gw_ip, 12345, 80, 0x02);

        let result = nat.rewrite_ingress(&mut frame);
        assert!(matches!(result, IngressResult::PassThrough));

        // NAT table should be empty
        assert_eq!(nat.lookup_real_dest(12345), None);
    }

    #[test]
    fn test_ingress_non_syn_tcp() {
        let mut nat = TcpNat::new();

        let external_ip = [93, 184, 216, 34];
        let vm_ip = VM_IP_STD.octets();
        // ACK flag only (not SYN)
        let mut frame = build_tcp_frame(vm_ip, external_ip, 12345, 80, 0x10);

        let result = nat.rewrite_ingress(&mut frame);
        assert!(matches!(result, IngressResult::Rewritten));
    }

    #[test]
    fn test_ingress_udp_passthrough() {
        let mut nat = TcpNat::new();

        // Build a UDP frame (protocol 17 instead of 6)
        let mut frame = build_tcp_frame([192, 168, 127, 3], [8, 8, 8, 8], 12345, 53, 0x02);
        frame[14 + 9] = 17; // Change protocol to UDP
        recompute_ip_checksum(&mut frame);

        let result = nat.rewrite_ingress(&mut frame);
        assert!(matches!(result, IngressResult::PassThrough));
    }

    #[test]
    fn test_egress_rewrite() {
        let mut nat = TcpNat::new();

        let external_ip = [93, 184, 216, 34];
        let vm_ip = VM_IP_STD.octets();

        // First do an ingress to populate the NAT table
        let mut ingress_frame = build_tcp_frame(vm_ip, external_ip, 12345, 80, 0x02);
        nat.rewrite_ingress(&mut ingress_frame);

        // Now build an egress frame: GATEWAY_IP -> VM_IP
        // src_port = 80 (server), dst_port = 12345 (VM's port)
        let gw_ip = GATEWAY_IP_STD.octets();
        let mut egress_frame = build_tcp_frame(gw_ip, vm_ip, 80, 12345, 0x12); // SYN-ACK

        nat.rewrite_egress(&mut egress_frame);

        // src_ip should now be the real external IP
        let ip_start = 14;
        let rewritten_src = Ipv4Addr::new(
            egress_frame[ip_start + 12],
            egress_frame[ip_start + 13],
            egress_frame[ip_start + 14],
            egress_frame[ip_start + 15],
        );
        assert_eq!(rewritten_src, Ipv4Addr::new(93, 184, 216, 34));
    }

    #[test]
    fn test_nat_table_remove() {
        let mut nat = TcpNat::new();

        let external_ip = [93, 184, 216, 34];
        let vm_ip = VM_IP_STD.octets();

        let mut frame = build_tcp_frame(vm_ip, external_ip, 12345, 80, 0x02);
        nat.rewrite_ingress(&mut frame);
        assert!(nat.lookup_real_dest(12345).is_some());

        nat.remove(12345);
        assert!(nat.lookup_real_dest(12345).is_none());
    }

    #[test]
    fn test_multiple_concurrent_connections() {
        let mut nat = TcpNat::new();

        let vm_ip = VM_IP_STD.octets();

        // Connection 1: VM:11111 -> 93.184.216.34:80
        let mut frame1 = build_tcp_frame(vm_ip, [93, 184, 216, 34], 11111, 80, 0x02);
        nat.rewrite_ingress(&mut frame1);

        // Connection 2: VM:22222 -> 142.250.80.46:443
        let mut frame2 = build_tcp_frame(vm_ip, [142, 250, 80, 46], 22222, 443, 0x02);
        nat.rewrite_ingress(&mut frame2);

        // Both should be in the NAT table with correct destinations
        assert_eq!(
            nat.lookup_real_dest(11111),
            Some(Ipv4Addr::new(93, 184, 216, 34))
        );
        assert_eq!(
            nat.lookup_real_dest(22222),
            Some(Ipv4Addr::new(142, 250, 80, 46))
        );

        // Egress for connection 1
        let gw_ip = GATEWAY_IP_STD.octets();
        let mut egress1 = build_tcp_frame(gw_ip, vm_ip, 80, 11111, 0x12);
        nat.rewrite_egress(&mut egress1);
        let src1 = Ipv4Addr::new(
            egress1[14 + 12],
            egress1[14 + 13],
            egress1[14 + 14],
            egress1[14 + 15],
        );
        assert_eq!(src1, Ipv4Addr::new(93, 184, 216, 34));

        // Egress for connection 2
        let mut egress2 = build_tcp_frame(gw_ip, vm_ip, 443, 22222, 0x12);
        nat.rewrite_egress(&mut egress2);
        let src2 = Ipv4Addr::new(
            egress2[14 + 12],
            egress2[14 + 13],
            egress2[14 + 14],
            egress2[14 + 15],
        );
        assert_eq!(src2, Ipv4Addr::new(142, 250, 80, 46));
    }

    #[test]
    fn test_checksum_validity_after_rewrite() {
        let mut nat = TcpNat::new();

        let vm_ip = VM_IP_STD.octets();
        let external_ip = [93, 184, 216, 34];
        let mut frame = build_tcp_frame(vm_ip, external_ip, 12345, 80, 0x02);

        nat.rewrite_ingress(&mut frame);

        // Verify IP checksum
        let ip_start = 14;
        let ihl = (frame[ip_start] & 0x0F) as usize * 4;
        let mut sum: u32 = 0;
        for i in (0..ihl).step_by(2) {
            let word = (frame[ip_start + i] as u32) << 8 | frame[ip_start + i + 1] as u32;
            sum += word;
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        assert_eq!(
            sum as u16, 0xFFFF,
            "IP checksum should be valid after rewrite"
        );

        // Verify TCP checksum
        let tcp_start = ip_start + ihl;
        let ip_total_len = u16::from_be_bytes([frame[ip_start + 2], frame[ip_start + 3]]) as usize;
        let tcp_len = ip_total_len - ihl;
        let mut sum: u32 = 0;
        // Pseudo-header
        let src = &frame[ip_start + 12..ip_start + 16];
        let dst = &frame[ip_start + 16..ip_start + 20];
        sum += (src[0] as u32) << 8 | src[1] as u32;
        sum += (src[2] as u32) << 8 | src[3] as u32;
        sum += (dst[0] as u32) << 8 | dst[1] as u32;
        sum += (dst[2] as u32) << 8 | dst[3] as u32;
        sum += 6u32;
        sum += tcp_len as u32;
        for i in (0..tcp_len).step_by(2) {
            let word = if i + 1 < tcp_len {
                (frame[tcp_start + i] as u32) << 8 | frame[tcp_start + i + 1] as u32
            } else {
                (frame[tcp_start + i] as u32) << 8
            };
            sum += word;
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        assert_eq!(
            sum as u16, 0xFFFF,
            "TCP checksum should be valid after rewrite"
        );
    }
}
