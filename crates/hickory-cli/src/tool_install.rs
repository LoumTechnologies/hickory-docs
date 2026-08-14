//! Installing a tool this product drives, inside the sandbox.
//!
//! Shared by `hick lsp install` and `hick dap install`, because the argument
//! for confining one is exactly the argument for confining the other.
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
//! * The writable directory is the catalogue's own prefix under
//!   `.hick-cache/`, not a cell workdir.
//! * The network is **granted**, because an install without it is nothing.
//!
//! ## Why it is never automatic
//!
//! Two of this product's promises are that it never reaches the network
//! without being asked and never runs code you did not ask it to run.
//! Auto-installing on first open would break both, silently, as a side
//! effect of opening a file. So discovery runs first (what is already on the
//! machine is always preferred), `hick init` *reports* what is missing, and
//! this only ever happens when a person types the command.
//!
//! ## Why `.hick-cache/`
//!
//! It is already created and git-ignored by `hick init`, so nothing here
//! lands in the user's repository. A tool is a build artifact of the
//! machine, not a fact about the project.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use hickory_executor_sandbox::policy::{self, Profile, Sandbox};

/// One way to install one language's tool.
pub struct Installer {
    pub language: &'static str,
    /// The tool that must exist on the machine for this to be possible.
    pub tool: &'static str,
    /// The package installed, named for the report.
    pub package: &'static str,
    /// The shell command, with `{prefix}` replaced by the install directory.
    pub command: &'static str,
    /// Why this package and not another, for the person reading the report.
    pub reason: &'static str,
}

/// A named set of installers with a home of its own.
pub struct Catalogue {
    /// `language server` / `debug adapter`, for messages.
    pub what: &'static str,
    /// Where installs land, relative to the project root.
    pub prefix: &'static str,
    /// The command a person types, for the report.
    pub command: &'static str,
    pub installers: &'static [Installer],
}

/// What a person is told about one language.
pub struct InstallPlan {
    pub language: String,
    pub package: String,
    pub tool: String,
    pub reason: String,
    /// None when the tool is present; otherwise why this cannot proceed.
    pub blocked: Option<String>,
}

/// Every language this catalogue can install, with whether the machine can.
pub fn plans(catalogue: &Catalogue) -> Vec<InstallPlan> {
    catalogue
        .installers
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

/// Install one language's tool, confined.
pub fn install(catalogue: &Catalogue, root: &Path, language: &str) -> Result<PathBuf> {
    let Some(installer) = catalogue.installers.iter().find(|i| i.language == language) else {
        let known: Vec<&str> = catalogue.installers.iter().map(|i| i.language).collect();
        bail!(
            "no {what} installer for '{language}'.\nInstallable languages: {}.\n\
             For anything else, install it however that ecosystem normally does — hick finds \
             what is already on the machine, and a tool you installed yourself is always \
             preferred over one installed here.",
            known.join(", "),
            what = catalogue.what,
        );
    };

    if which(installer.tool).is_none() {
        bail!(
            "`{tool}` is not installed, and it is what fetches {package}.\n\
             Install {tool} and try again, or install {package} yourself — hick searches the \
             project and the machine before it looks here, so either works.",
            tool = installer.tool,
            package = installer.package,
        );
    }

    let sandbox = Sandbox::detect();
    if sandbox == Sandbox::None {
        bail!(
            "refusing to install a {what} without a sandbox.\n\
             {}\n\
             An install runs the package's own setup scripts, which is someone else's code \
             on your machine — the one thing this tool will not do unconfined. Installing \
             {package} yourself is always an option; it will be found and used.",
            Sandbox::missing_hint(),
            what = catalogue.what,
            package = installer.package,
        );
    }

    let prefix = root.join(catalogue.prefix);
    std::fs::create_dir_all(&prefix).with_context(|| format!("creating {}", prefix.display()))?;

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

    /// Every catalogue in the product, checked by the same rules.
    fn catalogues() -> Vec<&'static Catalogue> {
        vec![
            &crate::lsp_install::CATALOGUE,
            &crate::dap_install::CATALOGUE,
        ]
    }

    #[test]
    fn every_installer_writes_only_into_its_prefix() {
        // The whole safety property: the command's writes are confined to the
        // directory the sandbox made writable, so every path it names must be
        // under {prefix}. A command that wrote elsewhere would fail
        // confusingly instead of being caught here.
        for catalogue in catalogues() {
            for installer in catalogue.installers {
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
    }

    #[test]
    fn tools_live_under_the_directory_init_already_ignores() {
        // If this moves, an installed tool starts showing up in `git status`
        // — a build artifact of the machine in the user's history.
        for catalogue in catalogues() {
            assert!(
                catalogue.prefix.starts_with(".hick-cache/"),
                "{} installs outside the ignored cache",
                catalogue.what
            );
        }
    }

    #[test]
    fn an_unknown_language_names_the_ones_that_do_work() {
        let root = std::env::temp_dir();
        for catalogue in catalogues() {
            let error = install(catalogue, &root, "cobol").unwrap_err().to_string();
            assert!(error.contains("no "), "{error}");
            assert!(error.contains("python"), "{error}");
        }
    }

    #[test]
    fn a_plan_names_the_tool_that_is_missing() {
        for catalogue in catalogues() {
            for plan in plans(catalogue) {
                assert!(
                    !plan.reason.is_empty(),
                    "{} has no stated reason",
                    plan.language
                );
                if let Some(blocked) = &plan.blocked {
                    assert!(blocked.contains(&plan.tool), "{blocked}");
                }
            }
        }
    }
}
