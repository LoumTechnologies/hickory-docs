//! Finding a debug adapter without being told where it is.
//!
//! The same play as `hick_lsp::discovery`, and deliberately the same rules,
//! because they were argued once already: the project's own copy wins over
//! the machine's, nothing is ever installed to satisfy a lookup, no network
//! is touched, and an adapter that is present but cannot work is **passed
//! over** rather than spawned to fail later.
//!
//! Debug adapters are less uniform than language servers, in one way that
//! shapes this module: several are not executables at all. `debugpy` is a
//! Python module you run with `-m`, and `js-debug` is a JavaScript file you
//! run with `node`. So a candidate is a *recipe* — an interpreter plus an
//! argument — rather than a binary name to find on `PATH`.

use std::path::{Path, PathBuf};

/// Where an adapter was found, for reporting to a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovered {
    /// Command and arguments, ready to spawn.
    pub command: Vec<String>,
    /// `project` or `machine`.
    pub origin: &'static str,
    /// The adapter's own name, for the report and for `<hick:needs>`.
    pub adapter: &'static str,
}

/// How one adapter is launched, once its home has been found.
enum Recipe {
    /// A binary on `PATH` or in a project directory: `dlv dap`.
    Binary {
        bin: &'static str,
        args: &'static [&'static str],
    },
    /// A Python module: `<python> -m debugpy.adapter`.
    PythonModule { module: &'static str },
    /// A JavaScript entry point run by node, relative to a package root.
    NodeScript {
        package: &'static str,
        entry: &'static str,
    },
}

struct Candidate {
    adapter: &'static str,
    recipe: Recipe,
}

fn candidates(language: &str) -> &'static [Candidate] {
    match language {
        "python" => C_PYTHON,
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => C_NODE,
        "go" => C_GO,
        "rust" | "c" | "cpp" => C_NATIVE,
        "ruby" => C_RUBY,
        "csharp" => C_CSHARP,
        _ => &[],
    }
}

const C_PYTHON: &[Candidate] = &[Candidate {
    adapter: "debugpy",
    recipe: Recipe::PythonModule {
        module: "debugpy.adapter",
    },
}];
const C_NODE: &[Candidate] = &[Candidate {
    adapter: "js-debug",
    recipe: Recipe::NodeScript {
        package: "@vscode/js-debug",
        entry: "src/dapDebugServer.js",
    },
}];
const C_GO: &[Candidate] = &[Candidate {
    adapter: "delve",
    recipe: Recipe::Binary {
        bin: "dlv",
        args: &["dap"],
    },
}];
const C_NATIVE: &[Candidate] = &[
    Candidate {
        adapter: "codelldb",
        recipe: Recipe::Binary {
            bin: "codelldb",
            args: &[],
        },
    },
    Candidate {
        // `lldb-dap` is what LLVM ships now; the older name is `lldb-vscode`.
        adapter: "lldb-dap",
        recipe: Recipe::Binary {
            bin: "lldb-dap",
            args: &[],
        },
    },
];
// netcoredbg (Samsung, MIT) is the C# adapter. Microsoft's `vsdbg` is
// licensed for use only with Visual Studio and VS Code, so it is not
// available to this product at all — not "not yet", not "behind a flag".
//
// netcoredbg ships as per-platform release archives and distro packages rather
// than as a `dotnet tool`, so `hick dap install csharp` needed an installer
// shape `tool_install` did not have — a URL and a pinned SHA-256 per platform.
// That shape exists now (`hickory-cli::dap_install`), so both roads are open:
// discovery still prefers a netcoredbg the user installed themselves.
const C_CSHARP: &[Candidate] = &[Candidate {
    adapter: "netcoredbg",
    recipe: Recipe::Binary {
        bin: "netcoredbg",
        args: &["--interpreter=vscode"],
    },
}];
const C_RUBY: &[Candidate] = &[Candidate {
    adapter: "rdbg",
    recipe: Recipe::Binary {
        bin: "rdbg",
        args: &["--open", "--stop-at-load"],
    },
}];

