<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Launch the artifact built from the selected compiled project

For: Authors debugging generated code while navigating its document source.

Debug a Rust or C# program described by the current project. The debugger must receive the selected build artifact rather than a guessed source-file stem.

## What must happen

Compiled-language launch uses the language's declared build step, artifact location and executable selection. Rust builds in its manifest directory, uses the package manifest to identify the program, and excludes dependency-note files from executable selection. Adapter discovery prefers a supported installed adapter; catalogue installation can supply supported archive-based C# and Rust adapters. Build failure retains actual diagnostics rather than starting a different program.

## Limits of the evidence

A successful build is not behavioral verification. Platform adapter availability and live stop/source mapping need independent witnesses.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold Rust/codelldb and the available C# archive installer into the earlier compiled-language contract. Retire the obsolete claim that C# has no installer.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-66164abc20c9"></a>
## `apps/web/src/debug/languages.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Launch the artifact built from the selected compiled project](contract.md).

<a id="f-66164abc20c9-DEBUGGABLE_LANGUAGES"></a>
### `DEBUGGABLE_LANGUAGES`

Debug availability comes from the same language plugins as the native host.

<storycode:copy id="f-66164abc20c9-DEBUGGABLE_LANGUAGES">
// Debug availability comes from the same language plugins as the native host.
import { DEBUGGABLE_LANGUAGE_IDS } from "../editor/generated/languages";

export const DEBUGGABLE_LANGUAGES = new Set(DEBUGGABLE_LANGUAGE_IDS);

</storycode:copy>
<a id="f-66164abc20c9-isDebuggable"></a>
### `isDebuggable`

<storycode:copy id="f-66164abc20c9-isDebuggable">
export function isDebuggable(language: string | null | undefined): boolean {
  return !!language && DEBUGGABLE_LANGUAGES.has(language);
}
</storycode:copy>

### Assemble `apps/web/src/debug/languages.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/languages.ts"><storycode:paste select="#f-66164abc20c9-DEBUGGABLE_LANGUAGES"/><storycode:paste select="#f-66164abc20c9-isDebuggable"/></storycode:file>

<a id="output-a2654634fd43"></a>
## `crates/hick-dap/tests/csharp_document.rs`

Related contracts: [Launch the artifact built from the selected compiled project](contract.md).

<a id="f-a2654634fd43-DOC"></a>
### `DOC`

A document that generates C#, taken from source to launchable program.
The unit tests in `build.rs` drive `build` directly. This drives the path
the app drives: weave the document into a scratch directory, pick the
entry point, and turn it into the thing a debugger launches. The step
being checked is the one `docs/specs/freeform/launching-what-a-document-builds.md`
is about — **`hick-dap` assumed the generated file IS the program**, which
is false for every compiled language.
It stops short of starting an adapter, and that boundary is honest rather
than convenient: netcoredbg is not installed on the machine this was
written on, and pretending otherwise is what the spec's fourth refusal is
about.

<storycode:copy id="f-a2654634fd43-DOC">
//! A document that generates C#, taken from source to launchable program.
//!
//! The unit tests in `build.rs` drive `build` directly. This drives the path
//! the app drives: weave the document into a scratch directory, pick the
//! entry point, and turn it into the thing a debugger launches. The step
//! being checked is the one `docs/specs/freeform/launching-what-a-document-builds.md`
//! is about — **`hick-dap` assumed the generated file IS the program**, which
//! is false for every compiled language.
//!
//! It stops short of starting an adapter, and that boundary is honest rather
//! than convenient: netcoredbg is not installed on the machine this was
//! written on, and pretending otherwise is what the spec's fourth refusal is
//! about.

use std::path::Path;

const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="o.md">

# A console app

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
    static void Main()
    {
        System.Console.WriteLine("hello");
    }
}
</hick:file>

</hick:doc>
"#;

</storycode:copy>
<a id="f-a2654634fd43-have_dotnet"></a>
### `have_dotnet`

