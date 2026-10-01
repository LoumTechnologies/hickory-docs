//! GitHub objects as workspace-tree nodes.
//!
//! Authentication belongs to the person's GitHub CLI. Commands are spawned
//! directly (never through a shell), so Hickory neither sees a token nor
//! writes one. The JSON emitted by `gh` is normalized here; React never learns
//! the CLI's response shape. Folder associations are the small, credentialless
//! `.hick-workspace.json` record that may travel with the repository.

use std::collections::HashMap;
use std::path::{Component, Path};
use std::process::{Command, Output};

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

const ASSOCIATIONS: &str = ".hick-workspace.json";
const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;
const MAX_LOG_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueAssociation {
    pub repository: String,
    pub number: u64,
    #[serde(default)]
    pub folder: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AssociationFile {
    #[serde(default = "version")]
    version: u8,
    #[serde(default)]
    github_issues: Vec<IssueAssociation>,
}

const fn version() -> u8 {
    1
}

#[derive(Deserialize)]
pub struct ObjectQuery {
    pub repository: Option<String>,
}

#[derive(Deserialize)]
pub struct AssociateIssue {
    repository: String,
    number: u64,
    #[serde(default)]
    folder: String,
}

#[derive(Deserialize)]
pub struct EditRequest {
    kind: String,
    number: u64,
    repository: Option<String>,
    field: String,
    value: String,
}

#[derive(Deserialize)]
pub struct CommentRequest {
    kind: String,
    number: u64,
    repository: Option<String>,
    body: String,
}

#[derive(Deserialize)]
pub struct LogQuery {
    run: u64,
    job: u64,
}

fn command(dir: &Path, program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|error| {
            format!(
                "could not start {program} in {}: {error}. Check that the folder exists \
                 and {program} is executable and on Hickory Docs' PATH; restart the app \
                 after installing it or changing your shell's PATH.",
                dir.display()
            )
        })
}

fn said(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        format!("GitHub CLI exited with {}", output.status)
    } else {
        stderr
    }
}

fn gh_json(dir: &Path, args: &[&str]) -> Result<Value, String> {
    let output = command(dir, "gh", args)?;
    if !output.status.success() {
        return Err(said(&output));
    }
    if output.stdout.len() > MAX_JSON_BYTES {
        return Err("GitHub returned more than 4 MiB; narrow the request".into());
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("GitHub returned invalid JSON: {error}"))
}

fn git_text(dir: &Path, args: &[&str]) -> Option<String> {
    let output = command(dir, "git", args).ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|text| !text.is_empty())
}

fn fetch_pull_head(dir: &Path, number: u64) -> Result<(), String> {
    let reference = format!("pull/{number}/head");
    let output = Command::new("git")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["fetch", "--quiet", "--no-tags", "origin", &reference])
        .output()
        .map_err(|error| format!("could not start git fetch: {error}"))?;
    output.status.success().then_some(()).ok_or_else(|| {
        let text = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if text.is_empty() {
            "git could not fetch the pull request head".into()
        } else {
            text
        }
    })
}

fn paths(value: &Value) -> std::collections::HashSet<&str> {
    value
        .get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|file| file.get("path")?.as_str())
        .collect()
}

fn commit_authors(value: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for commit in value
        .get("commits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for author in commit
            .get("authors")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(name) = author
                .get("login")
                .and_then(Value::as_str)
                .or_else(|| author.get("name").and_then(Value::as_str))
                && !out.iter().any(|known| known == name)
            {
                out.push(name.to_string());
            }
        }
    }
    out
}

