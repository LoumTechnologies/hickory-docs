//! Which language servers `hick lsp install` can fetch.
//!
//! The machinery — the sandbox, the prefix, the refusals — is shared with
//! `hick dap install`; see [`crate::tool_install`]. This file is the
//! catalogue and nothing else.

use crate::tool_install::{Catalogue, Installer};

/// Where a language server is installed, relative to the project root.
pub const SERVERS_DIR: &str = ".hick-cache/servers";

pub static CATALOGUE: std::sync::LazyLock<Catalogue> = std::sync::LazyLock::new(|| Catalogue {
    what: "language server",
    prefix: SERVERS_DIR,
    command: "hick lsp install",
    installers: &INSTALLERS,
});

/// The installable set.
///
/// Deliberately short. Every entry is a package this project is willing to
/// point a network install at, chosen because it is the server that language's
/// own ecosystem treats as the default — not because it was easy to wire up.

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(
        &CATALOGUE,
        root,
        hick_plugins::registry()
            .interface(language, hick_plugins::InterfaceKind::Lsp)
            .and_then(hick_plugins::Interface::installer)
            .map(|installer| installer.language)
            .unwrap_or(language),
    )
}

// Compatibility view; plugin declarations are owned by hick-plugins.
static INSTALLERS: std::sync::LazyLock<Vec<Installer>> = std::sync::LazyLock::new(|| {
    hick_plugins::registry()
        .installers(hick_plugins::InterfaceKind::Lsp)
        .copied()
        .collect()
});
