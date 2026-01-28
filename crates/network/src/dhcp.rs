//! Minimal DHCP server for the virtual network.
//!
//! Handles DISCOVER→OFFER and REQUEST→ACK to assign [`VM_IP`] to the
//! container guest. Only the options required for basic connectivity are
//! included: subnet mask, router (gateway), DNS server, and lease time.

use log::debug;

use crate::net_config::{DHCP_LEASE_SECS, GATEWAY_IP, SUBNET_MASK, VM_IP};

// DHCP message types (option 53 values)
const DHCP_DISCOVER: u8 = 1;
const DHCP_OFFER: u8 = 2;
const DHCP_REQUEST: u8 = 3;
const DHCP_ACK: u8 = 5;

// DHCP option codes
const OPT_SUBNET_MASK: u8 = 1;
const OPT_ROUTER: u8 = 3;
const OPT_DNS: u8 = 6;
const OPT_LEASE_TIME: u8 = 51;
const OPT_MSG_TYPE: u8 = 53;
const OPT_SERVER_ID: u8 = 54;
const OPT_END: u8 = 255;

// DHCP magic cookie
const MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];

// Fixed field offsets in a DHCP message (after UDP payload start)
const OFF_OP: usize = 0;
const OFF_HTYPE: usize = 1;
const OFF_HLEN: usize = 2;
const OFF_XID: usize = 4;
const OFF_YIADDR: usize = 16;
const OFF_SIADDR: usize = 20;
const OFF_CHADDR: usize = 28;
const OFF_MAGIC: usize = 236;
const OFF_OPTIONS: usize = 240;

// Minimum DHCP message size (up to start of options)
const MIN_DHCP_LEN: usize = OFF_OPTIONS;

/// A minimal DHCP server that assigns a single IP to the container guest.
pub struct DhcpServer {
    /// The IP to assign to the client.
    client_ip: [u8; 4],
    /// The gateway/server IP.
    server_ip: [u8; 4],
    /// The subnet mask.
    subnet_mask: [u8; 4],
}

impl Default for DhcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DhcpServer {
    pub fn new() -> Self {
        Self {
            client_ip: VM_IP.octets(),
            server_ip: GATEWAY_IP.octets(),
            subnet_mask: SUBNET_MASK.octets(),
        }
    }

    /// Process a DHCP message (the UDP payload on port 67).
    ///
    /// Returns `Some(response)` if the message is a DISCOVER or REQUEST,
    /// `None` for unrecognized or malformed messages.
    pub fn process(&self, msg: &[u8]) -> Option<Vec<u8>> {
        if msg.len() < MIN_DHCP_LEN {
            return None;
        }

        // Verify magic cookie
        if msg[OFF_MAGIC..OFF_MAGIC + 4] != MAGIC_COOKIE {
            return None;
        }

        // Must be a BOOTREQUEST (op=1)
        if msg[OFF_OP] != 1 {
            return None;
        }

        let msg_type = self.find_option(msg, OPT_MSG_TYPE)?;
        if msg_type.is_empty() {
            return None;
        }

        match msg_type[0] {
            DHCP_DISCOVER => {
                debug!("[dhcp] Received DISCOVER, sending OFFER");
                Some(self.build_response(msg, DHCP_OFFER))
            }
            DHCP_REQUEST => {
                debug!("[dhcp] Received REQUEST, sending ACK");
                Some(self.build_response(msg, DHCP_ACK))
            }
            _ => None,
        }
    }

