<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/dap.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3fdb9991751a"></a>
## `crates/hick-plugins/src/languages/dap.rs`

Related contracts: [Use the adapter’s transport and the current measured language support](../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-3fdb9991751a-C_PYTHON"></a>
### `C_PYTHON`

<storycode:copy id="f-3fdb9991751a-C_PYTHON">
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
</storycode:copy>
<a id="f-3fdb9991751a-C_NODE"></a>
### `C_NODE`

js-debug is NOT on npm — `npm install @vscode/js-debug` 404s, and always
has — so it is fetched as a tarball from its own releases and lands beside
the other adapters with no `node_modules` above it. It does not speak
stdio either: it prints "Debug server listening at…" and waits.

<storycode:copy id="f-3fdb9991751a-C_NODE">
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

</storycode:copy>
<a id="f-3fdb9991751a-C_GO"></a>
### `C_GO`

"Starts a headless TCP server communicating via Debug Adaptor Protocol
(DAP)" — delve's own help. `dlv dap` with no `--listen` picks its own port
and tells nobody; given one, it is an ordinary DAP server.

<storycode:copy id="f-3fdb9991751a-C_GO">
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
</storycode:copy>
<a id="f-3fdb9991751a-C_NATIVE"></a>
### `C_NATIVE`

<storycode:copy id="f-3fdb9991751a-C_NATIVE">
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
</storycode:copy>
<a id="f-3fdb9991751a-C_JAVA"></a>
### `C_JAVA`

java-debug is a plugin inside eclipse.jdt.ls, not a program. Discovery's
job here is only to say whether both halves are installed; obtaining a
port needs LSP, and that happens a layer up.

<storycode:copy id="f-3fdb9991751a-C_JAVA">
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
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/dap.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/dap.rs"><storycode:paste select="#f-3fdb9991751a-C_PYTHON"/><storycode:paste select="#f-3fdb9991751a-C_NODE"/><storycode:paste select="#f-3fdb9991751a-C_GO"/><storycode:paste select="#f-3fdb9991751a-C_NATIVE"/><storycode:paste select="#f-3fdb9991751a-C_JAVA"/></storycode:file>

</storycode:doc>
