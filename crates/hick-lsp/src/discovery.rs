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
    let mut dirs = Vec::new();

    // What `hick lsp install` put there, first: a server this project asked
    // for outranks whatever happens to be on the machine, for the same
    // reason a project-pinned one does.
    dirs.push(root.join(".hick-cache/servers/node/node_modules/.bin"));
    dirs.push(root.join(".hick-cache/servers/python/bin"));
    dirs.push(root.join(".hick-cache/servers/python/Scripts"));

    // Then each directory from here up to the repository root.
    //
    // Walking up rather than looking only at `root` is what makes a monorepo
    // work. In one, `apps/web/` has no `node_modules` of its own — the
    // workspace root has it, and every package manager that supports
    // workspaces (npm, yarn, pnpm, bun) hoists the binaries there. A document
    // in `apps/web/` whose search stopped at `apps/web/` would find nothing
    // and report the language as unsupported, on a machine where the server
    // is installed and every other tool in the repository finds it.
    for ancestor in ancestors_to_repo_root(root) {
        for layout in PROJECT_LAYOUTS {
            dirs.push(ancestor.join(layout));
        }
        dirs.extend(package_manager_dirs(&ancestor));
    }
    dirs
}

/// Fixed per-package-manager locations, in preference order.
///
/// Each is where that ecosystem's own tooling looks, so a server installed
/// the ordinary way for that ecosystem is found without anyone saying where.
const PROJECT_LAYOUTS: &[&str] = &[
    // npm, yarn, pnpm and bun all link executables here.
    "node_modules/.bin",
    // The conventional virtualenv names, and Windows' spelling of `bin`.
    ".venv/bin",
    ".venv/Scripts",
    "venv/bin",
    "venv/Scripts",
    "env/bin",
    // composer (PHP).
    "vendor/bin",
    // Bundler binstubs (Ruby) and the generic project-local convention.
    "bin",
    ".tools/bin",
];

/// Locations that need a glob or a probe rather than a fixed path.
fn package_manager_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // A virtualenv named something else is still a virtualenv: any directory
    // with `pyvenv.cfg` in it is one, by definition. This covers poetry's
    // in-project envs, pipenv with PIPENV_VENV_IN_PROJECT, and whatever a
    // person called theirs.
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("pyvenv.cfg").is_file() {
                dirs.push(path.join("bin"));
                dirs.push(path.join("Scripts"));
            }
        }
    }

    // PDM's PEP 582 layout: `__pypackages__/<python-version>/bin`.
    push_globbed(&mut dirs, &root.join("__pypackages__"), &["bin", "Scripts"]);
    // conda / mamba environments kept inside the project.
    push_globbed(&mut dirs, &root.join("envs"), &["bin", "Scripts"]);
    // Bundler's vendored install: `vendor/bundle/ruby/<abi>/bin`.
    push_globbed(&mut dirs, &root.join("vendor/bundle/ruby"), &["bin"]);

    dirs
}

/// Add `<parent>/<child>/<leaf>` for every child directory of `parent`.
///
/// The version component in these layouts is the Python or Ruby version, and
/// hardcoding one would work until the user upgraded.
fn push_globbed(dirs: &mut Vec<PathBuf>, parent: &Path, leaves: &[&str]) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    // Newest last in lexical order, so the highest version is preferred —
    // and sorted at all, so the answer does not change between runs.
    children.sort();
    for child in children.into_iter().rev() {
        for leaf in leaves {
            dirs.push(child.join(leaf));
        }
    }
}

