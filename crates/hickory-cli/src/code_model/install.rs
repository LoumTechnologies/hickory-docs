//! Building a code model server, and putting it where `hick` will find it.
//!
//! The other catalogues in this product FETCH: a language server is an npm
//! package or a release archive, pinned to a checksum, downloaded and
//! unpacked. A model server is not published anywhere, because there is
//! nowhere to publish four binaries in four ecosystems to — so this catalogue
//! **builds from source**, out of the `code-models/` directory of a
//! hickory-docs checkout, with the toolchain that ecosystem already needs.
//!
//! That difference is worth being explicit about rather than papering over,
//! because it decides what `hick lang` may claim. A fetchable tool can be
//! reported as *obtainable* on any machine; a tool that needs a checkout of
//! this repository and a Rust or Go or .NET toolchain cannot. So the model
//! column stays **measured by discovery** — `yes` when the binary is there
//! and `—` when it is not — and this is the command that turns one into the
//! other. Reporting `get` for a language whose server needs a checkout the
//! person does not have would be the fifth hand-maintained promise in
//! `docs/specs/freeform/language-tiers.md`'s list of four.
//!
//! The build runs confined, for the reason `tool_install` gives: a build
//! downloads dependencies and runs their build scripts, which is somebody
//! else's code on your machine. The writable directory is the source tree
//! itself, because that is where `target/` and `node_modules/` land.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use hickory_executor_sandbox::policy::{self, Profile, Sandbox};

pub use super::MODELS_DIR;

/// How one language's model server is built.
pub struct ServerBuild {
    pub language: &'static str,
    /// What `hick code-model <language>` looks for. Always
    /// `hick-model-<language>`, which the test below insists on.
    pub binary: &'static str,
    /// Relative to a hickory-docs checkout.
    pub source: &'static str,
    /// The toolchain that must be on the machine.
    pub tool: &'static str,
    /// The build, run with the source directory as the working directory.
    pub build: &'static str,
    /// What the build produced, relative to the source directory.
    pub produces: &'static str,
    /// Why it is built this way, for the person reading the report.
    pub reason: &'static str,
}

pub static CATALOGUE: &[ServerBuild] = &[
    ServerBuild {
        language: "rust",
        binary: "hick-model-rust",
        source: "code-models/rust",
        tool: "cargo",
        build: "cargo build --release",
        produces: "target/release/hick-model-rust",
        reason: "syn plus `cargo metadata`; no nightly and no rustdoc JSON, so it \
                 answers about a workspace that does not compile yet",
    },
    ServerBuild {
        language: "go",
        binary: "hick-model-go",
        source: "code-models/go",
        tool: "go",
        build: "go build -o hick-model-go ./...",
        produces: "hick-model-go",
        reason: "`go/packages` and `go/types`, which is the only thing that can \
                 answer about a Go method set",
    },
];

/// Why C# and TypeScript are not in that list, since they have servers.
///
/// Both build to a DIRECTORY whose parts have to stay together — a `dotnet
/// publish` output is an apphost beside its assemblies, and the TypeScript
/// launcher resolves `server.js` relative to its own real path. Installing
/// one file out of either produces a server that starts and then fails,
/// which is worse than not offering the command: `hick lang` would report
/// `yes` for a model nothing can query. They are built where they live and
/// put on `PATH`, which `code-models/README.md` says how to do.
pub const BUILT_IN_PLACE: &[(&str, &str, &str)] = &[
    (
        "csharp",
        "code-models/csharp",
        "dotnet publish -c Release -o <somewhere on PATH>",
    ),
    ("typescript", "code-models/typescript", "npm install"),
];

pub fn for_language(language: &str) -> Option<&'static ServerBuild> {
    CATALOGUE.iter().find(|s| s.language == language)
}

/// Every language this can build, and whether this machine could.
pub struct BuildPlan {
    pub language: String,
    pub binary: String,
    pub tool: String,
    pub reason: String,
    /// None when the build could run; otherwise why it could not.
    pub blocked: Option<String>,
}

