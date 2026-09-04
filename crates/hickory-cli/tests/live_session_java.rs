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

/// Found in the document rather than written down: a constant here was wrong
/// twice while these tests were being written, both times silently.
fn subtotal_line() -> u32 {
    DOC.lines()
        .position(|line| line.contains("double subtotal"))
        .expect("the document has that line") as u32
}

fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(unix)]
fn borrow_this_repos_installs(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if hick_dap::java::installed(&cache.join("..")).is_some() {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

#[cfg(not(unix))]
fn borrow_this_repos_installs(_into: &Path) {}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_java_document_line_stops_in_the_jvm() {
    if !have("java") {
        eprintln!("SKIPPED: no `java` on this machine — hick installs the server, never a JDK");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.hick"), DOC).expect("write the document");
    borrow_this_repos_installs(root);

    if hick_dap::java::installed(root).is_none() {
        eprintln!("SKIPPED: {}", hick_dap::how_to_get("java"));
        return;
    }

    let mapping =
        Arc::new(Mapping::for_document(Path::new("doc.hick"), DOC, root).expect("mapping"));
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