/// How to get an adapter for `language`, when discovery found none.
///
/// This is adapter knowledge, so it lives beside the candidates rather than
/// in the CLI: which ecosystems package an adapter as one installable command
/// is the same fact that decides what `hick dap install` can offer. Telling
/// someone to run a command that does not exist is worse than telling them
/// nothing — it costs them a shell round trip to find out.
///
/// `crates/hickory-cli/src/dap_install.rs` holds the catalogue that actually
/// installs, and a test there asserts the two agree.
pub fn how_to_get(language: &str) -> String {
    // Only two ecosystems package an adapter as one command hick can run
    // confined, and those two are what `hick dap install` offers. Every
    // other row here names the adapter and the ecosystem's own way to get
    // it — because this message used to say `hick dap install go`, which
    // prints "no installer for go" and costs a person a shell round trip to
    // find out.
    match language {
        "python" | "typescript" | "javascript" | "typescriptreact" | "javascriptreact"
        // netcoredbg ships as per-platform release archives rather than as
        // one installable command, which is why the catalogue grew an
        // archive shape rather than this row staying an exception.
        | "csharp" => {
            format!(
                "Install one with `hick dap install {language}`, or install it the way that \
                 ecosystem does — hick prefers whatever is already there."
            )
        }
        "go" => "delve is the Go debugger: `go install \
                 github.com/go-delve/delve/cmd/dlv@latest`, and hick will find `dlv` on PATH."
            .to_string(),
        "rust" | "c" | "cpp" => "hick uses LLVM's own adapter here. `lldb-dap` ships with LLVM \
                                 (`apt install lldb`, `brew install llvm`); codelldb is the \
                                 other one hick looks for. Either on PATH is enough."
            .to_string(),
        "ruby" => "rdbg comes from Ruby's debug gem: `gem install debug`, and hick will find \
                   `rdbg` on PATH."
            .to_string(),
        // A language with no candidates at all cannot get here through
        // `adapter_for`, but a caller asking directly deserves an answer.
        _ => format!(
            "hick has no debug adapter for {language}. `hick dap list` names the ones it does."
        ),
    }
}

/// Whether `how_to_get` points at `hick dap install`.
///
/// For the drift check in the CLI's catalogue, and nothing else.
pub fn suggests_hick_install(language: &str) -> bool {
    how_to_get(language).contains("hick dap install")
}

/// Every language this build can debug, for reporting and for tests.
pub fn known_languages() -> Vec<&'static str> {
    [
        "python",
        "typescript",
        // The React flavours, which `candidates` has always served through
        // the Node adapter and this list omitted — so a `.tsx` file was
        // reported as undebuggable while the adapter that debugs it was
        // installed. Same drift as `lang_detect`'s missing `cs`, on the other
        // side of the same feature.
        "typescriptreact",
        "javascript",
        "javascriptreact",
        "go",
        "rust",
        "c",
        "cpp",
        "ruby",
        // C# is listed only because the build step that makes it launchable
        // landed with it. On its own, this entry would make `hick dap list`
        // and `language_of` offer C#, pick `Program.cs`, and fail at launch —
        // worse than saying nothing. See `build.rs`.
        "csharp",
    ]
    .into()
}

/// Find an adapter for `language`, preferring one the project provides.
pub fn discover(language: &str, root: &Path) -> Option<Discovered> {
    for candidate in candidates(language) {
        if let Some(found) = resolve(candidate, root, true) {
            return Some(found);
        }
    }
    for candidate in candidates(language) {
        if let Some(found) = resolve(candidate, root, false) {
            return Some(found);
        }
    }
    None
}

fn resolve(candidate: &Candidate, root: &Path, project_only: bool) -> Option<Discovered> {
    let origin = if project_only { "project" } else { "machine" };
    match &candidate.recipe {
        Recipe::Binary { bin, args } => {
            let path = if project_only {
                search(&project_dirs(root), bin)?
            } else {
                search(&machine_dirs(), bin)?
            };
            Some(Discovered {
                command: once(path)
                    .chain(args.iter().map(|a| (*a).to_string()))
                    .collect(),
                origin,
                adapter: candidate.adapter,
            })
        }
        Recipe::PythonModule { module } => {
            // The interpreter matters more than the module: a project's
            // virtualenv has both its own python AND its own debugpy, and
            // debugging a cell with the machine's python against the venv's
            // debugpy is a mismatch that fails obscurely.
            let python = if project_only {
                search(&project_dirs(root), "python3")
                    .or_else(|| search(&project_dirs(root), "python"))?
            } else {
                search(&machine_dirs(), "python3").or_else(|| search(&machine_dirs(), "python"))?
            };
            // Present only if the module is importable BY THAT python.
            if !python_has_module(&python, module) {
                return None;
            }
            Some(Discovered {
                command: vec![python, "-m".into(), (*module).into()],
                origin,
                adapter: candidate.adapter,
            })
        }
        Recipe::NodeScript { package, entry } => {
            let script = if project_only {
                node_script(&project_dirs(root), package, entry)?
            } else {
                node_script(&machine_dirs(), package, entry)?
            };
            let node = search(&machine_dirs(), "node")?;
            Some(Discovered {
                command: vec![node, script],
                origin,
                adapter: candidate.adapter,
            })
        }
    }
}

