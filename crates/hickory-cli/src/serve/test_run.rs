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
//! The command comes from the manifest the walk finds. It is normalized as a
//! launch profile and uses the same terminal runner as optional manual profiles.

use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

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
    /// Optional current-buffer fingerprint; checked against saved bytes before admission.
    pub expected_source_hash: Option<String>,
}

pub use hick_plugins::testing::{TestCommand, test_command};

/// `POST /api/tests/run` — run one test (or a file's tests) in a terminal
/// session named after it. Answers the session, the way `POST /api/terminals`
/// does, so the client can show it.
pub async fn run(
    State(state): State<LocalState>,
    Json(body): Json<RunBody>,
) -> ApiResult<Json<Value>> {
    let root = state
        .index
        .root()
        .canonicalize()
        .map_err(|_| ApiError::unprocessable("Workspace is unavailable"))?;
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
    let real_file = file
        .canonicalize()
        .map_err(|_| ApiError::unprocessable("Test source is unavailable"))?;
    if !real_file.starts_with(&root) {
        return Err(ApiError::unprocessable(
            "Test source resolves outside the open folder",
        ));
    }
    let source = std::fs::read(&real_file)
        .map_err(|_| ApiError::unprocessable("Test source is unreadable"))?;
    let source_hash = format!("{:x}", Sha256::digest(&source));
    if let Some(expected) = &body.expected_source_hash {
        if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ApiError::bad_request(
                "A SHA-256 source fingerprint is required",
            ));
        }
        if !source_hash.eq_ignore_ascii_case(expected) {
            return Err(ApiError::conflict(
                "The editor differs from the saved test source. Save or reload before running this test",
            ));
        }
    }
    let command =
        test_command(&root, &file, &body.name, &body.language).map_err(ApiError::unprocessable)?;
    let cwd = command
        .cwd
        .strip_prefix(&root)
        .map_err(|e| ApiError::unprocessable(e.to_string()))?
        .to_string_lossy()
        .replace('\\', "/");
    let profile = crate::launch_profiles::command_profile(
        format!("test:{}:{}", body.path, body.name),
        command.title,
        command.argv,
        if cwd.is_empty() { ".".into() } else { cwd },
        "test gutter",
    );
    let summary = super::launch_profiles::open_run(
        &state,
        &profile,
        &root,
        &root,
        None,
        Some(profile.definition.name.clone()),
    )
    .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    let mut reply = json!(summary);
    reply["source_hash"] = json!(source_hash);
    reply["source_checked_at_admission"] = json!(body.expected_source_hash.is_some());
    reply["source_frozen"] = json!(false);
    Ok(Json(reply))
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
    fn jsx_and_tsx_aliases_retain_the_focused_test_filter() {
        let (_dir, root) = project();
        for language in ["jsx", "tsx"] {
            let command =
                test_command(&root, &root.join("web/src/a.test.ts"), "focused", language).unwrap();
            assert_eq!(
                command.argv,
                vec!["npx", "vitest", "run", "src/a.test.ts", "-t", "focused"]
            );
        }
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
