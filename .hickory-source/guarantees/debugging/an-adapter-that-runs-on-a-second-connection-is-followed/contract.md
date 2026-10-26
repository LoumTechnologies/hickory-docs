<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Adapter That Runs On A Second Connection Is Followed

For: Authors debugging generated code while navigating its document source.

Given an adapter that launches a supervisor rather than a program — js-debug
today — when a session starts, then hick answers its `startDebugging` request,
opens a **sibling connection to the same server**, and runs the real session
there: breakpoints, stack, values and stepping all belong to the child. The
supervising connection is held open for the life of the session, because it
owns the server process, and is shut down after the child.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-049e3521b988"></a>
## `crates/hick-dap/tests/live_session_js.rs`

Related contracts: [An Adapter That Runs On A Second Connection Is Followed](contract.md).

<a id="f-049e3521b988-DOC"></a>
### `DOC`

A real JavaScript debug session, against js-debug, over a real document.
Protects docs/guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed.md
js-debug does not run the program on the connection that launched it. It
answers `launch`, marks the breakpoint `provisionalBreakpoint`, and asks
the client — with a `startDebugging` reverse request carrying a
`__pendingTargetId` — to open a second connection and run the session
there. This is the test that says whether hick follows it.

<storycode:copy id="f-049e3521b988-DOC">
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

</storycode:copy>
<a id="f-049e3521b988-subtotal_line"></a>
### `subtotal_line`

Found in the document rather than written down. A constant here was wrong
twice while these tests were being written — both times pointing at a
blank line, and both times the status assertion passed anyway.

<storycode:copy id="f-049e3521b988-subtotal_line">
/// Found in the document rather than written down. A constant here was wrong
/// twice while these tests were being written — both times pointing at a
/// blank line, and both times the status assertion passed anyway.
fn subtotal_line() -> u32 {
    DOC.lines()
        .position(|line| line.contains("const subtotal ="))
        .expect("the document has that line") as u32
}

</storycode:copy>
<a id="f-049e3521b988-have"></a>
### `have`

<storycode:copy id="f-049e3521b988-have">
fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

</storycode:copy>
<a id="f-049e3521b988-borrow_this_repos_adapters"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-049e3521b988-borrow_this_repos_adapters">
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

</storycode:copy>
<a id="f-049e3521b988-borrow_this_repos_adapters-2"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-049e3521b988-borrow_this_repos_adapters-2">
#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

</storycode:copy>
<a id="f-049e3521b988-a_breakpoint_on_a_javascript_document_line_stops"></a>
### `a_breakpoint_on_a_javascript_document_line_stops`