fn once(value: String) -> std::iter::Once<String> {
    std::iter::once(value)
}

/// Can this interpreter import the module?
///
/// Asked by running it, because "the file is on disk" and "this python can
/// import it" are different questions — the second is the one that decides
/// whether the adapter starts, and answering the first taught us that lesson
/// already with `tsserver`.
fn python_has_module(python: &str, module: &str) -> bool {
    let root = module.split('.').next().unwrap_or(module);
    std::process::Command::new(python)
        .arg("-c")
        .arg(format!("import {root}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn node_script(dirs: &[PathBuf], package: &str, entry: &str) -> Option<String> {
    for dir in dirs {
        // `<…>/node_modules/.bin` -> `<…>/node_modules`
        let modules = if dir.ends_with(".bin") {
            dir.parent()?.to_path_buf()
        } else {
            dir.join("node_modules")
        };
        let candidate = modules.join(package).join(entry);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

fn search(dirs: &[PathBuf], name: &str) -> Option<String> {
    for dir in dirs {
        for spelling in [
            name.to_string(),
            format!("{name}.exe"),
            format!("{name}.cmd"),
        ] {
            let candidate = dir.join(spelling);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Where a project keeps its own tools. Deliberately the same list the
/// language-server discovery uses, including the walk to the repository root
/// that makes a monorepo work.
fn project_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for ancestor in ancestors_to_repo_root(root) {
        // The adapter cache belongs to the repository, not to whichever
        // directory a document happens to sit in. `hick dap install` run at
        // the top of a project has to serve a document three folders down,
        // or the install looks like it did nothing.
        dirs.push(ancestor.join(".hick-cache/adapters/python/bin"));
        dirs.push(ancestor.join(".hick-cache/adapters/node/node_modules/.bin"));
        // netcoredbg's archive unpacks to a directory of its own holding the
        // binary beside its managed DLLs, so the binary IS the directory's
        // name — there is no `bin/` to point at.
        dirs.push(ancestor.join(".hick-cache/adapters/netcoredbg"));
        for layout in [
            "node_modules/.bin",
            ".venv/bin",
            ".venv/Scripts",
            "venv/bin",
            "bin",
        ] {
            dirs.push(ancestor.join(layout));
        }
        if let Ok(entries) = std::fs::read_dir(&ancestor) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("pyvenv.cfg").is_file() {
                    dirs.push(path.join("bin"));
                    dirs.push(path.join("Scripts"));
                }
            }
        }
    }
    dirs
}

fn machine_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(paths) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&paths));
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home = PathBuf::from(home);
        for suffix in [
            ".local/bin",
            ".cargo/bin",
            "go/bin",
            ".local/share/uv/tools",
        ] {
            dirs.push(home.join(suffix));
        }
    }
    dirs
}

fn ancestors_to_repo_root(root: &Path) -> Vec<PathBuf> {
    // A relative root has no ancestors worth walking — `.` climbs to `""` and
    // stops — so make it absolute first. Callers pass whatever the run was
    // given, and "the adapter is not installed" is a bad way to find out that
    // a path was relative.
    let absolute = if root.as_os_str().is_empty() {
        // A document named without a directory gives the run an empty working
        // directory, and `absolute("")` is an error rather than the cwd.
        std::env::current_dir().ok()
    } else {
        std::path::absolute(root).ok()
    };
    let root = &absolute.unwrap_or_else(|| root.to_path_buf());
    let mut out = vec![root.to_path_buf()];
    if root.join(".git").exists() {
        return out;
    }
    for ancestor in root.ancestors().skip(1) {
        out.push(ancestor.to_path_buf());
        if ancestor.join(".git").exists() {
            return out;
        }
    }
    vec![root.to_path_buf()]
}

#[cfg(test)]
mod tests {

    /// Everything advertised as debuggable must have an adapter to try.
    #[test]
    fn known_languages_and_candidates_agree() {
        for language in known_languages() {
            assert!(
                !candidates(language).is_empty(),
                "`{language}` is advertised as debuggable and has no adapter candidate"
            );
        }
        for language in ["typescriptreact", "javascriptreact"] {
            assert!(
                known_languages().contains(&language),
                "`{language}` has an adapter and must be advertised, or `hick lang` \
                 calls a debuggable file undebuggable"
            );
        }
    }
    use super::*;
    use std::fs;

    fn executable(dir: &Path, name: &str) {
        executable_exiting(dir, name, 0);
    }

