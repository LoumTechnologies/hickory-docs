<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Plain File Has The Same Debugger A Document Has

For: Authors debugging generated code while navigating its document source.

Given a folder open in the app and a file in it that is not a `.md`
document — `src/main.rs`, `tools/app.py`, `index.ts`, `Program.cs` — in a
language hick can debug, when it is opened in a pane, then the pane has the
breakpoint gutter, one Debug icon beside the first line in the right gutter, and the same debugger
controls beside the paused line in the right gutter as a document; a breakpoint on line *n* is a breakpoint on line *n* of the file;
the program runs **in its own project**, built by that project's own tools
where the language needs a build; and while paused the pane shows the paused
line, the inline values, the stack, the watches, the hover value and the
inline eval exactly as a document's editor does. A frame in another file of
the folder is opened in that file's tab at the adapter's line.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-450dfe6078f4"></a>
## `crates/hick-dap/src/session/tests/tests.rs`

Related contracts: [A Plain File Has The Same Debugger A Document Has](contract.md).

<a id="f-450dfe6078f4-capabilities_are_read_from_the_adapters_own_answer"></a>
### `capabilities_are_read_from_the_adapters_own_answer`

<storycode:copy id="f-450dfe6078f4-capabilities_are_read_from_the_adapters_own_answer">
use super::*;

#[test]
fn capabilities_are_read_from_the_adapters_own_answer() {
    // Every control is gated on one of these, so reading them wrongly
    // means offering a button that does nothing.
    let caps = Capabilities::from_initialize(&json!({
        "supportsConditionalBreakpoints": true,
        "supportsRestartFrame": true,
        "supportsSetVariable": true,
        "exceptionBreakpointFilters": [
            { "filter": "raised", "label": "Raised Exceptions" },
            { "filter": "uncaught", "label": "Uncaught Exceptions" }
        ]
    }));
    assert!(caps.conditional_breakpoints);
    assert!(caps.restart_frame);
    assert!(caps.set_variable);
    // Absent means false, never "probably supported".
    assert!(!caps.step_back);
    assert!(!caps.log_points);
    assert_eq!(caps.exception_filters.len(), 2);
    assert_eq!(caps.exception_filters[0].id, "raised");
}

</storycode:copy>
<a id="f-450dfe6078f4-step_verbs_map_to_the_protocols_own_names"></a>
### `step_verbs_map_to_the_protocols_own_names`

<storycode:copy id="f-450dfe6078f4-step_verbs_map_to_the_protocols_own_names">
#[test]
fn step_verbs_map_to_the_protocols_own_names() {
    assert_eq!(Step::Over.command(), "next");
    assert_eq!(Step::In.command(), "stepIn");
    assert_eq!(Step::Out.command(), "stepOut");
    assert_eq!(Step::Continue.command(), "continue");
    assert_eq!(Step::DropFrame.command(), "restartFrame");
    assert_eq!(Step::Back.command(), "stepBack");
}

</storycode:copy>
<a id="f-450dfe6078f4-launch_arguments_can_be_overridden_per_language"></a>
### `launch_arguments_can_be_overridden_per_language`

<storycode:copy id="f-450dfe6078f4-launch_arguments_can_be_overridden_per_language">
#[test]
fn launch_arguments_can_be_overridden_per_language() {
    // `justMyCode` is right for Python and meaningless elsewhere, so the
    // defaults have to be overridable rather than final.
    let mut base = json!({ "program": "a.py", "justMyCode": true });
    merge(
        &mut base,
        &json!({ "justMyCode": false, "args": ["--fast"] }),
    );
    assert_eq!(base["justMyCode"], json!(false));
    assert_eq!(base["args"], json!(["--fast"]));
    assert_eq!(base["program"], json!("a.py"));
}

</storycode:copy>
<a id="f-450dfe6078f4-a_document_line_maps_into_the_file_it_generates"></a>
### `a_document_line_maps_into_the_file_it_generates`

<storycode:copy id="f-450dfe6078f4-a_document_line_maps_into_the_file_it_generates">
#[test]
fn a_document_line_maps_into_the_file_it_generates() {
    let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Title\n\n\
             <hick:file path=\"app.py\">\n\
             def total(x):\n\
             \x20   return x * 2\n\
             </hick:file>\n\
             </hick:doc>\n";
    let dir = tempfile::tempdir().unwrap();
    let mapping = Mapping::for_document(Path::new("d.md"), source, dir.path()).expect("maps");

    // The mapping and the bytes are one decision, so they are asserted
    // together: whatever `weave_into` writes is what the map describes.
    // A block's text begins with the newline after its opening tag, and
    // the file is written WITHOUT it — the engine's own weave writes no
    // such line, and a leading blank line moves whatever has to be at
    // byte 0 (a `#!`, a BOM, an XML declaration). Getting either half
    // wrong on its own puts every breakpoint one line from where it was
    // asked for.
    let written = crate::weave_into(source, dir.path()).expect("weaves");
    assert_eq!(written.len(), 1);
    let bytes = std::fs::read_to_string(&written[0]).unwrap();
    assert!(bytes.starts_with("def total(x):"), "{bytes:?}");

    // `def total(x):` is document line 5 (0-based), and line ZERO of the
    // file that was just written.
    let (path, line) = mapping.to_generated(5).expect("the code line maps");
    assert!(path.ends_with("app.py"), "{path:?}");
    assert_eq!(line, 0);
    assert_eq!(bytes.lines().nth(line as usize), Some("def total(x):"));

    // And back again, which is what a stack frame needs.
    assert_eq!(mapping.to_document(&path, 0), Some(5));
    assert_eq!(mapping.to_document(&path, 1), Some(6));
}

</storycode:copy>
<a id="f-450dfe6078f4-a_prose_line_maps_to_nothing"></a>
### `a_prose_line_maps_to_nothing`

<storycode:copy id="f-450dfe6078f4-a_prose_line_maps_to_nothing">
#[test]
fn a_prose_line_maps_to_nothing() {
    // A breakpoint on a heading is not a breakpoint the adapter can bind,
    // and saying so beats sending it and reporting `verified: false`
    // without explanation.
    let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Just prose here\n\
             </hick:doc>\n";
    let dir = tempfile::tempdir().unwrap();
    let mapping = Mapping::for_document(Path::new("d.md"), source, dir.path()).unwrap();
    assert_eq!(mapping.to_generated(2), None);
}

