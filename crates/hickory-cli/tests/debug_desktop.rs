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

/// A folder with no document in it at all: two plain Python files, and the
/// window's workspace socket — the one a plain file's pane talks over.
const APP_PY: &str = "from helpers import double\n\n\ndef run(n):\n    total = double(n)\n    return total\n\n\nprint(run(21))\n";
const HELPERS_PY: &str = "def double(x):\n    return x * 2\n";
/// 0-based line of `    total = double(n)` in app.py.
const TOTAL_LINE: u32 = 4;

async fn open_plain_workspace() -> App {
    let dir = tempfile::tempdir().expect("a temp project");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("tools")).unwrap();
    std::fs::write(root.join("tools/app.py"), APP_PY).unwrap();
    std::fs::write(root.join("tools/helpers.py"), HELPERS_PY).unwrap();

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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    let url = format!("ws://127.0.0.1:{port}/api/ws?doc=workspace");
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
    // debugpy binds while answering, so this is `bound` at set time. A
    // compiled language answers `pending` here and is promoted by the
    // adapter's `breakpoint` event — see `live_session_csharp.rs`.
    assert_eq!(
        started["breakpoints"][0]["state"], "bound",
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
    // `refused`, not `pending`: hick decided this itself, before any adapter
    // was asked, and it is the one refusal that is certain.
    assert_eq!(started["breakpoints"][0]["state"], "refused");
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
    let scratch = live
        .scratch_path()
        .expect("a document's debuggee has a scratch clone")
        .to_path_buf();
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

#[tokio::test(flavor = "multi_thread")]
async fn a_plain_file_debugs_as_itself_over_the_workspace_socket() {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    //
    // No document anywhere: `tools/app.py` is debugged at its own path, on
    // its own lines, over the socket a plain file's pane already has. A
    // frame in the neighbouring file is not "external" — it is a file in
    // this folder, named so the app can open it.
    let mut app = open_plain_workspace().await;
    if !python_available(&app.root) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }
    let before = std::fs::read_to_string(app.root.join("tools/app.py")).unwrap();

    send(
        &mut app.socket,
        json!({ "op": "start", "doc": "hick:///tools/app.py",
                "breakpoints": [{ "line": TOTAL_LINE }] }),
    )
    .await;
    let started = wait_for(&mut app.socket, "started", Duration::from_secs(60))
        .await
        .expect("the session starts");
    // The answer says which file it is for: every plain-file pane shares
    // this socket, and one of them must be able to tell this is its own.
    assert_eq!(started["doc"], "hick:///tools/app.py", "{started}");
    assert_eq!(started["breakpoints"][0]["line"], TOTAL_LINE);
    assert_eq!(started["breakpoints"][0]["state"], "bound", "{started}");
    let session = started["session"].as_str().unwrap().to_string();

    let stopped = wait_for(&mut app.socket, "stopped", Duration::from_secs(60))
        .await
        .expect("the program stops");
    assert_eq!(
        stopped["line"], TOTAL_LINE,
        "the paused line is the file's own: {stopped}"
    );
    assert_eq!(stopped["frames"][0]["name"], "run");
    assert_eq!(stopped["frames"][0]["source"], "tools/app.py", "{stopped}");
    let variables = stopped["variables"].as_array().unwrap();
    let n = variables
        .iter()
        .find(|v| v["name"] == "n")
        .expect("n is in scope for the inline display");
    assert_eq!(n["value"], "21");

    // Step into the neighbouring file. It is not the file being debugged,
    // so it has no line in THIS pane — but it is a file in this folder, and
    // the app can open it at the line the adapter reported.
    send(
        &mut app.socket,
        json!({ "op": "step", "session": session, "how": "in" }),
    )
    .await;
    let inside = wait_for(&mut app.socket, "stopped", Duration::from_secs(30))
        .await
        .expect("stops inside double");
    let top = &inside["frames"][0];
    assert_eq!(top["name"], "double", "{inside}");
    assert_eq!(top["in_document"], false);
    assert_eq!(top["source"], "tools/helpers.py", "{inside}");
    assert_eq!(top["source_line"], 1, "{inside}");
    // And the caller is still addressed in the debugged file's own lines.
    let caller = &inside["frames"][1];
    assert_eq!(caller["source"], "tools/app.py", "{inside}");
    assert_eq!(caller["line"], TOTAL_LINE, "{inside}");

    send(&mut app.socket, json!({ "op": "stop", "session": session })).await;
    wait_for(&mut app.socket, "ended", Duration::from_secs(30))
        .await
        .expect("the session ends");

    // Debugging a plain file runs it in place, and stepping through it
    // still changes nothing: the file is as it was.
    assert_eq!(
        std::fs::read_to_string(app.root.join("tools/app.py")).unwrap(),
        before
    );
}

