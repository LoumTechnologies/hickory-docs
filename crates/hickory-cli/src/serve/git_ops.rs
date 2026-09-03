//! The repository, as something to change.
//!
//! `git.rs` reads history and argued, for a while, that writing it belonged in
//! a terminal where the exact command is visible. The argument was right about
//! a HALF-built git UI — one that can commit but not amend, stage but not
//! stash — and wrong that the answer was none. What a person leaving an IDE
//! does every day is the loop below, and every verb in it is here: see what
//! changed, read a diff, stage and unstage, discard, commit and amend, push
//! and pull, switch and create a branch, stash and pop. Each one is one git
//! command, run as itself, with git's own words shown when it refuses.
//!
//! Three rules hold across all of them:
//!
//! * **Nothing prompts.** `GIT_TERMINAL_PROMPT=0` and an ssh in batch mode,
//!   so a push that needs a credential fails saying so rather than hanging a
//!   request forever waiting for a password nobody can type.
//! * **Nothing rewrites below the publication floor.** Amend is refused when
//!   `HEAD` is a record somebody else may hold — the same line `hick emit`
//!   draws. See docs/specs/freeform/expression-and-log.md.
//! * **Pull is fast-forward only.** A merge or a rebase is a decision with
//!   conflicts in it, and a button is the wrong place to make one.

use std::path::Path;
use std::process::Command;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// Run git in `dir`, never letting it prompt.
fn git(dir: &Path, args: &[&str]) -> Result<std::process::Output, ApiError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0");
    // The user's own ssh command wins; without one, batch mode so an unknown
    // host key or a passphrase fails instead of waiting for a terminal.
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        command.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    command.output().map_err(|e| {
        ApiError::unavailable(format!(
            "git could not be run: {e}. Install git and make sure it is on PATH; \
             the app runs the same `git` you would type."
        ))
    })
}

/// Run git and turn a non-zero exit into git's own explanation.
fn git_ok(dir: &Path, args: &[&str]) -> Result<String, ApiError> {
    let out = git(dir, args)?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let said = if stderr.is_empty() { stdout } else { stderr };
    Err(ApiError::unprocessable(format!(
        "git {} refused: {}",
        args.first().copied().unwrap_or(""),
        if said.is_empty() {
            "no reason given".to_string()
        } else {
            said
        }
    )))
}