</storycode:copy>
<a id="f-450dfe6078f4-a_plain_file_maps_every_line_to_itself"></a>
### `a_plain_file_maps_every_line_to_itself`

<storycode:copy id="f-450dfe6078f4-a_plain_file_maps_every_line_to_itself">
#[test]
fn a_plain_file_maps_every_line_to_itself() {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    //
    // `src/main.rs` is not woven: the breakpoint goes on the line it was
    // asked for, in the file itself, and a frame in that file comes back
    // on the same line. A frame in ANOTHER file of the project is not
    // in this one — but it keeps its source, so the app can open it.
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.rs");
    let lib = dir.path().join("lib.rs");
    std::fs::write(&main, "fn main() {}\n").unwrap();
    std::fs::write(&lib, "pub fn f() {}\n").unwrap();
    let mapping = Mapping::identity(&main);
    assert!(mapping.is_identity());
    assert_eq!(mapping.to_generated(7), Some((main.clone(), 7)));
    assert_eq!(mapping.to_document(&main, 7), Some(7));
    // The adapter's own spelling of the same file still counts.
    let spelled = dir.path().join(".").join("main.rs");
    assert_eq!(mapping.to_document(&spelled, 3), Some(3));
    assert_eq!(mapping.to_document(&lib, 3), None);
    assert_eq!(mapping.generated_files(), vec![main]);
}
</storycode:copy>

### Assemble `crates/hick-dap/src/session/tests/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/session/tests/tests.rs"><storycode:paste select="#f-450dfe6078f4-capabilities_are_read_from_the_adapters_own_answer"/><storycode:paste select="#f-450dfe6078f4-step_verbs_map_to_the_protocols_own_names"/><storycode:paste select="#f-450dfe6078f4-launch_arguments_can_be_overridden_per_language"/><storycode:paste select="#f-450dfe6078f4-a_document_line_maps_into_the_file_it_generates"/><storycode:paste select="#f-450dfe6078f4-a_prose_line_maps_to_nothing"/><storycode:paste select="#f-450dfe6078f4-a_plain_file_maps_every_line_to_itself"/></storycode:file>

<a id="output-a283ca144066"></a>
## `crates/hick-plugins/src/build.rs`

Related contracts: [Launch the artifact built from the selected compiled project](../a-compiled-language-launches-what-a-build-produced/contract.md), [A Plain File Has The Same Debugger A Document Has](contract.md), [Use the adapter’s transport and the current measured language support](../an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-a283ca144066-BuildOutput"></a>
### `BuildOutput`

What must happen before there is a program to launch.
`hick-dap` assumed the generated file IS the program. That is true of
Python and Node, and true of Go only because delve compiles it on the way
past. `Program.cs` is not a program at all: netcoredbg launches
`bin/Debug/net10.0/app.dll`, which does not exist until something builds
it. The missing concept is **a build between the weave and the launch**,
and what it produces — not the file the author wrote — is what is
launched.
See `docs/specs/freeform/launching-what-a-document-builds.md`. Two things
that spec says about this module are worth repeating where the code is:
* **This is the table version, and the table is scaffolding.** The shape
underneath is that a compile is a cell whose output is a file, which
`hick:exec` plus volumes already models — the debugger should run the
DAG rather than bypass it. The table is here because it is small,
testable without touching the DAG, and makes C# debuggable today. The
spec names the signal to stop extending it: **the moment a second
compiled language needs a fourth field.**
* **Nothing here is sandboxed, and that is not new.** `Adapter::spawn`
already starts the adapter directly; the isolation on this path is the
scratch copy, not the executor that confines cells. A build step is no
*less* confined than the debuggee it feeds — but it does run a build
tool the document's own content chose, and this is the honest place to
say so.

<storycode:copy id="f-a283ca144066-BuildOutput">
//! What must happen before there is a program to launch.
//!
//! `hick-dap` assumed the generated file IS the program. That is true of
//! Python and Node, and true of Go only because delve compiles it on the way
//! past. `Program.cs` is not a program at all: netcoredbg launches
//! `bin/Debug/net10.0/app.dll`, which does not exist until something builds
//! it. The missing concept is **a build between the weave and the launch**,
//! and what it produces — not the file the author wrote — is what is
//! launched.
//!
//! See `docs/specs/freeform/launching-what-a-document-builds.md`. Two things
//! that spec says about this module are worth repeating where the code is:
//!
//! * **This is the table version, and the table is scaffolding.** The shape
//!   underneath is that a compile is a cell whose output is a file, which
//!   `hick:exec` plus volumes already models — the debugger should run the
//!   DAG rather than bypass it. The table is here because it is small,
//!   testable without touching the DAG, and makes C# debuggable today. The
//!   spec names the signal to stop extending it: **the moment a second
//!   compiled language needs a fourth field.**
//! * **Nothing here is sandboxed, and that is not new.** `Adapter::spawn`
//!   already starts the adapter directly; the isolation on this path is the
//!   scratch copy, not the executor that confines cells. A build step is no
//!   *less* confined than the debuggee it feeds — but it does run a build
//!   tool the document's own content chose, and this is the honest place to
//!   say so.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// One line of a build, as it happens.
///
/// Deliberately the same four cases as `TranscriptEvent`
/// (`Cmd`/`Out`/`Err`/`Exit`), so a caller can hand them straight to the
/// watching terminal without a second vocabulary. They are *not* that type:
/// a build is not a cell, its output is never recorded under a cache key,
/// and nothing is ever verified against it. The terminal is the run
/// happening; the transcript is the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildOutput {
    /// The command about to run.
    Cmd(String),
    /// A line this app is saying, not the build tool — the restore notice.
    Note(String),
    Out(String),
    Err(String),
    Exit(i32),
}

</storycode:copy>
<a id="f-a283ca144066-Build"></a>
### `Build`

What must happen before there is a program to launch.

