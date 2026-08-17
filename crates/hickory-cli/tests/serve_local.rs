//! The local document server, over a real socket.
//!
//! Protects docs/guarantees/collaboration/local-session-writes-your-files.md.
//!
//! These drive the session the way the desktop app's window does — the same
//! Yjs protocol frames, the same REST calls — because the claim worth testing
//! is "an edit made in the app's editor reaches the file on disk", and that is
//! not observable from inside a unit test of a handler.
//!
//! There is nothing here about tokens, scopes, or share links: this server
//! binds loopback and answers one person. See
//! `docs/specs/freeform/local-only.md`.

use std::path::PathBuf;
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
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
    doc_id: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

impl Session {
    fn doc_path(&self) -> PathBuf {
        self.root.join("demo.hick")
    }
    fn source_on_disk(&self) -> String {
        std::fs::read_to_string(self.doc_path()).unwrap()
    }
}

async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
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
        doc_id,
        root,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(session: &Session) -> Ws {
    let url = format!(
        "ws://{}/api/ws?doc=doc:{}",
        session.base.trim_start_matches("http://"),
        session.doc_id
    );
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("ws connects");
    ws
}

/// The LSP bridge's channel byte, as `docs/specs/freeform/api.md` fixes it.
const CHANNEL_LSP: u8 = 0x02;

/// Send one JSON-RPC message on the language channel, framed the way the
/// notebook frames it: one message per binary frame, no Content-Length.
async fn send_lsp(ws: &mut Ws, message: Value) {
    let mut frame = vec![CHANNEL_LSP];
    frame.extend_from_slice(&serde_json::to_vec(&message).unwrap());
    ws.send(TtMessage::Binary(frame)).await.unwrap();
}

/// Wait for the first `0x02` frame whose `method` matches, ignoring the Yjs
/// handshake and the server's own log chatter.
async fn next_lsp(ws: &mut Ws, method: &str) -> Value {
    tokio::time::timeout(Duration::from_secs(20), async {
        while let Some(Ok(msg)) = ws.next().await {
            let TtMessage::Binary(data) = msg else {
                continue;
            };
            if data.first() != Some(&CHANNEL_LSP) {
                continue;
            }
            let message: Value = serde_json::from_slice(&data[1..]).unwrap();
            if message["method"] == method {
                return message;
            }
        }
        panic!("the socket closed before {method} arrived");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {method}"))
}

/// The notebook asks language questions over the same socket that carries its
/// edits, and gets answers in document coordinates.
///
/// Protects docs/guarantees/editor-intelligence/the-notebook-asks-the-same-questions-an-editor-does.md
#[tokio::test(flavor = "multi_thread")]
async fn the_language_channel_answers_in_document_coordinates() {
    let session = start().await;
    let mut ws = connect(&session).await;

    send_lsp(
        &mut ws,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "hick:///demo.hick",
                "languageId": "hick",
                "version": 1,
                "text": DOC,
            }},
        }),
    )
    .await;

    let published = next_lsp(&mut ws, "textDocument/publishDiagnostics").await;
    // The browser never learns where this project lives on disk.
    assert_eq!(published["params"]["uri"], "hick:///demo.hick");
    assert!(
        published["params"]["diagnostics"].is_array(),
        "a publish always carries a list, even an empty one: {published}"
    );

    // A question the hick layer answers by itself, so this holds on a machine
    // with no language servers installed at all.
    send_lsp(
        &mut ws,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "hick:///demo.hick" },
                // The `<hick:paste select="#greet" />` line.
                "position": { "line": 6, "character": 30 },
            },
        }),
    )
    .await;
    let hover = tokio::time::timeout(Duration::from_secs(20), async {
        while let Some(Ok(msg)) = ws.next().await {
            let TtMessage::Binary(data) = msg else {
                continue;
            };
            if data.first() != Some(&CHANNEL_LSP) {
                continue;
            }
            let message: Value = serde_json::from_slice(&data[1..]).unwrap();
            if message["id"] == 1 {
                return message;
            }
        }
        panic!("the socket closed before the hover answer arrived");
    })
    .await
    .expect("hover should be answered");
    assert!(
        hover.get("result").is_some() || hover.get("error").is_some(),
        "a request must get a reply of some kind: {hover}"
    );
}

/// Structural navigation is served from the same session, computed by
/// tree-sitter rather than a language server.
///
/// Protects docs/guarantees/editor-intelligence/structural-navigation-ships-in-the-download.md
#[tokio::test(flavor = "multi_thread")]
async fn structure_names_definitions_and_links_references_to_them() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("code.hick"),
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="code.md">
<hick:file path="app.py">
def load(path):
    return open(path).read()

def summarise(path):
    return len(load(path))