pub fn plans(checkout: &Path) -> Vec<BuildPlan> {
    CATALOGUE
        .iter()
        .map(|server| {
            let source = checkout.join(server.source);
            let blocked = if !source.is_dir() {
                Some(format!(
                    "{} is not here — this builds from a hickory-docs checkout, \
                     and `--from <checkout>` says where one is",
                    server.source
                ))
            } else if which(server.tool).is_none() {
                Some(format!(
                    "`{}` is not installed, and it is what builds {}",
                    server.tool, server.binary
                ))
            } else {
                None
            };
            BuildPlan {
                language: server.language.to_string(),
                binary: server.binary.to_string(),
                tool: server.tool.to_string(),
                reason: server.reason.to_string(),
                blocked,
            }
        })
        .collect()
}

/// Build one language's model server and install it into the project.
pub fn install(root: &Path, checkout: &Path, language: &str) -> Result<PathBuf> {
    let Some(server) = for_language(language) else {
        let known: Vec<&str> = CATALOGUE.iter().map(|s| s.language).collect();
        bail!(
            "no code model server for '{language}'.\n\
             Buildable languages: {}.\n\
             A model server is one project per language under `code-models/` in the \
             hickory-docs checkout; writing one is how a language reaches Gold, and \
             `docs/specs/freeform/language-tiers.md` says what it has to answer.",
            known.join(", "),
        );
    };

    let source = checkout.join(server.source);
    if !source.is_dir() {
        bail!(
            "no {} under {}.\n\
             This builds the server from source — there is nowhere to publish four \
             binaries in four ecosystems to — so it needs a hickory-docs checkout.\n\
             Point at one with `--from <path>`, or build it yourself:\n    \
             cd {} && {}\n    cp {} {}/{}",
            server.source,
            checkout.display(),
            server.source,
            server.build,
            server.produces,
            MODELS_DIR,
            server.binary,
        );
    }
    if which(server.tool).is_none() {
        bail!(
            "`{tool}` is not installed, and it is what builds {binary}.\n\
             Install {tool} and try again. A model server is built with its own \
             ecosystem's toolchain on purpose: that is the only place its compiler \
             front end exists.",
            tool = server.tool,
            binary = server.binary,
        );
    }

    let sandbox = Sandbox::detect();
    if sandbox == Sandbox::None {
        bail!(
            "refusing to build a code model server without a sandbox.\n{}\n\
             A build downloads dependencies and runs their build scripts, which is \
             someone else's code on your machine — the one thing this tool will not \
             do unconfined. Running `{}` in {} yourself is always an option; copy the \
             result to {}/{} and it will be found.",
            Sandbox::missing_hint(),
            server.build,
            server.source,
            MODELS_DIR,
            server.binary,
        );
    }

    let Some((program, args)) = policy::wrap(
        sandbox,
        &policy::Confinement {
            // The source tree, because `target/` and `node_modules/` land
            // beside the sources and nowhere else.
            workdir: &source,
            command: server.build,
            // A build that cannot fetch is not a build. Granted here and in
            // `tool_install`, and nowhere else in this product.
            allow_network: true,
            profile: Profile::Installer,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    ) else {
        bail!("the sandbox could not be prepared for the build");
    };

    let status = Command::new(&program)
        .args(&args)
        .status()
        .with_context(|| format!("running the confined build ({program})"))?;
    if !status.success() {
        bail!(
            "building {binary} failed ({status}).\n\
             The build ran confined: it could write only {source}, so a failure here \
             is the build's own rather than a permissions problem elsewhere.",
            binary = server.binary,
            source = source.display(),
        );
    }

    let built = source.join(server.produces);
    if !built.is_file() {
        bail!(
            "the build reported success and {} is not there.\n\
             That is a mismatch between this catalogue and {}'s own build, and the \
             catalogue is the thing to fix.",
            built.display(),
            server.source,
        );
    }

    let prefix = root.join(MODELS_DIR);
    std::fs::create_dir_all(&prefix).with_context(|| format!("creating {}", prefix.display()))?;
    let installed = prefix.join(server.binary);
    std::fs::copy(&built, &installed)
        .with_context(|| format!("copying {} to {}", built.display(), installed.display()))?;
    // Copying does not carry the executable bit on every platform, and a
    // model server that cannot be spawned is reported as absent.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&installed)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&installed, permissions)?;
    }
    Ok(installed)
}

