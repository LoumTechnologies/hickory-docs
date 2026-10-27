<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Prove the phone talks to an authorized desktop

For maintainers checking the mobile transport boundary. These focused integration tests use real Iroh connections and an HTTP/WebSocket stand-in on loopback, with direct-only reachability. They do not use a public relay or claim device acceptance.

## Paired loopback fixture

Enroll the phone mutually and start a desktop peer server with execution initially off. The HTTP stand-in counts runs, while the room echoes bytes so the test can distinguish an accepted upgrade from a denied edit grant.

<mobile:copy id="mobile-peer-witnesses-paired-loopback-fixture">
use axum::{
    Json, Router,
    extract::ws::WebSocketUpgrade,
    routing::{get, post},
};
use hickory_fleet::{Fleet, Grant, Identity, Kind};
use hickory_mobile_core::peer::{Peer, RoomEvent};
use hickory_peer::{PeerServer, Reach};
use serde_json::json;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[tokio::test]
async fn phone_reads_joins_live_rooms_and_runs_only_with_desktop_grants() {
    let host_dir = tempfile::tempdir().unwrap();
    let phone_dir = tempfile::tempdir().unwrap();
    let host = Identity::load_or_create(host_dir.path(), "desktop").unwrap();
    let phone = Identity::load_or_create(phone_dir.path(), "phone").unwrap();
    let host_fleet = Fleet::at(host.dir());
    host_fleet.add(&phone.invitation(Kind::Phone), "").unwrap();
    host_fleet.set_grant("phone", Grant::Edit, false).unwrap();
    let phone_fleet = Fleet::at(phone.dir());
    phone_fleet
        .add(&host.invitation(Kind::Desktop), "")
        .unwrap();
    let runs = Arc::new(AtomicUsize::new(0));
    let count = runs.clone();
    let app = Router::new()
        .route(
            "/api/docs/a",
            get(|| async { Json(json!({"id":"a", "source":"desktop source"})) }),
        )
        .route(
            "/api/docs/a/run",
            post(move || {
                let count = count.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Json(json!({"run_id":"one"}))
                }
            }),
        )
        .route(
            "/api/ws",
            get(|ws: WebSocketUpgrade| async {
                ws.on_upgrade(|mut socket| async move {
                    while let Some(Ok(message)) = socket.recv().await {
                        if socket.send(message).await.is_err() {
                            break;
                        }
                    }
                })
            }),
        );
    let local = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = local.local_addr().unwrap();
    let upstream = tokio::spawn(async move {
        axum::serve(local, app).await.unwrap();
    });
    let server = Arc::new(
        PeerServer::bind(&host, host_fleet, addr, Reach::Direct)
            .await
            .unwrap(),
    );
    let serving = server.clone();
    let accepting = tokio::spawn(async move {
        serving.serve().await.unwrap();
    });
    let peer = Arc::new(
        Peer::connect(
            &phone,
            &phone_fleet,
            "desktop",
            server.addr(),
            Reach::Direct,
        )
        .await
        .unwrap(),
    );
</mobile:copy>

## Grant and frame assertions

The read succeeds, the initial room and run are refused, then explicit grants admit exactly those operations. Revocation and re-enrollment do not restore execute. A mismatched endpoint key fails before dialing. The accepted room carries arbitrary binary bytes through the real upgrade path.

<mobile:copy id="mobile-peer-witnesses-grant-and-frame-assertions">
    assert_eq!(
        peer.request("GET", "/api/docs/a", json!(null))
            .await
            .unwrap()["source"],
        "desktop source"
    );
    assert!(peer.room("a", |_| {}).await.is_err());
    assert!(
        peer.request("POST", "/api/docs/a/run", json!({}))
            .await
            .is_err()
    );
    assert_eq!(runs.load(Ordering::SeqCst), 0);
    assert!(
        peer.request("GET", "/api/settings", json!(null))
            .await
            .is_err()
    );
    let grants = Fleet::at(host.dir());
    grants.set_grant("phone", Grant::Edit, true).unwrap();
    let (emit, mut events) = tokio::sync::mpsc::unbounded_channel();
    let room = peer
        .room("a", move |event| {
            let _ = emit.send(event);
        })
        .await
        .unwrap();
    room.send(vec![0, 1, 2, 255]).unwrap();
    match tokio::time::timeout(Duration::from_secs(5), events.recv())
        .await
        .unwrap()
        .unwrap()
    {
        RoomEvent::Frame { bytes } => assert_eq!(bytes, vec![0, 1, 2, 255]),
        RoomEvent::Closed { reason } => panic!("room unexpectedly closed: {reason}"),
    }
    grants.set_grant("phone", Grant::Execute, true).unwrap();
    assert_eq!(
        peer.request("POST", "/api/docs/a/run", json!({}))
            .await
            .unwrap()["run_id"],
        "one"
    );
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    grants.set_grant("phone", Grant::Execute, false).unwrap();
    grants.add(&phone.invitation(Kind::Phone), "").unwrap();
    assert!(
        peer.request("POST", "/api/docs/a/run", json!({}))
            .await
            .is_err()
    );
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    let mut wrong = server.addr();
    wrong.id = iroh::SecretKey::from_bytes(&[42; 32]).public();
    assert!(
        Peer::connect(&phone, &phone_fleet, "desktop", wrong, Reach::Direct)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("different desktop key")
    );
    drop(room);
    peer.close().await;
    server.endpoint().close().await;
    accepting.abort();
    upstream.abort();
}
</mobile:copy>

Assemble `apps/mobile/core/tests/peer.rs` in the order explained above.

<mobile:file path="apps/mobile/core/tests/peer.rs"><mobile:paste select="#mobile-peer-witnesses-paired-loopback-fixture"/><mobile:paste select="#mobile-peer-witnesses-grant-and-frame-assertions"/></mobile:file>

</mobile:doc>
