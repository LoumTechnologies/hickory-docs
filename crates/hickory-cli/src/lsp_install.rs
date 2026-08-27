//! Which language servers `hick lsp install` can fetch.
//!
//! The machinery — the sandbox, the prefix, the refusals — is shared with
//! `hick dap install`; see [`crate::tool_install`]. This file is the
//! catalogue and nothing else.

use crate::tool_install::{Catalogue, Installer};

/// Where a language server is installed, relative to the project root.
pub const SERVERS_DIR: &str = ".hick-cache/servers";

pub static CATALOGUE: Catalogue = Catalogue {
    what: "language server",
    prefix: SERVERS_DIR,
    command: "hick lsp install",
    installers: INSTALLERS,
};

/// The installable set.
///
/// Deliberately short. Every entry is a package this project is willing to
/// point a network install at, chosen because it is the server that language's
/// own ecosystem treats as the default — not because it was easy to wire up.
const INSTALLERS: &[Installer] = &[
    Installer {
        language: "python",
        tool: "uv",
        package: "basedpyright",
        // A real virtualenv rather than a --target install: the console
        // scripts land in bin/, which is where discovery already looks, and
        // `pyvenv.cfg` is how it recognises a virtualenv at all.
        command: "uv venv {prefix}/python && uv pip install --python {prefix}/python/bin/python basedpyright",
        // The answer to "why not pyright": pyright implements no semantic
        // tokens, no inlay hints and no folding ranges — those live in
        // Pylance, which is licensed for Microsoft's editors only.
        // basedpyright is the open-source fork that reimplements them, so
        // this is what makes Python code in a document actually coloured.
        assets: &[],
        reason: "pyright with the Pylance-only features (semantic tokens, inlay hints) added",
    },
    Installer {
        language: "typescript",
        tool: "npm",
        package: "typescript-language-server typescript@5",
        // Pinned to 5, and that pin is load-bearing rather than caution:
        // `typescript` on npm is 7.x now, the native port, which ships no
        // `tsserver.js` at all. typescript-language-server is a wrapper
        // AROUND tsserver, so the obvious `npm install typescript
        // typescript-language-server` installs two packages that cannot work
        // together — the server starts and fails `initialize` with "Could not
        // find a valid tsserver".
        command: "npm install --no-fund --no-audit --prefix {prefix}/node typescript-language-server typescript@5",
        assets: &[],
        reason: "the server the TypeScript ecosystem treats as the default",
    },
    Installer {
        language: "csharp",
        tool: "dotnet",
        package: "csharp-ls",
        // Three environment variables, and every one of them is what makes
        // this installable at all: `dotnet tool install` writes to
        // `$HOME/.nuget/packages` and `$HOME/.dotnet` by default, and the
        // installer sandbox makes ONLY the prefix writable. Redirected, the
        // whole install — tool, package cache, CLI state — lands in one
        // directory `hick init` already ignores.
        //
        // The telemetry opt-out is not incidental either. This product says
        // nothing to anyone; a tool it spawns on the user's behalf must not
        // be the exception, and the .NET CLI phones home on first run unless
        // told not to.
        command: "DOTNET_CLI_HOME={prefix}/dotnet DOTNET_CLI_TELEMETRY_OPTOUT=1                   NUGET_PACKAGES={prefix}/dotnet/nuget                   dotnet tool install --tool-path {prefix}/dotnet csharp-ls",
        // Why this and not OmniSharp, which discovery has always looked for:
        // OmniSharp ships as a per-platform release archive, so installing it
        // means picking a URL and a checksum, while csharp-ls is a `dotnet
        // tool` — one command, the SDK's own package path, and the SDK is
        // already there if you are writing C#. Both are MIT; a machine that
        // has OmniSharp keeps using it, because discovery prefers it.
        //
        // Not Microsoft's own Roslyn language server: it ships inside the C#
        // extension under a licence that permits use only with Microsoft's
        // editors, which is not something to install on someone's behalf.
        assets: &[],
        reason: "a Roslyn-based C# server that installs as a dotnet tool",
    },
    Installer {
        language: "json",
        tool: "npm",
        package: "vscode-langservers-extracted",
        command: "npm install --no-fund --no-audit --prefix {prefix}/node vscode-langservers-extracted",
        assets: &[],
        reason: "VS Code's own JSON, HTML and CSS servers, extracted; one install covers all three",
    },
];

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

/// Install one language's server, confined.
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(&CATALOGUE, root, language)
}