    /// A stand-in that exits with a chosen status.
    ///
    /// The status matters for `python3`: the module check RUNS the
    /// interpreter, so a fake that exits 0 for everything claims to import
    /// debugpy and the test proves the opposite of what it says. A real
    /// python without debugpy exits non-zero on the import.
    fn executable_exiting(dir: &Path, name: &str, code: i32) {
        fs::create_dir_all(dir).unwrap();
        // A `#!/bin/sh` file is not executable on Windows no matter what it
        // is called, and the python probe RUNS what it finds — so a shebang
        // stand-in there makes every adapter look absent and the tests pass
        // or fail for reasons that have nothing to do with discovery.
        // `search` already looks for a `.cmd` spelling, and cmd can run one.
        #[cfg(windows)]
        {
            let path = dir.join(format!("{name}.cmd"));
            fs::write(&path, format!("@echo off\r\nexit /b {code}\r\n")).unwrap();
        }
        #[cfg(not(windows))]
        {
            let path = dir.join(name);
            fs::write(&path, format!("#!/bin/sh\nexit {code}\n")).unwrap();
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn a_project_pinned_adapter_beats_the_machine() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        executable(&dir.path().join("bin"), "dlv");
        let found = discover("go", dir.path()).expect("found the project's dlv");
        assert_eq!(found.origin, "project");
        assert_eq!(found.adapter, "delve");
        assert_eq!(found.command.last().unwrap(), "dap");
    }

    #[test]
    fn an_installed_adapter_serves_a_document_further_down_the_tree() {
        // `hick dap install` writes one cache at the top of the project. A
        // document three folders down has to find it, or the install looks
        // like it did nothing.
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        executable(
            &dir.path().join(".hick-cache/adapters/python/bin"),
            "python3",
        );
        let deep = dir.path().join("docs/reports");
        fs::create_dir_all(&deep).unwrap();
        let found = discover("python", &deep).expect("found the project's adapter");
        assert_eq!(found.origin, "project");
        assert_eq!(found.adapter, "debugpy");
    }

    #[test]
    fn a_relative_root_still_walks_up_to_the_project() {
        // A run is given whatever working directory it was started with, and
        // `.` has no ancestors to walk: without absolutising it, the adapter
        // is reported missing on a machine where it is installed. Asserted on
        // the walk rather than by changing this process's directory, which
        // would race every other test in this binary.
        for relative in [".", ""] {
            let walked = ancestors_to_repo_root(Path::new(relative));
            assert!(walked[0].is_absolute(), "{relative:?} -> {walked:?}");
        }
        let walked = ancestors_to_repo_root(Path::new("."));
        assert!(
            walked.len() > 1 || walked[0].join(".git").exists(),
            "a relative root did not climb: {walked:?}"
        );
        assert!(walked[0].is_absolute(), "{walked:?}");
    }

    #[test]
    fn a_language_with_no_adapter_reports_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(discover("cobol", dir.path()).is_none());
    }

    #[test]
    fn a_python_without_debugpy_is_not_an_adapter() {
        // The tsserver lesson, applied before it can bite: a python is only
        // an adapter if it can actually import the module. A directory with
        // an executable called `python3` that cannot import debugpy must not
        // be offered.
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        executable_exiting(&dir.path().join("bin"), "python3", 1);
        let found = discover("python", dir.path());
        assert!(
            found.as_ref().is_none_or(|f| f.origin != "project"),
            "a python that cannot import debugpy was offered: {found:?}"
        );
    }

    #[test]
    fn a_node_adapter_is_a_script_run_by_node_not_a_binary() {
        // js-debug is a JavaScript file. Looking for an executable of that
        // name finds nothing, forever, on a machine where it is installed.
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        let pkg = dir.path().join("node_modules/@vscode/js-debug/src");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("dapDebugServer.js"), "// stand-in\n").unwrap();
        if search(&machine_dirs(), "node").is_none() {
            eprintln!("SKIPPED: no node on this machine to run it with");
            return;
        }
        let found = discover("typescript", dir.path()).expect("found js-debug");
        // `node` on Unix, `node.exe` on Windows — the stem is the claim.
        assert_eq!(
            Path::new(&found.command[0])
                .file_stem()
                .and_then(|stem| stem.to_str()),
            Some("node"),
            "{:?}",
            found.command
        );
        assert!(
            found.command[1].ends_with("dapDebugServer.js"),
            "{:?}",
            found.command
        );
    }

    #[test]
    fn every_known_language_has_a_candidate() {
        for language in known_languages() {
            assert!(
                !candidates(language).is_empty(),
                "{language} is advertised as debuggable and has no adapter"
            );
        }
    }
}
