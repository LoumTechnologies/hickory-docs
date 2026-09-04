//! A real Rust debug session, against codelldb or lldb-dap, over a real
//! document.
//!
//! Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
//!
//! The second compiled language. What is launched is the binary `cargo build`
//! wrote into the app's target directory, not `src/main.rs`; the breakpoint a
//! person set on a document line has to arrive in the right frame through
//! DWARF rather than a pdb. Skipped loudly for each thing it needs — cargo to
//! build, an adapter to debug — because "skipped" and "passed" must never
//! look alike.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in Rust

Every line is quantity times unit price.

<hick:file path="app/Cargo.toml">
[package]
name = "pricing"
version = "0.1.0"
edition = "2021"

[dependencies]
</hick:file>

<hick:file path="app/src/main.rs">
fn line_total(quantity: u32, unit_price: f64) -> f64 {
    let subtotal = quantity as f64 * unit_price;
    subtotal
}

fn main() {
    println!("{}", line_total(3, 1.25));
}
</hick:file>
</hick:doc>
"##;

/// 0-based document line of `    let subtotal = quantity as f64 * unit_price;`.
const SUBTOTAL_LINE: u32 = 17;

/// Point the scratch project at the adapter this repository installed, so a
/// developer who ran `hick dap install rust` once at the top of the repo
/// exercises this file instead of skipping it.
#[cfg(unix)]
fn borrow_this_repos_adapters(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache
        .join("adapters/codelldb/extension/adapter/codelldb")
        .exists()
    {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_rust_document_line_stops_inside_the_binary() {
    if !have("cargo") {
        eprintln!("SKIPPED: no cargo on this machine, so nothing can be built");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.hick"), DOC).expect("write the document");

    let mapping =
        Arc::new(Mapping::for_document(Path::new("doc.hick"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("main.rs is debuggable");
    assert!(entry.ends_with("main.rs"), "{entry:?}");

    borrow_this_repos_adapters(root);
    let Some(adapter) = hick_dap::discover("rust", root) else {
        eprintln!(
            "SKIPPED: no Rust debug adapter on this machine.\n{}",
            hick_dap::how_to_get("rust")
        );
        return;
    };
    eprintln!("using adapter {} ({})", adapter.adapter, adapter.origin);

    let mut said: Vec<String> = Vec::new();
    let program = hick_dap::build(&entry, root, root, &mut |line| {
        if let hick_dap::BuildOutput::Out(t) | hick_dap::BuildOutput::Err(t) = line {
            said.push(t);
        }
    })
    .await
    .unwrap_or_else(|e| panic!("the build failed:\n{e:#}\n{}", said.join("\n")));

    // The claim this test exists to make: the binary, not the source.
    assert_ne!(program, entry, "the source was launched, not the binary");
    assert!(
        program
            .file_name()
            .is_some_and(|n| n == "pricing" || n == "pricing.exe"),
        "{program:?}"
    );
    assert!(
        program.starts_with(root.join(".hick-cache/cargo-target")),
        "the build did not land in the app's target directory: {program:?}"
    );

    let breakpoints = vec![Breakpoint {
        line: SUBTOTAL_LINE,
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
            program: program.clone(),
            cwd: program.parent().unwrap().to_path_buf(),
            extra: serde_json::json!({}),
        },
        mapping,
        &breakpoints,
    )
    .await
    .expect("the session starts");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].line, SUBTOTAL_LINE);

    let stopped = session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .expect("waiting works")
        .expect("the program stopped");
    assert_eq!(stopped.reason, "breakpoint", "{stopped:?}");

    let frames = session.stack(stopped.thread_id).await.expect("a stack");
    let top = frames.first().expect("a top frame");
    assert!(
        top.name.contains("line_total"),
        "stopped in the wrong frame: {top:?}"
    );
    assert_eq!(
        top.line,
        Some(SUBTOTAL_LINE),
        "the frame is not on the document line the breakpoint was set on: {top:?}"
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
    eprintln!("OK rust: built with cargo, stopped on the document line, read a value");
}
