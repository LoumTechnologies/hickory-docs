//! Network constants for the virtual network.
//!
//! These match the defaults used by the upstream `c2w-net` bridge so that
//! container2wasm guests work without reconfiguration.

use smoltcp::wire::{EthernetAddress, Ipv4Address, Ipv4Cidr};

/// Gateway (host-side) IP address — the smoltcp interface address.
pub const GATEWAY_IP: Ipv4Address = Ipv4Address::new(192, 168, 127, 1);

/// VM (container-side) IP address — assigned via DHCP.
pub const VM_IP: Ipv4Address = Ipv4Address::new(192, 168, 127, 3);

/// Subnet mask as a prefix length.
pub const SUBNET_PREFIX: u8 = 24;

/// Gateway CIDR for the smoltcp interface.
pub const GATEWAY_CIDR: Ipv4Cidr = Ipv4Cidr::new(GATEWAY_IP, SUBNET_PREFIX);

/// MAC address of the gateway (smoltcp interface).
pub const GATEWAY_MAC: EthernetAddress = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);

/// MAC address of the VM (container guest).
pub const VM_MAC: EthernetAddress = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x02]);

/// Gateway IP as a `std::net::Ipv4Addr` (for use outside smoltcp, e.g. NAT).
pub const GATEWAY_IP_STD: std::net::Ipv4Addr = std::net::Ipv4Addr::new(192, 168, 127, 1);

/// VM IP as a `std::net::Ipv4Addr` (for use outside smoltcp, e.g. NAT).
pub const VM_IP_STD: std::net::Ipv4Addr = std::net::Ipv4Addr::new(192, 168, 127, 3);

/// Ethernet MTU (maximum payload size inside an Ethernet frame).
pub const MTU: usize = 1500;

/// Subnet mask as a dotted-quad address (for DHCP option).
pub const SUBNET_MASK: Ipv4Address = Ipv4Address::new(255, 255, 255, 0);

/// DHCP lease duration in seconds.
pub const DHCP_LEASE_SECS: u32 = 86400;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_consistent() {
        // Gateway and VM are on the same /24 subnet
        let gw = GATEWAY_IP.octets();
        let vm = VM_IP.octets();
        assert_eq!(gw[..3], vm[..3]);
        assert_eq!(SUBNET_PREFIX, 24);
        assert_eq!(SUBNET_MASK, Ipv4Address::new(255, 255, 255, 0));

        // MACs are locally-administered (bit 1 of first octet set)
        assert!(GATEWAY_MAC.0[0] & 0x02 != 0);
        assert!(VM_MAC.0[0] & 0x02 != 0);

        // MACs are distinct
        assert_ne!(GATEWAY_MAC, VM_MAC);
    }
}
