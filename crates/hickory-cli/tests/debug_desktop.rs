//! The app's debugger, over the real socket.
//!
//! Protects docs/specs/freeform/literate-debugging.md
//!
//! Driven the way the window drives it — a WebSocket, `0x03` frames, the
//! session API's verbs — because the claims worth testing are about what a
//! person sees: a breakpoint on the line they clicked, values from the frame
//! they are stopped in, and a file that does not change no matter what they
//! do in here.

use std::path::PathBuf;
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use hickory_collab::CHANNEL_DEBUG;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as TtMessage;

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing

<hick:file path="pricing.py">
LINES = [(2, 9.99), (1, 24.50)]


def line_total(quantity, unit_price):
    subtotal = quantity * unit_price
    return subtotal


with open("evidence.txt", "w") as handle:
    handle.write("the debuggee wrote this")

print(sum(line_total(q, p) for q, p in LINES))
</hick:file>
</hick:doc>
"##;

/// 0-based document line of `    subtotal = quantity * unit_price`.
const SUBTOTAL_LINE: u32 = 9;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

struct App {
    socket: Socket,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

async fn open_app() -> App {
    let dir = tempfile::tempdir().expect("a temp project");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
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

    let url = format!("ws://127.0.0.1:{port}/api/ws?doc=doc:{doc_id}");
    let (socket, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("the window connects");
    App {
        socket,
        root,
        _dir: dir,
    }
}

async fn send(socket: &mut Socket, request: Value) {
    let mut frame = vec![CHANNEL_DEBUG];
    frame.extend_from_slice(&serde_json::to_vec(&request).unwrap());
    socket.send(TtMessage::Binary(frame)).await.unwrap();
}

/// Read `0x03` frames until one has this event, or time out.
async fn wait_for(socket: &mut Socket, event: &str, budget: Duration) -> Option<Value> {
    wait_for_any(socket, &[event], budget).await
}

/// Read `0x03` frames until one has any of these events, or time out.
async fn wait_for_any(socket: &mut Socket, events: &[&str], budget: Duration) -> Option<Value> {
    tokio::time::timeout(budget, async {
        while let Some(Ok(message)) = socket.next().await {
            let TtMessage::Binary(bytes) = message else {
                continue;
            };
            if bytes.first() != Some(&CHANNEL_DEBUG) {
                continue;
            }
            let value: Value = serde_json::from_slice(&bytes[1..]).ok()?;
            // A failure is worth surfacing immediately rather than timing
            // out on the event that will never come.
            let seen = value["event"].as_str().unwrap_or_default();
            if seen == "failed" && !events.contains(&"failed") {
                eprintln!("debug channel failed: {}", value["message"]);
                return Some(value);
            }
            if events.contains(&seen) {
                return Some(value);
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

fn python_available(root: &std::path::Path) -> bool {
    // Borrow this repository's adapter cache, so a developer who ran
    // `hick dap install python` at the top of the repo exercises this file
    // instead of skipping it. CI installs debugpy for the machine.
    #[cfg(unix)]
    {
        let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
        if cache.join("adapters/python/bin/python3").exists() {
            let _ = std::os::unix::fs::symlink(cache, root.join(".hick-cache"));
        }
    }
    hick_dap::discover("python", root).is_some()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_window_can_set_a_breakpoint_step_and_read_values() {
    let mut app = open_app().await;
    if !python_available(&app.root) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }

    send(
        &mut app.socket,
        json!({ "op": "start", "doc": "hick:///doc.hick",
                "breakpoints": [{ "line": SUBTOTAL_LINE }] }),
    )
    .await;

    let started = wait_for(&mut app.socket, "started", Duration::from_secs(60))
        .await
        .expect("the session starts");
    assert_eq!(started["breakpoints"][0]["line"], SUBTOTAL_LINE);
    assert_eq!(
        started["breakpoints"][0]["verified"], true,
        "the gutter would draw this hollow: {started}"
    );
    // The capabilities the UI gates its controls on.
    assert!(started["capabilities"]["conditional_breakpoints"].is_boolean());
    let session = started["session"].as_str().unwrap().to_string();

    // Stopped, with everything the editor draws: the paused line, the stack,
    // and the values it shows inline.
    let stopped = wait_for(&mut app.socket, "stopped", Duration::from_secs(60))
        .await
        .expect("the program stops");
    assert_eq!(
        stopped["line"], SUBTOTAL_LINE,
        "the paused line is wrong: {stopped}"
    );
    assert_eq!(stopped["frames"][0]["name"], "line_total");
    let variables = stopped["variables"].as_array().unwrap();
    let quantity = variables
        .iter()
        .find(|v| v["name"] == "quantity")
        .expect("quantity is in scope for the inline display");
    assert_eq!(quantity["value"], "2");

    // Hovering an identifier while paused: the same evaluator, in the frame.
    send(
        &mut app.socket,
        json!({ "op": "eval", "session": session,
                "expression": "quantity * unit_price", "context": "hover" }),
    )
    .await;
    let value = wait_for(&mut app.socket, "value", Duration::from_secs(30))
        .await
        .expect("hover answers");
    assert!(
        value["value"].as_str().unwrap().starts_with("19.98"),
        "{value}"
    );

    // Step over, and the paused line moves one down.
    send(
        &mut app.socket,
        json!({ "op": "step", "session": session, "how": "over" }),
    )
    .await;
    let stepped = wait_for(&mut app.socket, "stopped", Duration::from_secs(30))
        .await
        .expect("stops after stepping");
    assert_eq!(stepped["line"], SUBTOTAL_LINE + 1);

    // And back again — the move that exists on this adapter.
    send(
        &mut app.socket,
        json!({ "op": "jump", "session": session, "line": SUBTOTAL_LINE }),
    )
    .await;
    let jumped = wait_for(&mut app.socket, "stopped", Duration::from_secs(30))
        .await
        .expect("stops after jumping");
    assert_eq!(
        jumped["line"], SUBTOTAL_LINE,
        "the jump did not land: {jumped}"
    );

    send(&mut app.socket, json!({ "op": "stop", "session": session })).await;
    wait_for(&mut app.socket, "ended", Duration::from_secs(30))
        .await
        .expect("the session ends");
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_a_debugger_does_touches_the_project() {
    // The isolation guarantee, checked rather than asserted. The debuggee in
    // this document WRITES A FILE, and stepping through it must leave the
    // project exactly as it was: no evidence.txt, no generated pricing.py,
    // and a document byte-for-byte unchanged.
    let mut app = open_app().await;
    if !python_available(&app.root) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }
    let before = std::fs::read_to_string(app.root.join("doc.hick")).unwrap();

    send(
        &mut app.socket,
        json!({ "op": "start", "doc": "hick:///doc.hick",
                "breakpoints": [{ "line": SUBTOTAL_LINE }] }),
    )
    .await;
    let started = wait_for(&mut app.socket, "started", Duration::from_secs(60))
        .await
        .expect("starts");
    let session = started["session"].as_str().unwrap().to_string();
    wait_for(&mut app.socket, "stopped", Duration::from_secs(60))
        .await
        .expect("stops");

    // Run the rest of the program, including the write.
    send(
        &mut app.socket,
        json!({ "op": "step", "session": session, "how": "continue" }),
    )
    .await;
    // It either stops again — this document's breakpoint is inside a
    // function called once per line of the order — or finishes. Both are
    // fine, and waiting for only one of them is waiting for a timeout.
    let _ = wait_for_any(
        &mut app.socket,
        &["stopped", "finished"],
        Duration::from_secs(60),
    )
    .await;

    send(&mut app.socket, json!({ "op": "stop", "session": session })).await;
    wait_for(&mut app.socket, "ended", Duration::from_secs(30)).await;

    assert!(
        !app.root.join("evidence.txt").exists(),
        "the debuggee's write landed in the project"
    );
    assert!(
        !app.root.join("pricing.py").exists(),
        "the session wove a file into the project; only a run may do that"
    );
    assert_eq!(
        std::fs::read_to_string(app.root.join("doc.hick")).unwrap(),
        before,
        "the document changed while it was being debugged"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_prose_is_reported_not_silently_dropped() {
    let mut app = open_app().await;
    if !python_available(&app.root) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }
    // Line 2 is the heading.
    send(
        &mut app.socket,
        json!({ "op": "start", "doc": "hick:///doc.hick", "breakpoints": [{ "line": 2 }] }),
    )
    .await;
    let started = wait_for(&mut app.socket, "started", Duration::from_secs(60))
        .await
        .expect("starts");
    assert_eq!(started["breakpoints"][0]["verified"], false);
    assert!(
        started["breakpoints"][0]["message"]
            .as_str()
            .is_some_and(|m| m.contains("prose")),
        "no explanation for the hollow dot: {started}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn asking_about_a_session_that_ended_says_why() {
    let mut app = open_app().await;
    send(
        &mut app.socket,
        json!({ "op": "state", "session": "dbg-does-not-exist" }),
    )
    .await;
    let failed = wait_for(&mut app.socket, "failed", Duration::from_secs(30))
        .await
        .expect("an answer, not silence");
    let message = failed["message"].as_str().unwrap();
    assert!(message.contains("dbg-does-not-exist"), "{message}");
    assert!(message.contains("untouched"), "{message}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_program_that_runs_to_completion_ends_its_own_session() {
    // The reported bug: the debuggee ran to the end and the session stayed —
    // adapter process alive, scratch clone on disk, registry entry waiting
    // fifteen minutes for the idle sweep, and the strip in the window still
    // dressed as a live session. Finishing must reap everything on its own.
    let mut app = open_app().await;
    if !python_available(&app.root) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }

    send(
        &mut app.socket,
        json!({ "op": "start", "doc": "hick:///doc.hick",
                "breakpoints": [{ "line": SUBTOTAL_LINE }] }),
    )
    .await;
    let started = wait_for(&mut app.socket, "started", Duration::from_secs(60))
        .await
        .expect("starts");
    let session = started["session"].as_str().unwrap().to_string();
    wait_for(&mut app.socket, "stopped", Duration::from_secs(60))
        .await
        .expect("stops");

    // Continue until the program is over. The breakpoint is inside a
    // function called once per order line, so this takes a few.
    let mut finished = None;
    for _ in 0..5 {
        send(
            &mut app.socket,
            json!({ "op": "step", "session": session, "how": "continue" }),
        )
        .await;
        let answer = wait_for_any(
            &mut app.socket,
            &["stopped", "finished"],
            Duration::from_secs(60),
        )
        .await
        .expect("the program stops again or ends");
        if answer["event"] == "finished" {
            finished = Some(answer);
            break;
        }
    }
    let finished = finished.expect("the program ran to completion");

    // The end says how it ended, so the strip can show it.
    assert_eq!(finished["exit_code"], 0, "{finished}");

    // The session went with the program: its id no longer answers.
    send(
        &mut app.socket,
        json!({ "op": "state", "session": session }),
    )
    .await;
    let failed = wait_for(&mut app.socket, "failed", Duration::from_secs(30))
        .await
        .expect("an answer, not silence");
    assert!(
        failed["message"]
            .as_str()
            .unwrap()
            .contains("no debug session"),
        "{failed}"
    );

    // And a stop arriving after the reap — the person pressing the button a
    // beat late — is the state they asked for, not an error.
    send(&mut app.socket, json!({ "op": "stop", "session": session })).await;
    wait_for(&mut app.socket, "ended", Duration::from_secs(30))
        .await
        .expect("a late stop still answers ended");
    send(&mut app.socket, json!({ "op": "stop", "session": session })).await;
    wait_for(&mut app.socket, "ended", Duration::from_secs(30))
        .await
        .expect("and so does a second one");
}

#[tokio::test(flavor = "multi_thread")]
async fn reaping_a_finished_session_deletes_the_scratch_clone() {
    // The registry's own half of the same guarantee, where the resources are
    // visible: the scratch directory the debuggee ran in must be gone once
    // the session is reaped, and reaping must be safe to do twice.
    let dir = tempfile::tempdir().expect("a temp project");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join("doc.hick"), DOC).unwrap();
    if !python_available(&root) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }

    let registry = hickory_cli::debug_sessions::Registry::new();
    // No breakpoints: the program runs straight to the end.
    let (id, live, _statuses) = registry
        .start(&root.join("doc.hick"), &[], None, &mut |_| {})
        .await
        .expect("the session starts");
    let scratch = live.scratch_path().to_path_buf();
    assert!(
        scratch.exists(),
        "the debuggee has a scratch clone to run in"
    );

    let stopped = live
        .session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .expect("waiting works");
    assert!(stopped.is_none(), "the program should have run to the end");

    // The first reap does the work; the second — the losing side of the
    // race with an explicit stop — finds nothing, quietly.
    assert!(registry.reap(&id).await);
    assert!(registry.is_empty().await);
    assert!(!registry.reap(&id).await);
    assert!(
        registry.stop(&id).await.is_err(),
        "an explicit stop of a reaped session still says there was nothing"
    );

    // The scratch clone dies with the last handle to the session. If the
    // adapter process were still holding it, the delete would fail and the
    // directory would remain.
    drop(live);
    assert!(
        !scratch.exists(),
        "the scratch clone survived the session: {}",
        scratch.display()
    );
}
