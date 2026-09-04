//! A real JavaScript debug session, against js-debug, over a real document.
//!
//! Protects docs/guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed.md
//!
//! js-debug does not run the program on the connection that launched it. It
//! answers `launch`, marks the breakpoint `provisionalBreakpoint`, and asks
//! the client — with a `startDebugging` reverse request carrying a
//! `__pendingTargetId` — to open a second connection and run the session
//! there. This is the test that says whether hick follows it.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in JavaScript

Every line is quantity times unit price.

<hick:file path="app/main.js">
function lineTotal(quantity, unitPrice) {
  const subtotal = quantity * unitPrice;
  return subtotal;
}

console.log(lineTotal(3, 1.25));
</hick:file>
</hick:doc>
"##;

/// Found in the document rather than written down. A constant here was wrong
/// twice while these tests were being written — both times pointing at a
/// blank line, and both times the status assertion passed anyway.
fn subtotal_line() -> u32 {
    DOC.lines()
        .position(|line| line.contains("const subtotal ="))
        .expect("the document has that line") as u32
}

fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(unix)]
fn borrow_this_repos_adapters(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache
        .join("adapters/js-debug/src/dapDebugServer.js")
        .exists()
    {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_javascript_document_line_stops() {
    if !have("node") {
        eprintln!("SKIPPED: no node on this machine");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.hick"), DOC).expect("write the document");

    let mapping =
        Arc::new(Mapping::for_document(Path::new("doc.hick"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("main.js is debuggable");
    assert!(entry.ends_with("main.js"), "{entry:?}");

    borrow_this_repos_adapters(root);
    let Some(adapter) = hick_dap::discover("javascript", root) else {
        eprintln!(
            "SKIPPED: no JavaScript debug adapter on this machine.\n{}",
            hick_dap::how_to_get("javascript")
        );
        return;
    };
    eprintln!("using adapter {} ({})", adapter.adapter, adapter.origin);
    assert!(
        adapter.multi_session,
        "js-debug runs the program on a second connection"
    );

    let line = subtotal_line();
    let breakpoints = vec![Breakpoint {
        line,
        condition: None,
        hit_condition: None,
        log_message: None,
    }];
    let (session, statuses) = Session::start(
        Launch {
            transport: adapter.transport,
            multi_session: adapter.multi_session,
            adapter_extra: adapter.launch_extra.clone(),
            adapter: adapter.command,
            program: entry.clone(),
            cwd: entry.parent().unwrap().to_path_buf(),
            extra: serde_json::json!({}),
        },
        mapping,
        &breakpoints,
    )
    .await
    .expect("the session starts");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].line, line);
    // The parent's breakpoints came back provisional. These are the child's,
    // which is the whole point: a provisional breakpoint never fires.
    assert_ne!(
        statuses[0].state,
        hick_dap::BindState::Refused,
        "the adapter refused the breakpoint: {:?}",
        statuses[0]
    );

    let stopped = session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .expect("waiting works")
        .expect("the program stopped");
    assert_eq!(stopped.reason, "breakpoint", "{stopped:?}");

    let frames = session.stack(stopped.thread_id).await.expect("a stack");
    let top = frames.first().expect("a top frame");
    assert!(
        top.name.contains("lineTotal"),
        "stopped in the wrong frame: {top:?}"
    );
    assert_eq!(top.line, Some(line), "wrong document line: {top:?}");

    let value = session
        .evaluate("quantity", Some(top.id), "watch")
        .await
        .expect("evaluating in the stopped frame");
    assert_eq!(
        value.value, "3",
        "argument read from the real frame: {value:?}"
    );

    session.shutdown().await;
    eprintln!("OK javascript: followed the child session, stopped, read a value");
}
