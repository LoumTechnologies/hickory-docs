//! A guest, a relay, and a session on a laptop — all three, for real.
//!
//! Protects docs/guarantees/collaboration/a-guest-reaches-the-host-through-the-relay.md.
//!
//! The relay runs in-process on an ephemeral port, the session dials it exactly
//! as `hickory serve --public` does, and the "guest" is an HTTP client that
//! knows only the public address. Nothing is stubbed between them: the same
//! frames, the same router, the same capability checks. What this proves is the
//! claim the whole relay exists for — that someone outside the host's network
//! can open a document — and the one it must never break, that arriving through
//! the relay does not skip the session's own door.

use std::sync::Arc;
use std::time::Duration;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::share::Scope;
use hickory_cli::serve::{ServeOptions, prepare, tunnel};
use hickory_relay::{Quota, TunnelCensus};
use hickory_relay_server::github::StubIdentifier;
use hickory_relay_server::tunnel::TunnelRegistry;
use hickory_relay_server::{RelayState, router as relay_router};
use tokio::sync::Mutex;

/// Signs the tokens the test relay issues and accepts.
const TOKEN_SECRET: &str = "a-test-secret-that-is-long-enough-to-sign";

/// A token the relay would have issued after a sign-in. The handshake verifies
/// its own signature, so a test does not need to run a sign-in first.
fn account_token(label: &str) -> String {
    hickory_identity::issue_token(TOKEN_SECRET, "acc-test", label, 1).unwrap()
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

struct Relay {
    base: String,
    apex: String,
    identifier: Arc<StubIdentifier>,
    state: RelayState,
}

/// Start a relay on loopback. `apex` is `localhost:<port>`, so a guest address
/// is `<slug>.localhost:<port>` — which is exactly the shape the real one uses,
/// one label in front of the apex.
async fn start_relay(account: &str) -> Relay {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let apex = format!("localhost:{port}");
    let identifier = Arc::new(StubIdentifier::new(account));

    let state = RelayState {
        tunnels: Arc::new(TunnelRegistry::default()),
        census: Arc::new(Mutex::new(TunnelCensus::default())),
        quota: Quota::default(),
        identifier: identifier.clone(),
        accounts: Some(
            hickory_relay_server::accounts::open("sqlite::memory:")
                .await
                .unwrap(),
        ),
        token_secret: TOKEN_SECRET.to_string(),
        github_client_id: None,
        apex: apex.clone(),
        scheme: "http".into(),
    };

    let serving = state.clone();
    tokio::spawn(async move {
        axum::serve(listener, relay_router(serving)).await.unwrap();
    });

    Relay {
        base: format!("http://127.0.0.1:{port}"),
        apex,
        identifier,
        state,
    }
}

struct Session {
    doc_id: String,
    host_token: String,
    guest_token: String,
    _dir: tempfile::TempDir,
}

/// Start a local session and open a tunnel for it on `relay`.
async fn start_session(relay: &Relay, scope: Scope) -> (Session, tunnel::TunnelHandle) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
        port: 0,
        lan: true,
        scope,
        web_dist: Some(root.join("no-such-dist")),
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        public: false,
    })
    .await
    .expect("session prepares");

    // The session's own listener, which the tunnel bridges guest sockets to.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let state = prepared.state.clone();
    let router = prepared.router.clone();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    let handle = tunnel::open(
        &relay.base,
        &account_token("nate@example.com"),
        None,
        router,
        &format!("ws://127.0.0.1:{port}"),
    )
    .await
    .expect("the tunnel opens");

    (
        Session {
            doc_id: state.index.sole().unwrap().0,
            host_token: state.host_token.to_string(),
            guest_token: state.guest_token.to_string(),
            _dir: dir,
        },
        handle,
    )
}