<storycode:copy id="f-a2654634fd43-have_dotnet">
fn have_dotnet() -> bool {
    std::process::Command::new("dotnet")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

</storycode:copy>
<a id="f-a2654634fd43-the_program_a_csharp_document_launches_is_the_assembly_not_the_source"></a>
### `the_program_a_csharp_document_launches_is_the_assembly_not_the_source`

<storycode:copy id="f-a2654634fd43-the_program_a_csharp_document_launches_is_the_assembly_not_the_source">
#[tokio::test]
async fn the_program_a_csharp_document_launches_is_the_assembly_not_the_source() {
    if !have_dotnet() {
        eprintln!("skipped: dotnet is not installed on this machine");
        return;
    }
    let scratch = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();

    let files = hick_dap::weave_into(DOC, scratch.path()).expect("the document weaves");
    // The `.csproj` is generated too, and it is not debuggable — the entry
    // point has to walk past it to the `.cs`.
    assert!(files.iter().any(|f| f.ends_with("app/app.csproj")));
    let entry = hick_dap::entry_point(&files).expect("Program.cs is debuggable");
    assert!(entry.ends_with("app/Program.cs"), "{entry:?}");
    assert_eq!(hick_dap::language_of(&entry), Some("csharp"));

    let mut lines: Vec<String> = Vec::new();
    let program = hick_dap::build(&entry, scratch.path(), cache.path(), &mut |line| {
        if let hick_dap::BuildOutput::Out(text) | hick_dap::BuildOutput::Err(text) = line {
            lines.push(text);
        }
    })
    .await
    .unwrap_or_else(|e| panic!("the build failed:\n{e:#}\n{}", lines.join("\n")));

    // The assumption this whole change exists to remove: what is launched is
    // NOT the generated file.
    assert_ne!(program, entry);
    assert!(program.ends_with("app.dll"), "{program:?}");
    assert!(program.exists(), "{program:?} was not written");
    // And it came from the project's own directory, not the scratch root —
    // `dotnet build` at the root of a tree whose project is in `app/` fails
    // with MSB1003 and nothing anybody can act on.
    assert!(
        program.starts_with(scratch.path().join("app")),
        "{program:?}"
    );
}

</storycode:copy>
<a id="f-a2654634fd43-a_csharp_document_with_no_project_file_refuses_and_writes_nothing"></a>
### `a_csharp_document_with_no_project_file_refuses_and_writes_nothing`

<storycode:copy id="f-a2654634fd43-a_csharp_document_with_no_project_file_refuses_and_writes_nothing">
#[tokio::test]
async fn a_csharp_document_with_no_project_file_refuses_and_writes_nothing() {
    // No SDK needed: the refusal happens before anything runs.
    let scratch = tempfile::tempdir().unwrap();
    let doc = DOC.replace(
        &DOC[DOC.find("<hick:file path=\"app/app.csproj\">").unwrap()
            ..DOC.find("</hick:file>").unwrap() + "</hick:file>\n".len()],
        "",
    );
    let files = hick_dap::weave_into(&doc, scratch.path()).expect("weaves");
    let entry = hick_dap::entry_point(&files).expect("Program.cs is still debuggable");

    let error = hick_dap::build(&entry, scratch.path(), scratch.path(), &mut |_| {})
        .await
        .expect_err("a C# document with no project file cannot be built");
    let text = format!("{error:#}");
    assert!(text.contains("which is compiled"), "{text}");
    assert!(text.contains("hick ingest"), "{text}");
    // Never invent a project file.
    assert!(!scratch.path().join("app/app.csproj").exists());
    assert!(!list(scratch.path()).iter().any(|p| p.ends_with(".csproj")));
}

</storycode:copy>
<a id="f-a2654634fd43-list"></a>
### `list`

<storycode:copy id="f-a2654634fd43-list">
fn list(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path.display().to_string());
            }
        }
    }
    out
}
</storycode:copy>

### Assemble `crates/hick-dap/tests/csharp_document.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/csharp_document.rs"><storycode:paste select="#f-a2654634fd43-DOC"/><storycode:paste select="#f-a2654634fd43-have_dotnet"/><storycode:paste select="#f-a2654634fd43-the_program_a_csharp_document_launches_is_the_assembly_not_the_source"/><storycode:paste select="#f-a2654634fd43-a_csharp_document_with_no_project_file_refuses_and_writes_nothing"/><storycode:paste select="#f-a2654634fd43-list"/></storycode:file>

<a id="output-c4d5afae8600"></a>
## `crates/hick-dap/tests/live_session_c.rs`

