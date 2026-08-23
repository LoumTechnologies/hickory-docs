//! Language intelligence, all the way to the desktop app's window.
//!
//! Protects docs/guarantees/editor-intelligence/the-meta-lsp-forwards-what-the-child-supports.md
//! Protects docs/guarantees/editor-intelligence/lsp-channel-degrades-never-errors.md
//!
//! `lsp_languages.rs` drives the `hick-lsp` binary the way an editor does.
//! This drives the OTHER path, the one the product's own UI uses: the local
//! server on a real port, the same WebSocket the app's window opens, and the
//! same `0x02` frames the notebook's LSP client sends. Nothing here imports
//! the front-end, so the answer holds whatever the UI is written in.
//!
//! The two paths are worth testing separately because the bridge does work
//! the editor path never exercises: it performs `initialize` on the browser's
//! behalf, rewrites every URI between `hick:///` and the filesystem, and
//! announces the server's capabilities as a notification — the browser never
//! sends `initialize`, so that announcement is the only way it can learn the
//! semantic-token legend.

use std::path::PathBuf;
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use hickory_collab::CHANNEL_LSP;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as TtMessage;

/// A Python document, because Python's server is the one most likely to be
/// installed and the fixture stays readable.
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# A document the desktop app opens

<hick:file path="app.py">
def summarise(path):
    return len(path)


def main():
    return summarise("x")
</hick:file>
</hick:doc>
"##;

/// 0-based document line of `def summarise`, counted from DOC above.
const DEFINITION_LINE: u32 = 5;
/// 0-based document line of the call inside `main`.
const USE_LINE: u32 = 10;

struct App {
    base: String,
    doc_id: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

async fn open_app() -> App {
    let dir = tempfile::tempdir().expect("a temp project");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(
        root.join("pyproject.toml"),
        "[project]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(root.join("doc.hick"), DOC).unwrap();

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("the session prepares");

    let doc_id = prepared.state.index.sole().expect("one document").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    App {
        base: format!("ws://127.0.0.1:{port}"),
        doc_id,
        root,
        _dir: dir,
    }
}

/// Is a Python server installed here? If not, this suite has nothing to say.
fn python_available(root: &std::path::Path) -> bool {
    hick_lsp::discovery::discover("python", root).is_some()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_apps_window_gets_answers_in_document_coordinates() {
    let app = open_app().await;
    if !python_available(&app.root) {
        eprintln!("SKIPPED: no Python language server on this machine");
        return;
    }

    let url = format!("{}/api/ws?doc=doc:{}", app.base, app.doc_id);
    let (mut socket, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("the app's window connects");

    // The document is addressed in the CLIENT's scheme. The browser never
    // learns where the file is on disk, and the bridge rewrites it both ways.
    let uri = "hick:///doc.hick";
    send(&mut socket, json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "hick", "version": 1, "text": DOC}},
    }))
    .await;

    // The capabilities announcement is what makes colouring possible at all:
    // semantic tokens are integers indexing into a legend, and this is the
    // only way the browser can be told what the legend is.
    let capabilities = wait_for_method(
        &mut socket,
        "hick/serverCapabilities",
        Duration::from_secs(30),
    )
    .await
    .expect("the bridge announces what the server can do");
    let legend =
        &capabilities["params"]["capabilities"]["semanticTokensProvider"]["legend"]["tokenTypes"];
    assert!(
        legend
            .as_array()
            .is_some_and(|types| types.contains(&json!("function"))),
        "no usable semantic token legend reached the window: {capabilities}"
    );

    // Hover, retried the way the UI does when the cursor moves: a server
    // still indexing answers null rather than waiting.
    let hover = request_until(
        &mut socket,
        "textDocument/hover",
        json!({"textDocument": {"uri": uri}, "position": {"line": DEFINITION_LINE, "character": 4}}),
        Duration::from_secs(60),
    )
    .await
    .expect("hover answers");
    assert!(
        hover.to_string().contains("summarise"),
        "hover in the app's window did not mention the symbol: {hover}"
    );

    // Definition, from the call site. It must come back named in the
    // client's scheme and on the document's line — not the virtual file's.
    let definition = request_until(
        &mut socket,
        "textDocument/definition",
        json!({"textDocument": {"uri": uri}, "position": {"line": USE_LINE, "character": 11}}),
        Duration::from_secs(60),
    )
    .await
    .expect("definition answers");
    let first = definition
        .as_array()
        .and_then(|items| items.first().cloned())
        .unwrap_or(definition.clone());
    assert_eq!(
        first["uri"].as_str().unwrap_or_default(),
        uri,
        "the window was handed a filesystem path instead of its own scheme: {definition}"
    );
    assert_eq!(
        first["range"]["start"]["line"].as_u64().unwrap_or_default() as u32,
        DEFINITION_LINE,
        "definition landed on the wrong document line: {definition}"
    );
}

/// A session record with two things a reader wants marked: a tool the agent
/// was refused (information — findable, not counted) and a last turn nobody
/// answered (a warning — the record is incomplete).
const SESSION: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-20T09:00:00Z">
<hick:user turn="t1">Edit it.</hick:user>
<hick:assistant>
<hick:tool name="write_doc">
<hick:input>x</hick:input>
</hick:tool>
</hick:assistant>
<hick:tool-result name="write_doc" ok="false">
declined — fixture
</hick:tool-result>
<hick:assistant>I could not.</hick:assistant>
<hick:user turn="t2" parent="t1">Try again.</hick:user>
</hick:session>
"##;