    fn build_response(&self, request: &[u8], response_type: u8) -> Vec<u8> {
        // Start with a zeroed buffer large enough for fixed fields + options
        let mut resp = vec![0u8; OFF_OPTIONS + 64];

        // Fixed fields
        resp[OFF_OP] = 2; // BOOTREPLY
        resp[OFF_HTYPE] = request[OFF_HTYPE];
        resp[OFF_HLEN] = request[OFF_HLEN];
        // Copy XID (4 bytes)
        resp[OFF_XID..OFF_XID + 4].copy_from_slice(&request[OFF_XID..OFF_XID + 4]);
        // Your (client) IP address
        resp[OFF_YIADDR..OFF_YIADDR + 4].copy_from_slice(&self.client_ip);
        // Server IP address
        resp[OFF_SIADDR..OFF_SIADDR + 4].copy_from_slice(&self.server_ip);
        // Copy client hardware address (16 bytes)
        resp[OFF_CHADDR..OFF_CHADDR + 16].copy_from_slice(&request[OFF_CHADDR..OFF_CHADDR + 16]);
        // Magic cookie
        resp[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC_COOKIE);

        // Options
        let mut pos = OFF_OPTIONS;
        // Message type
        pos = self.write_option(&mut resp, pos, OPT_MSG_TYPE, &[response_type]);
        // Server identifier
        pos = self.write_option(&mut resp, pos, OPT_SERVER_ID, &self.server_ip);
        // Lease time
        pos = self.write_option(
            &mut resp,
            pos,
            OPT_LEASE_TIME,
            &DHCP_LEASE_SECS.to_be_bytes(),
        );
        // Subnet mask
        pos = self.write_option(&mut resp, pos, OPT_SUBNET_MASK, &self.subnet_mask);
        // Router (gateway)
        pos = self.write_option(&mut resp, pos, OPT_ROUTER, &self.server_ip);
        // DNS server (use gateway as DNS — we forward internally)
        pos = self.write_option(&mut resp, pos, OPT_DNS, &self.server_ip);
        // End
        resp[pos] = OPT_END;

        resp.truncate(pos + 1);
        // Pad to minimum DHCP size (300 bytes) as required by some clients
        if resp.len() < 300 {
            resp.resize(300, 0);
        }

        resp
    }

    fn write_option(&self, buf: &mut Vec<u8>, pos: usize, code: u8, data: &[u8]) -> usize {
        // Ensure buffer is large enough
        let needed = pos + 2 + data.len();
        if buf.len() < needed {
            buf.resize(needed, 0);
        }
        buf[pos] = code;
        buf[pos + 1] = data.len() as u8;
        buf[pos + 2..pos + 2 + data.len()].copy_from_slice(data);
        pos + 2 + data.len()
    }

    fn find_option<'a>(&self, msg: &'a [u8], target: u8) -> Option<&'a [u8]> {
        let mut pos = OFF_OPTIONS;
        while pos < msg.len() {
            let code = msg[pos];
            if code == OPT_END {
                break;
            }
            if code == 0 {
                // Padding
                pos += 1;
                continue;
            }
            if pos + 1 >= msg.len() {
                break;
            }
            let len = msg[pos + 1] as usize;
            if pos + 2 + len > msg.len() {
                break;
            }
            if code == target {
                return Some(&msg[pos + 2..pos + 2 + len]);
            }
            pos += 2 + len;
        }
        None
    }
}

/// Extract the XID (transaction ID) from a DHCP message.
pub fn xid(msg: &[u8]) -> Option<u32> {
    if msg.len() < OFF_XID + 4 {
        return None;
    }
    Some(u32::from_be_bytes([
        msg[OFF_XID],
        msg[OFF_XID + 1],
        msg[OFF_XID + 2],
        msg[OFF_XID + 3],
    ]))
}