<storycode:copy id="f-049e3521b988-a_breakpoint_on_a_javascript_document_line_stops">
#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_javascript_document_line_stops() {
    if !have("node") {
        eprintln!("SKIPPED: no node on this machine");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.md"), DOC).expect("write the document");

    let mapping = Arc::new(Mapping::for_document(Path::new("doc.md"), DOC, root).expect("mapping"));
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
</storycode:copy>

### Assemble `crates/hick-dap/tests/live_session_js.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/live_session_js.rs"><storycode:paste select="#f-049e3521b988-DOC"/><storycode:paste select="#f-049e3521b988-subtotal_line"/><storycode:paste select="#f-049e3521b988-have"/><storycode:paste select="#f-049e3521b988-borrow_this_repos_adapters"/><storycode:paste select="#f-049e3521b988-borrow_this_repos_adapters-2"/><storycode:paste select="#f-049e3521b988-a_breakpoint_on_a_javascript_document_line_stops"/></storycode:file>

<a id="output-60905a0dec2e"></a>
## `crates/hick-dap/tests/live_session_ts.rs`

Related contracts: [An Adapter That Runs On A Second Connection Is Followed](contract.md).

<a id="f-60905a0dec2e-DOC"></a>
### `DOC`

A real JavaScript debug session, against js-debug, over a real document.
Protects docs/guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed.md
js-debug does not run the program on the connection that launched it. It
answers `launch`, marks the breakpoint `provisionalBreakpoint`, and asks
the client — with a `startDebugging` reverse request carrying a
`__pendingTargetId` — to open a second connection and run the session
there. This is the test that says whether hick follows it.

<storycode:copy id="f-60905a0dec2e-DOC">
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
# Pricing, in TypeScript

Every line is quantity times unit price.

<hick:file path="app/main.ts">
function lineTotal(quantity: number, unitPrice: number): number {
  const subtotal: number = quantity * unitPrice;
  return subtotal;
}

console.log(lineTotal(3, 1.25));
</hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-60905a0dec2e-subtotal_line"></a>
### `subtotal_line`

Found in the document rather than written down. A constant here was wrong
twice while these tests were being written — both times pointing at a
blank line, and both times the status assertion passed anyway.

<storycode:copy id="f-60905a0dec2e-subtotal_line">
/// Found in the document rather than written down. A constant here was wrong
/// twice while these tests were being written — both times pointing at a
/// blank line, and both times the status assertion passed anyway.
fn subtotal_line() -> u32 {
    DOC.lines()
        .position(|line| line.contains("const subtotal"))
        .expect("the document has that line") as u32
}

</storycode:copy>
<a id="f-60905a0dec2e-have"></a>
### `have`

<storycode:copy id="f-60905a0dec2e-have">
fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

</storycode:copy>
<a id="f-60905a0dec2e-borrow_this_repos_adapters"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-60905a0dec2e-borrow_this_repos_adapters">
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

</storycode:copy>
<a id="f-60905a0dec2e-borrow_this_repos_adapters-2"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-60905a0dec2e-borrow_this_repos_adapters-2">
#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

</storycode:copy>
<a id="f-60905a0dec2e-a_breakpoint_on_a_typescript_document_line_stops"></a>
### `a_breakpoint_on_a_typescript_document_line_stops`

<storycode:copy id="f-60905a0dec2e-a_breakpoint_on_a_typescript_document_line_stops">
#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_typescript_document_line_stops() {
    if !have("node") {
        eprintln!("SKIPPED: no node on this machine");
        return;
    }
    // A raw `.ts` file is run by node itself: type stripping is on by
    // default from v23.6, and before that node cannot execute TypeScript at
    // all without a loader. This is a real limit on the claim — a document
    // whose TypeScript is debugged needs a node new enough to run it — and
    // it is skipped loudly rather than failed, because an old node is not
    // hick being wrong.
    let version = std::process::Command::new("node")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let major: u32 = version
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|m| m.parse().ok())
        .unwrap_or(0);
    if major < 23 {
        eprintln!(
            "SKIPPED: node {version} cannot run TypeScript directly (type stripping arrived \
             in v23.6), so there is nothing here to debug without a loader"
        );
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.md"), DOC).expect("write the document");

    let mapping = Arc::new(Mapping::for_document(Path::new("doc.md"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("main.ts is debuggable");
    assert!(entry.ends_with("main.ts"), "{entry:?}");

    borrow_this_repos_adapters(root);
    let Some(adapter) = hick_dap::discover("typescript", root) else {
        eprintln!(
            "SKIPPED: no JavaScript debug adapter on this machine.\n{}",
            hick_dap::how_to_get("typescript")
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
    eprintln!("OK typescript: followed the child session, stopped, read a value");
}
</storycode:copy>

### Assemble `crates/hick-dap/tests/live_session_ts.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/live_session_ts.rs"><storycode:paste select="#f-60905a0dec2e-DOC"/><storycode:paste select="#f-60905a0dec2e-subtotal_line"/><storycode:paste select="#f-60905a0dec2e-have"/><storycode:paste select="#f-60905a0dec2e-borrow_this_repos_adapters"/><storycode:paste select="#f-60905a0dec2e-borrow_this_repos_adapters-2"/><storycode:paste select="#f-60905a0dec2e-a_breakpoint_on_a_typescript_document_line_stops"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-dap/src/adapter.rs](../../../mechanisms/crates/hick-dap/src/adapter.rs.md#output-2b5d1652c2d2)
- [crates/hick-dap/src/discovery.rs](../../../mechanisms/crates/hick-dap/src/discovery.rs.md#output-edc49b1ccd22)
- [crates/hick-dap/src/session.rs](../../../mechanisms/crates/hick-dap/src/session.rs.md#output-3c41c22e0551)
- [crates/hick-plugins/src/dap_discovery.rs](../../../mechanisms/crates/hick-plugins/src/dap_discovery.rs.md#output-72962e7f25e2)
- [crates/hickory-cli/src/dap_install.rs](../../../mechanisms/crates/hickory-cli/src/dap_install.rs.md#output-a1687642fec0)

### Tests and fixtures

- [crates/hick-dap/tests/live_session_js.rs](contract.md#output-049e3521b988)
- [crates/hick-dap/tests/live_session_ts.rs](contract.md#output-60905a0dec2e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
