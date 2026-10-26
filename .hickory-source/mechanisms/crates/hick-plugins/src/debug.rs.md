<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/debug.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-33040be2b1fc"></a>
## `crates/hick-plugins/src/debug.rs`

<a id="f-33040be2b1fc-PORT_PLACEHOLDER"></a>
### `PORT_PLACEHOLDER`

<storycode:copy id="f-33040be2b1fc-PORT_PLACEHOLDER">
pub const PORT_PLACEHOLDER: &str = "{port}";

</storycode:copy>
<a id="f-33040be2b1fc-Transport"></a>
### `Transport`

How an adapter expects to be talked to.
Three of the four ecosystems this product most wants to debug do NOT
speak DAP over stdio, which is the single reason Go, JavaScript,
TypeScript and Ruby were reported as debuggable and could not work:
* `dlv dap` — "Starts a headless TCP server communicating via Debug
Adaptor Protocol", in delve's own help text.
* `js-debug` — prints "Debug server listening at ::1:8123" and never
reads stdin.
* `rdbg --open` — a UNIX domain socket, or TCP with `--port`.
Only debugpy, netcoredbg and codelldb speak stdio, and those are exactly
the three that had a live test and worked. Everything without one was
broken.

<storycode:copy id="f-33040be2b1fc-Transport">
/// How an adapter expects to be talked to.
///
/// Three of the four ecosystems this product most wants to debug do NOT
/// speak DAP over stdio, which is the single reason Go, JavaScript,
/// TypeScript and Ruby were reported as debuggable and could not work:
///
/// * `dlv dap` — "Starts a headless TCP server communicating via Debug
///   Adaptor Protocol", in delve's own help text.
/// * `js-debug` — prints "Debug server listening at ::1:8123" and never
///   reads stdin.
/// * `rdbg --open` — a UNIX domain socket, or TCP with `--port`.
///
/// Only debugpy, netcoredbg and codelldb speak stdio, and those are exactly
/// the three that had a live test and worked. Everything without one was
/// broken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transport {
    /// The adapter reads requests on stdin and writes on stdout.
    #[default]
    Stdio,
    /// The adapter listens on a TCP port and is connected to. The command's
    /// arguments carry `{port}` where the port belongs.
    Tcp,
    /// Somebody else is already listening on this port, and nothing is
    /// spawned.
    ///
    /// Java's adapter is not a program: `java-debug` is a plugin inside the
    /// Java language server, and a client gets a session by asking that
    /// server for one and reading a port out of the reply. Obtaining the port
    /// needs LSP, which this crate must not know about — so the crate that
    /// owns both ends does the asking and hands the port here.
    Attached(u16),
}

</storycode:copy>
<a id="f-33040be2b1fc-DebugRecipe"></a>
### `DebugRecipe`

How one adapter is launched, once its home has been found.

<storycode:copy id="f-33040be2b1fc-DebugRecipe">
/// How one adapter is launched, once its home has been found.
#[derive(Debug, Clone, Copy)]
pub enum DebugRecipe {
    /// A binary on `PATH` or in a project directory: `dlv dap`.
    Binary {
        bin: &'static str,
        args: &'static [&'static str],
    },
    /// A Python module: `<python> -m debugpy.adapter`.
    PythonModule { module: &'static str },
    /// A JavaScript entry point run by node, relative to a package root.
    NodeScript {
        package: &'static str,
        entry: &'static str,
    },
    /// An adapter that runs INSIDE a language server, and is obtained by
    /// asking it. There is nothing here to spawn.
    LanguageServerHosted,
}

</storycode:copy>
<a id="f-33040be2b1fc-DebugCandidate"></a>
### `DebugCandidate`

<storycode:copy id="f-33040be2b1fc-DebugCandidate">
#[derive(Debug, Clone, Copy)]
pub struct DebugCandidate {
    pub adapter: &'static str,
    pub recipe: DebugRecipe,
    /// stdio unless the adapter is a server. See `adapter::Transport` for why
    /// this had to exist: three of the four ecosystems here listen on a
    /// socket and never read stdin.
    pub transport: Transport,
    /// Whether this adapter is hosted by a language server.
    pub hosted: bool,
    /// Launch keys this adapter requires, as a JSON object literal.
    pub launch: &'static str,
    /// Whether this adapter runs the program on a SECOND connection.
    ///
    /// js-debug does: the connection that launches is a supervisor, and it
    /// asks the client to open another one for the program's own target.
    /// Declared per adapter rather than detected, so no other language pays
    /// for a wait that only one of them needs.
    pub multi_session: bool,
}
</storycode:copy>

### Assemble `crates/hick-plugins/src/debug.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/debug.rs"><storycode:paste select="#f-33040be2b1fc-PORT_PLACEHOLDER"/><storycode:paste select="#f-33040be2b1fc-Transport"/><storycode:paste select="#f-33040be2b1fc-DebugRecipe"/><storycode:paste select="#f-33040be2b1fc-DebugCandidate"/></storycode:file>

</storycode:doc>