<storycode:copy id="f-a283ca144066-Build">
/// What must happen before there is a program to launch.
#[derive(Debug, Clone, Copy)]
pub enum Build {
    /// The generated file IS the program. Nothing to do.
    None,
    /// Build in the scratch directory, then launch what the build wrote.
    Command {
        /// The file whose presence means "this is buildable", and whose
        /// DIRECTORY the build runs in. A scratch tree holds whatever the
        /// document generates, at whatever depth it chose — `dotnet build`
        /// at the root of a tree whose project is in `app/` fails with
        /// MSB1003 and nothing anybody can act on.
        project: &'static str,
        argv: &'static [&'static str],
        /// Where the artifact lands, relative to the project's directory.
        artifact: &'static str,
        /// What a person is told is missing when no `project` file exists.
        needs: &'static str,
    },
    /// One translation unit: the source itself IS the compiler's input.
    ///
    /// The spec said the signal to stop extending the `Command` table is a
    /// second compiled language needing a fourth field. C is not that: it
    /// needs one field FEWER. There is no universal C project file — a
    /// Makefile, a CMakeLists.txt and a bare `cc main.c` are all normal —
    /// and a one-file program is a complete program, which is exactly what a
    /// literate document generates. Inventing a CMakeLists.txt on somebody's
    /// behalf would be the same mistake as writing them a `.csproj`.
    Compile {
        /// Compilers to try, in order. The first one on PATH wins.
        compilers: &'static [&'static str],
        /// Flags that make the result debuggable at all: symbols, and no
        /// optimisation to step through.
        flags: &'static [&'static str],
    },
}

</storycode:copy>
<a id="f-a283ca144066-build_for"></a>
### `build_for`

<storycode:copy id="f-a283ca144066-build_for">
fn build_for(language: &str) -> Build {
    match crate::registry().interface(language, crate::InterfaceKind::Build) {
        Some(crate::Interface::Build { recipe }) => *recipe,
        _ => Build::None,
    }
}

</storycode:copy>
<a id="f-a283ca144066-first_on_path"></a>
### `first_on_path`

<storycode:copy id="f-a283ca144066-first_on_path">
/// The first of `compilers` on PATH.
fn first_on_path(compilers: &[&str]) -> Option<String> {
    let paths = std::env::var_os("PATH")?;
    for name in compilers {
        for dir in std::env::split_paths(&paths) {
            if dir.join(name).is_file() {
                return Some((*name).to_string());
            }
        }
    }
    None
}

</storycode:copy>
<a id="f-a283ca144066-compile_one"></a>
### `compile_one`

Compile one source file into something a debugger can launch.
The binary lands beside the other build outputs this app owns, under
`.hick-cache/`, for the reason cargo's target directory does: a scratch
tree is deleted with the session, and a debugger that is still holding
the binary it is stepping through should not be racing that.

<storycode:copy id="f-a283ca144066-compile_one">
/// Compile one source file into something a debugger can launch.
///
/// The binary lands beside the other build outputs this app owns, under
/// `.hick-cache/`, for the reason cargo's target directory does: a scratch
/// tree is deleted with the session, and a debugger that is still holding
/// the binary it is stepping through should not be racing that.
async fn compile_one(
    source: &Path,
    compilers: &'static [&'static str],
    flags: &'static [&'static str],
    cache_root: &Path,
    display_root: &Path,
    on_output: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<PathBuf> {
    let name = source.file_name().unwrap_or_default().to_string_lossy();
    let compiler = first_on_path(compilers).with_context(|| {
        format!(
            "{name} is compiled, and none of {} is on this machine's PATH, so there is nothing              to build it with.
Next step: install a C toolchain — `apt install build-essential`,              `xcode-select --install`, or your platform's equivalent.",
            compilers.join(", ")
        )
    })?;
    let out_dir = cache_root.join(".hick-cache/cc-out");
    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let stem = source.file_stem().unwrap_or_default();
    let out = out_dir.join(stem);

    let mut argv: Vec<String> = vec![compiler];
    argv.extend(flags.iter().map(|f| (*f).to_string()));
    argv.push("-o".into());
    argv.push(out.to_string_lossy().into_owned());
    argv.push(source.to_string_lossy().into_owned());

    on_output(BuildOutput::Cmd(argv.join(" ")));
    let borrowed: Vec<&str> = argv.iter().map(String::as_str).collect();
    let dir = source.parent().unwrap_or(display_root);
    let code = run(&borrowed, dir, &BuildEnv::Own, on_output).await?;
    on_output(BuildOutput::Exit(code));
    if code != 0 {
        bail!(
            "compiling {} failed (exit {code}). The compiler's own output says why — it named \
             the file and the line, and that is what to read.",
            display_relative(source, display_root)
        );
    }
    Ok(out)
}

</storycode:copy>
<a id="f-a283ca144066-artifact_root"></a>
### `artifact_root`

Where a build's artifact lands, for `artifact` to be read relative to.
`dotnet build` writes under the project; `cargo build` writes to a target
directory this app points OUTSIDE the scratch tree, so a session's build
survives the session — a scratch copy is deleted with it, and rebuilding
every dependency on every debug session is a way to lose ten minutes.

<storycode:copy id="f-a283ca144066-artifact_root">
/// Where a build's artifact lands, for `artifact` to be read relative to.
///
/// `dotnet build` writes under the project; `cargo build` writes to a target
/// directory this app points OUTSIDE the scratch tree, so a session's build
/// survives the session — a scratch copy is deleted with it, and rebuilding
/// every dependency on every debug session is a way to lose ten minutes.
fn artifact_root(language: &str, project_dir: &Path, cache_root: &Path) -> PathBuf {
    match language {
        "rust" => cache_root.join(".hick-cache/cargo-target"),
        _ => project_dir.to_path_buf(),
    }
}

</storycode:copy>
<a id="f-a283ca144066-artifact_stem"></a>
### `artifact_stem`

What the program is called, from the project file that names it.
A .NET assembly is named after its project file; a cargo binary is named
after the package, which is inside `Cargo.toml` rather than on it.

<storycode:copy id="f-a283ca144066-artifact_stem">
/// What the program is called, from the project file that names it.
///
/// A .NET assembly is named after its project file; a cargo binary is named
/// after the package, which is inside `Cargo.toml` rather than on it.
fn artifact_stem(project_file: &Path) -> std::ffi::OsString {
    if project_file.file_name().is_some_and(|n| n == "Cargo.toml")
        && let Ok(text) = std::fs::read_to_string(project_file)
        && let Some(name) = cargo_package_name(&text)
    {
        return name.into();
    }
    project_file.file_stem().unwrap_or_default().to_owned()
}