/// Protects docs/guarantees/agent/a-session-is-the-conversation.md — the
/// problem markers a session carries reach the app's window through the same
/// bridge as every other diagnostic, with no language server installed.
#[tokio::test(flavor = "multi_thread")]
async fn a_sessions_problems_reach_the_window_as_diagnostics() {
    // The room is any open document's; the text the window sends is what
    // gets linted, exactly as when a session tab opens in the app.
    let app = open_app().await;
    std::fs::write(app.root.join("session.hick"), SESSION).unwrap();
    let url = format!("{}/api/ws?doc=doc:{}", app.base, app.doc_id);
    let (mut socket, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("the app's window connects");
    let uri = "hick:///session.hick";
    send(&mut socket, json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "hick", "version": 1, "text": SESSION}},
    }))
    .await;

    // The first publish may be empty (the parse succeeded, nothing from a
    // child yet); wait for the one that carries the session's own lints.
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    let mut diagnostics: Vec<Value> = Vec::new();
    while std::time::Instant::now() < deadline {
        let Some(message) = wait_for_method(
            &mut socket,
            "textDocument/publishDiagnostics",
            Duration::from_secs(10),
        )
        .await
        else {
            break;
        };
        let list = message["params"]["diagnostics"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if list.iter().any(|d| d["source"] == "hick-session") {
            diagnostics = list;
            break;
        }
    }
    assert!(
        !diagnostics.is_empty(),
        "no session diagnostics reached the window"
    );
    let refused = diagnostics
        .iter()
        .find(|d| d["message"].as_str().unwrap_or("").contains("refused"))
        .expect("the refused tool is marked");
    assert_eq!(
        refused["severity"], 3,
        "a refused tool is information, not a count"
    );
    // Line 8 (0-based) is the `<hick:tool-result …>` opening tag.
    assert_eq!(refused["range"]["start"]["line"], 8);
    let unanswered = diagnostics
        .iter()
        .find(|d| d["message"].as_str().unwrap_or("").contains("no reply"))
        .expect("the unanswered turn is marked");
    assert_eq!(unanswered["severity"], 2);
    assert_eq!(unanswered["range"]["start"]["line"], 12);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_window_opened_where_no_server_exists_still_works() {
    // The degradation guarantee, from the window's side: a language nothing
    // on this machine can serve must leave the editor usable rather than
    // erroring. COBOL is chosen because nobody has a COBOL server installed.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    std::fs::write(
        root.join("doc.hick"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
         # No server for this\n\n\
         <hick:file path=\"legacy.cbl\">\n\
         IDENTIFICATION DIVISION.\n\
         </hick:file>\n\
         </hick:doc>\n",
    )
    .unwrap();

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("the session prepares");
    let doc_id = prepared.state.index.sole().expect("one document").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    let url = format!("ws://127.0.0.1:{port}/api/ws?doc=doc:{doc_id}");
    let (mut socket, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let text = std::fs::read_to_string(root.join("doc.hick")).unwrap();
    send(&mut socket, json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": "hick:///doc.hick", "languageId": "hick", "version": 1, "text": text}},
    }))
    .await;

    // The window still gets its capabilities, and a hover simply answers
    // nothing — the editor stays open and editable, which is the guarantee.
    assert!(
        wait_for_method(
            &mut socket,
            "hick/serverCapabilities",
            Duration::from_secs(30)
        )
        .await
        .is_some(),
        "the bridge failed to come up for a language with no server"
    );
    let hover = request_until(
        &mut socket,
        "textDocument/hover",
        json!({"textDocument": {"uri": "hick:///doc.hick"}, "position": {"line": 5, "character": 2}}),
        Duration::from_secs(5),
    )
    .await;
    assert!(hover.is_none(), "a server appeared for COBOL: {hover:?}");
}

// ---------------------------------------------------------------------------
// The window's side of the wire
// ---------------------------------------------------------------------------

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Send one JSON-RPC message as an LSP-channel frame.
async fn send(socket: &mut Socket, message: Value) {
    let mut frame = vec![CHANNEL_LSP];
    frame.extend_from_slice(&serde_json::to_vec(&message).unwrap());
    socket.send(TtMessage::Binary(frame)).await.unwrap();
}

/// Read frames until one has this `method`, or time out.
async fn wait_for_method(socket: &mut Socket, method: &str, budget: Duration) -> Option<Value> {
    tokio::time::timeout(budget, async {
        while let Some(Ok(message)) = socket.next().await {
            if let Some(value) = decode(message)
                && value.get("method").and_then(Value::as_str) == Some(method)
            {
                return Some(value);
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

/// Read frames until the reply to `id` arrives, or time out.
async fn wait_for_id(socket: &mut Socket, id: i64, budget: Duration) -> Option<Value> {
    tokio::time::timeout(budget, async {
        while let Some(Ok(message)) = socket.next().await {
            if let Some(value) = decode(message)
                && value.get("id").and_then(Value::as_i64) == Some(id)
            {
                return value.get("result").cloned();
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

/// Ask until the answer is something, as the UI does when the cursor moves.
async fn request_until(
    socket: &mut Socket,
    method: &str,
    params: Value,
    budget: Duration,
) -> Option<Value> {
    let deadline = tokio::time::Instant::now() + budget;
    let mut id = 1;
    while tokio::time::Instant::now() < deadline {
        send(
            socket,
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params.clone()}),
        )
        .await;
        let remaining = deadline - tokio::time::Instant::now();
        match wait_for_id(socket, id, remaining.min(Duration::from_secs(10))).await {
            Some(Value::Null) | None => {}
            Some(result) => return Some(result),
        }
        id += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    None
}

/// One frame from the server, if it is an LSP-channel JSON message.
fn decode(message: TtMessage) -> Option<Value> {
    let TtMessage::Binary(bytes) = message else {
        return None;
    };
    if bytes.first() != Some(&CHANNEL_LSP) {
        return None;
    }
    serde_json::from_slice(&bytes[1..]).ok()
}
