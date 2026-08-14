//! Finding a language server without being told where it is.
//!
//! The goal is that nobody configures anything. You open a document with a
//! `hick:file` block of Python in it, and Python intelligence appears — no
//! settings file, no "which server do you use", no PATH surgery.
//!
//! That is achievable because language servers are conventional. Each
//! language has a handful of real candidates, each installs to a small set of
//! predictable places, and a project usually pins its own copy where the
//! project's own tools would look. So instead of one hardcoded command per
//! language, this searches:
//!
//!  1. **The project first.** `node_modules/.bin`, a Python virtualenv,
//!     `vendor/bin`. A repository that pins a server has already answered the
//!     question, and the answer differs from the machine's.
//!  2. **Then the machine.** `PATH`, plus the places tool installers use
//!     without adding themselves to `PATH` — `~/.local/bin`, `~/.cargo/bin`,
//!     `~/go/bin`, `~/.bun/bin`, rustup's toolchain directory, Mason's
//!     registry for Neovim users.
//!  3. **In preference order per language**, best first, so a machine with
//!     several installed gets the one most people would have chosen.
//!
//! What it will not do is guess. A server that is not on the machine is
//! reported as absent, never substituted — `pyright` standing in for `ruff`
//! would produce diagnostics the project's own tooling disagrees with, which
//! is worse than none.

use std::path::{Path, PathBuf};

/// One candidate server for a language.
struct Candidate {
    /// Executable name to look for.
    bin: &'static str,
    /// Arguments that make it speak LSP over stdio.
    args: &'static [&'static str],
}

const fn c(bin: &'static str, args: &'static [&'static str]) -> Candidate {
    Candidate { bin, args }
}

/// Candidates per language, best first.
///
/// "Best" means what a person setting this up by hand would most likely pick:
/// the language's own official server where there is one, then the widely
/// used alternative, then the fast newcomer. A machine with several gets the
/// first that exists.
// Const items rather than inline arrays: a `const fn` call inside a
// reference expression is not promoted to 'static, so returning
// `&[c(..)]` from a match arm borrows a temporary.
const C_RUST: &[Candidate] = &[c("rust-analyzer", &[])];
const C_PYTHON: &[Candidate] = &[
    c("basedpyright-langserver", &["--stdio"]),
    c("pyright-langserver", &["--stdio"]),
    c("pylsp", &[]),
    c("jedi-language-server", &[]),
    c("ruff", &["server"]),
];
const C_TYPESCRIPT: &[Candidate] = &[
    c("typescript-language-server", &["--stdio"]),
    c("vtsls", &["--stdio"]),
    c("deno", &["lsp"]),
];
const C_GO: &[Candidate] = &[c("gopls", &[])];
const C_C: &[Candidate] = &[c("clangd", &[]), c("ccls", &[])];
const C_JAVA: &[Candidate] = &[c("jdtls", &[])];
const C_RUBY: &[Candidate] = &[c("ruby-lsp", &[]), c("solargraph", &["stdio"])];
const C_PHP: &[Candidate] = &[
    c("intelephense", &["--stdio"]),
    c("phpactor", &["language-server"]),
];
const C_CSHARP: &[Candidate] = &[c("omnisharp", &["-lsp"])];
const C_KOTLIN: &[Candidate] = &[c("kotlin-language-server", &[])];
const C_SWIFT: &[Candidate] = &[c("sourcekit-lsp", &[])];
const C_SCALA: &[Candidate] = &[c("metals", &[])];
const C_ELIXIR: &[Candidate] = &[c("elixir-ls", &[]), c("lexical", &[])];
const C_HASKELL: &[Candidate] = &[c("haskell-language-server-wrapper", &["--lsp"])];
const C_LUA: &[Candidate] = &[c("lua-language-server", &[])];
const C_ZIG: &[Candidate] = &[c("zls", &[])];
const C_NIX: &[Candidate] = &[c("nil", &[]), c("nixd", &[])];
const C_JSON: &[Candidate] = &[
    c("vscode-json-language-server", &["--stdio"]),
    c("biome", &["lsp-proxy"]),
];
const C_YAML: &[Candidate] = &[c("yaml-language-server", &["--stdio"])];
const C_TOML: &[Candidate] = &[c("taplo", &["lsp", "stdio"])];
const C_HTML: &[Candidate] = &[c("vscode-html-language-server", &["--stdio"])];
const C_CSS: &[Candidate] = &[c("vscode-css-language-server", &["--stdio"])];
const C_MARKDOWN: &[Candidate] = &[c("marksman", &["server"]), c("harper-ls", &["--stdio"])];
const C_SQL: &[Candidate] = &[c("sqls", &[]), c("postgrestools", &["lsp-proxy"])];
const C_TERRAFORM: &[Candidate] = &[c("terraform-ls", &["serve"])];
const C_DOCKERFILE: &[Candidate] = &[c("docker-langserver", &["--stdio"])];
const C_SHELLSCRIPT: &[Candidate] = &[c("bash-language-server", &["start"])];
const C_R: &[Candidate] = &[c("air", &["language-server"])];
const C_DART: &[Candidate] = &[c("dart", &["language-server"])];

fn candidates(language: &str) -> &'static [Candidate] {
    match language {
        "rust" => C_RUST,
        "python" => C_PYTHON,
        "typescript" | "typescriptreact" | "javascript" | "javascriptreact" => C_TYPESCRIPT,
        "go" => C_GO,
        "c" | "cpp" => C_C,
        "java" => C_JAVA,
        "ruby" => C_RUBY,
        "php" => C_PHP,
        "csharp" => C_CSHARP,
        "kotlin" => C_KOTLIN,
        "swift" => C_SWIFT,
        "scala" => C_SCALA,
        "elixir" => C_ELIXIR,
        "haskell" => C_HASKELL,
        "lua" => C_LUA,
        "zig" => C_ZIG,
        "nix" => C_NIX,
        "json" => C_JSON,
        "yaml" => C_YAML,
        "toml" => C_TOML,
        "html" => C_HTML,
        "css" | "scss" | "less" => C_CSS,
        "markdown" => C_MARKDOWN,
        "sql" => C_SQL,
        "terraform" => C_TERRAFORM,
        "dockerfile" => C_DOCKERFILE,
        "shellscript" => C_SHELLSCRIPT,
        "r" => C_R,
        "dart" => C_DART,
        _ => &[],
    }
}