fn is_repository(dir: &Path) -> bool {
    git(dir, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn not_a_repository() -> ApiError {
    ApiError::unprocessable(
        "this folder is not a git repository. Open a terminal on the folder's row \
         in the tree and run `git init` if you want one.",
    )
}

/// A path the browser sent, kept inside the repository.
fn checked_path(path: &str) -> Result<&str, ApiError> {
    if path.is_empty() || Path::new(path).is_absolute() || path.split('/').any(|part| part == "..")
    {
        return Err(ApiError::bad_request(format!(
            "{path:?} is not a path inside this repository"
        )));
    }
    Ok(path)
}

fn paths_of(paths: &[String]) -> Result<Vec<&str>, ApiError> {
    if paths.is_empty() {
        return Err(ApiError::bad_request("no paths were given"));
    }
    paths.iter().map(|p| checked_path(p)).collect()
}

// ---------------------------------------------------------------------------
// Reading the working tree
// ---------------------------------------------------------------------------

/// One line of `git status --porcelain`, decoded.
fn parse_status_line(line: &str) -> Option<Value> {
    let mut chars = line.chars();
    let index = chars.next()?;
    let tree = chars.next()?;
    let rest: String = chars.skip(1).collect();
    // A rename is `R  old -> new`.
    let (from, path) = match rest.split_once(" -> ") {
        Some((from, to)) => (Some(from.to_string()), to.to_string()),
        None => (None, rest),
    };
    let mut item = json!({ "path": path, "index": index.to_string(), "tree": tree.to_string() });
    if let Some(from) = from {
        item["from"] = Value::String(from);
    }
    Some(item)
}

/// `## main...origin/main [ahead 1, behind 2]`, decoded.
fn parse_branch_line(line: &str) -> (String, Option<String>, u64, u64) {
    let line = line.trim_start_matches("## ");
    let (names, counts) = match line.split_once(" [") {
        Some((names, counts)) => (names, Some(counts.trim_end_matches(']'))),
        None => (line, None),
    };
    let (branch, upstream) = match names.split_once("...") {
        Some((b, u)) => (b.to_string(), Some(u.to_string())),
        None => (names.to_string(), None),
    };
    let mut ahead = 0;
    let mut behind = 0;
    if let Some(counts) = counts {
        for part in counts.split(", ") {
            if let Some(n) = part.strip_prefix("ahead ") {
                ahead = n.parse().unwrap_or(0);
            } else if let Some(n) = part.strip_prefix("behind ") {
                behind = n.parse().unwrap_or(0);
            }
        }
    }
    // An unborn branch reads `No commits yet on main`.
    let branch = branch
        .strip_prefix("No commits yet on ")
        .map(str::to_string)
        .unwrap_or(branch);
    (branch, upstream, ahead, behind)
}

/// `GET /api/git/changes` — the working tree: every changed file with its
/// index and tree state, the branch, its upstream, and how far apart they are.
pub async fn changes(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        if !is_repository(&root) {
            return Ok(Json(json!({ "repository": false, "files": [] })));
        }
        let text = git_ok(
            &root,
            &["status", "--porcelain=v1", "-b", "--untracked-files=all"],
        )?;
        let mut lines = text.lines();
        let (branch, upstream, ahead, behind) =
            parse_branch_line(lines.next().unwrap_or("## HEAD"));
        let files: Vec<Value> = lines.filter_map(parse_status_line).collect();
        Ok(Json(json!({
            "repository": true,
            "branch": branch,
            "upstream": upstream,
            "ahead": ahead,
            "behind": behind,
            "files": files,
        })))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?
}

#[derive(Deserialize)]
pub struct DiffParams {
    pub path: String,
    #[serde(default)]
    pub staged: bool,
}

/// `GET /api/git/diff?path=&staged=` — one file's diff, as git prints it.
/// An untracked file is shown as wholly added.
pub async fn diff(
    State(state): State<LocalState>,
    Query(params): Query<DiffParams>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        let path = checked_path(&params.path)?.to_string();
        if !is_repository(&root) {
            return Err(not_a_repository());
        }
        let tracked = git(&root, &["ls-files", "--error-unmatch", "--", &path])?
            .status
            .success();
        let out = if tracked {
            let mut args = vec!["diff", "--no-color", "--no-ext-diff"];
            if params.staged {
                args.push("--cached");
            }
            args.extend(["--", path.as_str()]);
            git(&root, &args)?
        } else {
            // `--no-index` exits 1 when the files differ, which they do.
            let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
            git(
                &root,
                &["diff", "--no-color", "--no-index", "--", null, &path],
            )?
        };
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let binary = text.lines().any(|l| l.starts_with("Binary files"));
        Ok(Json(
            json!({ "path": path, "diff": text, "binary": binary }),
        ))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?
}

// ---------------------------------------------------------------------------
// Changing the working tree
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PathsBody {
    #[serde(default)]
    pub paths: Vec<String>,
    /// Every changed file, rather than the ones named.
    #[serde(default)]
    pub all: bool,
}

async fn on_repo<F>(state: LocalState, f: F) -> ApiResult<Json<Value>>
where
    F: FnOnce(&Path) -> Result<Value, ApiError> + Send + 'static,
{
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        if !is_repository(&root) {
            return Err(not_a_repository());
        }
        f(&root).map(Json)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?
}

/// `POST /api/git/stage` — `git add`.
pub async fn stage(
    State(state): State<LocalState>,
    Json(body): Json<PathsBody>,
) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        if body.all {
            git_ok(root, &["add", "-A"])?;
        } else {
            let paths = paths_of(&body.paths)?;
            let mut args = vec!["add", "-A", "--"];
            args.extend(paths);
            git_ok(root, &args)?;
        }
        Ok(json!({ "ok": true }))
    })
    .await
}