</hick:file>
</hick:doc>
"##,
    )
    .unwrap();
    let root = dir.path().canonicalize().unwrap();

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    let body: Value = reqwest::get(format!("http://127.0.0.1:{port}/api/structure"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let files = body["files"].as_array().expect("files");
    let app = files
        .iter()
        .find(|f| f["path"] == "app.py")
        .expect("the generated python file was analysed");
    let names: Vec<&str> = app["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"load"), "{names:?}");
    assert!(names.contains(&"summarise"), "{names:?}");

    let links = body["links"].as_array().expect("links");
    let call = links
        .iter()
        .find(|l| l["name"] == "load")
        .expect("the call to load resolves to its definition");
    assert_eq!(call["to_path"], "app.py");
    // One definition of that name, so the link is not a guess among several.
    assert_eq!(call["candidates"], 1);
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
async fn an_edit_in_one_editor_reaches_the_other_and_the_file_on_disk() {
    let session = start().await;

    // Two editors on one document — the app's window and another client of
    // the same local server. One machine, one person, two writers.
    let mut alice = connect(&session).await;
    let mut bob = connect(&session).await;

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
async fn the_ribbons_have_their_data_without_a_database() {
    // The reason this mode exists: byte-precise lineage for a file on disk.
    let session = start().await;

    let (status, outputs) = get(&session, &format!("/api/docs/{}/outputs", session.doc_id)).await;
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

/// Editing PROSE in the woven markdown lands in the document, exactly like
/// editing a generated code file does. This is the desktop app's path — the
/// output pane loads `/outputs/file` and POSTs `/outputs/edit` — for the one
/// output every literate document has: its weave.
///
/// Protects docs/guarantees/authoring/an-output-edit-lands-in-its-document.md
/// (the serve-side half; the `hick up` half lives in up_loop.rs) and
/// docs/guarantees/execution/output-lineage-round-trips-byte-for-byte.md.
#[tokio::test(flavor = "multi_thread")]
async fn a_prose_edit_in_the_woven_markdown_lands_in_the_document() {
    let session = start().await;

    // The weave file is one of the document's outputs, listed like any other.
    let (status, outputs) = get(&session, &format!("/api/docs/{}/outputs", session.doc_id)).await;
    assert_eq!(status, 200, "{outputs}");
    let paths: Vec<&str> = outputs["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"demo.md"), "{paths:?}");

    // Its prose carries real (non-synthetic) provenance back to the document.
    let (status, file) = get(
        &session,
        &format!("/api/docs/{}/outputs/file?path=demo.md", session.doc_id),
    )
    .await;
    assert_eq!(status, 200, "{file}");
    let content = file["content"].as_str().unwrap();
    let at = content.find("# Demo").expect("the weave carries the prose");
    let covering = file["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| {
            p["start"].as_u64().unwrap() <= at as u64 && (at as u64) < p["end"].as_u64().unwrap()
        })
        .unwrap_or_else(|| panic!("no provenance covers the prose: {file}"));
    assert_ne!(
        covering["origin"]["kind"], "synthetic",
        "prose in the weave must map back to the document: {covering}"
    );

    // Edit the prose through the same endpoint the app's output pane uses.
    let (status, edited) = post(
        &session,
        &format!("/api/docs/{}/outputs/edit", session.doc_id),
        serde_json::json!({
            "path": "demo.md",
            "edits": [{ "start": at + 2, "end": at + 6, "text": "Demonstration" }]
        }),
    )
    .await;
    assert_eq!(status, 200, "{edited}");
    assert_eq!(edited["applied"], true);
    assert!(
        session.source_on_disk().contains("# Demonstration"),
        "the prose edit did not reach the document:\n{}",
        session.source_on_disk()
    );

    // A re-weave reproduces the edit: the round trip is byte-stable.
    let (status, rewoven) = get(
        &session,
        &format!("/api/docs/{}/outputs/file?path=demo.md", session.doc_id),
    )
    .await;
    assert_eq!(status, 200, "{rewoven}");
    assert!(
        rewoven["content"]
            .as_str()
            .unwrap()
            .contains("# Demonstration"),
        "the re-weave lost the edit: {rewoven}"
    );
}

/// A folder with no documents is the app's FIRST-RUN state, not an error:
/// the default workspace starts empty and the UI lands on a fresh untitled
/// document. The session starts with an empty index and fills as documents
/// are created.
#[tokio::test(flavor = "multi_thread")]
async fn a_session_with_no_documents_starts_empty() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.md"), "not a document").unwrap();
    let prepared = prepare(ServeOptions {
        target: dir.path().to_path_buf(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("an empty directory is a session waiting for its first document");
    assert!(prepared.state.index.entries().is_empty());
}
