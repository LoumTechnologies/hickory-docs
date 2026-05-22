//! In-process frame bridge for inter-container networking.
//!
//! Each container registers its VM IP and a channel sender with the bridge.
//! When a container sends a frame destined for another container's IP, the
//! network loop routes it through the bridge instead of the real network.
//! The bridge also provides internal DNS resolution by container name.

use std::collections::HashMap;
use std::net::Ipv4Addr;

use tokio::sync::mpsc;

/// Routes Ethernet frames between containers and resolves container names to IPs.
pub struct NetworkBridge {
    /// VM IP → frame sender for that container's network loop.
    routes: HashMap<Ipv4Addr, mpsc::Sender<Vec<u8>>>,
    /// Container name → VM IP for internal DNS.
    dns_records: HashMap<String, Ipv4Addr>,
}

impl NetworkBridge {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            dns_records: HashMap::new(),
        }
    }

    /// Register a container's VM IP and frame channel.
    /// Optionally associates a DNS name.
    pub fn register(&mut self, vm_ip: Ipv4Addr, sender: mpsc::Sender<Vec<u8>>, name: Option<&str>) {
        self.routes.insert(vm_ip, sender);
        if let Some(n) = name {
            self.dns_records.insert(n.to_string(), vm_ip);
        }
    }

    /// Remove a container from the bridge.
    pub fn unregister(&mut self, vm_ip: Ipv4Addr) {
        self.routes.remove(&vm_ip);
        self.dns_records.retain(|_, ip| *ip != vm_ip);
    }

    /// Add or update a DNS name for a VM IP.
    /// Removes any previous name pointing to this IP first.
    pub fn update_name(&mut self, vm_ip: Ipv4Addr, name: &str) {
        self.dns_records.retain(|_, ip| *ip != vm_ip);
        self.dns_records.insert(name.to_string(), vm_ip);
    }

    /// Look up the frame sender for a destination IP.
    pub fn route(&self, dest_ip: Ipv4Addr) -> Option<&mpsc::Sender<Vec<u8>>> {
        self.routes.get(&dest_ip)
    }

    /// Resolve a container name to its VM IP.
    pub fn resolve_name(&self, name: &str) -> Option<Ipv4Addr> {
        self.dns_records.get(name).copied()
    }
}

impl Default for NetworkBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_route() {
        let mut bridge = NetworkBridge::new();
        let (tx, _rx) = mpsc::channel(16);
        let ip = Ipv4Addr::new(10, 0, 1, 2);

        bridge.register(ip, tx, Some("web"));
        assert!(bridge.route(ip).is_some());
        assert_eq!(bridge.resolve_name("web"), Some(ip));
    }

    #[test]
    fn test_unregister() {
        let mut bridge = NetworkBridge::new();
        let (tx, _rx) = mpsc::channel(16);
        let ip = Ipv4Addr::new(10, 0, 1, 2);

        bridge.register(ip, tx, Some("web"));
        bridge.unregister(ip);

        assert!(bridge.route(ip).is_none());
        assert_eq!(bridge.resolve_name("web"), None);
    }

    #[test]
    fn test_update_name() {
        let mut bridge = NetworkBridge::new();
        let (tx, _rx) = mpsc::channel(16);
        let ip = Ipv4Addr::new(10, 0, 1, 2);

        bridge.register(ip, tx, Some("web"));
        bridge.update_name(ip, "api");

        assert_eq!(bridge.resolve_name("web"), None);
        assert_eq!(bridge.resolve_name("api"), Some(ip));
    }

    #[test]
    fn test_no_route_for_unknown_ip() {
        let bridge = NetworkBridge::new();
        assert!(bridge.route(Ipv4Addr::new(10, 0, 99, 2)).is_none());
    }
}
