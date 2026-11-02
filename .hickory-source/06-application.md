<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# 6. Assemble and check the application

We have the construction engine, native execution, editing and persistence.
Now connect them to the entry points an engineer uses. Keep one shared language
and project model beneath the command line, browser and desktop hosts.

## Give the tool one command-line entry

Use Clap to parse the public `hick` interface. Keep the default launch behavior
and explicit subcommands in the same parser, then delegate startup to the platform
main-thread boundary. Repository-only dogfood checks stay in their own scripts.

<a id="f-5607f2721a2d-Cli"></a>
<book:copy id="f-5607f2721a2d-Cli">
#[derive(Parser)]
#[command(
    name = "hick",
    version = main_thread::VERSION,
    about = "Open Hickory Docs or run, test, and weave executable documents",
    after_help = "Launch the editor: hick | hick . | hick notes.md\nUse hick open <path> for a path with the same name as a command."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

</book:copy>

<a id="f-5607f2721a2d-main"></a>
<book:copy id="f-5607f2721a2d-main">
fn main() -> ExitCode {
    main_thread::main_exit()
}

</book:copy>

Add the subcommand dispatch and handlers in the CLI assembly:
<book:include file="mechanisms/crates/hickory-cli/src/main.rs.md" />

## Compose the editor in its two application hosts

Mount the shared document workspace in the browser application. Wrap that same
application with Tauri for windows, OS events and the local engine lifecycle.
The native host supplies platform behavior rather than a second parser or editor.

<book:include file="mechanisms/apps/web/src/App.tsx.md" />
<book:include file="mechanisms/apps/desktop/src-tauri/src/lib.rs.md" />

## Build the complete construction and select its evidence

Materialize the complete Markdown closure before building. These chapters teach
the final implementation in dependency order; they do not claim that an isolated
chapter is a separately compiling intermediate release. Run the focused witnesses
where each decision was introduced, then verify the completed source/output graph.

```sh
python3 hickory-dogfood/tools/behavior.py generate
python3 hickory-dogfood/tools/behavior.py check
python3 hickory-dogfood/tools/behavior.py audit
cargo build --locked -p hickory-cli --bin hick
./target/debug/hick project verify project.md
```

Construct the opt-in CI policy beside the checks it selects. A selected check is
not a passing receipt; keep result evidence separate from source reconstruction.
<book:include file="ci.md" />

You have followed the core construction from workspace to application. Continue
with the [remaining implementation layers](mechanisms/index.md), or use the
[release contracts](guarantees/release/index.md) when distributing a build.
The [explanation ledger](../hickory-dogfood/narrative-map.json) retains unresolved
rationale in inherited modules; this tutorial does not claim those reviews are done.
</book:doc>
