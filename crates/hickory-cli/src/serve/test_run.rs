//! Run one test, from the line it is written on.
//!
//! The editor draws a run mark beside every test it can name — `#[test]`,
//! `it("…")`, `def test_…`, `[Fact]`, `func TestX` — and a click arrives
//! here as the file, the test's name and its language. What runs is that
//! ecosystem's own test command, in the nearest directory that owns the file
//! (the closest `Cargo.toml`, `package.json`, `pyproject.toml`, `.csproj` or
//! `go.mod`), as a **terminal session** named after the test: the output is
//! the test runner's own, in a terminal a person can read and scroll, and
//! the session says when it has finished or failed the way every other
//! session in the dock does.
//!
//! A terminal, not a summary, for the reason the build is watched rather
//! than reported: a failing test says why in its own words, and "1 failed"
//! throws all of that away. Nothing here is recorded, verified or woven —
//! the transcript is the record; the terminal is the run happening.
//!
//! Found without being configured: there is no `launch.json` and there must
//! not be. The command comes from the manifest the walk finds, which is the
//! same fact that decides which test runner the project uses.

use std::path::{Path, PathBuf};

use axum::Json;
use axum::extract::State;
use hick_term::SessionSpec;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct RunBody {
    /// The file, relative to the open folder.
    pub path: String,
    /// The test's name as written — `fn name`, `it("name")`, `def test_x`.
    /// Empty means every test in the file.
    #[serde(default)]
    pub name: String,
    /// The language id the editor routed the file to.
    pub language: String,
}

/// What will run, and where.
#[derive(Debug, PartialEq, Eq)]
pub struct TestCommand {
    pub title: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
}

/// The nearest ancestor of `file` (itself included) holding `manifest`,
/// stopping at `root`.
fn nearest(root: &Path, file: &Path, manifest: &str) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        if dir.join(manifest).is_file() {
            return Some(dir.to_path_buf());
        }
        if dir == root {
            return None;
        }
        dir = dir.parent()?;
    }
}

/// The ecosystem's own command for one test in `file`, or its reason why
/// not. `file` is absolute and under `root`.
pub fn test_command(
    root: &Path,
    file: &Path,
    name: &str,
    language: &str,
) -> Result<TestCommand, String> {
    let rel = file
        .strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    let name = name.trim();
    let title = if name.is_empty() {
        format!("test: {rel}")
    } else {
        format!("test: {name}")
    };
    let s = |text: &str| text.to_string();
    match language {
        "rust" => {
            let dir = nearest(root, file, "Cargo.toml").ok_or_else(|| {
                format!("{rel} is not inside a cargo package: no Cargo.toml above it")
            })?;
            let mut argv = vec![s("cargo"), s("test")];
            if !name.is_empty() {
                argv.push(name.to_string());
            }
            Ok(TestCommand {
                title,
                cwd: dir,
                argv,
            })
        }
        "python" => {
            let dir = nearest(root, file, "pyproject.toml")
                .or_else(|| nearest(root, file, "pytest.ini"))
                .or_else(|| nearest(root, file, "setup.py"))
                .unwrap_or_else(|| root.to_path_buf());
            let rel_to_dir = file
                .strip_prefix(&dir)
                .unwrap_or(file)
                .to_string_lossy()
                .replace('\\', "/");
            let target = if name.is_empty() {
                rel_to_dir
            } else {
                format!("{rel_to_dir}::{name}")
            };
            Ok(TestCommand {
                title,
                cwd: dir,
                argv: vec![
                    hick_project_env::python_interpreter(file.parent().unwrap_or(root), root)
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_else(|| s("python3")),
                    s("-m"),
                    s("pytest"),
                    target,
                ],
            })
        }
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => {
            let dir = nearest(root, file, "package.json").ok_or_else(|| {
                format!("{rel} is not inside a package: no package.json above it")
            })?;
            let rel_to_dir = file
                .strip_prefix(&dir)
                .unwrap_or(file)
                .to_string_lossy()
                .replace('\\', "/");
            // vitest and jest share the `-t <name>` spelling; which one is
            // the package's business, read from its own manifest.
            let manifest = std::fs::read_to_string(dir.join("package.json")).unwrap_or_default();
            let runner = if manifest.contains("\"vitest\"") {
                vec![s("npx"), s("vitest"), s("run"), rel_to_dir]
            } else if manifest.contains("\"jest\"") {
                vec![s("npx"), s("jest"), rel_to_dir]
            } else {
                return Err(format!(
                    "{} names neither vitest nor jest, so hick does not know how to run {rel}",
                    dir.join("package.json").display()
                ));
            };
            let mut argv = runner;
            if !name.is_empty() {
                argv.push(s("-t"));
                argv.push(name.to_string());
            }
            Ok(TestCommand {
                title,
                cwd: dir,
                argv,
            })
        }
        "csharp" => {
            let dir = nearest_glob(root, file, ".csproj")
                .ok_or_else(|| format!("{rel} is not inside a project: no .csproj above it"))?;
            let mut argv = vec![s("dotnet"), s("test")];
            if !name.is_empty() {
                argv.push(s("--filter"));
                argv.push(format!("FullyQualifiedName~{name}"));
            }
            Ok(TestCommand {
                title,
                cwd: dir,
                argv,
            })
        }
        "go" => {
            nearest(root, file, "go.mod")
                .ok_or_else(|| format!("{rel} is not inside a module: no go.mod above it"))?;
            let dir = file.parent().unwrap_or(root).to_path_buf();
            let mut argv = vec![s("go"), s("test"), s(".")];
            if !name.is_empty() {
                argv.push(s("-run"));
                argv.push(format!("^{name}$"));
            }
            Ok(TestCommand {
                title,
                cwd: dir,
                argv,
            })
        }
        other => Err(format!(
            "hick does not know how to run {other} tests. It runs cargo, pytest, vitest or \
             jest, dotnet test and go test; open a terminal on the file's folder for anything \
             else."
        )),
    }
}

