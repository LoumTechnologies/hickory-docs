//! Installing a language server, inside the sandbox.
//!
//! ## Why this is sandboxed rather than merely run
//!
//! `npm install` executes arbitrary `postinstall` scripts. `uv pip install`
//! executes arbitrary build backends. So "install a language server for the
//! user" is, precisely, *download code from the internet and run it on their
//! machine* — the exact thing [`hickory_executor_sandbox`] exists to confine.
//! Running the installer unconfined in order to enable an editor feature
//! would be a strange trade: the document you were sent is sandboxed, and
//! the tool that reads it is not.
//!
//! So the installer runs under the same confinement a cell gets, with two
//! differences that are both stated in the policy rather than assumed:
//!
//! * The writable directory is `.hick-cache/servers/`, not a cell workdir.
//! * The network is **granted**, because an install without it is nothing.
//!
//! ## Why it is never automatic
//!
//! Two of this product's promises are that it never reaches the network
//! without being asked and never runs code you did not ask it to run.
//! Auto-installing on first open would break both, silently, as a side
//! effect of opening a file. So discovery runs first (what is already on the
//! machine is always preferred), `hick init` *reports* what is missing, and
//! this only ever happens when a person types `hick lsp install`.
//!
//! ## Why `.hick-cache/`
//!
//! It is already created and git-ignored by `hick init`, so nothing here
//! lands in the user's repository. A language server is a build artifact of
//! the machine, not a fact about the project.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use hickory_executor_sandbox::policy::{self, Profile, Sandbox};

/// Where a language's server is installed, relative to the project root.
pub const SERVERS_DIR: &str = ".hick-cache/servers";

/// One way to install one language's server.
struct Installer {
    language: &'static str,
    /// The tool that must exist on the machine for this to be possible.
    tool: &'static str,
    /// The package installed, named for the report.
    package: &'static str,
    /// The shell command, with `{prefix}` replaced by the install directory.
    command: &'static str,
    /// Why this package and not another, for the person reading the report.
    reason: &'static str,
}

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
        package: "typescript-language-server typescript",
        command: "npm install --no-fund --no-audit --prefix {prefix}/node typescript-language-server typescript",
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

/// What a person is told about one language.
pub struct InstallPlan {
    pub language: String,
    pub package: String,
    pub tool: String,
    pub reason: String,
    /// None when the tool is present; otherwise why this cannot proceed.
    pub blocked: Option<String>,
}

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<InstallPlan> {
    INSTALLERS
        .iter()
        .map(|installer| InstallPlan {
            language: installer.language.to_string(),
            package: installer.package.to_string(),
            tool: installer.tool.to_string(),
            reason: installer.reason.to_string(),
            blocked: which(installer.tool).is_none().then(|| {
                format!(
                    "`{}` is not installed, and it is what fetches {}",
                    installer.tool, installer.package
                )
            }),
        })
        .collect()
}

/// Install the server for one language, confined.
///
/// Returns the directory it was installed into. The caller reports; this
/// function's only job is to make the install happen or explain why it did
/// not.
pub fn install(root: &Path, language: &str) -> Result<PathBuf> {
    let Some(installer) = INSTALLERS.iter().find(|i| i.language == language) else {
        let known: Vec<&str> = INSTALLERS.iter().map(|i| i.language).collect();
        bail!(
            "no installer for '{language}'.\nInstallable languages: {}.\n\
             For anything else, install the server however that ecosystem normally does — \
             `hick-lsp` finds what is already on the machine, and a server you installed \
             yourself is always preferred over one installed here.",
            known.join(", ")
        );
    };

    if which(installer.tool).is_none() {
        bail!(
            "`{tool}` is not installed, and it is what fetches {package}.\n\
             Install {tool} and try again, or install {package} yourself — `hick-lsp` searches \
             the project and the machine before it looks here, so either works.",
            tool = installer.tool,
            package = installer.package,
        );
    }

    let sandbox = Sandbox::detect();
    if sandbox == Sandbox::None {
        bail!(
            "refusing to install a language server without a sandbox.\n\
             {}\n\
             An install runs the package's own setup scripts, which is someone else's code \
             on your machine — the one thing this tool will not do unconfined. Installing \
             {package} yourself is always an option; it will be found and used.",
            Sandbox::missing_hint(),
            package = installer.package,
        );
    }

    let prefix = root.join(SERVERS_DIR);
    std::fs::create_dir_all(&prefix)
        .with_context(|| format!("creating the server directory {}", prefix.display()))?;

    let command = installer
        .command
        .replace("{prefix}", &prefix.to_string_lossy());

    // The network is granted here and nowhere else in this tool: an install
    // that cannot fetch is not an install.
    let Some((program, args)) =
        policy::wrap(sandbox, &prefix, &command, true, Profile::Installer, None)
    else {
        bail!("the sandbox could not be prepared for the install");
    };

    let status = Command::new(&program)
        .args(&args)
        .status()
        .with_context(|| format!("running the confined installer ({program})"))?;
    if !status.success() {
        bail!(
            "installing {package} failed ({status}).\n\
             The installer ran confined: it could write only {prefix}, so a failure here is \
             the package's own, not a permissions problem with the rest of your machine.",
            package = installer.package,
            prefix = prefix.display(),
        );
    }
    Ok(prefix)
}

/// First match for `name` on `PATH`.
fn which(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .flat_map(|dir| {
            [
                dir.join(name),
                dir.join(format!("{name}.exe")),
                dir.join(format!("{name}.cmd")),
            ]
        })
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_installer_writes_only_into_the_prefix() {
        // The whole safety property of the install: the command's writes are
        // confined to the directory the sandbox made writable, so every path
        // it names must be under {prefix}. A command that wrote elsewhere
        // would fail confusingly instead of being caught here.
        for installer in INSTALLERS {
            assert!(
                installer.command.contains("{prefix}"),
                "{} installs somewhere unconfined: {}",
                installer.language,
                installer.command
            );
            assert!(
                !installer.command.contains(" -g ") && !installer.command.contains("--global"),
                "{} installs globally, which the sandbox would (rightly) refuse",
                installer.language
            );
        }
    }

    #[test]
    fn an_unknown_language_names_the_ones_that_do_work() {
        let root = std::env::temp_dir();
        let error = install(&root, "cobol").unwrap_err().to_string();
        assert!(error.contains("python"), "{error}");
        assert!(error.contains("no installer for 'cobol'"), "{error}");
    }

    #[test]
    fn the_plans_say_what_is_installable_and_what_is_missing() {
        let plans = plans();
        assert!(plans.iter().any(|p| p.language == "python"));
        for plan in &plans {
            assert!(
                !plan.reason.is_empty(),
                "{} has no stated reason",
                plan.language
            );
            // `blocked` is the whole report for a machine without the tool;
            // it must name the tool, not just say "no".
            if let Some(blocked) = &plan.blocked {
                assert!(blocked.contains(&plan.tool), "{blocked}");
            }
        }
    }

    #[test]
    fn servers_live_under_the_directory_init_already_ignores() {
        // If this moves, an installed server starts showing up in `git
        // status` — a build artifact of the machine in the user's history.
        assert!(SERVERS_DIR.starts_with(".hick-cache/"));
    }
}