/// Extract the DHCP message type from options.
pub fn message_type(msg: &[u8]) -> Option<u8> {
    let server = DhcpServer::new();
    let opt = server.find_option(msg, OPT_MSG_TYPE)?;
    opt.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal DHCP DISCOVER/REQUEST message for testing.
    fn build_dhcp_message(msg_type: u8, xid_val: u32) -> Vec<u8> {
        let mut msg = vec![0u8; OFF_OPTIONS + 3];
        msg[OFF_OP] = 1; // BOOTREQUEST
        msg[OFF_HTYPE] = 1; // Ethernet
        msg[OFF_HLEN] = 6; // MAC length
        msg[OFF_XID..OFF_XID + 4].copy_from_slice(&xid_val.to_be_bytes());
        // Client MAC
        msg[OFF_CHADDR] = 0x02;
        msg[OFF_CHADDR + 1] = 0x00;
        msg[OFF_CHADDR + 2] = 0x00;
        msg[OFF_CHADDR + 3] = 0x00;
        msg[OFF_CHADDR + 4] = 0x00;
        msg[OFF_CHADDR + 5] = 0x02;
        // Magic cookie
        msg[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC_COOKIE);
        // Option: message type
        msg[OFF_OPTIONS] = OPT_MSG_TYPE;
        msg[OFF_OPTIONS + 1] = 1;
        msg[OFF_OPTIONS + 2] = msg_type;
        msg
    }

    #[test]
    fn test_dhcp_discover_offer() {
        let server = DhcpServer::new();
        let discover = build_dhcp_message(DHCP_DISCOVER, 0x12345678);
        let response = server.process(&discover).expect("should produce OFFER");

        // Verify it's an OFFER
        assert_eq!(message_type(&response), Some(DHCP_OFFER));
        // Verify XID matches
        assert_eq!(xid(&response), Some(0x12345678));
        // Verify assigned IP
        assert_eq!(&response[OFF_YIADDR..OFF_YIADDR + 4], &VM_IP.octets());
        // Verify server IP
        assert_eq!(&response[OFF_SIADDR..OFF_SIADDR + 4], &GATEWAY_IP.octets());

        // Verify subnet mask option
        let srv = DhcpServer::new();
        let mask = srv.find_option(&response, OPT_SUBNET_MASK).unwrap();
        assert_eq!(mask, SUBNET_MASK.octets());

        // Verify router option
        let router = srv.find_option(&response, OPT_ROUTER).unwrap();
        assert_eq!(router, GATEWAY_IP.octets());

        // Verify DNS option
        let dns = srv.find_option(&response, OPT_DNS).unwrap();
        assert_eq!(dns, GATEWAY_IP.octets());
    }

    #[test]
    fn test_dhcp_request_ack() {
        let server = DhcpServer::new();
        let request = build_dhcp_message(DHCP_REQUEST, 0xAABBCCDD);
        let response = server.process(&request).expect("should produce ACK");

        assert_eq!(message_type(&response), Some(DHCP_ACK));
        assert_eq!(xid(&response), Some(0xAABBCCDD));
        assert_eq!(&response[OFF_YIADDR..OFF_YIADDR + 4], &VM_IP.octets());
    }

    #[test]
    fn test_dhcp_unknown_type_ignored() {
        let server = DhcpServer::new();
        // DHCP INFORM (type 8) — not handled
        let msg = build_dhcp_message(8, 0x11111111);
        assert!(server.process(&msg).is_none());
    }

    #[test]
    fn test_dhcp_full_handshake() {
        let server = DhcpServer::new();
        let xid_val: u32 = 0xDEADBEEF;

        // Step 1: DISCOVER → OFFER
        let discover = build_dhcp_message(DHCP_DISCOVER, xid_val);
        let offer = server.process(&discover).expect("OFFER");
        assert_eq!(message_type(&offer), Some(DHCP_OFFER));
        assert_eq!(xid(&offer), Some(xid_val));

        // Step 2: REQUEST → ACK
        let request = build_dhcp_message(DHCP_REQUEST, xid_val);
        let ack = server.process(&request).expect("ACK");
        assert_eq!(message_type(&ack), Some(DHCP_ACK));
        assert_eq!(xid(&ack), Some(xid_val));

        // Both should assign the same IP
        assert_eq!(
            &offer[OFF_YIADDR..OFF_YIADDR + 4],
            &ack[OFF_YIADDR..OFF_YIADDR + 4]
        );

        // Verify lease time in ACK
        let srv = DhcpServer::new();
        let lease = srv.find_option(&ack, OPT_LEASE_TIME).unwrap();
        assert_eq!(
            u32::from_be_bytes([lease[0], lease[1], lease[2], lease[3]]),
            DHCP_LEASE_SECS
        );
    }

    #[test]
    fn test_malformed_message_rejected() {
        let server = DhcpServer::new();
        // Too short
        assert!(server.process(&[0u8; 10]).is_none());
        // Wrong magic cookie
        let mut msg = build_dhcp_message(DHCP_DISCOVER, 1);
        msg[OFF_MAGIC] = 0;
        assert!(server.process(&msg).is_none());
        // Wrong op (reply, not request)
        let mut msg = build_dhcp_message(DHCP_DISCOVER, 1);
        msg[OFF_OP] = 2;
        assert!(server.process(&msg).is_none());
    }
}
