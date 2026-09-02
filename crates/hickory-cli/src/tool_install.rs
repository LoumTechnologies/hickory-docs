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
    ///
    /// For an [`Installer::assets`] install this is the fetcher (`curl`); the
    /// unpacker is named per-asset, because a `.tar.gz` and a `.zip` do not
    /// need the same program.
    pub tool: &'static str,
    /// The package installed, named for the report.
    pub package: &'static str,
    /// The shell command, with `{prefix}` replaced by the install directory.
    ///
    /// Empty when this installs from [`Installer::assets`] instead.
    pub command: &'static str,
    /// Prebuilt archives, one per platform, when the tool is not published
    /// through any package manager.
    ///
    /// This is the shape `netcoredbg` needs and `csharp-ls` does not: a
    /// `dotnet tool` is one command on every platform, while a release
    /// archive is a different URL per target and a checksum that has to be
    /// pinned, because "download and run whatever is at this URL" is not
    /// something to do on somebody's machine.
    pub assets: &'static [Asset],
    /// Why this package and not another, for the person reading the report.
    pub reason: &'static str,
}

/// One platform's prebuilt archive, pinned.
pub struct Asset {
    /// `std::env::consts::OS`.
    pub os: &'static str,
    /// `std::env::consts::ARCH`.
    pub arch: &'static str,
    pub url: &'static str,
    /// SHA-256 of the archive, checked before anything is unpacked.
    ///
    /// A pin, not a signature: it proves the bytes are the ones somebody
    /// looked at when this line was written, and proves nothing about
    /// whether those bytes are trustworthy. Upstream publishes no
    /// signatures, so this is the strongest available statement and it is
    /// worth being precise about which one it is.
    pub sha256: &'static str,
    /// The directory under the prefix to unpack into, or empty for the
    /// prefix itself. An archive that carries a top-level directory of its
    /// own (netcoredbg's `netcoredbg/`) needs nothing; one that does not
    /// (a `.vsix` is a zip of `extension/…`) would otherwise scatter its
    /// contents across every other adapter's home.
    pub into: &'static str,
    /// The program that unpacks it — `tar` or `unzip`.
    pub unpack: &'static str,
}

impl Asset {
    /// The shell that fetches, verifies and unpacks this asset.
    ///
    /// One `&&` chain on purpose: a failed checksum must stop before the
    /// archive is opened, and a shell that carried on would unpack bytes
    /// nobody vouched for.
    fn command(&self) -> String {
        // `sha256sum` is coreutils and absent on macOS, where the same job is
        // `shasum -a 256`. Both read the same "<hex>  <path>" line, so only
        // the program name differs.
        let checker = if cfg!(target_os = "macos") {
            "shasum -a 256 -c -"
        } else {
            "sha256sum -c -"
        };
        let dest = if self.into.is_empty() {
            "{prefix}".to_string()
        } else {
            format!("{{prefix}}/{}", self.into)
        };
        let unpack = match self.unpack {
            "unzip" => format!("mkdir -p {dest} && unzip -q {{prefix}}/download.archive -d {dest}"),
            _ => format!("mkdir -p {dest} && tar -xzf {{prefix}}/download.archive -C {dest}"),
        };
        format!(
            "curl -fsSL {url} -o {{prefix}}/download.archive && \
             printf '%s  %s\\n' {sha} {{prefix}}/download.archive | {checker} && \
             {unpack} && rm {{prefix}}/download.archive",
            url = self.url,
            sha = self.sha256,
        )
    }
}

