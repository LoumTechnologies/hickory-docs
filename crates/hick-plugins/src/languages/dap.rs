use crate::{DebugCandidate as Candidate, DebugRecipe as Recipe, Transport};
pub const C_PYTHON: &[Candidate] = &[Candidate {
    adapter: "debugpy",
    recipe: Recipe::PythonModule {
        module: "debugpy.adapter",
    },
    transport: Transport::Stdio,
    hosted: false,
    launch: "{}",
    multi_session: false,
}];
// js-debug is NOT on npm — `npm install @vscode/js-debug` 404s, and always
// has — so it is fetched as a tarball from its own releases and lands beside
// the other adapters with no `node_modules` above it. It does not speak
// stdio either: it prints "Debug server listening at…" and waits.
pub const C_NODE: &[Candidate] = &[Candidate {
    adapter: "js-debug",
    recipe: Recipe::NodeScript {
        package: "js-debug",
        entry: "src/dapDebugServer.js",
    },
    transport: Transport::Tcp,
    hosted: false,
    // Without a `type` js-debug answers "Unknown config" and lists back
    // everything it was sent. `pwa-node` is the Node debugger; the browser
    // ones are a different product decision and are not offered.
    launch: r#"{"type": "pwa-node", "request": "launch"}"#,
    multi_session: true,
}];

// "Starts a headless TCP server communicating via Debug Adaptor Protocol
// (DAP)" — delve's own help. `dlv dap` with no `--listen` picks its own port
// and tells nobody; given one, it is an ordinary DAP server.
pub const C_GO: &[Candidate] = &[Candidate {
    adapter: "delve",
    recipe: Recipe::Binary {
        bin: "dlv",
        args: &["dap", "--listen=127.0.0.1:{port}"],
    },
    transport: Transport::Tcp,
    hosted: false,
    launch: "{}",
    multi_session: false,
}];
pub const C_NATIVE: &[Candidate] = &[
    Candidate {
        adapter: "codelldb",
        recipe: Recipe::Binary {
            bin: "codelldb",
            args: &[],
        },
        transport: Transport::Stdio,
        hosted: false,
        launch: "{}",
        multi_session: false,
    },
    Candidate {
        // `lldb-dap` is what LLVM ships now; the older name is `lldb-vscode`.
        adapter: "lldb-dap",
        recipe: Recipe::Binary {
            bin: "lldb-dap",
            args: &[],
        },
        transport: Transport::Stdio,
        hosted: false,
        launch: "{}",
        multi_session: false,
    },
];
// java-debug is a plugin inside eclipse.jdt.ls, not a program. Discovery's
// job here is only to say whether both halves are installed; obtaining a
// port needs LSP, and that happens a layer up.
pub const C_JAVA: &[Candidate] = &[Candidate {
    adapter: "java-debug",
    recipe: Recipe::LanguageServerHosted,
    transport: Transport::Stdio,
    hosted: true,
    launch: "{}",
    multi_session: false,
}];
