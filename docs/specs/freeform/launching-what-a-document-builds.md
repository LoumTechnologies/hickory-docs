# Launching what a document builds

A debugger for C# is not a C# feature. It is the discovery that **`hick-dap`
assumes the generated file IS the program**, which is true of Python, Node and
Go and false of every compiled language there is. C# is just the first one to
walk into it.

## The assumption, named

`program.rs::entry_point` picks the first generated file in a debuggable
language and hands that path to the adapter as `program`. `app.py` is a
program. `app.js` is a program. `main.go` is a program *because delve compiles
it for you*. `Program.cs` is not a program at all — netcoredbg launches
`bin/Debug/net10.0/app.dll`, an artifact that does not exist until something
builds it.

So the missing concept is **a build between the weave and the launch**, and
the thing it produces — not the file the author wrote — is what gets launched.

## The precedent this is not inventing

VS Code hit this exact wall and solved it in the open. A debug extension there
contributes three things, and hick already has one and a half of them:

| VS Code | hick | state |
|---|---|---|
| a debug adapter binary | `Recipe` in `hick-dap`'s `discovery.rs` | **has it** |
| `program` in `launch.json` | `Launch::program` | has it, pointed at the wrong file for compiled languages |
| **`preLaunchTask`** in `launch.json` + a `tasks.json` entry | — | **missing; this is the whole gap** |

The C# extension's generated `launch.json` says `"preLaunchTask": "build"` and
`"program": "${workspaceFolder}/bin/Debug/net8.0/app.dll"`. That pair — run
this first, then launch *that* — is the shape. Copying a solved design beats
inventing one, and it means every compiled language's answer already exists in
public as a `tasks.json` somebody maintains.

## Where the analogy breaks, which is where the work is

Three differences, and each one removes something VS Code leans on:

1. **There is no `launch.json`, and there must not be.** VS Code's answer is a
   file the user writes; this product's whole editor-intelligence family is
   *found without being configured*
   (`docs/guarantees/editor-intelligence/a-language-server-is-found-without-being-configured.md`).
   A build step that needs a config file to exist is not a feature here, it is
   a regression. **The build must be derived.**
2. **The program is woven into a scratch directory, not opened in a
   workspace.** `program.rs` writes the document's files to a temp dir so the
   debuggee runs against a copy. A build needs everything a build needs — for
   C#, a `.csproj` beside the `.cs`, and a package cache — and the scratch
   directory holds only what the document generates. A document with no
   project file has nothing to build, and that must be a clear refusal rather
   than an obscure build error.
3. **The debug path is not sandboxed.** Worth stating plainly because it is
   easy to assume otherwise: `Adapter::spawn` starts the adapter directly, and
   the isolation is the scratch copy, not the executor that confines cells. A
   build step is therefore no *less* confined than the debuggee already is —
   it crosses no new line — but it does mean running a build tool the
   document's own content chose, and the honest place to say so is here.

## The shape

A per-language **`Build`** beside the existing `Recipe`, returning the program
rather than assuming it:

- `None` for Python, Node and Go — the file is the program, and nothing
  changes for them.
- For C#: run the project's build in the scratch directory, then the program
  is the assembly it produced.

`entry_point` stops meaning "the first debuggable file" and starts meaning
"the thing to launch", with the build as the step that can turn one into the
other. Every compiled language then has a place to go: Java, Kotlin, Rust,
C++ — the same shape, a different command.

**Where the build command comes from is the open question**, and there are two
honest answers:

- **A built-in recipe per language** (`dotnet build`). Zero configuration,
  which is the guarantee; wrong for any project that builds differently.
- **The document's own cells.** A document that builds C# almost certainly
  already has an `<hick:exec>` that builds it — that is what a literate
  document *is*. Deriving the build from the cell that already exists invents
  no configuration at all and is exactly this product's argument. It needs a
  way to say which cell, which is a design problem rather than a mechanism
  one.

The first is shippable and the second is right. They are not exclusive: the
built-in recipe is the default, and a document that says otherwise wins.

## What it looks like

Both surfaces that debug — the interactive session and `<hick:capture>` — say
the same three lines today:

```rust
let files   = weave_into(source, scratch.path())?;
let program = entry_point(&files)?;          // the first debuggable FILE
let adapter = adapter_for(&program, project)?;
```