Related contracts: [Launch the artifact built from the selected compiled project](contract.md), [Use the adapter’s transport and the current measured language support](../an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-c4d5afae8600-DOC"></a>
### `DOC`

A real C debug session, against codelldb or lldb-dap, over a real document.
Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
`hick lang` reports C and C++ as debuggable — Silver — because an adapter
is discoverable for them. This is the test that says whether that is true.

<storycode:copy id="f-c4d5afae8600-DOC">
//! A real C debug session, against codelldb or lldb-dap, over a real document.
//!
//! Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
//!
//! `hick lang` reports C and C++ as debuggable — Silver — because an adapter
//! is discoverable for them. This is the test that says whether that is true.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in C

Every line is quantity times unit price.

<hick:file path="app/main.c">
#include <stdio.h>

double line_total(int quantity, double unit_price) {
    double subtotal = quantity * unit_price;
    return subtotal;
}

int main(void) {
    printf("%f\n", line_total(3, 1.25));
    return 0;
}
</hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-c4d5afae8600-SUBTOTAL_LINE"></a>
### `SUBTOTAL_LINE`

0-based document line of `    double subtotal = quantity * unit_price;`.

<storycode:copy id="f-c4d5afae8600-SUBTOTAL_LINE">
/// 0-based document line of `    double subtotal = quantity * unit_price;`.
const SUBTOTAL_LINE: u32 = 10;

</storycode:copy>
<a id="f-c4d5afae8600-borrow_this_repos_adapters"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-c4d5afae8600-borrow_this_repos_adapters">
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

</storycode:copy>
<a id="f-c4d5afae8600-borrow_this_repos_adapters-2"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-c4d5afae8600-borrow_this_repos_adapters-2">
#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

</storycode:copy>
<a id="f-c4d5afae8600-have"></a>
### `have`

<storycode:copy id="f-c4d5afae8600-have">
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
<a id="f-c4d5afae8600-a_breakpoint_on_a_c_document_line_stops_inside_the_binary"></a>
### `a_breakpoint_on_a_c_document_line_stops_inside_the_binary`

<storycode:copy id="f-c4d5afae8600-a_breakpoint_on_a_c_document_line_stops_inside_the_binary">
#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_c_document_line_stops_inside_the_binary() {
    if !have("cc") && !have("gcc") {
        eprintln!("SKIPPED: no C compiler on this machine, so nothing can be built");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.md"), DOC).expect("write the document");

    let mapping = Arc::new(Mapping::for_document(Path::new("doc.md"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("main.c is debuggable");
    assert!(entry.ends_with("main.c"), "{entry:?}");

    borrow_this_repos_adapters(root);
    let Some(adapter) = hick_dap::discover("c", root) else {
        eprintln!(
            "SKIPPED: no C debug adapter on this machine.\n{}",
            hick_dap::how_to_get("c")
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

    // The claim: the executable, not the source. A `.c` file handed to a
    // debugger as a program is not a program.
    assert_ne!(program, entry, "the source was launched, not a binary");

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
    eprintln!("OK c: compiled, stopped on the document line, read a value");
}
</storycode:copy>

### Assemble `crates/hick-dap/tests/live_session_c.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/live_session_c.rs"><storycode:paste select="#f-c4d5afae8600-DOC"/><storycode:paste select="#f-c4d5afae8600-SUBTOTAL_LINE"/><storycode:paste select="#f-c4d5afae8600-borrow_this_repos_adapters"/><storycode:paste select="#f-c4d5afae8600-borrow_this_repos_adapters-2"/><storycode:paste select="#f-c4d5afae8600-have"/><storycode:paste select="#f-c4d5afae8600-a_breakpoint_on_a_c_document_line_stops_inside_the_binary"/></storycode:file>

<a id="output-e8bb12be9677"></a>
## `crates/hick-dap/tests/live_session_cpp.rs`

Related contracts: [Launch the artifact built from the selected compiled project](contract.md), [Use the adapter’s transport and the current measured language support](../an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-e8bb12be9677-DOC"></a>
### `DOC`

A real C++ debug session, against codelldb or lldb-dap, over a real document.
Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
C++ is not C, and the difference is a whole row of the build table: its
own compilers (`c++`, `g++`, `clang++` rather than `cc`, `gcc`, `clang`)
and its own source extension. The C suite exercised neither, so C++
claimed a debugger that nothing had ever driven — the same shape as the
four false claims of 2026-09-04, one language further along.

<storycode:copy id="f-e8bb12be9677-DOC">
//! A real C++ debug session, against codelldb or lldb-dap, over a real document.
//!
//! Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
//!
//! C++ is not C, and the difference is a whole row of the build table: its
//! own compilers (`c++`, `g++`, `clang++` rather than `cc`, `gcc`, `clang`)
//! and its own source extension. The C suite exercised neither, so C++
//! claimed a debugger that nothing had ever driven — the same shape as the
//! four false claims of 2026-09-04, one language further along.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, Launch, Mapping, Session};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing, in C++

Every line is quantity times unit price.

<hick:file path="app/main.cpp">
#include <iostream>

double line_total(int quantity, double unit_price) {
    double subtotal = quantity * unit_price;
    return subtotal;
}

int main() {
    std::cout << line_total(3, 1.25) << std::endl;
    return 0;
}
</hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-e8bb12be9677-SUBTOTAL_LINE"></a>
### `SUBTOTAL_LINE`

0-based document line of `    double subtotal = quantity * unit_price;`.

<storycode:copy id="f-e8bb12be9677-SUBTOTAL_LINE">
/// 0-based document line of `    double subtotal = quantity * unit_price;`.
const SUBTOTAL_LINE: u32 = 10;

</storycode:copy>
<a id="f-e8bb12be9677-borrow_this_repos_adapters"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-e8bb12be9677-borrow_this_repos_adapters">
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

</storycode:copy>
<a id="f-e8bb12be9677-borrow_this_repos_adapters-2"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-e8bb12be9677-borrow_this_repos_adapters-2">
#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

</storycode:copy>
<a id="f-e8bb12be9677-have"></a>
### `have`

<storycode:copy id="f-e8bb12be9677-have">
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
<a id="f-e8bb12be9677-a_breakpoint_on_a_cpp_document_line_stops_inside_the_binary"></a>
### `a_breakpoint_on_a_cpp_document_line_stops_inside_the_binary`

<storycode:copy id="f-e8bb12be9677-a_breakpoint_on_a_cpp_document_line_stops_inside_the_binary">
#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_cpp_document_line_stops_inside_the_binary() {
    if !have("c++") && !have("g++") && !have("clang++") {
        eprintln!("SKIPPED: no C++ compiler on this machine, so nothing can be built");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.md"), DOC).expect("write the document");

    let mapping = Arc::new(Mapping::for_document(Path::new("doc.md"), DOC, root).expect("mapping"));
    let files = hick_dap::weave_into(DOC, root).expect("the document weaves");
    let entry = hick_dap::entry_point(&files).expect("main.c is debuggable");
    assert!(entry.ends_with("main.cpp"), "{entry:?}");

    borrow_this_repos_adapters(root);
    let Some(adapter) = hick_dap::discover("cpp", root) else {
        eprintln!(
            "SKIPPED: no C++ debug adapter on this machine.\n{}",
            hick_dap::how_to_get("cpp")
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

    // The claim: the executable, not the source. A `.c` file handed to a
    // debugger as a program is not a program.
    assert_ne!(program, entry, "the source was launched, not a binary");

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
    eprintln!("OK cpp: compiled, stopped on the document line, read a value");
}
</storycode:copy>

### Assemble `crates/hick-dap/tests/live_session_cpp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/live_session_cpp.rs"><storycode:paste select="#f-e8bb12be9677-DOC"/><storycode:paste select="#f-e8bb12be9677-SUBTOTAL_LINE"/><storycode:paste select="#f-e8bb12be9677-borrow_this_repos_adapters"/><storycode:paste select="#f-e8bb12be9677-borrow_this_repos_adapters-2"/><storycode:paste select="#f-e8bb12be9677-have"/><storycode:paste select="#f-e8bb12be9677-a_breakpoint_on_a_cpp_document_line_stops_inside_the_binary"/></storycode:file>

<a id="output-d2e66d639bf7"></a>
## `crates/hick-dap/tests/live_session_rust.rs`

Related contracts: [Launch the artifact built from the selected compiled project](contract.md).

<a id="f-d2e66d639bf7-DOC"></a>
### `DOC`

A real Rust debug session, against codelldb or lldb-dap, over a real
document.
Protects docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md
The second compiled language. What is launched is the binary `cargo build`
wrote into the app's target directory, not `src/main.rs`; the breakpoint a
person set on a document line has to arrive in the right frame through
DWARF rather than a pdb. Skipped loudly for each thing it needs — cargo to
build, an adapter to debug — because "skipped" and "passed" must never
look alike.

<storycode:copy id="f-d2e66d639bf7-DOC">
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

</storycode:copy>
<a id="f-d2e66d639bf7-SUBTOTAL_LINE"></a>
### `SUBTOTAL_LINE`

0-based document line of `    let subtotal = quantity as f64 * unit_price;`.

<storycode:copy id="f-d2e66d639bf7-SUBTOTAL_LINE">
/// 0-based document line of `    let subtotal = quantity as f64 * unit_price;`.
const SUBTOTAL_LINE: u32 = 17;

</storycode:copy>
<a id="f-d2e66d639bf7-borrow_this_repos_adapters"></a>
### `borrow_this_repos_adapters`

Point the scratch project at the adapter this repository installed, so a
developer who ran `hick dap install rust` once at the top of the repo
exercises this file instead of skipping it.

<storycode:copy id="f-d2e66d639bf7-borrow_this_repos_adapters">
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

</storycode:copy>
<a id="f-d2e66d639bf7-borrow_this_repos_adapters-2"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-d2e66d639bf7-borrow_this_repos_adapters-2">
#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

</storycode:copy>
<a id="f-d2e66d639bf7-have"></a>
### `have`

<storycode:copy id="f-d2e66d639bf7-have">
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
<a id="f-d2e66d639bf7-a_breakpoint_on_a_rust_document_line_stops_inside_the_binary"></a>
### `a_breakpoint_on_a_rust_document_line_stops_inside_the_binary`

<storycode:copy id="f-d2e66d639bf7-a_breakpoint_on_a_rust_document_line_stops_inside_the_binary">
#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_rust_document_line_stops_inside_the_binary() {
    if !have("cargo") {
        eprintln!("SKIPPED: no cargo on this machine, so nothing can be built");
        return;
    }
    let dir = tempfile::tempdir().expect("scratch");
    let root = dir.path();
    std::fs::write(root.join("doc.md"), DOC).expect("write the document");

    let mapping = Arc::new(Mapping::for_document(Path::new("doc.md"), DOC, root).expect("mapping"));
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
</storycode:copy>

### Assemble `crates/hick-dap/tests/live_session_rust.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/live_session_rust.rs"><storycode:paste select="#f-d2e66d639bf7-DOC"/><storycode:paste select="#f-d2e66d639bf7-SUBTOTAL_LINE"/><storycode:paste select="#f-d2e66d639bf7-borrow_this_repos_adapters"/><storycode:paste select="#f-d2e66d639bf7-borrow_this_repos_adapters-2"/><storycode:paste select="#f-d2e66d639bf7-have"/><storycode:paste select="#f-d2e66d639bf7-a_breakpoint_on_a_rust_document_line_stops_inside_the_binary"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/DebugStrip.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-09306f4ec1a5)
- [apps/web/src/debug/languages.ts](contract.md#output-66164abc20c9)
- [crates/hick-dap/src/build.rs](../../../mechanisms/crates/hick-dap/src/build.rs.md#output-0fdc81f45247)
- [crates/hick-dap/src/capture.rs](../../../mechanisms/crates/hick-dap/src/capture.rs.md#output-9b1310a1716d)
- [crates/hick-dap/src/discovery.rs](../../../mechanisms/crates/hick-dap/src/discovery.rs.md#output-edc49b1ccd22)
- [crates/hick-lsp/src/lang_detect.rs](../../../mechanisms/crates/hick-lsp/src/lang_detect.rs.md#output-0053bd574cca)
- [crates/hick-plugins/src/build.rs](../a-plain-file-has-the-same-debugger/contract.md#output-a283ca144066)
- [crates/hick-plugins/src/dap_discovery.rs](../../../mechanisms/crates/hick-plugins/src/dap_discovery.rs.md#output-72962e7f25e2)
- [crates/hick-plugins/src/routing.rs](../../../mechanisms/crates/hick-plugins/src/routing.rs.md#output-007f87027669)
- [crates/hickory-cli/src/dap_install.rs](../../../mechanisms/crates/hickory-cli/src/dap_install.rs.md#output-a1687642fec0)
- [crates/hickory-cli/src/debug_sessions.rs](../../../mechanisms/crates/hickory-cli/src/debug_sessions.rs.md#output-24a9413a429b)
- [crates/hickory-cli/src/mcp.rs](../../../mechanisms/crates/hickory-cli/src/mcp.rs.md#output-5cbce5960257)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)

### Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/debug/cmDebug.test.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-8d12c7278102)
- [crates/hick-dap/tests/csharp_document.rs](contract.md#output-a2654634fd43)
- [crates/hick-dap/tests/live_session_c.rs](contract.md#output-c4d5afae8600)
- [crates/hick-dap/tests/live_session_cpp.rs](contract.md#output-e8bb12be9677)
- [crates/hick-dap/tests/live_session_csharp.rs](../../../mechanisms/crates/hick-dap/tests/live_session_csharp.rs.md#output-095fece59d06)
- [crates/hick-dap/tests/live_session_go.rs](../../../mechanisms/crates/hick-dap/tests/live_session_go.rs.md#output-eb0b9ddf7573)
- [crates/hick-dap/tests/live_session_rust.rs](contract.md#output-d2e66d639bf7)
- [crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs](../../../support/files/crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs.md)
- [crates/hick-term/tests/full_screen_apps.rs](../../../mechanisms/crates/hick-term/tests/full_screen_apps.rs.md#output-31c9ccf64945)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