/// The nearest ancestor holding a file with `extension`.
fn nearest_glob(root: &Path, file: &Path, extension: &str) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        if let Ok(entries) = std::fs::read_dir(dir)
            && entries.flatten().any(|e| {
                e.path()
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(extension))
            })
        {
            return Some(dir.to_path_buf());
        }
        if dir == root {
            return None;
        }
        dir = dir.parent()?;
    }
}

/// `POST /api/tests/run` — run one test (or a file's tests) in a terminal
/// session named after it. Answers the session, the way `POST /api/terminals`
/// does, so the client can show it.
pub async fn run(
    State(state): State<LocalState>,
    Json(body): Json<RunBody>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    if body.path.is_empty()
        || Path::new(&body.path).is_absolute()
        || body.path.split('/').any(|p| p == "..")
    {
        return Err(ApiError::bad_request(format!(
            "{:?} is not a path inside the open folder",
            body.path
        )));
    }
    let file = root.join(&body.path);
    if !file.is_file() {
        return Err(ApiError::not_found(format!(
            "{} is not a file here",
            body.path
        )));
    }
    let command =
        test_command(&root, &file, &body.name, &body.language).map_err(ApiError::unprocessable)?;
    let session = state
        .terminals
        .open(SessionSpec {
            title: command.title,
            cwd: command.cwd,
            argv: command.argv,
            monitor: false,
        })
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!(session.summary())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("crates/a/src")).unwrap();
        std::fs::write(
            root.join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\n",
        )
        .unwrap();
        std::fs::write(root.join("crates/a/src/lib.rs"), "").unwrap();
        std::fs::create_dir_all(root.join("web/src")).unwrap();
        std::fs::write(
            root.join("web/package.json"),
            "{\"devDependencies\":{\"vitest\":\"3\"}}",
        )
        .unwrap();
        std::fs::write(root.join("web/src/a.test.ts"), "").unwrap();
        std::fs::write(root.join("pyproject.toml"), "[project]\n").unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(root.join("tests/test_x.py"), "").unwrap();
        (dir, root)
    }

    #[test]
    fn a_rust_test_runs_in_its_own_crate() {
        let (_dir, root) = project();
        let cmd = test_command(&root, &root.join("crates/a/src/lib.rs"), "adds", "rust").unwrap();
        assert_eq!(cmd.cwd, root.join("crates/a"));
        assert_eq!(cmd.argv, vec!["cargo", "test", "adds"]);
        assert_eq!(cmd.title, "test: adds");
    }

    #[test]
    fn a_vitest_test_is_named_to_its_runner() {
        let (_dir, root) = project();
        let cmd = test_command(
            &root,
            &root.join("web/src/a.test.ts"),
            "renders",
            "typescript",
        )
        .unwrap();
        assert_eq!(cmd.cwd, root.join("web"));
        assert_eq!(
            cmd.argv,
            vec!["npx", "vitest", "run", "src/a.test.ts", "-t", "renders"]
        );
    }

    #[test]
    fn a_python_test_is_addressed_by_file_and_name() {
        let (_dir, root) = project();
        let cmd = test_command(&root, &root.join("tests/test_x.py"), "test_it", "python").unwrap();
        assert_eq!(cmd.cwd, root);
        assert_eq!(
            cmd.argv,
            vec!["python3", "-m", "pytest", "tests/test_x.py::test_it"]
        );
        // The whole file, when no test is named.
        let all = test_command(&root, &root.join("tests/test_x.py"), "", "python").unwrap();
        assert_eq!(all.argv.last().unwrap(), "tests/test_x.py");
        assert_eq!(all.title, "test: tests/test_x.py");
    }

    #[test]
    fn a_file_with_no_manifest_above_it_is_refused_by_name() {
        let (_dir, root) = project();
        std::fs::write(root.join("stray.rs"), "").unwrap();
        let err = test_command(&root, &root.join("stray.rs"), "x", "rust").unwrap_err();
        assert!(err.contains("Cargo.toml"), "{err}");
        let err = test_command(&root, &root.join("stray.rs"), "x", "haskell").unwrap_err();
        assert!(err.contains("does not know"), "{err}");
    }
}