</storycode:copy>
<a id="f-a283ca144066-cargo_package_name"></a>
### `cargo_package_name`

`name = "…"` under `[package]`, by a line scan.
A scan and not a TOML parser, deliberately: this crate takes no TOML
dependency for one key, and a manifest that fools a line scan — a
`name` inside a multi-line string — is a manifest nobody has written.

<storycode:copy id="f-a283ca144066-cargo_package_name">
/// `name = "…"` under `[package]`, by a line scan.
///
/// A scan and not a TOML parser, deliberately: this crate takes no TOML
/// dependency for one key, and a manifest that fools a line scan — a
/// `name` inside a multi-line string — is a manifest nobody has written.
fn cargo_package_name(text: &str) -> Option<String> {
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = line.strip_prefix("name")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

</storycode:copy>
<a id="f-a283ca144066-is_compiled"></a>
### `is_compiled`

Whether a language needs a build before it has a program.
For reporting only — the answer that matters is what [`build`] returns.

<storycode:copy id="f-a283ca144066-is_compiled">
/// Whether a language needs a build before it has a program.
///
/// For reporting only — the answer that matters is what [`build`] returns.
pub fn is_compiled(language: &str) -> bool {
    matches!(build_for(language), Build::Command { .. })
}

</storycode:copy>
<a id="f-a283ca144066-build"></a>
### `build`

Turn the entry-point **source** file into the **program** to launch.
`source` is what `entry_point` chose and what `adapter_for` read the
language from; both are unchanged, deliberately, because the source is
what has a language. What that source *produces* is what gets launched,
and for Python, Node and Go the two are the same path.
`cache_root` is the real project directory — the one holding the `.md`
document — not the scratch copy. Package caches have to survive a session,
and the scratch tree is deleted with it.

<storycode:copy id="f-a283ca144066-build">
/// Turn the entry-point **source** file into the **program** to launch.
///
/// `source` is what `entry_point` chose and what `adapter_for` read the
/// language from; both are unchanged, deliberately, because the source is
/// what has a language. What that source *produces* is what gets launched,
/// and for Python, Node and Go the two are the same path.
///
/// `cache_root` is the real project directory — the one holding the `.md`
/// document — not the scratch copy. Package caches have to survive a session,
/// and the scratch tree is deleted with it.
pub async fn build(
    source: &Path,
    scratch: &Path,
    cache_root: &Path,
    on_output: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<PathBuf> {
    let Some(language) =
        crate::routing::language_id(&source.to_string_lossy()).filter(|language| {
            crate::registry()
                .interface(language, crate::InterfaceKind::Dap)
                .is_some()
        })
    else {
        return Ok(source.to_path_buf());
    };
    let build = build_for(language);
    if let Build::Compile { compilers, flags } = build {
        return compile_one(source, compilers, flags, cache_root, scratch, on_output).await;
    }
    let Build::Command {
        project,
        argv,
        artifact,
        needs,
    } = build
    else {
        return Ok(source.to_path_buf());
    };

    let name = source.file_name().unwrap_or_default().to_string_lossy();
    // Never invent a project file. A scratch directory with a `.cs` and no
    // `.csproj` cannot be built, and writing a plausible one on somebody's
    // behalf produces a program that is not theirs.
    let project_file = find_one(scratch, project).with_context(|| {
        format!(
            "{name} is {language}, which is compiled: the debugger launches the assembly a build \
             produces, not the source you wrote.\n\n\
             This document generates no project file ({project}), so there is nothing to build.\n\
             Next step: generate one — {needs} — and the debugger will build it."
        )
    })?;
    let dir = project_file.parent().unwrap_or(scratch).to_path_buf();

    on_output(BuildOutput::Cmd(format!(
        "{} ({})",
        argv.join(" "),
        display_relative(&project_file, scratch)
    )));

    // Never fetch silently. `dotnet build` restores from the network by
    // default, and the debug path has never needed the network before — so
    // it is said out loud, in the terminal, where the person is already
    // looking, and BEFORE it happens rather than in a log afterwards.
    //
    // The wording is "may fetch", not "fetching", and that is not hedging.
    // Whether a restore reaches the network cannot be known before running
    // it: a console app with no `PackageReference` resolves entirely from
    // the SDK's own reference assemblies and downloads nothing at all —
    // measured, not assumed. Announcing a fetch that does not happen is the
    // same class of dishonesty as not announcing one that does.
    //
    // The directory is created here so the notice is once-per-project rather
    // than once-per-build: `dotnet` only creates it when something actually
    // restores into it, so keying the notice on its existence alone would
    // repeat "this happens once" on every build of a project that never
    // fetches anything.
    let packages = cache_root.join(".hick-cache/dotnet/nuget");
    if !packages.exists() {
        on_output(BuildOutput::Note(format!(
            "packages restore into {}; the first build of a project may fetch from the network",
            display_relative(&packages, cache_root)
        )));
        std::fs::create_dir_all(&packages)
            .with_context(|| format!("creating {}", packages.display()))?;
    }
    let out_dir = artifact_root(language, &dir, cache_root);
    if language == "rust" && !out_dir.exists() {
        // cargo's registry cache is the user's own (`~/.cargo`); what this
        // app redirects is only where the build lands.
        on_output(BuildOutput::Note(format!(
            "cargo builds into {}; the first build of a project may fetch crates from the network",
            display_relative(&out_dir, cache_root)
        )));
    }

    let env = BuildEnv::Confined {
        packages,
        cli_home: cache_root.join(".hick-cache/dotnet"),
        out_dir: out_dir.clone(),
    };
    finish(
        argv,
        &dir,
        &env,
        &project_file,
        scratch,
        &out_dir,
        artifact,
        on_output,
    )
    .await
}

</storycode:copy>
<a id="f-a283ca144066-build_plain"></a>
### `build_plain`

Build a file that is not a document — `src/main.rs`, `Program.cs` — the
way the person's own tools would, and return what to launch.
The plain-file sibling of [`build`], and it differs in exactly the ways
a plain file differs from a document:
* **The project is the nearest one above the file**, not the one file
in a scratch tree. A repository holds many `Cargo.toml`s, and the one
that owns `crates/foo/src/main.rs` is the closest — the same rule the
run-test gutter uses to pick a directory. Never below the folder the
app opened: a manifest outside it is outside the project.
* **It builds in place, into the project's own output.** No scratch
copy — copying a checkout with its `target/` per session is not a
cost anyone would pay — and no redirected `CARGO_TARGET_DIR`, so the
build shares its incremental state with the person's own `cargo
build`. Where cargo puts it is asked of cargo (`cargo metadata`),
because a workspace member's target directory is the workspace's.
* **Nothing is invented.** A `.cs` with no `.csproj` above it is refused
by name, exactly as for a document; the fix is named too.

<storycode:copy id="f-a283ca144066-build_plain">
/// Build a file that is not a document — `src/main.rs`, `Program.cs` — the
/// way the person's own tools would, and return what to launch.
///
/// The plain-file sibling of [`build`], and it differs in exactly the ways
/// a plain file differs from a document:
///
/// * **The project is the nearest one above the file**, not the one file
///   in a scratch tree. A repository holds many `Cargo.toml`s, and the one
///   that owns `crates/foo/src/main.rs` is the closest — the same rule the
///   run-test gutter uses to pick a directory. Never below the folder the
///   app opened: a manifest outside it is outside the project.
/// * **It builds in place, into the project's own output.** No scratch
///   copy — copying a checkout with its `target/` per session is not a
///   cost anyone would pay — and no redirected `CARGO_TARGET_DIR`, so the
///   build shares its incremental state with the person's own `cargo
///   build`. Where cargo puts it is asked of cargo (`cargo metadata`),
///   because a workspace member's target directory is the workspace's.
/// * **Nothing is invented.** A `.cs` with no `.csproj` above it is refused
///   by name, exactly as for a document; the fix is named too.
pub async fn build_plain(
    source: &Path,
    root: &Path,
    on_output: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<PathBuf> {
    let Some(language) =
        crate::routing::language_id(&source.to_string_lossy()).filter(|language| {
            crate::registry()
                .interface(language, crate::InterfaceKind::Dap)
                .is_some()
        })
    else {
        return Ok(source.to_path_buf());
    };
    let Build::Command {
        project,
        argv,
        artifact,
        ..
    } = build_for(language)
    else {
        return Ok(source.to_path_buf());
    };

    let name = source.file_name().unwrap_or_default().to_string_lossy();
    let project_file = nearest_matching(root, source, project).with_context(|| {
        format!(
            "{name} is {language}, which is compiled: the debugger launches what a build \
             produces, not the source you wrote.\n\n\
             No project file ({project}) was found above {name} inside {}, so there is \
             nothing to build.\n\
             Next step: open the folder that holds the project, or add a {project} beside \
             the code and the debugger will build it.",
            root.display()
        )
    })?;
    let dir = project_file.parent().unwrap_or(root).to_path_buf();

    on_output(BuildOutput::Cmd(format!(
        "{} ({})",
        argv.join(" "),
        display_relative(&project_file, root)
    )));
    let out_dir = match language {
        "rust" => cargo_target_dir(&dir).await,
        _ => dir.clone(),
    };
    finish(
        argv,
        &dir,
        &BuildEnv::Own,
        &project_file,
        root,
        &out_dir,
        artifact,
        on_output,
    )
    .await
}

</storycode:copy>
<a id="f-a283ca144066-finish"></a>
### `finish`

Run a build whose project has been found, and locate what it wrote.

<storycode:copy id="f-a283ca144066-finish">
/// Run a build whose project has been found, and locate what it wrote.
#[allow(clippy::too_many_arguments)]
async fn finish(
    argv: &[&str],
    dir: &Path,
    env: &BuildEnv,
    project_file: &Path,
    display_root: &Path,
    out_dir: &Path,
    artifact: &str,
    on_output: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<PathBuf> {
    let code = run(argv, dir, env, on_output).await?;
    on_output(BuildOutput::Exit(code));
    if code != 0 {
        // Deliberately neutral about WHERE the output is: this same error
        // reaches a terminal in the app, a log under `hick test`, and a tool
        // result in MCP. Each surface says where its copy is; naming one of
        // them here would be wrong on the other two.
        bail!(
            "building {} failed (exit {code}). The build's own output says why — the compiler \
             named the file and the line, and that is what to read.",
            display_relative(project_file, display_root)
        );
    }

    locate_artifact(out_dir, artifact, project_file)
}

</storycode:copy>
<a id="f-a283ca144066-cargo_target_dir"></a>
### `cargo_target_dir`

Where cargo puts this package's build, asked of cargo itself.
`<project>/target` is only right for a package that is its own
workspace; a member builds into the workspace root's `target/`, and a
`.cargo/config.toml` can move it anywhere. `cargo metadata` answers all
three, and falls back to the plain guess when it cannot run at all — the
build that follows will say why in its own words.

<storycode:copy id="f-a283ca144066-cargo_target_dir">
/// Where cargo puts this package's build, asked of cargo itself.
///
/// `<project>/target` is only right for a package that is its own
/// workspace; a member builds into the workspace root's `target/`, and a
/// `.cargo/config.toml` can move it anywhere. `cargo metadata` answers all
/// three, and falls back to the plain guess when it cannot run at all — the
/// build that follows will say why in its own words.
async fn cargo_target_dir(dir: &Path) -> PathBuf {
    let output = tokio::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .await;
    if let Ok(output) = output
        && output.status.success()
        && let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        && let Some(target) = metadata.get("target_directory").and_then(|v| v.as_str())
    {
        return PathBuf::from(target);
    }
    dir.join("target")
}

</storycode:copy>
<a id="f-a283ca144066-nearest_matching"></a>
### `nearest_matching`

The nearest ancestor of `file` (its own directory first, `root` last)
holding a file matching `pattern`.
Sorted within a directory, like [`find_one`], so two project files side
by side pick the same one every time.

<storycode:copy id="f-a283ca144066-nearest_matching">
/// The nearest ancestor of `file` (its own directory first, `root` last)
/// holding a file matching `pattern`.
///
/// Sorted within a directory, like [`find_one`], so two project files side
/// by side pick the same one every time.
fn nearest_matching(root: &Path, file: &Path, pattern: &str) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .map(|n| wildcard(pattern, &n.to_string_lossy()))
                        .unwrap_or(false)
            })
            .collect();
        found.sort();
        if let Some(first) = found.into_iter().next() {
            return Some(first);
        }
        if dir == root {
            return None;
        }
        dir = dir.parent()?;
    }
}