/// A guest request: addressed to the relay's socket, but claiming the tunnel's
/// public hostname — which is what a browser resolving `<slug>.<apex>` does.
async fn guest_get(
    relay: &Relay,
    handle: &tunnel::TunnelHandle,
    path: &str,
    token: &str,
) -> (u16, String) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", relay.base))
        .header("host", format!("{}.{}", handle.slug, relay.apex))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.text().await.unwrap_or_default())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_guest_outside_the_network_reaches_the_document_through_the_relay() {
    let relay = start_relay("nate").await;
    let (session, handle) = start_session(&relay, Scope::Edit).await;

    // The address the host would send someone.
    assert!(
        handle.url.starts_with(&format!("http://{}.", handle.slug)),
        "{}",
        handle.url
    );
    assert_eq!(handle.account, "nate@example.com");

    // The document itself, fetched by someone who knows only the public URL.
    let (status, body) = guest_get(
        &relay,
        &handle,
        &format!("/api/docs/{}", session.doc_id),
        &session.guest_token,
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("hick:copy"), "{body}");

    // And the lineage the ribbons draw — the thing worth travelling for.
    let (status, body) = guest_get(
        &relay,
        &handle,
        &format!("/api/docs/{}/outputs/file?path=greet.rs", session.doc_id),
        &session.guest_token,
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("\"kind\":\"paste\""), "{body}");

    // GitHub is never called during a session: the handshake verifies a token
    // the relay signed at sign-in. A relay that phoned a third party on every
    // page load would be rate limited into uselessness — and would stop
    // working whenever GitHub did.
    assert_eq!(relay.identifier.calls(), 0);
}

