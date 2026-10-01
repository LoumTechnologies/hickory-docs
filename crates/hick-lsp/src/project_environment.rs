//! Host Python tools use the same selected environment as tests and debug launch.
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub fn boundary(root: &Path) -> &Path {
    root.ancestors()
        .find(|p| p.join(".git").exists())
        .unwrap_or(root)
}

pub fn plain_root(path: &Path, workspace: Option<PathBuf>) -> PathBuf {
    let base = workspace
        .filter(|w| path.starts_with(w))
        .unwrap_or_else(|| {
            path.ancestors()
                .find(|p| p.join(".git").exists())
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.parent().unwrap_or(path).to_path_buf())
        });
    if path.extension().is_some_and(|e| e == "py") {
        hick_project_env::providers::uv::owner(path.parent().unwrap_or(&base), &base)
            .unwrap_or(base)
    } else {
        base
    }
}

pub fn settings(kind: Option<&str>, root: &Path) -> Value {
    if crate::server_config::command_for("python").is_some() {
        return json!({});
    }
    let python = hick_project_env::python_interpreter(root, boundary(root));
    match kind {
        Some("pyright") => json!({ "python": { "pythonPath": python } }),
        Some("pylsp") => {
            json!({ "pylsp": { "plugins": { "jedi": { "environment": python.and_then(|p| p.parent()?.parent().map(Path::to_path_buf)) } } } })
        }
        _ => json!({}),
    }
}

/// LSP configuration items request sections, not the entire settings object.
pub fn configuration(settings: &Value, params: &Value) -> Value {
    let values: Vec<Value> = params
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|item| {
            let Some(section) = item.get("section").and_then(Value::as_str) else {
                return settings.clone();
            };
            section
                .split('.')
                .try_fold(settings, |v, part| v.get(part))
                .cloned()
                .unwrap_or(Value::Null)
        })
        .collect();
    json!(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configurations_return_the_requested_sections() {
        let settings = json!({"python":{"pythonPath":"/project/.venv/bin/python"}});
        assert_eq!(
            configuration(
                &settings,
                &json!({"items":[{"section":"python"},{"section":"python.pythonPath"},{"section":"unknown"}]})
            ),
            json!([{"pythonPath":"/project/.venv/bin/python"},"/project/.venv/bin/python",null])
        );
    }

    #[test]
    fn independent_python_projects_get_different_child_roots() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for name in ["a", "b"] {
            let project = root.join(name);
            std::fs::create_dir_all(&project).unwrap();
            std::fs::write(project.join("pyproject.toml"), "[project]\nname='fixture'\n").unwrap();
            std::fs::write(project.join("uv.lock"), "version=1\n").unwrap();
            assert_eq!(plain_root(&project.join("app.py"), Some(root.to_path_buf())), project);
        }
        assert_eq!(plain_root(&root.join("a/app.ts"), Some(root.to_path_buf())), root);
    }
}
