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

pub static CATALOGUE: std::sync::LazyLock<Catalogue> = std::sync::LazyLock::new(|| Catalogue {
    what: "debug adapter",
    prefix: ADAPTERS_DIR,
    command: "hick dap install",
    installers: &INSTALLERS,
});

/// The installable set.
///
/// Shorter than the language-server catalogue, because debug adapters are
/// less uniformly packaged: several ship only inside an editor's extension,
/// which is not something to fetch on a user's behalf. Where an ecosystem has
/// no installable adapter, discovery still finds one the user installed
/// themselves — that path is unaffected by anything here.

/// java-debug, as shipped inside `vscjava.vscode-java-debug` 0.59.0.
///
/// One archive for every platform, for the same reason jdt.ls is: it is a
/// Java jar. It unpacks into `java-debug/`, and the bundle jdt.ls is handed
/// is then `java-debug/extension/server/com.microsoft.java.debug.plugin-*.jar`.
///
/// The checksum was taken on 2026-09-04 by fetching the asset.
/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

/// Install one language's adapter, confined.
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(&CATALOGUE, root, installer_language(language))
}

// JavaScript and TypeScript use the same js-debug distribution. Discovery
// returns the source language; installation is keyed by ecosystem.
fn installer_language(language: &str) -> &str {
    hick_plugins::registry()
        .interface(language, hick_plugins::InterfaceKind::Dap)
        .and_then(hick_plugins::Interface::installer)
        .map(|installer| installer.language)
        .or_else(|| {
            hick_plugins::registry()
                .interface(language, hick_plugins::InterfaceKind::Lsp)
                .and_then(hick_plugins::Interface::installer)
                .map(|installer| installer.language)
        })
        .unwrap_or(language)
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_javascript_flavour_resolves_to_a_real_installer() {
        for language in [
            "javascript",
            "typescript",
            "javascriptreact",
            "typescriptreact",
        ] {
            let canonical = super::installer_language(language);
            assert_eq!(canonical, "typescript");
            assert!(super::INSTALLERS.iter().any(|i| i.language == canonical));
        }
        assert_eq!(super::installer_language("python"), "python");
        assert_eq!(super::installer_language("unknown"), "unknown");
    }

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

// Compatibility view; plugin declarations are owned by hick-plugins.
static INSTALLERS: std::sync::LazyLock<Vec<Installer>> = std::sync::LazyLock::new(|| {
    hick_plugins::registry()
        .installers(hick_plugins::InterfaceKind::Dap)
        .copied()
        .collect()
});
