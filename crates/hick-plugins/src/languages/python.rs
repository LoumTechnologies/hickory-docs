use super::{dap, dap_install, index_install, lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "python",
    languages: &["python"],
    interfaces: &[
        Interface::SessionWrites {
            aliases: &["python", "py"],
            extract: crate::session_writes::extract_python_writes,
        },
        Interface::EditorEnvironment {
            kinds: &["pyright", "pylsp"],
            settings: editor_settings,
            root: editor_root,
        },
        Interface::Structure {
            languages: &["python"],
            language: |_| "python",
            grammar: |_| Some(tree_sitter_python::LANGUAGE.into()),
        },
        Interface::ProjectEnvironment {
            provider: || Box::new(hick_project_env::providers::uv::Uv),
        },
        Interface::Project {
            markers: &[
                "pyproject.toml",
                "pytest.ini",
                "setup.py",
                "requirements.txt",
            ],
        },
        Interface::RuntimeObservation {
            provider: "cpython-hooks-v1",
            script: include_str!("python_observer.py"),
        },
        Interface::Profiling {
            provider: "cpython-cprofile-v1",
            script: include_str!("python_observer.py"),
        },
        Interface::TypedClient {
            languages: &["python"],
            target: crate::typed_client::emit::Target::Python,
        },
        Interface::TestRunner { plan: test_command },
        Interface::Formula(FormulaBackend {
            language: "python",
            aliases: &["python", "py", "python3"],
            file_name: "python_backend.py",
            interpreters: &["python3", "python"],
            source: include_str!("python_backend.py"),
        }),
        Interface::Syntax {
            file_names: &[],
            extensions: &[("py", "python", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_PYTHON,
            project_dirs: &[],
            installer: Some(&lsp_install::PYTHON),
        },
        Interface::Dap {
            candidates: dap::C_PYTHON,
            project_dirs: &[],
            installer: Some(&dap_install::PYTHON),
        },
        Interface::Indexer {
            bin: "scip-python",
            args: &["index", "."],
            output_flag: "--output",
            installer: Some(&index_install::PYTHON),
        },
    ],
};

fn test_command(
    root: &std::path::Path,
    file: &std::path::Path,
    name: &str,
) -> Result<crate::testing::TestCommand, String> {
    use crate::testing::{TestCommand, nearest};
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

fn editor_root(path: &std::path::Path, base: &std::path::Path) -> std::path::PathBuf {
    hick_project_env::providers::uv::owner(path.parent().unwrap_or(base), base)
        .unwrap_or_else(|| base.to_path_buf())
}
fn editor_settings(kind: Option<&str>, root: &std::path::Path) -> serde_json::Value {
    use serde_json::json;
    let boundary = root
        .ancestors()
        .find(|p| p.join(".git").exists())
        .unwrap_or(root);
    let python = hick_project_env::python_interpreter(root, boundary);
    match kind {
        Some("pyright") => json!({ "python": { "pythonPath": python } }),
        Some("pylsp") => {
            json!({ "pylsp": { "plugins": { "jedi": { "environment": python.and_then(|p| p.parent()?.parent().map(std::path::Path::to_path_buf)) } } } })
        }
        _ => json!({}),
    }
}