fn current_head_conflicts(root: &Path, repository: &str, current: &Value) -> Vec<Value> {
    let current_number = current.get("number").and_then(Value::as_u64).unwrap_or(0);
    let Some(current_sha) = current.get("headRefOid").and_then(Value::as_str) else {
        return Vec::new();
    };
    let Some(base) = current.get("baseRefName").and_then(Value::as_str) else {
        return Vec::new();
    };
    let current_paths = paths(current);
    if current_paths.is_empty() {
        return Vec::new();
    }
    let Ok(Value::Array(candidates)) = gh_json(
        root,
        &[
            "pr",
            "list",
            "--repo",
            repository,
            "--base",
            base,
            "--state",
            "open",
            "--limit",
            "30",
            "--json",
            "number,title,url,author,headRefName,headRefOid,files,commits",
        ],
    ) else {
        return Vec::new();
    };
    let current_fetch = fetch_pull_head(root, current_number);

    candidates
        .into_iter()
        .filter_map(|candidate| {
            let number = candidate.get("number")?.as_u64()?;
            if number == current_number || current_paths.is_disjoint(&paths(&candidate)) {
                return None;
            }
            let other_sha = candidate.get("headRefOid")?.as_str()?;
            let result = (|| {
                current_fetch.as_ref().map_err(Clone::clone)?;
                fetch_pull_head(root, number)?;
                let output = command(
                    root,
                    "git",
                    &["merge-tree", "--write-tree", current_sha, other_sha],
                )?;
                match output.status.code() {
                    Some(0) => Ok(("clean", None)),
                    Some(1) => Ok(("conflicting", None)),
                    _ => Err({
                        let detail = [output.stdout.as_slice(), output.stderr.as_slice()].concat();
                        let detail = String::from_utf8_lossy(&detail)
                            .trim()
                            .chars()
                            .take(400)
                            .collect::<String>();
                        if detail.is_empty() {
                            "git merge-tree could not compare these heads".into()
                        } else {
                            detail
                        }
                    }),
                }
            })();
            let (status, reason) = match result {
                Ok(found) => found,
                Err(reason) => ("unavailable", Some(reason)),
            };
            Some(json!({
                "number": number,
                "title": candidate.get("title"),
                "url": candidate.get("url"),
                "author": author(&candidate),
                "commit_authors": commit_authors(&candidate),
                "head": candidate.get("headRefName"),
                "head_sha": other_sha,
                "status": status,
                "reason": reason,
                "claim": "these current head commits conflict if merged together",
            }))
        })
        .collect()
}

/// Normalize the common HTTPS and SSH spellings to GitHub's `owner/repo`.
pub fn github_repository(remote: &str) -> Option<String> {
    let trimmed = remote.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("ssh://git@github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else {
        trimmed.strip_prefix("http://github.com/")?
    };
    valid_repository(path).then(|| path.to_string())
}