</storycode:copy>
<a id="f-a283ca144066-BuildEnv"></a>
### `BuildEnv`

The environment a build runs with, beyond the tool's own defaults.
Two shapes, because the two things built here belong to different
people. A **document's** build is this app's: it runs in a scratch copy,
its package cache and its artifacts are redirected under `.hick-cache/`,
and the .NET CLI's own writes to `$HOME` are redirected with them — the
same three redirects `hick lsp install csharp` makes, for the same
reason: this product says nothing to anyone, and a tool it spawns on the
user's behalf must not be the exception. A **plain file's** build is the
person's own: their `cargo build`, in their checkout, into their
`target/`, with their NuGet cache — redirecting any of it would make the
debugger build a second copy of the project beside the one their own
tools use. Telemetry stays off in both.

<storycode:copy id="f-a283ca144066-BuildEnv">
/// The environment a build runs with, beyond the tool's own defaults.
///
/// Two shapes, because the two things built here belong to different
/// people. A **document's** build is this app's: it runs in a scratch copy,
/// its package cache and its artifacts are redirected under `.hick-cache/`,
/// and the .NET CLI's own writes to `$HOME` are redirected with them — the
/// same three redirects `hick lsp install csharp` makes, for the same
/// reason: this product says nothing to anyone, and a tool it spawns on the
/// user's behalf must not be the exception. A **plain file's** build is the
/// person's own: their `cargo build`, in their checkout, into their
/// `target/`, with their NuGet cache — redirecting any of it would make the
/// debugger build a second copy of the project beside the one their own
/// tools use. Telemetry stays off in both.
enum BuildEnv {
    Confined {
        packages: PathBuf,
        cli_home: PathBuf,
        out_dir: PathBuf,
    },
    Own,
}

