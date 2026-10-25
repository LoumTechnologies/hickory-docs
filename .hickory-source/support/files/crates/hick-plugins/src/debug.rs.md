<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/debug.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1eee2062265d5533c4d096370a85a0605ad42573590a4ab0ed71fbe171a92289`.

<book:entry path="crates/hick-plugins/src/debug.rs">
pub const PORT_PLACEHOLDER: &str = "{port}";

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
</book:entry>

</book:doc>
