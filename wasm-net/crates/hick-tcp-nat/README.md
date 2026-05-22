# tcp-nat

Stateful TCP NAT table for transparent proxying of external connections through
the smoltcp virtual network.

## Operations

- `TcpNat::rewrite_ingress` — rewrites destination IPs from external addresses
  to `GATEWAY_IP` (working around smoltcp 0.12's 2-slot IP address list limit)
  and records the mapping keyed by the VM's source port
- `TcpNat::rewrite_egress` — restores the real external IP as the source on
  reply frames
- `IngressResult::Syn` — signals the network loop to open a new listener for a
  new outbound connection

Both operations recompute IPv4 and TCP checksums in-place.