</storycode:copy>
<a id="f-a283ca144066-run"></a>
### `run`

Run the build, streaming both streams as they arrive.

<storycode:copy id="f-a283ca144066-run">
/// Run the build, streaming both streams as they arrive.
async fn run(
    argv: &[&str],
    dir: &Path,
    env: &BuildEnv,
    on_output: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<i32> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut command = tokio::process::Command::new(argv[0]);
    command
        .args(&argv[1..])
        .current_dir(dir)
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
    if let BuildEnv::Confined {
        packages,
        cli_home,
        out_dir,
    } = env
    {
        command
            .env("NUGET_PACKAGES", packages)
            .env("DOTNET_CLI_HOME", cli_home)
            // Harmless to dotnet; the whole point for cargo. See
            // `artifact_root`.
            .env("CARGO_TARGET_DIR", out_dir);
    }
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());

    let mut child = command.spawn().with_context(|| {
        format!(
            "could not run `{}`. Install the toolchain that provides it, or debug a language that \
             needs no build.",
            argv[0]
        )
    })?;
    let mut out = BufReader::new(child.stdout.take().expect("piped")).lines();
    let mut err = BufReader::new(child.stderr.take().expect("piped")).lines();

    loop {
        tokio::select! {
            line = out.next_line() => match line? {
                Some(line) => on_output(BuildOutput::Out(line)),
                None => break,
            },
            line = err.next_line() => match line? {
                Some(line) => on_output(BuildOutput::Err(line)),
                None => break,
            },
        }
    }
    // Whichever stream ended first, drain the other: a build tool that writes
    // its errors to stderr and finishes stdout early would otherwise have the
    // one thing worth reading dropped.
    while let Some(line) = out.next_line().await? {
        on_output(BuildOutput::Out(line));
    }
    while let Some(line) = err.next_line().await? {
        on_output(BuildOutput::Err(line));
    }

    Ok(child.wait().await?.code().unwrap_or(-1))
}

</storycode:copy>
<a id="f-a283ca144066-locate_artifact"></a>
### `locate_artifact`

The artifact the build wrote.
The glob can match more than one file — a project with dependencies
copies them next to its own assembly — so the project's own stem wins,
which is the assembly name .NET uses unless a project overrides it. A
project that DID override it, with several candidates and no stem match,
is a refusal that lists them rather than a guess.