/// Guarantee: arriving through the relay is not a way around the session's own
/// door. The relay forwards; it does not vouch for anyone.
#[tokio::test(flavor = "multi_thread")]
async fn the_relay_does_not_admit_anyone_the_session_would_refuse() {
    let relay = start_relay("nate").await;
    let (session, handle) = start_session(&relay, Scope::Read).await;

    // No capability token at all.
    let (status, _) = guest_get(
        &relay,
        &handle,
        &format!("/api/docs/{}", session.doc_id),
        "not-the-token",
    )
    .await;
    assert_eq!(status, 403, "the session's own check must still run");

    // A read-only link still cannot write, even from the far side of a relay.
    let resp = reqwest::Client::new()
        .put(format!("{}/api/docs/{}", relay.base, session.doc_id))
        .header("host", format!("{}.{}", handle.slug, relay.apex))
        .bearer_auth(&session.guest_token)
        .json(&serde_json::json!({ "source": "replaced" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 403);

    // The host's own token works through the tunnel, which is what makes the
    // public address usable by the person who opened it.
    let (status, _) = guest_get(
        &relay,
        &handle,
        &format!("/api/docs/{}", session.doc_id),
        &session.host_token,
    )
    .await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_address_with_no_session_behind_it_says_so_plainly() {
    let relay = start_relay("nate").await;

    // A slug that never existed: someone following a stale link.
    let resp = reqwest::Client::new()
        .get(format!("{}/", relay.base))
        .header("host", format!("gone-away-1.{}", relay.apex))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    let body = resp.text().await.unwrap();
    assert!(body.contains("No session is running"), "{body}");
    assert!(body.contains("hickory serve"), "{body}");

    // The apex itself is not a product surface.
    let resp = reqwest::Client::new()
        .get(format!("{}/", relay.base))
        .header("host", relay.apex.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    assert!(resp.text().await.unwrap().contains("relay"));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_quota_stops_an_account_opening_tunnels_without_end() {
    let relay = start_relay("nate").await;
    let quota = relay.state.quota.max_tunnels_per_account;

    let mut held = Vec::new();
    for _ in 0..quota {
        let (_, handle) = start_session(&relay, Scope::Edit).await;
        held.push(handle);
    }

    // One more than the limit, from the same account.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let prepared = prepare(ServeOptions {
        target: dir.path().canonicalize().unwrap().join("demo.hick"),
        port: 0,
        lan: true,
        scope: Scope::Edit,
        web_dist: Some(dir.path().join("no-such-dist")),
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        public: false,
    })
    .await
    .unwrap();

    let err = tunnel::open(
        &relay.base,
        &account_token("nate@example.com"),
        None,
        prepared.router,
        "ws://127.0.0.1:1",
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("the quota must refuse the next tunnel"));
    let message = format!("{err:#}");
    assert!(message.contains("limit is"), "{message}");
    // The refusal has to tell the host what to do about it.
    assert!(message.contains("Close one"), "{message}");
}

/// A token this relay did not sign — expired, forged, or issued by a different
/// relay — cannot open a tunnel.
#[tokio::test(flavor = "multi_thread")]
async fn a_token_this_relay_did_not_sign_opens_nothing() {
    let relay = start_relay("nate").await;

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let prepared = prepare(ServeOptions {
        target: dir.path().canonicalize().unwrap().join("demo.hick"),
        port: 0,
        lan: true,
        scope: Scope::Edit,
        web_dist: Some(dir.path().join("no-such-dist")),
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        public: false,
    })
    .await
    .unwrap();

    let err = tunnel::open(
        &relay.base,
        "expired",
        None,
        prepared.router,
        "ws://127.0.0.1:1",
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("an unusable token cannot open a tunnel"));
    let message = format!("{err:#}");
    assert!(message.contains("hickory login"), "{message}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_closed_session_takes_its_address_with_it() {
    let relay = start_relay("nate").await;
    let (session, handle) = {
        let (session, handle) = start_session(&relay, Scope::Edit).await;
        // Prove it works before proving it stops.
        let (status, _) = guest_get(
            &relay,
            &handle,
            &format!("/api/docs/{}", session.doc_id),
            &session.guest_token,
        )
        .await;
        assert_eq!(status, 200);
        (session, handle)
    };

    // Drop the tunnel by closing the relay's view of it — the same thing that
    // happens when a laptop lid closes.
    relay.state.tunnels.remove_for_test(&handle.slug).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (status, body) = guest_get(
        &relay,
        &handle,
        &format!("/api/docs/{}", session.doc_id),
        &session.guest_token,
    )
    .await;
    assert_eq!(status, 404, "{body}");
    assert!(body.contains("No session is running"), "{body}");
}

/// The document room, through the relay.
///
/// This is the one that matters most and the one most likely to break: an HTTP
/// request survives a naive proxy, but a WebSocket carrying binary CRDT updates
/// in both directions does not. If this passes, two people in different
/// countries can edit the same file on someone's laptop.
#[tokio::test(flavor = "multi_thread")]
async fn the_document_room_works_through_the_relay() {
    use futures::{SinkExt as _, StreamExt as _};
    use tokio_tungstenite::tungstenite::Message as TtMessage;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
    use yrs::updates::decoder::Decode as _;
    use yrs::updates::encoder::Encode as _;
    use yrs::{GetString as _, ReadTxn as _, Transact as _};

    let relay = start_relay("nate").await;
    let (session, handle) = start_session(&relay, Scope::Edit).await;

    // A guest's browser would resolve <slug>.<apex> to the relay's address.
    // Here the address is loopback and the Host header carries the name.
    let url = format!(
        "ws://{}/api/ws?doc=doc:{}&token={}",
        relay.base.trim_start_matches("http://"),
        session.doc_id,
        session.guest_token
    );
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "host",
        format!("{}.{}", handle.slug, relay.apex).parse().unwrap(),
    );

    let (mut socket, response) = tokio_tungstenite::connect_async(request)
        .await
        .expect("the room's socket survives the tunnel");
    assert_eq!(response.status().as_u16(), 101);

    // Ask for the document, exactly as the client does.
    let doc = yrs::Doc::with_options(yrs::Options {
        offset_kind: yrs::OffsetKind::Utf16,
        ..yrs::Options::default()
    });
    let sv = doc.transact().state_vector();
    let mut frame = vec![0x00];
    frame.extend_from_slice(
        &yrs::sync::Message::Sync(yrs::sync::SyncMessage::SyncStep1(sv)).encode_v1(),
    );
    socket.send(TtMessage::Binary(frame)).await.unwrap();

    // The reply comes back down the tunnel as binary, byte for byte.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut got_document = false;
    while tokio::time::Instant::now() < deadline && !got_document {
        let Ok(Some(Ok(message))) =
            tokio::time::timeout(Duration::from_secs(5), socket.next()).await
        else {
            break;
        };
        let TtMessage::Binary(bytes) = message else {
            continue;
        };
        if bytes.first() != Some(&0x00) {
            continue;
        }
        let mut decoder =
            yrs::updates::decoder::DecoderV1::new(yrs::encoding::read::Cursor::new(&bytes[1..]));
        for message in yrs::sync::MessageReader::new(&mut decoder).flatten() {
            if let yrs::sync::Message::Sync(
                yrs::sync::SyncMessage::SyncStep2(update) | yrs::sync::SyncMessage::Update(update),
            ) = message
                && let Ok(update) = yrs::Update::decode_v1(&update)
            {
                let mut txn = doc.transact_mut();
                let _ = txn.apply_update(update);
                got_document = true;
            }
        }
    }

    assert!(
        got_document,
        "the room never sent the document through the tunnel"
    );
    let text = {
        let text = doc.get_or_insert_text("source");
        let txn = doc.transact();
        text.get_string(&txn)
    };
    assert!(
        text.contains("println!(\"hello\")"),
        "the document arrived corrupted: {text}"
    );
}
