//! Which debug adapters `hick dap install` can fetch.
//!
//! The machinery — the sandbox, the prefix, the refusals — is shared with
//! `hick lsp install`; see [`crate::tool_install`]. This file is the
//! catalogue and nothing else.

use crate::tool_install::{Catalogue, Installer};

/// Where a debug adapter is installed, relative to the project root.
///
/// Beside the language servers rather than among them: the two are found by
/// different code and a mixed directory would make "which of these is which"
/// a question somebody has to answer at a shell prompt.
pub const ADAPTERS_DIR: &str = ".hick-cache/adapters";

pub static CATALOGUE: Catalogue = Catalogue {
    what: "debug adapter",
    prefix: ADAPTERS_DIR,
    command: "hick dap install",
    installers: INSTALLERS,
};

/// The installable set.
///
/// Shorter than the language-server catalogue, because debug adapters are
/// less uniformly packaged: several ship only inside an editor's extension,
/// which is not something to fetch on a user's behalf. Where an ecosystem has
/// no installable adapter, discovery still finds one the user installed
/// themselves — that path is unaffected by anything here.
const INSTALLERS: &[Installer] = &[
    Installer {
        language: "python",
        tool: "uv",
        package: "debugpy",
        // A real virtualenv, for the same reason the language-server
        // installer uses one: `pyvenv.cfg` is how discovery recognises it,
        // and the interpreter that has debugpy must be the interpreter that
        // runs the cell — a machine python paired with a venv's debugpy
        // fails in a way that reads as our bug.
        command: "uv venv {prefix}/python && uv pip install --python {prefix}/python/bin/python debugpy",
        reason: "Microsoft's Python debugger, and the reference DAP implementation",
    },
    Installer {
        language: "typescript",
        tool: "npm",
        package: "@vscode/js-debug",
        // The same adapter VS Code uses for Node, published standalone. It
        // is a JavaScript entry point rather than a binary, which is why
        // discovery looks for a script to hand to `node` rather than a name
        // on PATH.
        command: "npm install --no-fund --no-audit --prefix {prefix}/node @vscode/js-debug",
        reason: "the Node debugger VS Code ships, usable outside it",
    },
];

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

/// Install one language's adapter, confined.
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(&CATALOGUE, root, language)
}

#[cfg(test)]
mod tests {
    /// The two halves of "how do I get an adapter" must agree.
    ///
    /// `hick_dap::how_to_get` writes the sentence a person reads when
    /// discovery finds nothing, and this file decides what `hick dap install`
    /// can actually do. They live in different crates because one is adapter
    /// knowledge and the other is a catalogue — so nothing but this test
    /// stops them drifting into a message that names a command that does not
    /// exist.
    #[test]
    fn nothing_is_offered_that_cannot_be_installed_and_nothing_installable_is_hidden() {
        for language in hick_dap::known_languages() {
            let installable = super::INSTALLERS.iter().any(|i| i.language == language)
                // The catalogue is keyed by ecosystem, not by language id:
                // one `typescript` installer serves every JavaScript flavour
                // discovery routes to the same adapter.
                || matches!(
                    language,
                    "javascript" | "typescriptreact" | "javascriptreact"
                );
            assert_eq!(
                hick_dap::suggests_hick_install(language),
                installable,
                "`{language}`: the message and the catalogue disagree about whether \
                 `hick dap install {language}` exists"
            );
        }
    }
}