<storycode:copy id="f-a283ca144066-locate_artifact">
/// The artifact the build wrote.
///
/// The glob can match more than one file — a project with dependencies
/// copies them next to its own assembly — so the project's own stem wins,
/// which is the assembly name .NET uses unless a project overrides it. A
/// project that DID override it, with several candidates and no stem match,
/// is a refusal that lists them rather than a guess.
fn locate_artifact(dir: &Path, pattern: &str, project_file: &Path) -> Result<PathBuf> {
    let stem = artifact_stem(project_file);
    let mut found: Vec<PathBuf> = glob(dir, pattern)
        .into_iter()
        // Files, and not cargo's `.d` dependency notes beside them, which
        // share the binary's stem and are not programs.
        .filter(|p| p.is_file() && p.extension().is_none_or(|e| e != "d"))
        .collect();
    found.sort();
    if let Some(exact) = found.iter().find(|p| p.file_stem() == Some(&stem)) {
        return Ok(exact.clone());
    }
    match found.len() {
        0 => bail!(
            "the build reported success but wrote nothing matching {pattern} under {}. Nothing \
             can be launched, so this is a bug in hick's build step rather than in your document.",
            dir.display()
        ),
        1 => Ok(found.remove(0)),
        _ => bail!(
            "the build wrote several candidates and none is named after {}, so which one is the \
             program is a guess: {}. Name the project's assembly after the project file, or debug \
             it by naming the program explicitly.",
            project_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            found
                .iter()
                .map(|p| p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

</storycode:copy>
<a id="f-a283ca144066-find_one"></a>
### `find_one`

The one file under `root` matching `pattern`, or `None`.
Sorted, so a tree with two project files picks the same one every time
rather than whatever the filesystem happened to hand back first.

<storycode:copy id="f-a283ca144066-find_one">
/// The one file under `root` matching `pattern`, or `None`.
///
/// Sorted, so a tree with two project files picks the same one every time
/// rather than whatever the filesystem happened to hand back first.
fn find_one(root: &Path, pattern: &str) -> Option<PathBuf> {
    let mut found = walk(root)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .map(|n| wildcard(pattern, &n.to_string_lossy()))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    found.sort();
    found.into_iter().next()
}

</storycode:copy>
<a id="f-a283ca144066-glob"></a>
### `glob`

Paths under `dir` matching a `/`-separated pattern whose segments may
contain `*`.
Enough of a glob for the table above and no more: `*` matches within one
path component and never across a separator, so `bin/Debug/*/*.dll` finds
an assembly under an unknown target-framework directory without wandering
off into `obj/`.

<storycode:copy id="f-a283ca144066-glob">
/// Paths under `dir` matching a `/`-separated pattern whose segments may
/// contain `*`.
///
/// Enough of a glob for the table above and no more: `*` matches within one
/// path component and never across a separator, so `bin/Debug/*/*.dll` finds
/// an assembly under an unknown target-framework directory without wandering
/// off into `obj/`.
fn glob(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut level = vec![dir.to_path_buf()];
    for segment in pattern.split('/') {
        let mut next = Vec::new();
        for base in &level {
            let Ok(entries) = std::fs::read_dir(base) else {
                continue;
            };
            for entry in entries.flatten() {
                if wildcard(segment, &entry.file_name().to_string_lossy()) {
                    next.push(entry.path());
                }
            }
        }
        level = next;
    }
    level
}

</storycode:copy>
<a id="f-a283ca144066-walk"></a>
### `walk`

<storycode:copy id="f-a283ca144066-walk">
/// Every file under `root`, recursively.
fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

</storycode:copy>
<a id="f-a283ca144066-wildcard"></a>
### `wildcard`

`*` matches any run of characters, including none. Nothing else is
special — these patterns are ours, not a user's.

<storycode:copy id="f-a283ca144066-wildcard">
/// `*` matches any run of characters, including none. Nothing else is
/// special — these patterns are ours, not a user's.
fn wildcard(pattern: &str, name: &str) -> bool {
    let mut parts = pattern.split('*');
    let Some(first) = parts.next() else {
        return false;
    };
    if !name.starts_with(first) {
        return false;
    }
    let mut rest = &name[first.len()..];
    let tail: Vec<&str> = parts.collect();
    if tail.is_empty() {
        return rest.is_empty();
    }
    for (i, part) in tail.iter().enumerate() {
        if i == tail.len() - 1 {
            return rest.len() >= part.len() && rest.ends_with(part);
        }
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    true
}

</storycode:copy>
<a id="f-a283ca144066-display_relative"></a>
### `display_relative`

<storycode:copy id="f-a283ca144066-display_relative">
fn display_relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

</storycode:copy>
<a id="f-a283ca144066-tests"></a>
### `tests`

<storycode:copy id="f-a283ca144066-tests">
#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;

</storycode:copy>
<a id="f-a283ca144066-csharp"></a>
### `csharp`

The C# build, run for real.
Gated on `dotnet` being present rather than skipped silently: a test that
returns before its first assertion on a machine without the toolchain is a
green suite covering nothing, and this repository has been bitten by that
exact shape before (`crates/hick-term/tests/full_screen_apps.rs`).

<storycode:copy id="f-a283ca144066-csharp">
/// The C# build, run for real.
///
/// Gated on `dotnet` being present rather than skipped silently: a test that
/// returns before its first assertion on a machine without the toolchain is a
/// green suite covering nothing, and this repository has been bitten by that
/// exact shape before (`crates/hick-term/tests/full_screen_apps.rs`).
#[cfg(test)]
mod csharp {
    use super::*;

    fn have_dotnet() -> bool {
        std::process::Command::new("dotnet")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    const CSPROJ: &str = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>
"#;

    #[tokio::test]
    async fn a_document_that_generates_a_project_builds_and_yields_an_assembly() {
        if !have_dotnet() {
            eprintln!("skipped: dotnet is not installed on this machine");
            return;
        }
        let scratch = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        // The shape `scaffolding.md` generates: the project is NOT at the
        // root of the scratch tree, which is the case `dotnet build` at the
        // root fails on with MSB1003.
        let app = scratch.path().join("app");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(app.join("app.csproj"), CSPROJ).unwrap();
        let source = app.join("Program.cs");
        std::fs::write(
            &source,
            "class Program { static void Main() { System.Console.WriteLine(\"hi\"); } }\n",
        )
        .unwrap();

        let mut seen: Vec<BuildOutput> = Vec::new();
        let program = build(&source, scratch.path(), cache.path(), &mut |o| seen.push(o))
            .await
            .expect("the build should succeed");

        // What is launched is the assembly, not the source that was handed in.
        assert_ne!(program, source);
        assert_eq!(program.extension().unwrap(), "dll");
        assert_eq!(program.file_stem().unwrap(), "app");
        assert!(program.exists(), "{program:?} does not exist");
        assert!(program.starts_with(&app), "{program:?} escaped the project");

        // The person was told what ran, and told about the fetch BEFORE it
        // happened rather than in a log afterwards.
        assert!(
            matches!(seen.first(), Some(BuildOutput::Cmd(_))),
            "{seen:?}"
        );
        assert!(
            seen.iter().any(
                |o| matches!(o, BuildOutput::Note(n) if n.contains("may fetch from the network"))
            ),
            "the restore was silent: {seen:?}"
        );
        assert_eq!(seen.last(), Some(&BuildOutput::Exit(0)));
        // And the packages landed where they were said to.
        assert!(cache.path().join(".hick-cache/dotnet/nuget").exists());
    }

    #[tokio::test]
    async fn a_build_that_fails_keeps_the_tools_own_words() {
        if !have_dotnet() {
            eprintln!("skipped: dotnet is not installed on this machine");
            return;
        }
        let scratch = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let app = scratch.path().join("app");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(app.join("app.csproj"), CSPROJ).unwrap();
        let source = app.join("Program.cs");
        std::fs::write(&source, "class Program { this is not C# }\n").unwrap();

        let mut seen: Vec<BuildOutput> = Vec::new();
        let error = build(&source, scratch.path(), cache.path(), &mut |o| seen.push(o))
            .await
            .unwrap_err();

        // The error points at the output rather than replacing it — the
        // compiler already said which line and why, and a spinner's worth of
        // summary would throw that away.
        assert!(
            format!("{error}").contains("named the file and the line"),
            "{error}"
        );
        let words: String = seen
            .iter()
            .filter_map(|o| match o {
                BuildOutput::Out(l) | BuildOutput::Err(l) => Some(l.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            words.contains("error CS"),
            "no compiler error surfaced:\n{words}"
        );
    }

    #[test]
    fn the_nearest_project_file_above_a_plain_file_wins() {
        // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
        //
        // A repository holds many manifests; the one that owns a file is the
        // closest above it — and never one above the folder the app opened.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        std::fs::create_dir_all(root.join("crates/foo/src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        std::fs::write(root.join("crates/foo/Cargo.toml"), "").unwrap();
        let file = root.join("crates/foo/src/main.rs");
        std::fs::write(&file, "").unwrap();
        assert_eq!(
            nearest_matching(&root, &file, "Cargo.toml"),
            Some(root.join("crates/foo/Cargo.toml"))
        );
        let orphan = root.join("scripts/x.rs");
        std::fs::create_dir_all(orphan.parent().unwrap()).unwrap();
        std::fs::write(&orphan, "").unwrap();
        assert_eq!(
            nearest_matching(&root, &orphan, "Cargo.toml"),
            Some(root.join("Cargo.toml"))
        );
        // Above the root is outside the project, even when a manifest sits there.
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("Cargo.toml"), "").unwrap();
        let sub = outside.path().join("a");
        std::fs::create_dir_all(&sub).unwrap();
        let file = sub.join("main.rs");
        std::fs::write(&file, "").unwrap();
        assert_eq!(nearest_matching(&sub, &file, "Cargo.toml"), None);
    }

    #[tokio::test]
    async fn a_plain_interpreted_file_is_its_own_program() {
        // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("app.py");
        std::fs::write(&file, "print(1)\n").unwrap();
        let mut seen = Vec::new();
        let program = build_plain(&file, dir.path(), &mut |line| seen.push(line))
            .await
            .unwrap();
        assert_eq!(program, file);
        assert!(
            seen.is_empty(),
            "nothing to build, nothing to say: {seen:?}"
        );
    }

    #[tokio::test]
    async fn a_plain_compiled_file_with_no_project_is_refused_by_name() {
        // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Program.cs");
        std::fs::write(&file, "").unwrap();
        let error = build_plain(&file, dir.path(), &mut |_| {})
            .await
            .expect_err("nothing to build");
        let text = format!("{error:#}");
        assert!(text.contains("Program.cs is csharp"), "{text}");
        assert!(text.contains("*.csproj"), "{text}");
        assert!(text.contains("Next step"), "{text}");
    }

    #[test]
    fn a_cargo_package_is_named_by_its_manifest() {
        assert_eq!(
            cargo_package_name(
                "[package]\nname = \"pricing\"\nversion = \"0.1.0\"\n[dependencies]\nname = \"not-this\"\n"
            ),
            Some("pricing".to_string())
        );
        assert_eq!(cargo_package_name("[workspace]\nmembers = []\n"), None);
        assert!(is_compiled("rust"));
    }
}
</storycode:copy>

### Assemble `crates/hick-plugins/src/build.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/build.rs"><storycode:paste select="#f-a283ca144066-BuildOutput"/><storycode:paste select="#f-a283ca144066-Build"/><storycode:paste select="#f-a283ca144066-build_for"/><storycode:paste select="#f-a283ca144066-first_on_path"/><storycode:paste select="#f-a283ca144066-compile_one"/><storycode:paste select="#f-a283ca144066-artifact_root"/><storycode:paste select="#f-a283ca144066-artifact_stem"/><storycode:paste select="#f-a283ca144066-cargo_package_name"/><storycode:paste select="#f-a283ca144066-is_compiled"/><storycode:paste select="#f-a283ca144066-build"/><storycode:paste select="#f-a283ca144066-build_plain"/><storycode:paste select="#f-a283ca144066-finish"/><storycode:paste select="#f-a283ca144066-cargo_target_dir"/><storycode:paste select="#f-a283ca144066-nearest_matching"/><storycode:paste select="#f-a283ca144066-BuildEnv"/><storycode:paste select="#f-a283ca144066-run"/><storycode:paste select="#f-a283ca144066-locate_artifact"/><storycode:paste select="#f-a283ca144066-find_one"/><storycode:paste select="#f-a283ca144066-glob"/><storycode:paste select="#f-a283ca144066-walk"/><storycode:paste select="#f-a283ca144066-wildcard"/><storycode:paste select="#f-a283ca144066-display_relative"/><storycode:paste select="#f-a283ca144066-tests"/><storycode:paste select="#f-a283ca144066-csharp"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/debug/cmDebug.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-9df55a31763b)
- [apps/web/src/debug/plainDebugHosts.tsx](../launch-profiles-choose-targets-within-the-viewed-project/contract.md#output-238f66c701b0)
- [apps/web/src/debug/useDebugger.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-597e991b795b)
- [apps/web/src/lib/pausedElsewhere.ts](../../../mechanisms/apps/web/src/lib/pausedElsewhere.ts.md#output-f89ec0ad0c79)
- [crates/hick-dap/src/build.rs](../../../mechanisms/crates/hick-dap/src/build.rs.md#output-0fdc81f45247)
- [crates/hick-dap/src/session.rs](../../../mechanisms/crates/hick-dap/src/session.rs.md#output-3c41c22e0551)
- [crates/hick-plugins/src/build.rs](contract.md#output-a283ca144066)
- [crates/hickory-cli/src/debug_sessions.rs](../../../mechanisms/crates/hickory-cli/src/debug_sessions.rs.md#output-24a9413a429b)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)
- [crates/hickory-cli/src/serve/socket.rs](../../../mechanisms/crates/hickory-cli/src/serve/socket.rs.md#output-ce0f0e119d3c)

### Tests and fixtures

- [apps/web/src/components/PlainFilePane.test.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-d923b2fdae48)
- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/debug/cmDebug.test.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-8d12c7278102)
- [apps/web/src/debug/useDebugger.test.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-101aa3d58173)
- [crates/hick-dap/src/session/tests/tests.rs](contract.md#output-450dfe6078f4)
- [crates/hickory-cli/tests/debug_desktop.rs](../../../mechanisms/crates/hickory-cli/tests/debug_desktop.rs.md#output-2ee357f70dc3)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
