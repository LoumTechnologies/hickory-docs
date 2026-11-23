//! Immutable commit readings: message and exact before/after blobs, never a checkout.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    revision,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::Path, process::Command};

#[derive(Deserialize)]
pub struct CommitQuery {
    pub sha: String,
    pub base: Option<String>,
    pub path: Option<String>,
}

fn blob(root: &Path, revision: Option<&str>, path: &str) -> ApiResult<(String, bool)> {
    let Some(revision) = revision else {
        return Ok((String::new(), false));
    };
    let object = format!("{revision}:{path}");
    let kind = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-t", &object])
        .output()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !kind.status.success() {
        return Ok((String::new(), false));
    }
    if kind.stdout != b"blob\n" {
        return Ok((String::new(), true));
    }
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "blob", &object])
        .output()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !result.status.success() {
        return Err(ApiError::unprocessable(
            "Could not read this commit's file.",
        ));
    }
    if result.stdout.contains(&0) {
        return Ok((String::new(), true));
    }
    match String::from_utf8(result.stdout) {
        Ok(text) => Ok((text, false)),
        Err(_) => Ok((String::new(), true)),
    }
}

pub async fn read(
    State(state): State<LocalState>,
    Query(query): Query<CommitQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || read_history(&root, query).map(Json))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
}

fn read_history(root: &Path, query: CommitQuery) -> ApiResult<Value> {
    let sha = revision::resolve(root, &query.sha)?;
    if sha == "INDEX" {
        return Err(ApiError::bad_request("Choose a commit to read."));
    }
    let comparison = query.base.is_some();
    let previous = if let Some(reference) = query.base {
        let base = revision::resolve(root, &reference)?;
        if base == "INDEX" {
            return Err(ApiError::bad_request(
                "Choose a commit as the comparison base.",
            ));
        }
        Some(base)
    } else {
        let parents = revision::git(root, &["rev-list", "--parents", "-n", "1", &sha])?;
        parents.split_whitespace().nth(1).map(str::to_owned)
    };
    let parent = previous.as_deref();
    let message = if comparison {
        String::new()
    } else {
        revision::git(root, &["show", "-s", "--format=%B", &sha])?
    };
    let prefix = revision::git(root, &["rev-parse", "--show-prefix"])?;
    let prefix = prefix.strip_suffix('\n').unwrap_or(&prefix);
    let changed = if let Some(parent) = parent {
        revision::git(
            root,
            &[
                "diff",
                "--no-relative",
                "--name-status",
                "-z",
                "-M",
                parent,
                &sha,
                "--",
                ".",
            ],
        )?
    } else {
        revision::git(
            root,
            &[
                "diff-tree",
                "--root",
                "--no-commit-id",
                "-r",
                "--name-status",
                "-z",
                &sha,
                "--",
                ".",
            ],
        )?
    };
    let mut tokens = changed.split_terminator('\0');
    let mut files = Vec::new();
    while let Some(status) = tokens.next() {
        let first = tokens
            .next()
            .ok_or_else(|| ApiError::internal("Incomplete git change."))?;
        let (old_path, path) = if status.starts_with('R') || status.starts_with('C') {
            (
                first,
                tokens
                    .next()
                    .ok_or_else(|| ApiError::internal("Incomplete git rename."))?,
            )
        } else {
            (first, first)
        };
        let Some(local_path) = path.strip_prefix(prefix) else {
            continue;
        };
        if comparison
            && query
                .path
                .as_deref()
                .is_some_and(|selected| selected != local_path)
        {
            continue;
        }
        if comparison && query.path.is_none() {
            if files.len() == 1000 {
                return Err(ApiError::unprocessable(
                    "These commits change more than 1,000 workspace files. Compare a narrower pair.",
                ));
            }
            files.push(json!({"path": local_path, "status": status}));
            continue;
        }
        let old_local = old_path.strip_prefix(prefix);
        let (before, old_binary) = if status.starts_with('A') || comparison && old_local.is_none() {
            (String::new(), false)
        } else {
            blob(root, parent, old_path)?
        };
        let (after, new_binary) = if status.starts_with('D') {
            (String::new(), false)
        } else {
            blob(root, Some(&sha), path)?
        };
        let mut file = json!({"path": local_path, "from": old_local.unwrap_or(old_path), "status": status,
            "before": before, "after": after, "binary": old_binary || new_binary});
        if comparison {
            let literal = format!(":(top,literal){path}");
            let mut args = vec![
                "diff",
                "--no-relative",
                "--no-color",
                "--no-ext-diff",
                "--no-textconv",
                "-M",
                parent.unwrap(),
                &sha,
                "--",
                &literal,
            ];
            let previous_literal = old_local
                .filter(|_| old_path != path)
                .map(|_| format!(":(top,literal){old_path}"));
            if let Some(previous_literal) = &previous_literal {
                args.push(previous_literal);
            }
            let patch = revision::git(root, &args)?;
            let truncated = patch.chars().nth(200_000).is_some();
            file["patch"] = json!(patch.chars().take(200_000).collect::<String>());
            file["patch_truncated"] = json!(truncated);
        }
        files.push(file);
    }
    if comparison && query.path.is_some() && files.is_empty() {
        return Err(ApiError::not_found(
            "This path does not change between the selected commits.",
        ));
    }
    Ok(json!({"sha": sha, "parent": parent, "message": message, "files": files}))
}
#[cfg(test)]
mod compare_tests {
    use super::*;

    #[test]
    fn compares_two_git_objects_inside_the_workspace_without_writes() {
        let temporary = tempfile::tempdir().unwrap();
        let repository = temporary.path();
        let workspace = repository.join("docs");
        std::fs::create_dir(&workspace).unwrap();
        let path = "literal[1].md";
        let file = workspace.join(path);
        let git = |args: &[&str]| revision::git(repository, args).unwrap();
        git(&["init", "-q"]);
        git(&["config", "user.email", "test@example.invalid"]);
        git(&["config", "user.name", "Comparison fixture"]);
        for text in ["base", "intermediate", "target"] {
            std::fs::write(&file, format!("{text}\n")).unwrap();
            std::fs::write(repository.join("sibling.txt"), text).unwrap();
            git(&["add", "."]);
            git(&["commit", "-qm", text]);
        }
        std::fs::write(&file, "working draft\n").unwrap();
        let base = git(&["rev-parse", "HEAD~2"]).trim().to_owned();
        let target = git(&["rev-parse", "HEAD"]).trim().to_owned();
        let compare = |path| {
            read_history(
                &workspace,
                CommitQuery {
                    sha: target.clone(),
                    base: Some(base.clone()),
                    path,
                },
            )
            .unwrap()
        };
        let list = compare(None);
        assert_eq!(list["files"], json!([{"status": "M", "path": path}]));
        let reading = compare(Some(path.into()));
        assert_eq!(reading["sha"], target);
        assert_eq!(reading["parent"], base);
        assert_eq!(reading["files"][0]["before"], "base\n");
        assert_eq!(reading["files"][0]["after"], "target\n");
        let patch = reading["files"][0]["patch"].as_str().unwrap();
        assert!(patch.contains("-base\n+target\n"), "{patch}");
        assert_eq!(std::fs::read_to_string(file).unwrap(), "working draft\n");
        assert_eq!(git(&["show", ":docs/literal[1].md"]), "target\n");
    }
}