The change is one line, and deliberately at the end: `entry_point` keeps
naming the **source** file, because the source is what has a language and
`adapter_for` needs the language. What it produces stops being what gets
launched.

```rust
let files   = weave_into(source, scratch.path())?;
let source  = entry_point(&files)?;          // unchanged
let adapter = adapter_for(&source, project)?;
let program = build(&source, scratch.path(), &mut on_progress)?;   // new
```

`Launch` does not change at all. For Python, Node and Go, `build` returns the
path it was given.

### The table

```rust
/// What must happen before there is a program to launch.
enum Build {
    /// The generated file IS the program. Python and Node run it; delve
    /// compiles Go on the way past. Nothing to do.
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
    },
}

fn build_for(language: &str) -> Build {
    match language {
        "csharp" => Build::Command {
            project: "*.csproj",
            argv: &["dotnet", "build", "--nologo", "--configuration", "Debug"],
            artifact: "bin/Debug/*/*.dll",
        },
        _ => Build::None,
    }
}
```

Java, Kotlin, Rust and C++ are then four more rows, not four more designs.

### On `scaffolding.hick`, concretely

The document generates `app/Program.cs` and `app/app.csproj` into the scratch
directory. `entry_point` picks `Program.cs` → language `csharp` →
`adapter_for` finds netcoredbg → `build` locates `app/app.csproj`, runs
`dotnet build` in `app/`, and returns `app/bin/Debug/net10.0/app.dll`. That is
what netcoredbg launches, and the breakpoint the reader set on line 2 of
`Program.cs` maps back through the same `Mapping` every other language uses.

### The document's own spelling

The second design option needs one attribute, not an element:

```xml
<hick:exec container="sdk" builds="app/bin/Debug/net10.0/app.dll">
dotnet build app --configuration Debug
</hick:exec>
```

`builds=` is a claim about what the cell produces, which is the same kind of
claim `hick:file` and `hick:volume` already make. It removes the glob guessing
entirely — the document says where the artifact is — and it carries an
advantage the built-in recipe cannot have: **the build then runs the way cells
run, in the document's own container, confined.** The recipe path spawns a
build tool directly, exactly as unconfined as the adapter it feeds.

That is the argument for making the document's answer win where it exists, and
for treating the built-in table as the default for a document that says
nothing.

### What a person sees: the run, in a read-only terminal

The spinner is the wrong answer. A build that fails says why in its own output
— MSBuild's errors, the missing package, the syntax error on line 12 — and a
`Building…` label throws all of that away and replaces it with the one fact
the person already knew.

So **you watch the build in a terminal**, and the terminal is **read-only**.

That is not a limitation of the view, it is the only honest form it can take:
**a terminal you can type into is a second input the document does not have.**
A cell's stdin is not in the document; anything typed into it would be
unrecorded input that changed the output, and the run would no longer be
something the document reproduces. If a cell needs input, the input belongs in
the document — in the command, in a file it reads, in a declared variable —
where a re-run finds it too.

This gives the app **kinds of terminal that must never be confused** — a third
one, where the typing IS the document, is
`docs/specs/freeform/a-terminal-that-writes-the-document.md`, and it is what
shows the rule above is stated too narrowly: the rule is not *no typing*, it
is *no unrecorded input*.

| | what it is | input |
|---|---|---|
| a **session** terminal (`hick-term`) | a person's own shell, a named piece of work that keeps its state | a real PTY, typed into |
| a **cell** terminal, watching | one exec happening | **none — there is no input path at all** |
| a **cell** terminal, anchored | a container in a document; the typing becomes the cell | typed, and written down

The second has no PTY behind it. It is the same xterm emulator with
`disableStdin`, fed from the `TranscriptEvent` stream (`Cmd`, `Out`, `Err`,
`Exit`) the executor already publishes on the run channel. Nothing new is
plumbed; the events exist and are already timed.

**Why a terminal rather than the text card that exists.** Build tools emit
ANSI colour and rewrite lines with carriage returns — progress bars, restore
counters, MSBuild's warning colours. A card that concatenates the bytes shows
that as garbage. An emulator is what turns those same bytes back into what the
tool meant, which is the whole reason terminals exist.

**The transcript and the terminal are the same bytes for different purposes,
and only one of them is evidence.** The transcript is the record — woven,
compared, recorded under the cell's key. The terminal is the run happening;
you watch it and close it. Nothing is ever verified against what a terminal
showed.