/// `POST /api/git/unstage` — take a file out of the index, leaving the
/// working tree alone.
pub async fn unstage(
    State(state): State<LocalState>,
    Json(body): Json<PathsBody>,
) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let mut args = vec!["reset", "-q", "--"];
        let paths;
        if !body.all {
            paths = paths_of(&body.paths)?;
            args.extend(paths.iter().copied());
        }
        // A repository with no commit yet has no HEAD to reset to; there the
        // index is emptied of the file instead, which is the same answer.
        if git_ok(root, &args).is_err() {
            let mut rm = vec!["rm", "-r", "-q", "--cached", "--"];
            if body.all {
                rm.push(".");
            } else {
                rm.extend(paths_of(&body.paths)?);
            }
            git_ok(root, &rm)?;
        }
        Ok(json!({ "ok": true }))
    })
    .await
}

/// `POST /api/git/discard` — throw the working-tree changes to these files
/// away: a tracked file goes back to the index, an untracked one is deleted.
/// The pane confirms first; this does not, because it is the pane's verb.
pub async fn discard(
    State(state): State<LocalState>,
    Json(body): Json<PathsBody>,
) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let paths = paths_of(&body.paths)?;
        let mut tracked = Vec::new();
        let mut untracked = Vec::new();
        for path in paths {
            if git(root, &["ls-files", "--error-unmatch", "--", path])?
                .status
                .success()
            {
                tracked.push(path);
            } else {
                untracked.push(path);
            }
        }
        if !tracked.is_empty() {
            let mut args = vec!["checkout", "--"];
            args.extend(tracked);
            git_ok(root, &args)?;
        }
        if !untracked.is_empty() {
            let mut args = vec!["clean", "-f", "-q", "--"];
            args.extend(untracked);
            git_ok(root, &args)?;
        }
        Ok(json!({ "ok": true }))
    })
    .await
}

#[derive(Deserialize)]
pub struct CommitBody {
    pub message: String,
    #[serde(default)]
    pub amend: bool,
}

/// `POST /api/git/commit` — commit what is staged, or amend `HEAD` — never
/// a `HEAD` below the publication floor, which is a record.
pub async fn commit(
    State(state): State<LocalState>,
    Json(body): Json<CommitBody>,
) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let message = body.message.trim();
        if message.is_empty() {
            return Err(ApiError::bad_request(
                "a commit needs a message: say what changed and why",
            ));
        }
        if body.amend {
            let head = git_ok(root, &["rev-parse", "HEAD"])
                .map_err(|_| ApiError::unprocessable("there is no commit yet to amend"))?;
            let head = head.trim();
            let floor = crate::floor::compute(root);
            let is_draft = floor
                .as_ref()
                .map(|f| crate::floor::draft_set(f).contains(head))
                // No floor means nothing is published: everything is a draft.
                .unwrap_or(true);
            if !is_draft {
                return Err(ApiError::forbidden(format!(
                    "HEAD ({}) is below the publication floor — somebody else may already \
                     hold it, so it is a record and cannot be amended. Make a new commit \
                     instead. See docs/specs/freeform/expression-and-log.md.",
                    &head[..head.len().min(7)]
                )));
            }
        } else {
            let staged = git_ok(root, &["diff", "--cached", "--name-only"])?;
            if staged.trim().is_empty() {
                return Err(ApiError::unprocessable(
                    "nothing is staged. Stage a file first, or amend the last commit.",
                ));
            }
        }
        let mut args = vec!["commit", "-q", "-m", message];
        if body.amend {
            args.push("--amend");
        }
        git_ok(root, &args)?;
        let line = git_ok(root, &["log", "-1", "--pretty=format:%H\u{1f}%h\u{1f}%s"])?;
        let mut parts = line.trim().split('\u{1f}');
        Ok(json!({
            "sha": parts.next().unwrap_or_default(),
            "short": parts.next().unwrap_or_default(),
            "subject": parts.next().unwrap_or_default(),
        }))
    })
    .await
}

/// `POST /api/git/push` — push the branch, setting an upstream the first
/// time. Nothing is forced, ever.
pub async fn push(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let branch = git_ok(root, &["rev-parse", "--abbrev-ref", "HEAD"])?
            .trim()
            .to_string();
        let has_upstream = git(root, &["rev-parse", "--abbrev-ref", "@{upstream}"])?
            .status
            .success();
        let said = if has_upstream {
            git_ok(root, &["push", "--porcelain"])?
        } else {
            git_ok(root, &["push", "--porcelain", "-u", "origin", &branch])?
        };
        Ok(json!({ "ok": true, "said": said.trim() }))
    })
    .await
}

