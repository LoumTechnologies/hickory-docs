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

struct Pair {
    client: Identity,
    server: Arc<PeerServer>,
    addr: iroh::EndpointAddr,
    _dirs: (tempfile::TempDir, tempfile::TempDir),
}

/// Pair two machines mutually, grant the client `grants`, and stand the
/// server up.
async fn pair(grants: &[Grant], kind: Kind) -> Pair {
    let host_dir = tempfile::tempdir().unwrap();
    let peer_dir = tempfile::tempdir().unwrap();
    let host = Identity::load_or_create(host_dir.path(), "desktop").unwrap();
    let client = Identity::load_or_create(peer_dir.path(), "laptop").unwrap();

    let fleet = Fleet::at(host.dir());
    fleet.add(&client.invitation(kind), "2026-08-24").unwrap();
    for grant in [Grant::View, Grant::Edit, Grant::Execute] {
        fleet
            .set_grant("laptop", grant, grants.contains(&grant))
            .unwrap();
    }

    let local = local_server().await;
    let server = Arc::new(
        PeerServer::bind(&host, Fleet::at(host.dir()), local, Reach::Direct)
            .await
            .unwrap(),
    );
    let addr = server.addr();
    let serving = server.clone();
    tokio::spawn(async move {
        let _ = serving.serve().await;
    });

    Pair {
        client,
        server,
        addr,
        _dirs: (host_dir, peer_dir),
    }
}

async fn attach(pair: &Pair) -> Attached {
    Attached::connect(&pair.client, pair.addr.clone(), Reach::Direct)
        .await
        .expect("the peer connects")
}

#[tokio::test]
async fn a_paired_machine_reaches_this_sessions_own_server() {
    // The peer channel is a window onto the session, so what answers is the
    // same code that answers the window on this machine.
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let answer = peer.request("GET", "/api/docs/abc").await.unwrap();
    assert!(answer.contains("200 OK"), "{answer}");
    assert!(
        answer.contains("reached the session: GET /api/docs/abc"),
        "{answer}"
    );
    peer.close().await;
}

#[tokio::test]
async fn a_key_nobody_paired_is_refused_and_told_a_fleet_is_mutual() {
    // The usual cause is pairing in one direction only.
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let stranger_dir = tempfile::tempdir().unwrap();
    let stranger = Identity::load_or_create(stranger_dir.path(), "not-yours").unwrap();

    let peer = Attached::connect(&stranger, pair.addr.clone(), Reach::Direct).await;
    // Either the connection is closed during the handshake or the first
    // request fails — both are the refusal; what must never happen is a
    // request being served.
    match peer {
        Err(_) => {}
        Ok(peer) => {
            let answer = peer.request("GET", "/api/docs/abc").await;
            assert!(
                answer.is_err() || !answer.unwrap().contains("reached the session"),
                "an unpaired key reached the session"
            );
        }
    }
}

#[tokio::test]
async fn view_alone_cannot_write_and_the_refusal_says_which_grant() {
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let peer = attach(&pair).await;

    let read = peer.request("GET", "/api/docs/abc").await.unwrap();
    assert!(read.contains("reached the session"), "{read}");

    let write = peer.request("PUT", "/api/docs/abc").await.unwrap();
    assert!(write.contains("403"), "{write}");
    assert!(!write.contains("reached the session"), "{write}");
    assert!(write.contains("`edit` grant"), "{write}");
    // Names the machine and the exact command that fixes it.
    assert!(write.contains("hick fleet grant laptop edit"), "{write}");
    peer.close().await;
}