And it generalises past the build, which is the argument for putting it here
rather than in a build-shaped corner: **watching a build is just watching a
cell.** Any `<hick:exec>` a person starts can have one. That is also what
makes the `builds=` spelling cheap — if the build is an ordinary cell, its
live output, its transcript, its recording and its cache key all already
exist, and the attribute adds nothing but a claim about the artifact.

### The restore, said out loud

`dotnet build` cannot work without a restore, and a fresh scratch tree never
has one, so `--no-restore` would mean C# never debugs at all. The packages
land in the project's own `.hick-cache` — the same `NUGET_PACKAGES` redirect
`hick lsp install csharp` already uses — and the first build says so before it
does it, in the terminal, where it is already looking:

```
Building app/app.csproj…
  fetching packages into .hick-cache/nuget — this happens once
```

Silent is what the refusals forbid. Slow and legible is fine.

### When it cannot

```
Program.cs is C#, which is compiled: the debugger launches the assembly a
build produces, not the source you wrote.

This document generates no project file, so there is nothing to build.
Next step: generate one — a `hick:file path="app/app.csproj"` block, or
`hick ingest` the one `dotnet new` writes — and the debugger will build it.
```

## The shape underneath, and which to build

The table above treats a build as a new phase. It is not: **a compile is a
cell whose output is a file rather than text**, and `hick:exec` plus volumes
already models that exactly.

Notice what the debugger does today — `weave_into` writes the document's files
to a scratch directory and launches one. **It never runs the DAG at all.** It
bypasses the graph that already knows what produces what.

So the deeper formulation is: **the debugger is a consumer of the DAG, not a
bypass of it.** "Debug this" means *run the graph up to the cell that produces
the artifact, then launch that artifact under an adapter*. Under that reading
there is no build step, no per-language table and no new concept: incremental
builds come from the cache key, and the build appears in a read-only terminal
because the build is a cell.

**Build the table first anyway.** It is small, it is testable without touching
the DAG, and it makes C# debuggable — which is the thing a person can use. But
build it knowing it is scaffolding for the second shape, and do not grow it:
the moment a second compiled language needs a fourth field, that is the signal
to stop extending the table and make the debugger run the graph instead.

`builds=` is the same idea arriving from the document's side. Under the table
it is an override; under the DAG reading it is simply how a cell says its
output is a program, and the table becomes the default for a document that
says nothing.

## Refusals

- **No `launch.json`, no `tasks.json`, no per-project debug config.** If the
  answer needs the user to write configuration, it is the wrong answer.
- **Never invent a project file.** A scratch directory with a `.cs` and no
  `.csproj` cannot be built, and writing a plausible one on the user's behalf
  produces a program that is not theirs. Refuse, and say what is missing.
- **A cell terminal never gains an input path.** Not a "just this once" prompt
  answer, not a paste target, not a signal key. The moment it takes input, the
  run stops being something the document reproduces, and the terminal stops
  being safe to show beside a transcript that claims it is.
- **Never fetch silently.** `dotnet build` restores from the network by
  default. The debug path has never needed the network, and gaining it
  invisibly is the kind of change that should be a decision, not a side
  effect.
- **Do not claim a language is debuggable before it is.** Adding netcoredbg to
  discovery on its own would make `hick dap list` and `language_of` offer C#,
  pick `Program.cs`, and fail at launch — worse than saying nothing. The
  discovery entry lands in the same change as the build step.

## What is settled

**netcoredbg** (Samsung, MIT, verified 2026-08-26) is the C# adapter:
`netcoredbg --interpreter=vscode`, needing only the .NET runtime. Microsoft's
`vsdbg` is licensed for use only with Visual Studio and VS Code and is
therefore not available to this product at all.

Distribution is release archives and distro packages rather than a `dotnet
tool`, so it does not fit the one-command installer shape that
`hick lsp install csharp` uses — a `hick dap install csharp` would need a new
installer shape (a per-platform URL and a checksum), which is a separate piece
of work from the build step and not a prerequisite for it. Discovery finds a
netcoredbg the user installed themselves, which is the documented fallback for
exactly this case.

**The language server half is already built** (2026-08-26): `hick lsp install
csharp` fetches csharp-ls (MIT), sandboxed, and discovery finds both it and
OmniSharp. Highlighting for C# and XML is in the app. Nothing about the
debugger is built.
