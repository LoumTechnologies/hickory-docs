<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Peer Channel Admits Only Keys You Hold, And Serves Them Only What Their Grants Allow

For: Engineers using local sessions or pairing their own machines.

Given a machine paired into this fleet, when it connects to this session, then
the connection is authenticated as its ed25519 public key by the transport
itself, checked against the mutual key list, and every request it makes is
gated on that machine's grants before anything reaches this session's server.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-peer-channel-carries-grants.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Test the behavior: `local_server`

This is the maintained test fragment from `crates/hickory-peer/tests/peer_channel.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-669e41b69a09-local_server"></a>
### `local_server`

Two machines, one engineer, over a real QUIC connection.
Protects `docs/guarantees/collaboration/the-peer-channel-carries-grants.md`.
**Offline by construction.** Both endpoints use `Reach::Direct` — no
relays, no address publishing — and dial by direct address, so this passes
on a machine that has never had internet. That is not merely a test
convenience: `config-and-environments` forbids requiring network access,
and a suite that reached number0 to prove a local property would be
testing their uptime.

<storycode:copy id="f-669e41b69a09-local_server">
//! Two machines, one engineer, over a real QUIC connection.
//!
//! Protects `docs/guarantees/collaboration/the-peer-channel-carries-grants.md`.
//!
//! **Offline by construction.** Both endpoints use `Reach::Direct` — no
//! relays, no address publishing — and dial by direct address, so this passes
//! on a machine that has never had internet. That is not merely a test
//! convenience: `config-and-environments` forbids requiring network access,
//! and a suite that reached number0 to prove a local property would be
//! testing their uptime.

use std::net::SocketAddr;
use std::sync::Arc;

use hickory_fleet::{Fleet, Grant, Identity, Kind};
use hickory_peer::{Attached, PeerServer, Reach, Reachability};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

/// A stand-in for the session's own loopback server: answers every request
/// with the request line it was given, so a test can see exactly what got
/// through.
async fn local_server() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while !head.ends_with(b"\r\n\r\n") {
                    match sock.read(&mut byte).await {
                        Ok(0) | Err(_) => return,
                        Ok(_) => head.push(byte[0]),
                    }
                }
                let line = String::from_utf8_lossy(&head)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                let body = format!("reached the session: {line}");
                let _ = sock
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                        .as_bytes(),
                    )
                    .await;
            });
        }
    });
    addr
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/views/FleetPane.tsx](../../../mechanisms/apps/web/src/views/FleetPane.tsx.md#output-6e68b623335b)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-fleet/src/lib.rs](../../../mechanisms/crates/hickory-fleet/src/lib.rs.md#output-b1e920114639)
- [crates/hickory-peer/src/grants.rs](../../../mechanisms/crates/hickory-peer/src/grants.rs.md#output-fb44d9f1f9e0)
- [crates/hickory-peer/src/tunnel.rs](../../../mechanisms/crates/hickory-peer/src/tunnel.rs.md#output-17fc8ed55ce5)

### Tests and fixtures

- [crates/hickory-peer/tests/peer_channel.rs](../../../mechanisms/crates/hickory-peer/tests/peer_channel.rs.md#output-669e41b69a09)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