/// `POST /api/git/pull` — fast-forward only. A merge or a rebase is a
/// decision with conflicts in it, not a button.
pub async fn pull(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let said = git_ok(root, &["pull", "--ff-only"])?;
        Ok(json!({ "ok": true, "said": said.trim() }))
    })
    .await
}

/// `GET /api/git/branches` — local branches, each with its upstream.
pub async fn branches(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let text = git_ok(
            root,
            &[
                "branch",
                "--list",
                "--format=%(refname:short)\u{1f}%(upstream:short)\u{1f}%(HEAD)",
            ],
        )?;
        let branches: Vec<Value> = text
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\u{1f}');
                let name = parts.next()?.to_string();
                if name.is_empty() {
                    return None;
                }
                let upstream = parts.next().unwrap_or_default();
                let current = parts.next().unwrap_or_default().trim() == "*";
                Some(json!({
                    "name": name,
                    "upstream": if upstream.is_empty() { Value::Null } else { Value::String(upstream.into()) },
                    "current": current,
                }))
            })
            .collect();
        Ok(json!({ "branches": branches }))
    })
    .await
}

#[derive(Deserialize)]
pub struct CheckoutBody {
    pub branch: String,
    #[serde(default)]
    pub create: bool,
}

/// `POST /api/git/checkout` — switch to a branch, or create it here.
pub async fn checkout(
    State(state): State<LocalState>,
    Json(body): Json<CheckoutBody>,
) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let name = body.branch.trim();
        if name.is_empty() || name.starts_with('-') {
            return Err(ApiError::bad_request("a branch needs a name"));
        }
        let said = if body.create {
            git_ok(root, &["switch", "-c", name])?
        } else {
            git_ok(root, &["switch", name])?
        };
        Ok(json!({ "ok": true, "branch": name, "said": said.trim() }))
    })
    .await
}

#[derive(Deserialize)]
pub struct StashBody {
    /// `push` or `pop`.
    pub action: String,
}

/// `POST /api/git/stash` — put the working tree aside, or take it back.
pub async fn stash(
    State(state): State<LocalState>,
    Json(body): Json<StashBody>,
) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let said = match body.action.as_str() {
            "push" => git_ok(root, &["stash", "push", "--include-untracked"])?,
            "pop" => git_ok(root, &["stash", "pop"])?,
            other => {
                return Err(ApiError::bad_request(format!(
                    "stash action must be \"push\" or \"pop\", not {other:?}"
                )));
            }
        };
        Ok(json!({ "ok": true, "said": said.trim() }))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_line_names_index_and_tree_apart() {
        let item = parse_status_line("MM src/main.rs").unwrap();
        assert_eq!(item["path"], "src/main.rs");
        assert_eq!(item["index"], "M");
        assert_eq!(item["tree"], "M");
        let untracked = parse_status_line("?? notes.md").unwrap();
        assert_eq!(untracked["index"], "?");
    }

    #[test]
    fn a_rename_keeps_both_names() {
        let item = parse_status_line("R  old.rs -> new.rs").unwrap();
        assert_eq!(item["from"], "old.rs");
        assert_eq!(item["path"], "new.rs");
    }

    #[test]
    fn the_branch_line_says_how_far_from_upstream() {
        let (branch, upstream, ahead, behind) =
            parse_branch_line("## master...origin/master [ahead 2, behind 1]");
        assert_eq!(branch, "master");
        assert_eq!(upstream.as_deref(), Some("origin/master"));
        assert_eq!((ahead, behind), (2, 1));
        let (branch, upstream, ..) = parse_branch_line("## feature");
        assert_eq!(branch, "feature");
        assert!(upstream.is_none());
        let (branch, ..) = parse_branch_line("## No commits yet on main");
        assert_eq!(branch, "main");
    }

    #[test]
    fn a_path_that_leaves_the_repository_is_refused() {
        assert!(checked_path("../etc/passwd").is_err());
        assert!(checked_path("/etc/passwd").is_err());
        assert!(checked_path("").is_err());
        assert!(checked_path("src/main.rs").is_ok());
    }
}