/// Point a temp folder at the adapters this repository installed, so a
/// developer who ran `hick dap install rust` once at the top of the repo
/// exercises the compiled path instead of skipping it.
fn rust_available(root: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
        if cache
            .join("adapters/codelldb/extension/adapter/codelldb")
            .exists()
        {
            let _ = std::os::unix::fs::symlink(cache, root.join(".hick-cache"));
        }
    }
    let have_cargo = std::process::Command::new("cargo")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    have_cargo && hick_dap::discover("rust", root).is_some()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_plain_rust_file_builds_in_its_own_project_and_stops_on_its_own_line() {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    //
    // The compiled case, and the one where "in its own project" has teeth: a
    // workspace MEMBER's binary lands in the workspace's `target/`, which is
    // cargo's answer and not `<member>/target`; nothing is built under
    // `.hick-cache/`; and there is no scratch copy at all.
    let dir = tempfile::tempdir().expect("a temp workspace");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("crates/pricing/src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/pricing\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("crates/pricing/Cargo.toml"),
        "[package]\nname = \"pricing\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let main = root.join("crates/pricing/src/main.rs");
    std::fs::write(
        &main,
        "fn line_total(quantity: u32, unit_price: f64) -> f64 {\n\
         \x20   let subtotal = quantity as f64 * unit_price;\n\
         \x20   subtotal\n\
         }\n\n\
         fn main() {\n\
         \x20   println!(\"{}\", line_total(3, 1.25));\n\
         }\n",
    )
    .unwrap();
    if !rust_available(&root) {
        eprintln!("SKIPPED: no cargo or no Rust debug adapter (`hick dap install rust`)");
        return;
    }

    let registry = hickory_cli::debug_sessions::Registry::new();
    let mut said: Vec<String> = Vec::new();
    let breakpoints = vec![hick_dap::Breakpoint {
        line: 1,
        condition: None,
        hit_condition: None,
        log_message: None,
    }];
    let (id, live, _statuses) = registry
        .start_plain(&main, &root, &breakpoints, &mut |line| {
            if let hick_dap::BuildOutput::Out(t) | hick_dap::BuildOutput::Err(t) = line {
                said.push(t);
            }
        })
        .await
        .unwrap_or_else(|e| panic!("the session did not start:\n{e:#}\n{}", said.join("\n")));

    assert!(
        live.scratch_path().is_none(),
        "a plain file has no scratch copy"
    );
    // The binary is where the person's own `cargo build` would have put
    // it — the WORKSPACE's target directory, cargo's answer — and not under
    // a redirected `.hick-cache/cargo-target`, which is the document path's
    // rule. (The repo's adapter cache is symlinked in above, so the cache
    // directory's mere existence proves nothing; the binary's location does.)
    let binary = root.join("target/debug/pricing");
    assert!(
        binary.exists() || binary.with_extension("exe").exists(),
        "the build did not land in the workspace's own target directory"
    );

    let stopped = live
        .session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .expect("waiting for the stop")
        .expect("the program stops rather than finishing");
    let frames = live
        .session
        .stack(stopped.thread_id)
        .await
        .expect("a stack");
    assert_eq!(frames[0].line, Some(1), "{frames:?}");
    assert!(frames[0].in_document, "{frames:?}");
    assert_eq!(frames[0].source_line, Some(1));

    registry.stop(&id).await.expect("stops");
}

/// Point a temp folder at the C# adapter this repository installed.
fn csharp_available(root: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
        if cache.join("adapters/netcoredbg/netcoredbg").exists()
            && !root.join(".hick-cache").exists()
        {
            let _ = std::os::unix::fs::symlink(cache, root.join(".hick-cache"));
        }
    }
    let have_dotnet = std::process::Command::new("dotnet")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    have_dotnet && hick_dap::discover("csharp", root).is_some()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_plain_csharp_file_builds_its_own_project_in_place_and_stops_on_its_own_line() {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    //
    // The other compiled language, in place: `dotnet build` runs in the
    // nearest project above the file, the assembly lands under that project's
    // own `bin/`, there is no scratch copy, and the breakpoint stops on the
    // file's own line through netcoredbg's pdb mapping.
    let dir = tempfile::tempdir().expect("a temp repository");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("app")).unwrap();
    std::fs::write(
        root.join("app/app.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <OutputType>Exe</OutputType>\n\
         \x20   <TargetFramework>net10.0</TargetFramework>\n  </PropertyGroup>\n</Project>\n",
    )
    .unwrap();
    let program = root.join("app/Program.cs");
    std::fs::write(
        &program,
        "class Program\n{\n    static decimal LineTotal(int quantity, decimal unitPrice)\n    {\n\
         \x20       decimal subtotal = quantity * unitPrice;\n        return subtotal;\n    }\n\n\
         \x20   static void Main()\n    {\n        System.Console.WriteLine(LineTotal(3, 1.25m));\n    }\n}\n",
    )
    .unwrap();
    /// 0-based line of `        decimal subtotal = quantity * unitPrice;`.
    const SUBTOTAL: u32 = 4;
    if !csharp_available(&root) {
        eprintln!("SKIPPED: no .NET SDK or no C# debug adapter (`hick dap install csharp`)");
        return;
    }

    let registry = hickory_cli::debug_sessions::Registry::new();
    let mut said: Vec<String> = Vec::new();
    let breakpoints = vec![hick_dap::Breakpoint {
        line: SUBTOTAL,
        condition: None,
        hit_condition: None,
        log_message: None,
    }];
    let (id, live, _statuses) = registry
        .start_plain(&program, &root, &breakpoints, &mut |line| {
            if let hick_dap::BuildOutput::Out(t) | hick_dap::BuildOutput::Err(t) = line {
                said.push(t);
            }
        })
        .await
        .unwrap_or_else(|e| panic!("the session did not start:\n{e:#}\n{}", said.join("\n")));

    assert!(
        live.scratch_path().is_none(),
        "a plain file has no scratch copy"
    );
    assert!(
        root.join("app/bin/Debug").exists(),
        "the build did not land in the project's own bin/: {}",
        said.join("\n")
    );

    let stopped = live
        .session
        .wait_for_stop(Duration::from_secs(120))
        .await
        .expect("waiting for the stop")
        .expect("the program stops rather than finishing");
    let frames = live
        .session
        .stack(stopped.thread_id)
        .await
        .expect("a stack");
    let top = &frames[0];
    assert!(top.name.contains("LineTotal"), "{frames:?}");
    assert_eq!(top.line, Some(SUBTOTAL), "{frames:?}");
    assert!(top.in_document, "{frames:?}");
    let value = live
        .session
        .evaluate("quantity", Some(top.id), "watch")
        .await
        .expect("evaluating in the stopped frame");
    assert_eq!(value.value, "3");

    registry.stop(&id).await.expect("stops");
}
