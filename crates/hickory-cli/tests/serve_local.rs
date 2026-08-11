//! `hickory serve`: the collaboration guarantees, over a real socket.
//!
//! Protects docs/guarantees/collaboration/local-session-writes-your-files.md
//! and docs/guarantees/collaboration/shared-runs-require-a-sandbox.md.
//!
//! These drive the session the way a browser does — the same Yjs protocol
//! frames, the same REST calls, the same token in the same places — because
//! the claims worth testing are "a collaborator's keystroke reaches the host's
//! file" and "a link that should not run code cannot", and neither is
//! observable from inside a unit test of a handler.

use std::path::PathBuf;
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::share::Scope;
use hickory_cli::serve::{LocalState, ServeOptions, prepare};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message as TtMessage;
use yrs::sync::{Message, SyncMessage};
use yrs::updates::decoder::Decode as _;
use yrs::updates::encoder::Encode as _;
use yrs::{GetString as _, ReadTxn as _, Text as _, Transact as _, Update};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

struct Session {
    base: String,
    state: LocalState,
    doc_id: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

impl Session {
    fn host_token(&self) -> String {
        self.state.host_token.to_string()
    }
    fn guest_token(&self) -> String {
        self.state.guest_token.to_string()
    }
    fn doc_path(&self) -> PathBuf {
        self.root.join("demo.hick")
    }
    fn source_on_disk(&self) -> String {
        std::fs::read_to_string(self.doc_path()).unwrap()
    }
}

async fn start(scope: Scope, lan: bool) -> Session {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
        port: 0,
        lan,
        scope,
        // No web client in a test; the API is what is under test.
        web_dist: Some(root.join("no-such-dist")),
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        public: false,
    })
    .await
    .expect("session prepares");

    let state = prepared.state.clone();
    let doc_id = state.index.sole().expect("one document").0;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        state,
        doc_id,
        root,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str, token: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn post(session: &Session, path: &str, token: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(session: &Session, token: &str) -> Ws {
    let url = format!(
        "ws://{}/api/ws?doc=doc:{}&token={token}",
        session.base.trim_start_matches("http://"),
        session.doc_id
    );
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("ws connects");
    ws
}

/// Yjs frames are channel 0x00; run events are 0x01.
fn yjs_frame(msg: &Message) -> Vec<u8> {
    let mut frame = vec![0x00];
    frame.extend_from_slice(&msg.encode_v1());
    frame
}

/// Drain frames until one carries a Yjs sync update, applying each to `doc`.
/// Returns false on timeout.
async fn absorb_updates(ws: &mut Ws, doc: &yrs::Doc, deadline: Duration) -> bool {
    let mut saw_update = false;
    let _ = tokio::time::timeout(deadline, async {
        while let Some(Ok(msg)) = ws.next().await {
            let TtMessage::Binary(bytes) = msg else {
                continue;
            };
            if bytes.first() != Some(&0x00) {
                continue;
            }
            let mut decoder = yrs::updates::decoder::DecoderV1::new(
                yrs::encoding::read::Cursor::new(&bytes[1..]),
            );
            let reader = yrs::sync::MessageReader::new(&mut decoder);
            for message in reader.flatten() {
                match message {
                    Message::Sync(SyncMessage::SyncStep2(update))
                    | Message::Sync(SyncMessage::Update(update)) => {
                        if let Ok(update) = Update::decode_v1(&update) {
                            let mut txn = doc.transact_mut();
                            let _ = txn.apply_update(update);
                            saw_update = true;
                        }
                    }
                    _ => {}
                }
            }
            if saw_update {
                return;
            }
        }
    })
    .await;
    saw_update
}

fn text_of(doc: &yrs::Doc) -> String {
    let text = doc.get_or_insert_text("source");
    let txn = doc.transact();
    text.get_string(&txn)
}

/// A client Y.Doc in the same shape the browser builds (UTF-16 offsets).
fn client_doc() -> yrs::Doc {
    yrs::Doc::with_options(yrs::Options {
        offset_kind: yrs::OffsetKind::Utf16,
        ..yrs::Options::default()
    })
}

// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn a_collaborators_edit_reaches_the_other_client_and_the_hosts_file() {
    let session = start(Scope::Edit, true).await;

    // Two people on the same document: the host, and someone holding the link.
    let mut alice = connect(&session, &session.host_token()).await;
    let mut bob = connect(&session, &session.guest_token()).await;

    let alice_doc = client_doc();
    let bob_doc = client_doc();

    // Ask for the server's state, and absorb it.
    let sv = alice_doc.transact().state_vector();
    alice
        .send(TtMessage::Binary(yjs_frame(&Message::Sync(
            SyncMessage::SyncStep1(sv),
        ))))
        .await
        .unwrap();
    assert!(
        absorb_updates(&mut alice, &alice_doc, Duration::from_secs(5)).await,
        "the server must send the document to a client that asks"
    );
    assert!(text_of(&alice_doc).contains("println!(\"hello\")"));

    let sv = bob_doc.transact().state_vector();
    bob.send(TtMessage::Binary(yjs_frame(&Message::Sync(
        SyncMessage::SyncStep1(sv),
    ))))
    .await
    .unwrap();
    assert!(absorb_updates(&mut bob, &bob_doc, Duration::from_secs(5)).await);

    // Alice edits. The update goes out exactly as the browser sends it.
    // Compute the offset BEFORE opening the write transaction: yrs takes an
    // internal lock per document, so reading inside a `transact_mut` block
    // deadlocks the test against itself.
    let at = text_of(&alice_doc)
        .find("hello")
        .expect("the document has text");
    let update = {
        let text = alice_doc.get_or_insert_text("source");
        let mut txn = alice_doc.transact_mut();
        let before = txn.state_vector();
        // The document is ASCII here, so byte and UTF-16 offsets coincide.
        text.insert(&mut txn, at as u32, "collaborative ");
        txn.encode_diff_v1(&before)
    };
    alice
        .send(TtMessage::Binary(yjs_frame(&Message::Sync(
            SyncMessage::Update(update),
        ))))
        .await
        .unwrap();

    // Bob sees it — that is collaboration.
    assert!(
        absorb_updates(&mut bob, &bob_doc, Duration::from_secs(5)).await,
        "the other client never received the edit"
    );
    assert!(
        text_of(&bob_doc).contains("collaborative hello"),
        "bob has: {}",
        text_of(&bob_doc)
    );

    // …and the host's FILE has it, after the persist debounce. This is the
    // claim that separates this from a chat room: the durable state is the
    // working tree, not a server.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if session.source_on_disk().contains("collaborative hello") {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the edit never reached the file on disk:\n{}",
            session.source_on_disk()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_read_only_link_can_watch_but_not_write() {
    let session = start(Scope::Read, true).await;
    let mut watcher = connect(&session, &session.guest_token()).await;
    let doc = client_doc();

    let sv = doc.transact().state_vector();
    watcher
        .send(TtMessage::Binary(yjs_frame(&Message::Sync(
            SyncMessage::SyncStep1(sv),
        ))))
        .await
        .unwrap();
    assert!(
        absorb_updates(&mut watcher, &doc, Duration::from_secs(5)).await,
        "a read-only link must still be able to READ"
    );

    // Try to write anyway.
    let update = {
        let text = doc.get_or_insert_text("source");
        let mut txn = doc.transact_mut();
        let before = txn.state_vector();
        text.insert(&mut txn, 0, "SNEAKY");
        txn.encode_diff_v1(&before)
    };
    watcher
        .send(TtMessage::Binary(yjs_frame(&Message::Sync(
            SyncMessage::Update(update),
        ))))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(
        !session.source_on_disk().contains("SNEAKY"),
        "a read-only link wrote to the document:\n{}",
        session.source_on_disk()
    );

    // The REST write path refuses too, and says what to ask for.
    let (status, body) = reqwest::Client::new()
        .put(format!("{}/api/docs/{}", session.base, session.doc_id))
        .bearer_auth(session.guest_token())
        .json(&serde_json::json!({ "source": "replaced" }))
        .send()
        .await
        .map(|r| (r.status().as_u16(), r))
        .unwrap();
    assert_eq!(status, 403);
    let text = body.text().await.unwrap();
    assert!(text.contains("read-only"), "{text}");
}

/// Guarantee: docs/guarantees/collaboration/shared-runs-require-a-sandbox.md
#[tokio::test(flavor = "multi_thread")]
async fn a_guest_cannot_run_unless_the_link_says_so_and_the_host_always_can() {
    let session = start(Scope::Edit, true).await;

    // The guest may edit, so they may not run: executing is a separate grant
    // because it is code on someone else's machine.
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/run", session.doc_id),
        &session.guest_token(),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    let msg = body["error"].as_str().unwrap();
    assert!(msg.contains("executes code on the machine"), "{msg}");
    assert!(msg.contains("--scope run"), "{msg}");

    // The host is not a guest: it is their machine, and their session.
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/run", session.doc_id),
        &session.host_token(),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status, 202,
        "the host must be able to run their own document: {body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_api_is_closed_to_anyone_without_the_link() {
    let session = start(Scope::Edit, true).await;

    for token in ["", "not-the-token"] {
        let (status, _) = get(&session, "/api/me", token).await;
        assert_eq!(status, 403, "token {token:?} must not be admitted");
    }
    // A socket with no valid token is refused before it can join a room.
    let url = format!(
        "ws://{}/api/ws?doc=doc:{}&token=nope",
        session.base.trim_start_matches("http://"),
        session.doc_id
    );
    assert!(
        tokio_tungstenite::connect_async(url).await.is_err(),
        "an unauthenticated socket must not be upgraded"
    );

    let (status, _) = get(&session, "/api/me", &session.host_token()).await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_ribbons_have_their_data_without_a_database() {
    // The reason this mode exists: byte-precise lineage for a file on disk.
    let session = start(Scope::Edit, false).await;
    let token = session.host_token();

    let (status, outputs) = get(
        &session,
        &format!("/api/docs/{}/outputs", session.doc_id),
        &token,
    )
    .await;
    assert_eq!(status, 200, "{outputs}");
    let paths: Vec<&str> = outputs["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"greet.rs"), "{paths:?}");

    let (status, file) = get(
        &session,
        &format!("/api/docs/{}/outputs/file?path=greet.rs", session.doc_id),
        &token,
    )
    .await;
    assert_eq!(status, 200, "{file}");
    assert_eq!(file["language"], "rust");
    let provenance = file["provenance"].as_array().unwrap();
    assert!(!provenance.is_empty(), "no lineage: {file}");
    assert_eq!(provenance[0]["origin"]["kind"], "paste");

    // Editing through the generated file lands in the DOCUMENT, byte-exactly.
    let content = file["content"].as_str().unwrap();
    let at = content.find("hello").unwrap();
    let (status, edited) = post(
        &session,
        &format!("/api/docs/{}/outputs/edit", session.doc_id),
        &token,
        serde_json::json!({
            "path": "greet.rs",
            "edits": [{ "start": at, "end": at + 5, "text": "hello, world" }]
        }),
    )
    .await;
    assert_eq!(status, 200, "{edited}");
    assert_eq!(edited["applied"], true);
    assert!(
        session
            .source_on_disk()
            .contains(r#"println!("hello, world")"#),
        "the edit did not reach the document:\n{}",
        session.source_on_disk()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_session_with_nothing_to_serve_says_so() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.md"), "not a document").unwrap();
    let err = prepare(ServeOptions {
        target: dir.path().to_path_buf(),
        port: 0,
        lan: false,
        scope: Scope::Edit,
        web_dist: None,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        public: false,
    })
    .await
    .err()
    .expect("an empty directory has nothing to serve");
    assert!(
        err.to_string().contains("no .hick documents to serve"),
        "{err}"
    );
}

/// The guard is enforced where the session is built, not merely where the flag
/// is parsed — a caller that skips the CLI must not be able to skip it.
#[tokio::test(flavor = "multi_thread")]
async fn preparing_a_shared_runnable_session_on_the_local_executor_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let err = prepare(ServeOptions {
        target: dir.path().to_path_buf(),
        port: 0,
        lan: true,
        scope: Scope::Run,
        web_dist: None,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        public: false,
    })
    .await
    .err()
    .expect("sharing + run + local executor must refuse");
    assert!(err.to_string().contains("HICKORY_EXECUTOR=docker"), "{err}");
}
