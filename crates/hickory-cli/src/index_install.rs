//! Which code indexers `hick index install` can fetch, and how one is run.
//!
//! `docs/specs/freeform/an-index-beside-the-language-server.md`.
//!
//! **SCIP is in addition to LSP, never in place of it.** A language server
//! answers about the file you are looking at, including the parts you have
//! not saved; an index answers about the whole project, as it was when the
//! index was built. Every editor that has only one of them feels like the
//! half it has.
//!
//! ## What is built here, and what is deliberately not
//!
//! Built: the catalogue, the sandboxed install, discovery that prefers a copy
//! you installed yourself, and **spawning** an indexer to produce a
//! `index.scip`.
//!
//! Not built: **nothing reads the index.** Until 2026-08-27 that was blocked
//! by a contradiction — reading it means linking the `scip` crate, which is
//! Apache-2.0, permissive but not MIT — and `AGENTS.md`'s heading and its
//! rule disagreed about whether that was allowed. The heading was amended:
//! the rule (no copyleft) was always the policy, so Apache-2.0 is allowed and
//! the crate may be linked. Nothing blocks the reading half now except that
//! nobody has written it. The indexers themselves never needed the decision:
//! they are **spawned**, exactly as language servers and debug adapters
//! already are, and nothing links them.
//!
//! So `hick index build` produces an index and says, in as many words, that
//! nothing consumes it yet. That is a strange thing to ship on its own, and
//! shipping it while claiming navigation had improved would be worse.

use crate::tool_install::{Catalogue, Installer};

/// Where an indexer is installed, relative to the project root.
pub const INDEXERS_DIR: &str = ".hick-cache/indexers";

pub static CATALOGUE: std::sync::LazyLock<Catalogue> = std::sync::LazyLock::new(|| Catalogue {
    what: "code indexer",
    prefix: INDEXERS_DIR,
    command: "hick index install",
    installers: &INSTALLERS,
});

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

/// Install one language's indexer, confined.
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(
        &CATALOGUE,
        root,
        hick_plugins::registry()
            .interface(language, hick_plugins::InterfaceKind::Indexer)
            .and_then(hick_plugins::Interface::installer)
            .map(|installer| installer.language)
            .unwrap_or(language),
    )
}

/// How to get an indexer for `language` when discovery finds none.
///
/// The same rule the debug adapters learned the hard way: never name a
/// command that does not exist. Most SCIP indexers are not things this
/// product installs, and saying so costs a person nothing while a wrong
/// suggestion costs them a shell round trip.
pub use hick_plugins::metadata::index_advice as how_to_get;

/// Whether [`how_to_get`] points at `hick index install`.
///
/// For the drift check below, and nothing else.
pub fn suggests_hick_install(language: &str) -> bool {
    how_to_get(language).contains("hick index install")
}

/// How one indexer is invoked, once found.
pub struct Recipe {
    /// The binary, as discovery finds it.
    pub bin: &'static str,
    /// Arguments before the output path.
    pub args: &'static [&'static str],
    /// The flag that names the output file.
    pub output_flag: &'static str,
}

/// The recipe for a language, or `None` when hick cannot drive its indexer.
pub fn recipe(language: &str) -> Option<Recipe> {
    match hick_plugins::registry().interface(language, hick_plugins::InterfaceKind::Indexer) {
        Some(hick_plugins::Interface::Indexer {
            bin,
            args,
            output_flag,
            ..
        }) => Some(Recipe {
            bin,
            args,
            output_flag,
        }),
        _ => None,
    }
}

/// Find an indexer, preferring the copy this project installed.
///
/// The same rule as language servers and debug adapters, argued once already:
/// what is already on the machine wins, nothing is ever installed to satisfy
/// a lookup, and no network is touched.
pub fn discover(language: &str, root: &std::path::Path) -> Option<std::path::PathBuf> {
    let recipe = recipe(language)?;
    let project = root.join(INDEXERS_DIR).join("node/node_modules/.bin");
    for dir in [Some(project), None].into_iter().flatten() {
        let candidate = dir.join(recipe.bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(recipe.bin))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_command_the_message_names_is_a_command_that_exists() {
        // The drift check the debug adapters grew, and a stronger version of
        // it: the first draft here said `hick index install javascript`,
        // which fails with "no installer for 'javascript'" because the
        // catalogue is keyed by ecosystem. Asserting that SOME install
        // command is named was not enough — the named one has to be real.
        let installable: Vec<&str> = super::INSTALLERS.iter().map(|i| i.language).collect();
        for language in [
            "typescript",
            "javascript",
            "typescriptreact",
            "javascriptreact",
            "python",
        ] {
            let message = super::how_to_get(language);
            let named = message
                .split("hick index install ")
                .nth(1)
                .and_then(|rest| rest.split(['`', ' ', '.']).next())
                .unwrap_or_default();
            assert!(
                installable.contains(&named),
                "`{language}` is told to run `hick index install {named}`, which does not exist"
            );
        }
        for language in ["rust", "csharp", "java", "cobol"] {
            assert!(
                !super::suggests_hick_install(language),
                "`{language}` is not installable and the message offers it anyway"
            );
        }
    }

    #[test]
    fn every_installable_language_has_a_recipe_to_run_it() {
        // Installing an indexer nothing knows how to invoke is a download
        // that does nothing.
        for installer in super::INSTALLERS.iter() {
            assert!(
                super::recipe(installer.language).is_some(),
                "{} can be installed and not run",
                installer.language
            );
        }
    }
}

static INSTALLERS: std::sync::LazyLock<Vec<Installer>> = std::sync::LazyLock::new(|| {
    hick_plugins::registry()
        .installers(hick_plugins::InterfaceKind::Indexer)
        .copied()
        .collect()
});
