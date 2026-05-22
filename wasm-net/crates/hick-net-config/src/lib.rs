//! Network constants and configuration for the virtual network.
//!
//! The default constants match the upstream `c2w-net` bridge so that
//! container2wasm guests work without reconfiguration. [`NetworkConfig`]
//! allows each container to use a unique subnet for multi-container setups.

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

/// Per-container network configuration.
///
/// Each container gets its own subnet so traffic can be routed between
/// containers via the in-process [`NetworkBridge`].
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub gateway_ip: Ipv4Address,
    pub vm_ip: Ipv4Address,
    pub subnet_prefix: u8,
    pub gateway_mac: EthernetAddress,
    pub vm_mac: EthernetAddress,
    pub subnet_mask: Ipv4Address,
    pub dhcp_lease_secs: u32,
}

impl NetworkConfig {
    /// Build a config for container N: gateway 10.0.{N+1}.1, VM 10.0.{N+1}.2, /24.
    pub fn for_container(index: u8) -> Self {
        let subnet = index.wrapping_add(1);
        Self {
            gateway_ip: Ipv4Address::new(10, 0, subnet, 1),
            vm_ip: Ipv4Address::new(10, 0, subnet, 2),
            subnet_prefix: 24,
            gateway_mac: EthernetAddress([0x02, 0x00, 0x00, 0x00, subnet, 0x01]),
            vm_mac: EthernetAddress([0x02, 0x00, 0x00, 0x00, subnet, 0x02]),
            subnet_mask: Ipv4Address::new(255, 255, 255, 0),
            dhcp_lease_secs: DHCP_LEASE_SECS,
        }
    }

    /// Gateway CIDR for the smoltcp interface.
    pub fn gateway_cidr(&self) -> Ipv4Cidr {
        Ipv4Cidr::new(self.gateway_ip, self.subnet_prefix)
    }

    /// Gateway IP as a `std::net::Ipv4Addr`.
    pub fn gateway_ip_std(&self) -> std::net::Ipv4Addr {
        let o = self.gateway_ip.octets();
        std::net::Ipv4Addr::new(o[0], o[1], o[2], o[3])
    }

    /// VM IP as a `std::net::Ipv4Addr`.
    pub fn vm_ip_std(&self) -> std::net::Ipv4Addr {
        let o = self.vm_ip.octets();
        std::net::Ipv4Addr::new(o[0], o[1], o[2], o[3])
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            gateway_ip: GATEWAY_IP,
            vm_ip: VM_IP,
            subnet_prefix: SUBNET_PREFIX,
            gateway_mac: GATEWAY_MAC,
            vm_mac: VM_MAC,
            subnet_mask: SUBNET_MASK,
            dhcp_lease_secs: DHCP_LEASE_SECS,
        }
    }
}

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

    #[test]
    fn test_default_config_matches_constants() {
        let cfg = NetworkConfig::default();
        assert_eq!(cfg.gateway_ip, GATEWAY_IP);
        assert_eq!(cfg.vm_ip, VM_IP);
        assert_eq!(cfg.subnet_prefix, SUBNET_PREFIX);
        assert_eq!(cfg.gateway_mac, GATEWAY_MAC);
        assert_eq!(cfg.vm_mac, VM_MAC);
        assert_eq!(cfg.subnet_mask, SUBNET_MASK);
        assert_eq!(cfg.dhcp_lease_secs, DHCP_LEASE_SECS);
    }

    #[test]
    fn test_for_container_addressing() {
        let cfg = NetworkConfig::for_container(0);
        assert_eq!(cfg.gateway_ip, Ipv4Address::new(10, 0, 1, 1));
        assert_eq!(cfg.vm_ip, Ipv4Address::new(10, 0, 1, 2));
        assert_eq!(cfg.subnet_prefix, 24);

        let cfg2 = NetworkConfig::for_container(4);
        assert_eq!(cfg2.gateway_ip, Ipv4Address::new(10, 0, 5, 1));
        assert_eq!(cfg2.vm_ip, Ipv4Address::new(10, 0, 5, 2));

        // MACs are unique per container
        assert_ne!(cfg.gateway_mac, cfg2.gateway_mac);
        assert_ne!(cfg.vm_mac, cfg2.vm_mac);
    }
}