/// The archive for this machine, when the catalogue has one.
fn asset_for(installer: &Installer) -> Option<&'static Asset> {
    installer
        .assets
        .iter()
        .find(|a| a.os == std::env::consts::OS && a.arch == std::env::consts::ARCH)
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
        .map(|installer| {
            let asset = asset_for(installer);
            // An archive installer needs an unpacker as well as a fetcher,
            // and on a platform the catalogue has no archive for it needs
            // saying so — not a missing-`curl` message about a machine that
            // has curl.
            let blocked = if !installer.assets.is_empty() && asset.is_none() {
                Some(format!(
                    "{} publishes no build for {}-{}",
                    installer.package,
                    std::env::consts::OS,
                    std::env::consts::ARCH
                ))
            } else if which(installer.tool).is_none() {
                Some(format!(
                    "`{}` is not installed, and it is what fetches {}",
                    installer.tool, installer.package
                ))
            } else {
                asset.filter(|a| which(a.unpack).is_none()).map(|a| {
                    format!(
                        "`{}` is not installed, and it is what unpacks {}",
                        a.unpack, installer.package
                    )
                })
            };
            InstallPlan {
                language: installer.language.to_string(),
                package: installer.package.to_string(),
                tool: installer.tool.to_string(),
                reason: installer.reason.to_string(),
                blocked,
            }
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

    if !installer.assets.is_empty() && asset_for(installer).is_none() {
        bail!(
            "{package} publishes no build for {os}-{arch}, so there is nothing to install here.\n\
             Build it yourself or install it however this platform does — hick searches the \
             project and the machine before it looks here, so either works.",
            package = installer.package,
            os = std::env::consts::OS,
            arch = std::env::consts::ARCH,
        );
    }
    if let Some(asset) = asset_for(installer)
        && which(asset.unpack).is_none()
    {
        bail!(
            "`{unpack}` is not installed, and it is what unpacks {package}.\n\
             Install {unpack} and try again, or install {package} yourself — it will be found \
             and used.",
            unpack = asset.unpack,
            package = installer.package,
        );
    }
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

    let command = match asset_for(installer) {
        Some(asset) => asset.command(),
        None => installer.command.to_string(),
    }
    .replace("{prefix}", &prefix.to_string_lossy());

    // The network is granted here and nowhere else in this tool: an install
    // that cannot fetch is not an install.
    let Some((program, args)) =
        // An installer has no peers to hide from: it runs before any cell,
        // into a prefix of its own.
        // No project tools for an installer: it is fetching one, not using
        // one, and the prefix it writes into is already its workdir.
        policy::wrap(
            sandbox,
            &policy::Confinement {
                workdir: &prefix,
                command: &command,
                allow_network: true,
                profile: Profile::Installer,
                tmpdir: None,
                peers: &[],
                tools: &[],
            },
        )
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
                // An archive installer's command is built from the asset for
                // this machine, so that is what has to be checked — and it
                // has to be checked for EVERY platform's asset, not only the
                // one this test happens to be running on.
                let commands: Vec<String> = if installer.assets.is_empty() {
                    vec![installer.command.to_string()]
                } else {
                    installer.assets.iter().map(Asset::command).collect()
                };
                for command in commands {
                    assert!(
                        command.contains("{prefix}"),
                        "{} installs somewhere unconfined: {command}",
                        installer.language,
                    );
                    assert!(
                        !command.contains(" -g ") && !command.contains("--global"),
                        "{} installs globally, which the sandbox would (rightly) refuse",
                        installer.language
                    );
                }
            }
        }
    }

    #[test]
    fn every_archive_is_pinned_to_bytes_somebody_looked_at() {
        // A URL with no checksum is "download and run whatever is there
        // now", which is the one thing an installer must not be. The
        // checksum is a pin rather than a signature — upstream publishes no
        // signatures — and the honest version of that is: these are the
        // bytes somebody saw, and nothing more is claimed.
        for catalogue in catalogues() {
            for installer in catalogue.installers {
                for asset in installer.assets {
                    assert_eq!(
                        asset.sha256.len(),
                        64,
                        "{} / {}-{}: not a SHA-256",
                        installer.language,
                        asset.os,
                        asset.arch
                    );
                    assert!(
                        asset.sha256.chars().all(|c| c.is_ascii_hexdigit()),
                        "{} / {}-{}: not hex",
                        installer.language,
                        asset.os,
                        asset.arch
                    );
                    assert!(
                        asset.url.starts_with("https://"),
                        "{} fetches over something other than TLS: {}",
                        installer.language,
                        asset.url
                    );
                    assert!(
                        matches!(asset.unpack, "tar" | "unzip"),
                        "{} needs an unpacker nothing checks for: {}",
                        installer.language,
                        asset.unpack
                    );
                }
                // A half-bumped version is the mistake this catches: four
                // URLs that do not agree about which release they are would
                // install one release's binary against another's checksum.
                let versions: std::collections::BTreeSet<&str> = installer
                    .assets
                    .iter()
                    .filter_map(|a| a.url.split("/download/").nth(1))
                    .filter_map(|rest| rest.split('/').next())
                    .collect();
                assert!(
                    versions.len() <= 1,
                    "{} pins assets from more than one release: {versions:?}",
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
                    // Either the missing TOOL (the fetcher or the unpacker),
                    // or the PACKAGE — because a catalogue that publishes no
                    // build for this platform is blocked with nothing missing
                    // on the machine at all, and naming a tool there would be
                    // a lie. Found on macos-x86_64, where netcoredbg has no
                    // asset: the old assertion demanded a tool name and got
                    // the package name, correctly.
                    assert!(
                        blocked.contains(&plan.tool) || blocked.contains(&plan.package),
                        "a blocked plan for {} names neither its tool ({}) nor its package \
                         ({}): {blocked}",
                        plan.language,
                        plan.tool,
                        plan.package,
                    );
                }
            }
        }
    }
}