/// Directories inside a project that hold project-pinned tools.
///
/// Looked at before the machine, because a repository that ships its own
/// server has already decided, and its choice is the one its other tooling
/// agrees with.
fn project_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![
        root.join("node_modules/.bin"),
        root.join(".venv/bin"),
        root.join("venv/bin"),
        root.join(".venv/Scripts"),
        root.join("vendor/bin"),
        root.join(".tools/bin"),
        root.join("bin"),
    ];
    // A virtualenv named something else is still a virtualenv: any directory
    // with `pyvenv.cfg` in it is one, by definition.
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("pyvenv.cfg").is_file() {
                dirs.push(path.join("bin"));
                dirs.push(path.join("Scripts"));
            }
        }
    }
    dirs
}

/// Every rustup toolchain's `bin`, `stable` first.
///
/// Sorted for a stable answer across runs: `read_dir` order is arbitrary, and
/// a server that changes between runs is worse than either choice.
fn toolchain_dirs(home: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(home.join(".rustup/toolchains")) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries.flatten().map(|e| e.path().join("bin")).collect();
    found.sort();
    found.sort_by_key(|dir| !dir.to_string_lossy().contains("/stable-"));
    found
}

/// Directories tool installers use without joining `PATH`.
fn machine_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let home = home_dir();

    // Toolchain directories come BEFORE `PATH`, and only these do.
    //
    // `~/.cargo/bin/rust-analyzer` is on everyone's PATH and is not a
    // language server: it is a rustup shim that re-dispatches to whichever
    // toolchain the current directory selects. In a repository that pins a
    // toolchain lacking the component — this one does — the shim exits with
    // "Unknown binary 'rust-analyzer'" and Rust blocks get no intelligence at
    // all, while a working binary sits in the toolchain directory unused.
    // Nothing but rustc's own tools lives here, so preferring it cannot
    // shadow another language's server.
    if let Some(home) = &home {
        dirs.extend(toolchain_dirs(home));
    }

    if let Some(paths) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&paths));
    }
    let Some(home) = home else {
        return dirs;
    };
    for suffix in [
        ".local/bin",
        ".cargo/bin",
        "go/bin",
        ".bun/bin",
        ".deno/bin",
        ".npm-global/bin",
        ".local/share/nvim/mason/bin", // Mason, for Neovim users
        ".vscode/extensions",
        "Library/Application Support/Code/User/globalStorage",
    ] {
        dirs.push(home.join(suffix));
    }
    dirs
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn executable_in(dir: &Path, name: &str) -> Option<PathBuf> {
    // The Windows spellings are tried too: a server installed by npm is a
    // `.cmd` shim there, and looking only for the bare name finds nothing.
    [
        dir.join(name),
        dir.join(format!("{name}.exe")),
        dir.join(format!("{name}.cmd")),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

/// What was found for a language, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovered {
    /// Command and arguments, ready to spawn.
    pub command: Vec<String>,
    /// Where it came from, for reporting to a person: `project` or `machine`.
    pub origin: &'static str,
}

/// Find a server for `language`, preferring one the project pins.
///
/// `root` is the project directory; pass the document's directory when there
/// is nothing better.
pub fn discover(language: &str, root: &Path) -> Option<Discovered> {
    let candidates = candidates(language);
    if candidates.is_empty() {
        return None;
    }

    // Project first, in candidate preference order — a repository that pins
    // its own server has answered the question already.
    for candidate in candidates {
        for dir in project_dirs(root) {
            if let Some(path) = executable_in(&dir, candidate.bin) {
                return Some(Discovered {
                    command: with_args(path, candidate.args),
                    origin: "project",
                });
            }
        }
    }

    for candidate in candidates {
        for dir in machine_dirs() {
            if let Some(path) = executable_in(&dir, candidate.bin) {
                return Some(Discovered {
                    command: with_args(path, candidate.args),
                    origin: "machine",
                });
            }
        }
    }
    None
}

fn with_args(path: PathBuf, args: &[&str]) -> Vec<String> {
    let mut command = vec![path.to_string_lossy().to_string()];
    command.extend(args.iter().map(|a| (*a).to_string()));
    command
}

/// Every language this build knows candidates for — what a person can expect
/// to work without configuring anything.
pub fn known_languages() -> Vec<&'static str> {
    let mut out = vec![
        "rust",
        "python",
        "typescript",
        "javascript",
        "go",
        "c",
        "cpp",
        "java",
        "ruby",
        "php",
        "csharp",
        "kotlin",
        "swift",
        "scala",
        "elixir",
        "haskell",
        "lua",
        "zig",
        "nix",
        "json",
        "yaml",
        "toml",
        "html",
        "css",
        "markdown",
        "sql",
        "terraform",
        "dockerfile",
        "shellscript",
        "r",
        "dart",
    ];
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fake_executable(dir: &Path, name: &str) {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn a_project_pinned_server_beats_the_machine() {
        // A repository that ships its own server has already chosen, and its
        // choice is the one the rest of its tooling agrees with.
        let dir = tempfile::tempdir().unwrap();
        fake_executable(
            &dir.path().join("node_modules/.bin"),
            "typescript-language-server",
        );
        let found = discover("typescript", dir.path()).expect("found the pinned server");
        assert_eq!(found.origin, "project");
        assert!(
            found.command[0].contains("node_modules/.bin"),
            "{:?}",
            found.command
        );
        assert_eq!(found.command[1], "--stdio");
    }

    #[test]
    fn a_virtualenv_counts_as_the_project_even_when_oddly_named() {
        let dir = tempfile::tempdir().unwrap();
        let env = dir.path().join("some-env");
        fs::create_dir_all(&env).unwrap();
        fs::write(env.join("pyvenv.cfg"), "home = /usr\n").unwrap();
        fake_executable(&env.join("bin"), "pylsp");
        let found = discover("python", dir.path()).expect("found the venv server");
        assert_eq!(found.origin, "project");
        assert!(found.command[0].contains("some-env"), "{:?}", found.command);
    }

    #[test]
    fn preference_order_decides_when_several_are_present() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("node_modules/.bin");
        fake_executable(&bin, "pylsp");
        fake_executable(&bin, "pyright-langserver");
        let found = discover("python", dir.path()).unwrap();
        // pyright outranks pylsp in the table, so it wins even though both
        // are right there.
        assert!(
            found.command[0].ends_with("pyright-langserver"),
            "{:?}",
            found.command
        );
    }

    #[test]
    fn a_rustup_toolchain_outranks_the_shim_on_path() {
        // `~/.cargo/bin/rust-analyzer` is not a language server: it is a
        // rustup shim that re-dispatches to whatever toolchain the current
        // directory pins, and exits with "Unknown binary" when that toolchain
        // lacks the component. The real binary in the toolchain must be
        // searched first, or Rust blocks silently get nothing.
        let home = tempfile::tempdir().unwrap();
        let toolchains = home.path().join(".rustup/toolchains");
        for name in [
            "1.96.1-x86_64-unknown-linux-gnu",
            "stable-x86_64-unknown-linux-gnu",
        ] {
            fake_executable(&toolchains.join(name).join("bin"), "rust-analyzer");
        }
        let dirs = toolchain_dirs(home.path());
        assert_eq!(dirs.len(), 2);
        assert!(
            dirs[0].to_string_lossy().contains("stable-"),
            "stable must be searched first: {dirs:?}"
        );
        // And the ordering is the same on every run, whatever read_dir says.
        assert_eq!(dirs, toolchain_dirs(home.path()));
    }

    #[test]
    fn a_machine_with_no_rustup_looks_in_no_toolchains() {
        let home = tempfile::tempdir().unwrap();
        assert!(toolchain_dirs(home.path()).is_empty());
    }

    #[test]
    fn a_language_with_nothing_installed_reports_nothing_rather_than_a_substitute() {
        // Standing in another language's server would produce diagnostics the
        // project's own tooling disagrees with — worse than silence.
        let dir = tempfile::tempdir().unwrap();
        assert!(discover("cobol", dir.path()).is_none());
    }

    #[test]
    fn the_known_language_list_is_what_it_claims() {
        let languages = known_languages();
        for language in &languages {
            assert!(
                !candidates(language).is_empty(),
                "{language} is advertised but has no candidates"
            );
        }
        assert!(languages.contains(&"python"));
        assert!(languages.contains(&"rust"));
    }
}
