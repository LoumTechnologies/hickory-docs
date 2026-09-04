//! A real C# debug session, against netcoredbg, over a real document.
//!
//! Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
//!
//! This is the test the build step existed for and could not have. Its whole
//! reason to be is the step every other language skips: what is launched here
//! is **not** the file the document generated. `Program.cs` is source;
//! netcoredbg launches `app.dll`, and the breakpoint a person set on a
//! document line still has to arrive in the right frame.
//!
//! Skipped loudly, and separately, for each of the two things it needs — the
//! .NET SDK to build and netcoredbg to debug — because "skipped" and "passed"
//! must never look alike.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in C#

Every line is quantity times unit price.

<hick:file path="app/app.csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
  </PropertyGroup>
</Project>
</hick:file>

<hick:file path="app/Program.cs">
class Program
{
    static decimal LineTotal(int quantity, decimal unitPrice)
    {
        decimal subtotal = quantity * unitPrice;
        return subtotal;
    }

    static void Main()
    {
        System.Console.WriteLine(LineTotal(3, 1.25m));
    }
}
</hick:file>
</hick:doc>
"##;

/// 0-based document line of `        decimal subtotal = quantity * unitPrice;`.
///
/// Counted from `<?xml` as line 0. One line out is not a rounding error: the
/// line above is the method signature, which is a different frame.
const SUBTOTAL_LINE: u32 = 20;

/// Point the scratch project at the adapter this repository installed.
///
/// A developer runs `hick dap install csharp` once, at the top of this repo;
/// a test project in a temp directory is nowhere near it. Without this the
/// test skips on the machine most likely to be running it, which is how a
/// suite ends up green while testing nothing — the same borrow
/// `live_session.rs` does for debugpy.
#[cfg(unix)]
fn borrow_this_repos_adapters(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache.join("adapters/netcoredbg/netcoredbg").exists() {
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
async fn a_breakpoint_on_a_csharp_document_line_stops_inside_the_assembly() {
    if !have("dotnet") {
        eprintln!("SKIPPED: no .NET SDK on this machine, so nothing can be built");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.hick"), DOC).expect("write the document");

    let mapping =
        Arc::new(Mapping::for_document(Path::new("doc.hick"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("Program.cs is debuggable");

    // Discovery BEFORE the build, the way the real path orders it: a machine
    // with no netcoredbg should not spend a build finding that out.
    borrow_this_repos_adapters(root);
    let Some(adapter) = hick_dap::discover("csharp", root) else {
        eprintln!(
            "SKIPPED: no C# debug adapter on this machine.\n{}",
            hick_dap::how_to_get("csharp")
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

    // The claim this whole change exists to make.
    assert_ne!(program, entry, "the source was launched, not the assembly");
    assert!(program.ends_with("app.dll"), "{program:?}");

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
    // **netcoredbg does not verify at set time**, and this asserts that
    // rather than working around it. debugpy answers `setBreakpoints` with
    // `verified: true`; netcoredbg answers "The breakpoint is pending and
    // will be resolved when debugging starts" and binds it when the module
    // loads. Both then stop the program in the same place — the rest of this
    // test is the proof.
    //
    // It matters because the gutter draws an unverified breakpoint hollow,
    // which is how it says "this will not bind". Under netcoredbg every C#
    // breakpoint would wear that mark for the life of the session while
    // working perfectly. Nothing here consumes DAP's `breakpoint` EVENT, so
    // the later verification is never seen. See the guarantee's caveats.
    assert_eq!(
        statuses[0].state,
        hick_dap::BindState::Pending,
        "netcoredbg confirmed at set time — if this now happens, the promotion \
         below is dead code rather than the fix: {statuses:?}"
    );

    let stopped = session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .expect("waiting works")
        .expect("the program stopped");
    assert_eq!(stopped.reason, "breakpoint");

    // The stack comes back in DOCUMENT coordinates, through the same
    // `Mapping` every other language uses — via a pdb this time, which is
    // the half no unit test could reach.
    let frames = session.stack(stopped.thread_id).await.expect("a stack");
    let top = frames.first().expect("a top frame");
    assert!(
        top.name.contains("LineTotal"),
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
    assert_eq!(value.value, "3", "argument read from the real frame");

    // The promotion: netcoredbg sent a `breakpoint` event when the module
    // loaded, and the status the app reads has moved from pending to bound.
    // Without this the gutter draws every C# breakpoint half-filled for the
    // life of a session while the program stops on it perfectly.
    let settled = session.breakpoint_statuses();
    assert_eq!(
        settled
            .iter()
            .find(|s| s.line == SUBTOTAL_LINE)
            .map(|s| s.state),
        Some(hick_dap::BindState::Bound),
        "the breakpoint never left `pending` even though the program stopped on it: {settled:?}"
    );
}
