//! A real Go debug session, against delve, over a real document.
//!
//! Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
//!
//! `hick lang` reports Go as debuggable. Nothing had ever run it. Go is
//! compiled, but unlike C# and Rust its debugger does its own building —
//! `dlv dap` takes a launch request naming the package and compiles it —
//! so the interesting question is whether hick hands delve something it
//! accepts, not whether hick built anything.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in Go

Every line is quantity times unit price.

<hick:file path="app/go.mod">
module pricing

go 1.21
</hick:file>

<hick:file path="app/main.go">
package main

import "fmt"

func lineTotal(quantity int, unitPrice float64) float64 {
	subtotal := float64(quantity) * unitPrice
	return subtotal
}

func main() {
	fmt.Println(lineTotal(3, 1.25))
}
</hick:file>
</hick:doc>
"##;

/// 0-based document line of `\tsubtotal := float64(quantity) * unitPrice`.
const SUBTOTAL_LINE: u32 = 18;

fn have(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_go_document_line_stops_inside_the_program() {
    if !have("go") {
        eprintln!("SKIPPED: no go on this machine, so nothing can be built");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.hick"), DOC).expect("write the document");

    let mapping =
        Arc::new(Mapping::for_document(Path::new("doc.hick"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("main.go is debuggable");
    assert!(entry.ends_with("main.go"), "{entry:?}");

    let Some(adapter) = hick_dap::discover("go", root) else {
        eprintln!(
            "SKIPPED: no Go debug adapter on this machine.\n{}",
            hick_dap::how_to_get("go")
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

    let breakpoints = vec![Breakpoint {
        line: SUBTOTAL_LINE,
        condition: None,
        hit_condition: None,
        log_message: None,
    }];
    let (session, statuses) = Session::start(
        Launch {
            transport: adapter.transport,
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
    // Asserted, because echoing back the line that was asked for is not
    // evidence: a breakpoint on a blank line comes back looking identical
    // and then never fires.
    assert_ne!(
        statuses[0].state,
        hick_dap::BindState::Refused,
        "delve refused the breakpoint: {:?}",
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
    eprintln!("OK go: stopped on the document line, read a value");
}
