<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Debugger That Lives In A Language Server Is Asked For

For: Authors debugging generated code while navigating its document source.

Given Java, when a debug session starts, then hick starts **eclipse.jdt.ls**
with the `java-debug` plugin named in its `initializationOptions.bundles`,
waits for the server to report `ServiceReady`, sends
`workspace/executeCommand` for `vscode.java.startDebugSession`, and connects
to the **port** in the reply. From there it is ordinary DAP: breakpoints in
document coordinates, the stack, values, stepping.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-debugger-that-lives-in-a-language-server-is-asked-for.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-f0679f10df7f"></a>
## `crates/hick-dap/src/java.rs`

Implementation owned by the language plugin package.

Related contracts: [A Debugger That Lives In A Language Server Is Asked For](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hick-dap/src/java.rs">
//! Implementation owned by the language plugin package.
pub use hick_plugins::java::*;
</storycode:entry>

<a id="output-3a77193c14e0"></a>
## `crates/hickory-cli/src/java_debug.rs`

Implementation owned by the language plugin package.

Related contracts: [A Debugger That Lives In A Language Server Is Asked For](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hickory-cli/src/java_debug.rs">
//! Implementation owned by the language plugin package.
pub use hick_plugins::java_debug::*;
</storycode:entry>

<a id="output-134eb93d24b2"></a>
## `crates/hickory-cli/tests/live_session_java.rs`

Related contracts: [A Debugger That Lives In A Language Server Is Asked For](contract.md).

<a id="f-134eb93d24b2-DOC"></a>
### `DOC`

A real Java debug session, over a real document.
Protects docs/guarantees/debugging/a-debugger-that-lives-in-a-language-server-is-asked-for.md
This test lives in `hickory-cli` rather than beside the other live debug
tests in `hick-dap`, and that placement is the point: Java's adapter is
not a program. `java-debug` is a plugin inside eclipse.jdt.ls, and a
session is obtained by asking the language server for one — which needs
LSP, which `hick-dap` must not know about. The crate that owns both ends
is the one that can write this test.

<storycode:copy id="f-134eb93d24b2-DOC">
//! A real Java debug session, over a real document.
//!
//! Protects docs/guarantees/debugging/a-debugger-that-lives-in-a-language-server-is-asked-for.md
//!
//! This test lives in `hickory-cli` rather than beside the other live debug
//! tests in `hick-dap`, and that placement is the point: Java's adapter is
//! not a program. `java-debug` is a plugin inside eclipse.jdt.ls, and a
//! session is obtained by asking the language server for one — which needs
//! LSP, which `hick-dap` must not know about. The crate that owns both ends
//! is the one that can write this test.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in Java

Every line is quantity times unit price.

<hick:file path="app/Main.java">
public class Main {
    static double lineTotal(int quantity, double unitPrice) {
        double subtotal = quantity * unitPrice;
        return subtotal;
    }

    public static void main(String[] args) {
        System.out.println(lineTotal(3, 1.25));
    }
}
</hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-134eb93d24b2-subtotal_line"></a>
### `subtotal_line`

Found in the document rather than written down: a constant here was wrong
twice while these tests were being written, both times silently.

<storycode:copy id="f-134eb93d24b2-subtotal_line">
/// Found in the document rather than written down: a constant here was wrong
/// twice while these tests were being written, both times silently.
fn subtotal_line() -> u32 {
    DOC.lines()
        .position(|line| line.contains("double subtotal"))
        .expect("the document has that line") as u32
}

</storycode:copy>
<a id="f-134eb93d24b2-have"></a>
### `have`

<storycode:copy id="f-134eb93d24b2-have">
fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

</storycode:copy>
<a id="f-134eb93d24b2-borrow_this_repos_installs"></a>
### `borrow_this_repos_installs`

<storycode:copy id="f-134eb93d24b2-borrow_this_repos_installs">
#[cfg(unix)]
fn borrow_this_repos_installs(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if hick_dap::java::installed(&cache.join("..")).is_some() {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

</storycode:copy>
<a id="f-134eb93d24b2-borrow_this_repos_installs-2"></a>
### `borrow_this_repos_installs`

<storycode:copy id="f-134eb93d24b2-borrow_this_repos_installs-2">
#[cfg(not(unix))]
fn borrow_this_repos_installs(_into: &Path) {}

</storycode:copy>
<a id="f-134eb93d24b2-a_breakpoint_on_a_java_document_line_stops_in_the_jvm"></a>
### `a_breakpoint_on_a_java_document_line_stops_in_the_jvm`

<storycode:copy id="f-134eb93d24b2-a_breakpoint_on_a_java_document_line_stops_in_the_jvm">
#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_java_document_line_stops_in_the_jvm() {
    if !have("java") {
        eprintln!("SKIPPED: no `java` on this machine — hick installs the server, never a JDK");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.md"), DOC).expect("write the document");
    borrow_this_repos_installs(root);

    if hick_dap::java::installed(root).is_none() {
        eprintln!("SKIPPED: {}", hick_dap::how_to_get("java"));
        return;
    }

    let mapping = Arc::new(Mapping::for_document(Path::new("doc.md"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("Main.java is debuggable");
    assert!(entry.ends_with("Main.java"), "{entry:?}");

    let adapter = hick_dap::discover("java", root).expect("both halves are installed");
    assert!(
        adapter.hosted,
        "Java's adapter is asked for, not spawned: {:?}",
        adapter.command
    );
    assert!(
        adapter.command.is_empty(),
        "there is nothing to spawn: {:?}",
        adapter.command
    );

    // The expensive part, and the reason it is reported: this starts a JVM,
    // an OSGi framework, and a project import.
    let mut said: Vec<String> = Vec::new();
    let project = entry.parent().expect("the file's directory");
    let mut prepared = hickory_cli::java_debug::prepare(&entry, project, root, &mut |line| {
        if let hick_dap::BuildOutput::Out(t)
        | hick_dap::BuildOutput::Err(t)
        | hick_dap::BuildOutput::Note(t) = line
        {
            said.push(t);
        }
    })
    .await
    .unwrap_or_else(|e| panic!("preparing the session failed:\n{e:#}\n{}", said.join("\n")));

    let line = subtotal_line();
    let breakpoints = vec![Breakpoint {
        line,
        condition: None,
        hit_condition: None,
        log_message: None,
    }];
    let (session, statuses) = Session::start(
        Launch {
            // Nothing is spawned: the port came from the language server.
            transport: hick_dap::Transport::Attached(prepared.port),
            multi_session: false,
            adapter: Vec::new(),
            adapter_extra: prepared.launch.clone(),
            // java-debug takes a main class and a classpath, not a program.
            program: entry.clone(),
            cwd: project.to_path_buf(),
            extra: serde_json::json!({}),
        },
        mapping,
        &breakpoints,
    )
    .await
    .expect("the session starts");

    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].line, line);
    assert_ne!(
        statuses[0].state,
        hick_dap::BindState::Refused,
        "the adapter refused the breakpoint: {:?}",
        statuses[0]
    );

    let stopped = session
        .wait_for_stop(Duration::from_secs(90))
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
    assert_eq!(
        top.line,
        Some(line),
        "the frame is not on the document line: {top:?}"
    );

    let value = session
        .evaluate("quantity", Some(top.id), "watch")
        .await
        .expect("evaluating in the stopped frame");
    assert_eq!(
        value.value, "3",
        "argument read from the real frame: {value:?}"
    );

    session.shutdown().await;
    prepared.shutdown().await;
    eprintln!("OK java: asked the language server for a session, stopped, read a value");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/live_session_java.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/live_session_java.rs"><storycode:paste select="#f-134eb93d24b2-DOC"/><storycode:paste select="#f-134eb93d24b2-subtotal_line"/><storycode:paste select="#f-134eb93d24b2-have"/><storycode:paste select="#f-134eb93d24b2-borrow_this_repos_installs"/><storycode:paste select="#f-134eb93d24b2-borrow_this_repos_installs-2"/><storycode:paste select="#f-134eb93d24b2-a_breakpoint_on_a_java_document_line_stops_in_the_jvm"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-dap/src/adapter.rs](../../../mechanisms/crates/hick-dap/src/adapter.rs.md#output-2b5d1652c2d2)
- [crates/hick-dap/src/discovery.rs](../../../mechanisms/crates/hick-dap/src/discovery.rs.md#output-edc49b1ccd22)
- [crates/hick-dap/src/java.rs](contract.md#output-f0679f10df7f)
- [crates/hick-plugins/src/dap_discovery.rs](../../../mechanisms/crates/hick-plugins/src/dap_discovery.rs.md#output-72962e7f25e2)
- [crates/hick-plugins/src/java.rs](../../../mechanisms/crates/hick-plugins/src/java.rs.md#output-7df2afd7c160)
- [crates/hick-plugins/src/java_debug.rs](../../../mechanisms/crates/hick-plugins/src/java_debug.rs.md#output-2d734a88c587)
- [crates/hickory-cli/src/java_debug.rs](contract.md#output-3a77193c14e0)

### Tests and fixtures

- [crates/hickory-cli/tests/live_session_java.rs](contract.md#output-134eb93d24b2)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