/// This directory and its ancestors, stopping at the repository root.
///
/// Bounded by `.git` so the search never wanders into a parent that has
/// nothing to do with this project — someone's home directory with a stray
/// `node_modules` in it should not decide which server a repository uses.
/// Without a `.git` anywhere, only the directory itself is searched, because
/// there is no way to tell where the project ends.
fn ancestors_to_repo_root(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    if !root.join(".git").exists() {
        for ancestor in root.ancestors().skip(1) {
            out.push(ancestor.to_path_buf());
            if ancestor.join(".git").exists() {
                return out;
            }
        }
        // No repository root found: the walk was unbounded, so trust only
        // the directory we started in.
        return vec![root.to_path_buf()];
    }
    out
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
    /// `initializationOptions` this server needs to work here, if any.
    ///
    /// Configuration and not arguments because that is where these servers
    /// take it: `typescript-language-server` HAD a `--tsserver-path` flag and
    /// removed it in 4.x, so passing one now makes the server exit with
    /// `unknown option` before it has said anything — which surfaces as
    /// "the language server crashed", several layers from the cause.
    pub init_options: Option<serde_json::Value>,
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
                if !usable(candidate.bin, &path, root) {
                    continue;
                }
                return Some(Discovered {
                    command: with_args(path.clone(), candidate.args),
                    origin: "project",
                    init_options: init_options(candidate.bin, &path, root),
                });
            }
        }
    }

    for candidate in candidates {
        for dir in machine_dirs() {
            if let Some(path) = executable_in(&dir, candidate.bin) {
                if !usable(candidate.bin, &path, root) {
                    continue;
                }
                return Some(Discovered {
                    command: with_args(path.clone(), candidate.args),
                    origin: "machine",
                    init_options: init_options(candidate.bin, &path, root),
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

/// Configuration a server needs that depends on where things are, not on
/// which server it is.
///
/// So far exactly one server needs this, and the reason generalises badly,
/// so it is written out rather than turned into a mechanism.
///
/// `typescript-language-server` is a wrapper: the actual analysis is
/// `tsserver`, from the `typescript` package, which it locates relative to
/// the WORKSPACE ROOT. Our workspace root is the directory the virtual files
/// are written to, and that directory has no `node_modules` — so the server
/// starts, fails `initialize` with "Could not find a valid tsserver", and
/// every TypeScript block in every document silently gets nothing. Pointing
/// it at a tsserver is what editors do too.
///
/// The project's own copy is preferred over the one beside the server: a
/// repository pinning a TypeScript version means that version, and analysing
/// its code with a different one is how a document ends up disagreeing with
/// the project's build.
fn init_options(bin: &str, discovered: &Path, root: &Path) -> Option<serde_json::Value> {
    if bin != "typescript-language-server" {
        return None;
    }
    let tsserver = find_tsserver(discovered, root)?;
    Some(serde_json::json!({
        "tsserver": { "path": tsserver.to_string_lossy() }
    }))
}

/// A `tsserver.js` this server could actually use, if one exists.
fn find_tsserver(discovered: &Path, root: &Path) -> Option<PathBuf> {
    tsserver_near(root).or_else(|| discovered.parent().and_then(tsserver_beside))
}

/// Is this candidate worth spawning, given where it was found?
///
/// A candidate can be installed and still unusable, and spawning it anyway
/// is worse than not finding it: the child fails `initialize`, every request
/// for that language returns nothing, and the editor looks broken in a way
/// that points at us rather than at the install.
///
/// One rule so far. `typescript-language-server` is a wrapper around
/// `tsserver`, which comes from the `typescript` package — and `typescript`
/// on npm is 7.x now, the native port, which ships no `tsserver.js`. So the
/// obvious install produces a server that cannot start, and the honest thing
/// is to pass over it and try the next candidate.
fn usable(bin: &str, discovered: &Path, root: &Path) -> bool {
    if bin != "typescript-language-server" {
        return true;
    }
    find_tsserver(discovered, root).is_some()
}

/// `node_modules/typescript/lib/tsserver.js` from here up to the repo root.
fn tsserver_near(root: &Path) -> Option<PathBuf> {
    ancestors_to_repo_root(root)
        .into_iter()
        .find_map(|dir| tsserver_in(&dir.join("node_modules")))
}

/// The `typescript` package sitting beside a discovered `.bin` entry.
fn tsserver_beside(bin_dir: &Path) -> Option<PathBuf> {
    // `<…>/node_modules/.bin/typescript-language-server` → `<…>/node_modules`
    tsserver_in(bin_dir.parent()?)
}

fn tsserver_in(node_modules: &Path) -> Option<PathBuf> {
    let candidate = node_modules.join("typescript/lib/tsserver.js");
    candidate.is_file().then_some(candidate)
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

    /// A `typescript` package beside a planted wrapper.
    ///
    /// Without it the fixture is not a TypeScript install but a broken one:
    /// the wrapper needs a `tsserver.js` to wrap, and discovery passes over
    /// one that has none. Planting it makes the fixture model what npm
    /// actually produces.
    fn fake_tsserver(node_modules: &Path) {
        let lib = node_modules.join("typescript/lib");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("tsserver.js"), "// stand-in\n").unwrap();
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
        fake_tsserver(&dir.path().join("node_modules"));
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
    fn a_server_this_project_installed_is_found_without_configuration() {
        // The point of `hick lsp install`: nothing is written to a config
        // file afterwards, so if discovery does not look here, the install
        // silently does nothing.
        let dir = tempfile::tempdir().unwrap();
        fake_executable(
            &dir.path().join(".hick-cache/servers/python/bin"),
            "basedpyright-langserver",
        );
        let found = discover("python", dir.path()).expect("the installed server is found");
        assert_eq!(found.origin, "project");
        assert!(
            found.command[0].contains(".hick-cache"),
            "{:?}",
            found.command
        );
    }

    #[test]
    fn a_monorepo_package_finds_the_workspace_root_install() {
        // npm, yarn, pnpm and bun all hoist workspace binaries to the root.
        // A document in `apps/web` whose search stopped there would report
        // TypeScript as unsupported in a repository where every other tool
        // finds the server.
        let repo = tempfile::tempdir().unwrap();
        fs::create_dir_all(repo.path().join(".git")).unwrap();
        fake_executable(
            &repo.path().join("node_modules/.bin"),
            "typescript-language-server",
        );
        fake_tsserver(&repo.path().join("node_modules"));
        let package = repo.path().join("apps/web");
        fs::create_dir_all(&package).unwrap();

        let found = discover("typescript", &package).expect("the hoisted server is found");
        assert_eq!(found.origin, "project");
    }

    #[test]
    fn the_walk_stops_at_the_repository_root() {
        // Someone's home directory with a stray node_modules in it must not
        // decide which server a repository uses.
        let outer = tempfile::tempdir().unwrap();
        let planted = outer.path().join("node_modules/.bin");
        fake_executable(&planted, "typescript-language-server");
        let repo = outer.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();

        // Asserting "finds nothing" would be asserting something else: on a
        // machine that HAS a TypeScript server installed — as CI now does —
        // discovery correctly finds that one, and a test demanding absence
        // fails for a reason that has nothing to do with the walk. What the
        // walk promises is narrower: never the one outside the repository.
        if let Some(found) = discover("typescript", &repo) {
            assert!(
                !found.command[0].starts_with(&planted.to_string_lossy().to_string()),
                "the search escaped the repository and used {}",
                found.command[0]
            );
            assert_eq!(
                found.origin, "machine",
                "a server outside the repository was reported as the project's own: {:?}",
                found.command
            );
        }
    }

    #[test]
    fn pdms_pep_582_layout_is_a_project_install() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fake_executable(
            &dir.path().join("__pypackages__/3.12/bin"),
            "pyright-langserver",
        );
        let found = discover("python", dir.path()).expect("found the PDM install");
        assert_eq!(found.origin, "project");
        assert!(
            found.command[0].contains("__pypackages__"),
            "{:?}",
            found.command
        );
    }

    #[test]
    fn the_newest_versioned_directory_wins_and_does_not_vary() {
        // The version component is the Python (or Ruby) version; a machine
        // with two must pick the higher one, and pick the SAME one every run.
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        for version in ["3.11", "3.12"] {
            fake_executable(
                &dir.path().join(format!("__pypackages__/{version}/bin")),
                "pyright-langserver",
            );
        }
        let found = discover("python", dir.path()).unwrap();
        assert!(found.command[0].contains("3.12"), "{:?}", found.command);
        assert_eq!(
            found.command,
            discover("python", dir.path()).unwrap().command
        );
    }

    #[test]
    fn a_bundler_vendored_install_is_found() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fake_executable(&dir.path().join("vendor/bundle/ruby/3.3.0/bin"), "ruby-lsp");
        let found = discover("ruby", dir.path()).expect("found the bundled server");
        assert_eq!(found.origin, "project");
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