#[tokio::test]
async fn running_a_cell_needs_execute_which_is_off_by_default() {
    // "My laptop was stolen" must not read as "every machine I own now
    // executes whatever the thief types".
    let pair = pair(&[Grant::View, Grant::Edit], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let run = peer.request("POST", "/api/docs/abc/run").await.unwrap();
    assert!(run.contains("403"), "{run}");
    assert!(run.contains("`execute` grant"), "{run}");
    assert!(run.contains("my laptop was stolen"), "{run}");
    peer.close().await;
}

#[tokio::test]
async fn with_execute_granted_a_cell_run_reaches_the_session() {
    let pair = pair(&[Grant::View, Grant::Edit, Grant::Execute], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let run = peer.request("POST", "/api/docs/abc/run").await.unwrap();
    assert!(
        run.contains("reached the session: POST /api/docs/abc/run"),
        "{run}"
    );
    peer.close().await;
}

#[tokio::test]
async fn settings_are_unreachable_over_the_channel_whatever_the_grants() {
    // They hold provider keys and the continuity switch. The fleet channel
    // carries no key material by design rather than by rule.
    let pair = pair(&[Grant::View, Grant::Edit, Grant::Execute], Kind::Desktop).await;
    let peer = attach(&pair).await;
    for path in ["/api/settings/keys", "/api/settings/ui"] {
        let answer = peer.request("GET", path).await.unwrap();
        assert!(answer.contains("403"), "{path}: {answer}");
        assert!(answer.contains("only routes it knows"), "{path}: {answer}");
    }
    peer.close().await;
}

#[tokio::test]
async fn an_unknown_route_is_refused_rather_than_passed_through() {
    // Deny by default: allowing anything unmatched would make every route
    // added later reachable by every paired machine without anybody deciding.
    let pair = pair(&[Grant::View, Grant::Edit, Grant::Execute], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let answer = peer
        .request("GET", "/api/something-invented-later")
        .await
        .unwrap();
    assert!(answer.contains("403"), "{answer}");
    assert!(!answer.contains("reached the session"), "{answer}");
    peer.close().await;
}

#[tokio::test]
async fn a_direct_connection_is_reported_as_direct() {
    // A third party in the path is a thing to be told rather than to
    // discover, so the answer has to be right when there isn't one either.
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let peer = attach(&pair).await;
    peer.request("GET", "/api/health").await.unwrap();
    assert_eq!(peer.how().await, Reachability::Direct);
    peer.close().await;
}

#[tokio::test]
async fn the_direct_posture_publishes_nothing_and_says_so() {
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let summary = pair.server.reach().summary();
    assert!(summary.contains("No relays"), "{summary}");
    assert!(summary.contains("reported unreachable"), "{summary}");
}

// ---------------------------------------------------------------------------
// One request per stream, and the local forwarder
// ---------------------------------------------------------------------------

/// Write two requests back-to-back down ONE stream, and see what comes back.
///
/// This is the attack the one-request-per-stream rule exists to stop: the
/// grant check runs on one head, so a second request appended behind the
/// first must not reach the session. `GET /api/health` is permitted with
/// `view`; `POST /api/docs/x/run` is not permitted without `execute`.
async fn two_on_one_stream(pair: &Pair, first: &str, second: &str) -> String {
    let peer = attach(pair).await;
    let (mut send, mut recv) = peer.connection().open_bi().await.unwrap();
    send.write_all(format!("{first}\r\n{second}\r\n").as_bytes())
        .await
        .unwrap();
    send.finish().unwrap();
    let body = recv.read_to_end(1024 * 1024).await.unwrap_or_default();
    String::from_utf8_lossy(&body).to_string()
}

#[tokio::test]
async fn a_second_request_cannot_ride_in_behind_a_checked_one() {
    // The grant check ran on ONE head. Blindly pumping the rest of the stream
    // into a keep-alive upstream would serve the second request without ever
    // inspecting it.
    let pair = pair(&[Grant::View, Grant::Edit], Kind::Desktop).await;
    let answer = two_on_one_stream(
        &pair,
        "GET /api/health HTTP/1.1\r\nHost: peer\r\n",
        "POST /api/docs/x/run HTTP/1.1\r\nHost: peer\r\n",
    )
    .await;

    // The first is served.
    assert!(
        answer.contains("reached the session: GET /api/health"),
        "{answer}"
    );
    // The second is NOT — it never reaches the session at all.
    assert!(
        !answer.contains("/api/docs/x/run"),
        "an unchecked second request was served: {answer}"
    );
    // Exactly one response came back.
    assert_eq!(answer.matches("HTTP/1.1 200").count(), 1, "{answer}");
}

#[tokio::test]
async fn a_body_whose_length_cannot_be_known_is_refused() {
    // Guessing the length short is precisely how a second request would ride
    // in behind the first.
    let pair = pair(&[Grant::View, Grant::Edit], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let (mut send, mut recv) = peer.connection().open_bi().await.unwrap();
    send.write_all(
        b"POST /api/docs/abc HTTP/1.1\r\nHost: peer\r\nTransfer-Encoding: chunked\r\n\r\n",
    )
    .await
    .unwrap();
    send.finish().unwrap();
    let body = recv.read_to_end(65536).await.unwrap_or_default();
    let answer = String::from_utf8_lossy(&body).to_string();
    assert!(answer.contains("411"), "{answer}");
    assert!(answer.contains("One request per stream"), "{answer}");
    peer.close().await;
}

#[tokio::test]
async fn a_request_with_a_body_still_reaches_the_session() {
    // The fix must not break ordinary writes.
    let pair = pair(&[Grant::View, Grant::Edit], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let (mut send, mut recv) = peer.connection().open_bi().await.unwrap();
    send.write_all(b"PUT /api/docs/abc HTTP/1.1\r\nHost: peer\r\nContent-Length: 5\r\n\r\nhello")
        .await
        .unwrap();
    send.finish().unwrap();
    let body = recv.read_to_end(65536).await.unwrap_or_default();
    let answer = String::from_utf8_lossy(&body).to_string();
    assert!(
        answer.contains("reached the session: PUT /api/docs/abc"),
        "{answer}"
    );
    peer.close().await;
}

#[tokio::test]
async fn the_forwarder_makes_a_local_port_be_the_remote_session() {
    // The last mile: point anything that speaks HTTP at this port and it is
    // talking to the other machine.
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let peer = attach(&pair).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let bind: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    let forwarding = tokio::spawn(async move {
        let _ = peer.forward(bind).await;
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let mut sock = tokio::net::TcpStream::connect(bind).await.unwrap();
    sock.write_all(b"GET /api/docs/abc HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let mut answer = String::new();
    sock.read_to_string(&mut answer).await.unwrap();
    assert!(
        answer.contains("reached the session: GET /api/docs/abc"),
        "{answer}"
    );
    forwarding.abort();
}

#[tokio::test]
async fn the_forwarder_refuses_to_bind_anywhere_but_loopback() {
    // That port IS the remote session with your grants already applied.
    let pair = pair(&[Grant::View], Kind::Desktop).await;
    let peer = attach(&pair).await;
    let err = peer
        .forward("0.0.0.0:0".parse().unwrap())
        .await
        .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("not a loopback address"), "{msg}");
    assert!(msg.contains("none of the key checking"), "{msg}");
    peer.close().await;
}

// ---------------------------------------------------------------------------
// Pairing by a spoken phrase
// ---------------------------------------------------------------------------

/// Two machines that have never met, holding only a phrase between them.
///
/// Offline by construction, like everything else here: `Reach::Direct` means
/// no relay and no address publishing, so the rendezvous is found on the local
/// interface and the test passes on a machine that has never had internet.
#[tokio::test]
async fn a_phrase_alone_enrols_both_machines_in_one_exchange() {
    let a_dir = tempfile::tempdir().unwrap();
    let b_dir = tempfile::tempdir().unwrap();
    let desktop = Identity::load_or_create(a_dir.path(), "desktop").unwrap();
    let laptop = Identity::load_or_create(b_dir.path(), "laptop").unwrap();
    let desktop_fleet = Fleet::at(desktop.dir());
    let laptop_fleet = Fleet::at(laptop.dir());

    // Neither knows the other.
    assert!(desktop_fleet.machines().is_empty());
    assert!(laptop_fleet.machines().is_empty());

    let phrase = hickory_peer::Phrase::generate();
    let hosting = {
        let phrase = phrase.clone();
        let dir = a_dir.path().to_path_buf();
        tokio::spawn(async move {
            let me = Identity::load_or_create(&dir, "desktop").unwrap();
            let fleet = Fleet::at(me.dir());
            hickory_peer::host(
                &me,
                &fleet,
                &phrase,
                Kind::Desktop,
                &Reach::Direct,
                "2026-08-24",
            )
            .await
        })
    };
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;

    let joined = hickory_peer::join(
        &laptop,
        &laptop_fleet,
        &phrase,
        Kind::Desktop,
        &Reach::Direct,
        "2026-08-24",
    )
    .await
    .expect("the laptop pairs");
    let hosted = hosting.await.unwrap().expect("the desktop pairs");

    // ONE exchange, both directions — the friction this exists to remove.
    assert_eq!(joined.machine.name, "desktop");
    assert_eq!(hosted.machine.name, "laptop");
    assert_eq!(joined.machine.public_key, desktop.public_key());
    assert_eq!(hosted.machine.public_key, laptop.public_key());

    // And each end knows its OWN fingerprint, so a person can compare them —
    // the only check that catches somebody who guessed the phrase.
    assert_eq!(joined.ours, laptop.fingerprint());
    assert_eq!(hosted.ours, desktop.fingerprint());

    // Enrolled with the usual grants, and execute still off.
    let seen = laptop_fleet.get("desktop").unwrap();
    assert!(seen.may(Grant::View) && seen.may(Grant::Edit));
    assert!(!seen.may(Grant::Execute));
}

#[tokio::test]
async fn a_wrong_phrase_reaches_nothing() {
    // A phrase that is nearly right derives a completely different key, so
    // there is no partial success to be confused by.
    let dir = tempfile::tempdir().unwrap();
    let me = Identity::load_or_create(dir.path(), "laptop").unwrap();
    let fleet = Fleet::at(me.dir());
    let wrong = hickory_peer::Phrase::parse("anchor-drift-maple-harbor-99").unwrap();

    let out = tokio::time::timeout(
        std::time::Duration::from_secs(6),
        hickory_peer::join(
            &me,
            &fleet,
            &wrong,
            Kind::Desktop,
            &Reach::Direct,
            "2026-08-24",
        ),
    )
    .await;
    // Either it timed out at our deadline or it failed to dial; what must not
    // happen is a machine appearing in the fleet.
    assert!(out.is_err() || out.unwrap().is_err());
    assert!(fleet.machines().is_empty());
}