fn valid_repository(repository: &str) -> bool {
    let mut parts = repository.split('/');
    let valid = |part: &str| {
        !part.is_empty()
            && part.len() <= 100
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    matches!((parts.next(), parts.next(), parts.next()), (Some(owner), Some(repo), None) if valid(owner) && valid(repo))
}

fn safe_folder(root: &Path, folder: &str) -> Result<String, String> {
    let normalized = folder.trim().trim_matches('/').replace('\\', "/");
    let path = Path::new(&normalized);
    if path
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
        && !normalized.is_empty()
    {
        return Err("the associated folder must be inside this workspace".into());
    }
    let absolute = root.join(&normalized);
    if !absolute.is_dir() {
        return Err(format!(
            "{} is not a folder in this workspace",
            if normalized.is_empty() {
                "."
            } else {
                &normalized
            }
        ));
    }
    Ok(normalized)
}

fn association_path(root: &Path) -> std::path::PathBuf {
    root.join(ASSOCIATIONS)
}

fn read_associations(root: &Path) -> Result<AssociationFile, String> {
    let path = association_path(root);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|error| format!("could not read {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AssociationFile {
            version: 1,
            github_issues: Vec::new(),
        }),
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

fn write_associations(root: &Path, associations: &AssociationFile) -> Result<(), String> {
    let path = association_path(root);
    let body = serde_json::to_vec_pretty(associations).map_err(|error| error.to_string())?;
    super::store::write_atomic(&path, &body)
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}

fn author(value: &Value) -> Option<String> {
    value
        .get("author")?
        .get("login")?
        .as_str()
        .map(str::to_string)
}

fn labels(value: &Value) -> Vec<String> {
    value
        .get("labels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|label| label.get("name")?.as_str().map(str::to_string))
        .collect()
}

fn check(value: &Value) -> Value {
    let kind = value
        .get("__typename")
        .and_then(Value::as_str)
        .unwrap_or("CheckRun");
    if kind == "StatusContext" {
        json!({
            "name": value.get("context").and_then(Value::as_str).unwrap_or("check"),
            "status": value.get("state").and_then(Value::as_str).unwrap_or("UNKNOWN"),
            "url": value.get("targetUrl").and_then(Value::as_str),
        })
    } else {
        let url = value.get("detailsUrl").and_then(Value::as_str);
        let (run, job) = url.and_then(action_ids).unwrap_or((0, 0));
        json!({
            "name": value.get("name").and_then(Value::as_str).unwrap_or("check"),
            "workflow": value.get("workflowName").and_then(Value::as_str),
            "status": value.get("status").and_then(Value::as_str).unwrap_or("UNKNOWN"),
            "conclusion": value.get("conclusion").and_then(Value::as_str),
            "url": url,
            "run": (run != 0).then_some(run),
            "job": (job != 0).then_some(job),
        })
    }
}

fn action_ids(url: &str) -> Option<(u64, u64)> {
    if !url.starts_with("https://github.com/") {
        return None;
    }
    let (_, rest) = url.split_once("/actions/runs/")?;
    let (run, job) = rest.split_once("/job/")?;
    Some((
        run.split('/').next()?.parse().ok()?,
        job.split(['/', '?']).next()?.parse().ok()?,
    ))
}

fn summary_pr(value: &Value, repository: &str, notification: Option<&String>) -> Value {
    json!({
        "repository": repository,
        "number": value.get("number"),
        "title": value.get("title"),
        "state": value.get("state"),
        "draft": value.get("isDraft"),
        "url": value.get("url"),
        "author": author(value),
        "review_decision": value.get("reviewDecision"),
        "mergeable": value.get("mergeable"),
        "merge_state": value.get("mergeStateStatus"),
        "head": value.get("headRefName"),
        "head_sha": value.get("headRefOid"),
        "base": value.get("baseRefName"),
        "updated_at": value.get("updatedAt"),
        "checks": value.get("statusCheckRollup").and_then(Value::as_array)
            .map(|items| items.iter().map(check).collect::<Vec<_>>()).unwrap_or_default(),
        "unread": u8::from(notification.is_some()),
        "notification_thread": notification,
    })
}

fn summary_issue(
    value: &Value,
    association: &IssueAssociation,
    notification: Option<&String>,
) -> Value {
    json!({
        "repository": association.repository,
        "number": association.number,
        "folder": association.folder,
        "title": value.get("title").and_then(Value::as_str).unwrap_or("GitHub issue"),
        "state": value.get("state").and_then(Value::as_str).unwrap_or("UNKNOWN"),
        "url": value.get("url"),
        "author": author(value),
        "labels": labels(value),
        "updated_at": value.get("updatedAt"),
        "freshness": "live",
        "unread": u8::from(notification.is_some()),
        "notification_thread": notification,
    })
}

fn issue_summary(
    root: &Path,
    association: &IssueAssociation,
    notification: Option<&String>,
) -> Value {
    let number = association.number.to_string();
    match gh_json(
        root,
        &[
            "issue",
            "view",
            &number,
            "--repo",
            &association.repository,
            "--json",
            "number,title,state,url,author,labels,updatedAt",
        ],
    ) {
        Ok(value) => summary_issue(&value, association, notification),
        Err(reason) => json!({
            "repository": association.repository,
            "number": association.number,
            "folder": association.folder,
            "title": format!("Issue #{}", association.number),
            "state": "UNAVAILABLE",
            "freshness": "unavailable",
            "reason": reason,
            "unread": u8::from(notification.is_some()),
            "notification_thread": notification,
        }),
    }
}

/// `owner/repo#number` → notification thread. GitHub notifications are the
/// unread record; expansion only moves their badge and never acknowledges it.
fn notifications(root: &Path) -> HashMap<String, String> {
    let Ok(Value::Array(values)) = gh_json(root, &["api", "notifications?per_page=100"]) else {
        return HashMap::new();
    };
    notifications_from(&values)
}

fn notifications_from(values: &[Value]) -> HashMap<String, String> {
    values
        .iter()
        .filter_map(|value| {
            if value.get("unread").and_then(Value::as_bool) != Some(true) {
                return None;
            }
            let repository = value.get("repository")?.get("full_name")?.as_str()?;
            let kind = value.get("subject")?.get("type")?.as_str()?;
            if !matches!(kind, "Issue" | "PullRequest") {
                return None;
            }
            let url = value.get("subject")?.get("url")?.as_str()?;
            let number = url.rsplit('/').next()?.parse::<u64>().ok()?;
            let thread = value.get("id")?.as_str()?.to_string();
            Some((format!("{repository}#{number}"), thread))
        })
        .collect()
}

fn workspace_sync(root: &Path) -> Value {
    let associations = match read_associations(root) {
        Ok(file) => file,
        Err(reason) => {
            return json!({ "status": "unavailable", "reason": reason, "reviews": [], "issues": [] });
        }
    };
    let remote = git_text(root, &["remote", "get-url", "origin"]);
    let current_repository = remote.as_deref().and_then(github_repository);
    let unread = if current_repository.is_some() || !associations.github_issues.is_empty() {
        notifications(root)
    } else {
        HashMap::new()
    };
    let issues: Vec<Value> = associations
        .github_issues
        .iter()
        .map(|item| {
            issue_summary(
                root,
                item,
                unread.get(&format!("{}#{}", item.repository, item.number)),
            )
        })
        .collect();
    let Some(_remote) = remote else {
        return json!({ "status": "not_repository", "reviews": [], "issues": issues });
    };
    let Some(repository) = current_repository else {
        return json!({ "status": "not_github", "reviews": [], "issues": issues });
    };
    let branch = git_text(root, &["branch", "--show-current"]);
    let Some(branch_name) = branch.as_deref() else {
        return json!({ "status": "detached", "repository": repository, "reviews": [], "issues": issues });
    };
    let fields = "number,title,state,isDraft,url,author,reviewDecision,mergeable,mergeStateStatus,headRefName,headRefOid,baseRefName,statusCheckRollup,updatedAt";
    match gh_json(
        root,
        &[
            "pr",
            "list",
            "--repo",
            &repository,
            "--head",
            branch_name,
            "--state",
            "open",
            "--limit",
            "50",
            "--json",
            fields,
        ],
    ) {
        Ok(Value::Array(values)) => json!({
            "status": "available", "repository": repository, "branch": branch_name,
            "reviews": values.iter().map(|value| {
                let number = value.get("number").and_then(Value::as_u64).unwrap_or(0);
                summary_pr(value, &repository, unread.get(&format!("{repository}#{number}")))
            }).collect::<Vec<_>>(),
            "issues": issues,
        }),
        Ok(_) => {
            json!({ "status": "unavailable", "repository": repository, "branch": branch_name, "reason": "GitHub returned a non-list for pull requests", "reviews": [], "issues": issues })
        }
        Err(reason) => {
            json!({ "status": "unavailable", "repository": repository, "branch": branch_name, "reason": reason, "reviews": [], "issues": issues })
        }
    }
}

pub async fn workspace(State(state): State<LocalState>) -> Json<Value> {
    let root = state.index.root().to_path_buf();
    Json(tokio::task::spawn_blocking(move || workspace_sync(&root)).await.unwrap_or_else(|error| {
        json!({ "status": "unavailable", "reason": error.to_string(), "reviews": [], "issues": [] })
    }))
}

fn object_repository(root: &Path, requested: Option<&str>) -> Result<String, String> {
    if let Some(repository) = requested {
        return valid_repository(repository)
            .then(|| repository.to_string())
            .ok_or_else(|| "a GitHub repository must be owner/name".into());
    }
    let remote = git_text(root, &["remote", "get-url", "origin"])
        .ok_or_else(|| "this folder has no origin remote".to_string())?;
    github_repository(&remote).ok_or_else(|| "this folder's origin is not on GitHub".into())
}

pub async fn pull_request(
    State(state): State<LocalState>,
    AxumPath(number): AxumPath<u64>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let value = tokio::task::spawn_blocking(move || {
        let repository = object_repository(&root, None)?;
        let n = number.to_string();
        let raw = gh_json(&root, &["pr", "view", &n, "--repo", &repository, "--json", "number,title,body,state,isDraft,url,author,reviewDecision,mergeable,mergeStateStatus,headRefName,headRefOid,baseRefName,statusCheckRollup,reviews,comments,files,updatedAt"])?;
        let conflicts = current_head_conflicts(&root, &repository, &raw);
        let mut answer = summary_pr(&raw, &repository, None);
        let object = answer.as_object_mut().unwrap();
        object.insert("body".into(), raw.get("body").cloned().unwrap_or(Value::String(String::new())));
        object.insert("reviews".into(), raw.get("reviews").cloned().unwrap_or_else(|| json!([])));
        object.insert("comments".into(), raw.get("comments").cloned().unwrap_or_else(|| json!([])));
        object.insert("current_head_conflicts".into(), Value::Array(conflicts));
        Ok::<_, String>(answer)
    }).await.map_err(|error| ApiError::internal(error.to_string()))?
        .map_err(ApiError::unavailable)?;
    Ok(Json(value))
}

pub async fn issue(
    State(state): State<LocalState>,
    AxumPath(number): AxumPath<u64>,
    Query(query): Query<ObjectQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let value = tokio::task::spawn_blocking(move || {
        let repository = object_repository(&root, query.repository.as_deref())?;
        let n = number.to_string();
        let raw = gh_json(&root, &["issue", "view", &n, "--repo", &repository, "--json", "number,title,body,state,stateReason,url,author,assignees,labels,comments,updatedAt"])?;
        Ok::<_, String>(json!({ "repository": repository, "number": number, "title": raw.get("title"), "body": raw.get("body"), "state": raw.get("state"), "state_reason": raw.get("stateReason"), "url": raw.get("url"), "author": author(&raw), "assignees": raw.get("assignees"), "labels": labels(&raw), "comments": raw.get("comments"), "updated_at": raw.get("updatedAt") }))
    }).await.map_err(|error| ApiError::internal(error.to_string()))?.map_err(ApiError::unavailable)?;
    Ok(Json(value))
}

pub async fn associate_issue(
    State(state): State<LocalState>,
    Json(body): Json<AssociateIssue>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let association = tokio::task::spawn_blocking(move || {
        if !valid_repository(&body.repository) {
            return Err("a GitHub repository must be owner/name".to_string());
        }
        if body.number == 0 {
            return Err("a GitHub issue number must be greater than zero".to_string());
        }
        let folder = safe_folder(&root, &body.folder)?;
        let item = IssueAssociation {
            repository: body.repository,
            number: body.number,
            folder,
        };
        let mut file = read_associations(&root)?;
        if !file.github_issues.contains(&item) {
            file.github_issues.push(item.clone());
        }
        write_associations(&root, &file)?;
        Ok::<_, String>(item)
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?
    .map_err(ApiError::bad_request)?;
    Ok(Json(json!(association)))
}

fn edit_sync(root: &Path, body: EditRequest) -> Result<(), String> {
    if !matches!(body.kind.as_str(), "pr" | "issue") {
        return Err("kind must be pr or issue".into());
    }
    if !matches!(body.field.as_str(), "title" | "body") {
        return Err("field must be title or body".into());
    }
    if body.value.trim().is_empty() && body.field == "title" {
        return Err("a title cannot be empty".into());
    }
    let repository = object_repository(root, body.repository.as_deref())?;
    let number = body.number.to_string();
    let flag = if body.field == "title" {
        "--title"
    } else {
        "--body"
    };
    let output = command(
        root,
        "gh",
        &[
            &body.kind,
            "edit",
            &number,
            "--repo",
            &repository,
            flag,
            &body.value,
        ],
    )?;
    output
        .status
        .success()
        .then_some(())
        .ok_or_else(|| said(&output))
}

pub async fn edit(
    State(state): State<LocalState>,
    Json(body): Json<EditRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || edit_sync(&root, body))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map_err(ApiError::unprocessable)?;
    Ok(Json(json!({ "ok": true })))
}

fn comment_sync(root: &Path, body: CommentRequest) -> Result<(), String> {
    if !matches!(body.kind.as_str(), "pr" | "issue") {
        return Err("kind must be pr or issue".into());
    }
    if body.body.trim().is_empty() {
        return Err("a comment cannot be empty".into());
    }
    let repository = object_repository(root, body.repository.as_deref())?;
    let number = body.number.to_string();
    let output = command(
        root,
        "gh",
        &[
            &body.kind,
            "comment",
            &number,
            "--repo",
            &repository,
            "--body",
            &body.body,
        ],
    )?;
    output
        .status
        .success()
        .then_some(())
        .ok_or_else(|| said(&output))
}

pub async fn comment(
    State(state): State<LocalState>,
    Json(body): Json<CommentRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || comment_sync(&root, body))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map_err(ApiError::unprocessable)?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn check_log(
    State(state): State<LocalState>,
    Query(query): Query<LogQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let answer = tokio::task::spawn_blocking(move || {
        let run = query.run.to_string();
        let job = query.job.to_string();
        let output = command(&root, "gh", &["run", "view", &run, "--job", &job, "--log"])?;
        if !output.status.success() {
            return Err(said(&output));
        }
        let truncated = output.stdout.len() > MAX_LOG_BYTES;
        let bytes = &output.stdout[..output.stdout.len().min(MAX_LOG_BYTES)];
        Ok::<_, String>(json!({ "text": String::from_utf8_lossy(bytes), "truncated": truncated }))
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?
    .map_err(ApiError::unavailable)?;
    Ok(Json(answer))
}

pub async fn mark_notification_read(
    State(state): State<LocalState>,
    AxumPath(thread): AxumPath<String>,
) -> ApiResult<Json<Value>> {
    if thread.is_empty() || !thread.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::bad_request(
            "a GitHub notification thread id must be numeric",
        ));
    }
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        let endpoint = format!("notifications/threads/{thread}");
        let output = command(&root, "gh", &["api", "-X", "PATCH", &endpoint])?;
        output
            .status
            .success()
            .then_some(())
            .ok_or_else(|| said(&output))
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?
    .map_err(ApiError::unprocessable)?;
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Protects docs/guarantees/integrations/the-desktop-finds-tools-from-the-shell-path.md
    #[test]
    fn launch_failure_does_not_claim_a_binary_is_uninstalled() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing-workspace");
        let error = command(&missing, "git", &["--version"]).unwrap_err();
        assert!(error.contains("could not start git"), "{error}");
        assert!(error.contains("missing-workspace"), "{error}");
        assert!(error.contains("Check that the folder exists"), "{error}");
        assert!(!error.contains("not installed"), "{error}");
    }

    #[test]
    fn common_github_remotes_have_one_identity() {
        for remote in [
            "https://github.com/acme/widget.git",
            "http://github.com/acme/widget/",
            "git@github.com:acme/widget.git",
            "ssh://git@github.com/acme/widget.git",
        ] {
            assert_eq!(github_repository(remote).as_deref(), Some("acme/widget"));
        }
        assert_eq!(github_repository("git@gitlab.com:acme/widget.git"), None);
        assert_eq!(
            github_repository("https://github.com/acme/widget/extra"),
            None
        );
    }

    #[test]
    fn actions_urls_yield_only_numeric_run_and_job_ids() {
        assert_eq!(
            action_ids("https://github.com/a/b/actions/runs/123/job/456?pr=7"),
            Some((123, 456))
        );
        assert_eq!(action_ids("https://example.com/actions/runs/x/job/2"), None);
    }

    #[test]
    fn associations_are_credentialless_and_folder_checked() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("docs")).unwrap();
        assert_eq!(safe_folder(dir.path(), "docs/").unwrap(), "docs");
        assert!(safe_folder(dir.path(), "../elsewhere").is_err());
        let file = AssociationFile {
            version: 1,
            github_issues: vec![IssueAssociation {
                repository: "acme/widget".into(),
                number: 4,
                folder: "docs".into(),
            }],
        };
        write_associations(dir.path(), &file).unwrap();
        let raw = std::fs::read_to_string(association_path(dir.path())).unwrap();
        assert!(!raw.to_ascii_lowercase().contains("token"));
        assert_eq!(
            read_associations(dir.path()).unwrap().github_issues,
            file.github_issues
        );
    }

    #[test]
    fn unread_notifications_keep_their_thread_identity() {
        let values = vec![
            json!({
                "id": "991", "unread": true,
                "repository": { "full_name": "acme/widget" },
                "subject": { "type": "PullRequest", "url": "https://api.github.com/repos/acme/widget/pulls/12" }
            }),
            json!({
                "id": "992", "unread": false,
                "repository": { "full_name": "acme/widget" },
                "subject": { "type": "Issue", "url": "https://api.github.com/repos/acme/widget/issues/7" }
            }),
        ];
        let found = notifications_from(&values);
        assert_eq!(found.get("acme/widget#12").map(String::as_str), Some("991"));
        assert!(!found.contains_key("acme/widget#7"));
    }

    #[test]
    fn conflict_candidates_are_bounded_by_overlapping_paths_and_name_commit_authors() {
        let one = json!({ "files": [{ "path": "src/retry.rs" }, { "path": "README.md" }] });
        let two = json!({
            "files": [{ "path": "src/retry.rs" }],
            "commits": [{ "authors": [{ "login": "ada" }, { "name": "Lin" }] }, { "authors": [{ "login": "ada" }] }]
        });
        assert!(!paths(&one).is_disjoint(&paths(&two)));
        assert_eq!(commit_authors(&two), ["ada", "Lin"]);
    }

    #[test]
    fn a_plain_folder_has_a_complete_empty_workspace_response() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            workspace_sync(dir.path()),
            json!({ "status": "not_repository", "reviews": [], "issues": [] })
        );
    }
}
