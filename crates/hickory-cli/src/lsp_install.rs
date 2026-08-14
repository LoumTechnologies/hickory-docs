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
        reason: "the server the TypeScript ecosystem treats as the default",
    },
    Installer {
        language: "json",
        tool: "npm",
        package: "vscode-langservers-extracted",
        command: "npm install --no-fund --no-audit --prefix {prefix}/node vscode-langservers-extracted",
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