/// What to say when `hick code-model <language>` finds nothing.
pub fn how_to_get(language: &str) -> String {
    match for_language(language) {
        Some(server) => format!(
            "Build one with `hick code-model {language} --install` — it builds \
             {} with {} and installs {} into {}.",
            server.source, server.tool, server.binary, MODELS_DIR
        ),
        None => match BUILT_IN_PLACE.iter().find(|(l, _, _)| *l == language) {
            // A server that exists and is not installable from here. Saying
            // "hick has no model server for C#" would be false, and saying
            // "run --install" would name a command that installs a broken one.
            // One line: this is a cell of `hick lang`'s table, and a message
            // with newlines in it breaks the column it is in.
            Some((_, source, build)) => format!(
                "built where it lives — its output is a directory, not one binary: \
                 `cd {source} && {build}`, then put it on PATH"
            ),
            None => format!(
                "hick has no code model server for {language}. `code-models/README.md` \
                 says what one has to answer; writing it is how {language} reaches Gold."
            ),
        },
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .flat_map(|dir| [dir.join(name), dir.join(format!("{name}.exe"))])
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_server_is_named_the_way_discovery_looks_for_it() {
        // `code_model::discover` builds the name from the language, so a
        // catalogue entry whose binary is spelled differently installs a file
        // nothing will ever find — the exact shape of bug
        // `language-tiers.md` records four of.
        for server in CATALOGUE {
            assert_eq!(
                server.binary,
                format!("hick-model-{}", server.language),
                "{} installs a binary discovery will not look for",
                server.language
            );
        }
    }

    #[test]
    fn every_server_is_a_language_the_survey_reports_on() {
        let surveyed = hick_lsp::lang_detect::known_language_ids();
        for server in CATALOGUE {
            assert!(
                surveyed.contains(&server.language),
                "`{}` has a model server and no file extension routes to it, so the \
                 server could never be reached",
                server.language
            );
        }
    }

    #[test]
    fn the_advice_names_a_command_that_exists() {
        // The drift check `index_install` grew: never name a command that
        // does not exist.
        for server in CATALOGUE {
            let message = how_to_get(server.language);
            assert!(
                message.contains(&format!("hick code-model {} --install", server.language)),
                "{}: {message}",
                server.language
            );
        }
        assert!(
            !how_to_get("cobol").contains("--install"),
            "a language with no server must not be offered a build"
        );
        // A server that exists and is not installable from here must say so
        // rather than claim there is none.
        for (language, source, _) in BUILT_IN_PLACE {
            let message = how_to_get(language);
            assert!(!message.contains("--install"), "{language}: {message}");
            assert!(message.contains(source), "{language}: {message}");
            assert!(
                for_language(language).is_none(),
                "{language} is in both lists"
            );
        }
    }

    #[test]
    fn every_build_writes_only_into_its_own_source_tree() {
        // The safety property, and the same one `tool_install` asserts: the
        // build's writes are confined to the directory the sandbox made
        // writable, so a command naming an absolute path or a global install
        // would fail confusingly instead of being caught here.
        for server in CATALOGUE {
            assert!(
                !server.build.contains(" /") && !server.build.starts_with('/'),
                "{} builds into an absolute path: {}",
                server.language,
                server.build
            );
            assert!(
                !server.build.contains("--global") && !server.build.contains(" -g "),
                "{} installs globally, which the sandbox would rightly refuse",
                server.language
            );
        }
    }
}
