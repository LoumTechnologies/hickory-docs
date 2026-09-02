//! The repository, as something to read.
//!
//! One `git log` invocation answers everything the graph shows — the commits,
//! their parents, and every file each one touched with its line counts. That
//! is deliberate: the alternative is a `git show` per commit when a row is
//! expanded, which is a process per click and a visible pause on a repository
//! with any history. `--numstat` costs almost nothing on top of the log walk,
//! so expanding a row is free and stays free.
//!
//! Changing the repository — stage, commit, push, and the rest of the daily
//! loop — is `git_ops.rs`, kept apart so this file stays what it says: the
//! repository as something to read.

use std::process::Command;

use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// How many commits the graph walks by default.
///
/// A graph is read, not scrolled through forever: past a few hundred rows the
/// lanes are unreadable anyway, and the answer is a filter rather than more
/// rows.
const DEFAULT_LIMIT: usize = 120;
const MAX_LIMIT: usize = 1_000;

/// A separator that cannot occur in a commit message, an author name, or a
/// path. `\x1f` is ASCII "unit separator", which is exactly what it is for —
/// and unlike a tab or a pipe, nobody has ever put one in a subject line.
const FIELD: char = '\u{1f}';
const RECORD: char = '\u{1e}';

#[derive(Serialize)]
struct FileChange {
    path: String,
    /// The path it came from, on a rename. Absent otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    /// `A`, `M`, `D`, `R`, or `?` when git said something else.
    status: char,
    /// `None` for a binary file, where lines are not a meaningful count —
    /// which is different from zero and must not render as zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    added: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<u64>,
}

#[derive(Serialize)]
struct Commit {
    sha: String,
    /// The short form, for the node. Git's own abbreviation, not a slice:
    /// how many characters are unambiguous is the repository's business.
    short: String,
    parents: Vec<String>,
    author: String,
    email: String,
    /// Unix seconds. Formatted by the client, in the reader's own locale.
    time: i64,
    subject: String,
    body: String,
    /// Branch and tag names pointing here.
    refs: Vec<String>,
    files: Vec<FileChange>,
    added: u64,
    removed: u64,
    /// Above the publication floor: still a draft, because nobody else can
    /// be holding it yet. Filled in after the log walk from
    /// `crate::floor`; see docs/specs/freeform/expression-and-log.md.
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize)]
pub struct LogParams {
    #[serde(default)]
    pub limit: Option<usize>,
    /// Only commits touching this path, when given.
    #[serde(default)]
    pub path: Option<String>,
}

fn git(dir: &std::path::Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()
}

/// `GET /api/git/log` — the commit graph, with every commit's files.
///
/// A folder that is not a repository answers `{"commits": [], "repository":
/// false}` rather than an error: opening a folder of notes that happens not
/// to be under version control is entirely normal, and the pane says so.
pub async fn log(
    State(state): State<LocalState>,
    Query(params): Query<LogParams>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let path = params.path.clone();

    let answer = tokio::task::spawn_blocking(move || {
        let format = format!(
            "{RECORD}%H{FIELD}%h{FIELD}%P{FIELD}%an{FIELD}%ae{FIELD}%at{FIELD}%D{FIELD}%s{FIELD}%b{FIELD}"
        );
        let limit_arg = format!("-n{limit}");
        let mut args: Vec<String> = vec![
            "log".into(),
            limit_arg,
            "--numstat".into(),
            // Renames as renames: a file that moved is one change, and
            // showing it as a delete plus an add is showing a story that did
            // not happen.
            "--find-renames".into(),
            "--date-order".into(),
            format!("--pretty=format:{format}"),
        ];
        if let Some(path) = path {
            args.push("--".into());
            args.push(path);
        }
        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = git(&root, &borrowed)?;
        if !out.status.success() {
            return None;
        }
        let mut commits = parse_log(&String::from_utf8_lossy(&out.stdout));
        // Which of these are still drafts. Computed here, in the same
        // blocking task, because it is the same repository and the same
        // handful of git calls.
        let floor = crate::floor::compute(&root);
        if let Some(floor) = &floor {
            let drafts = crate::floor::draft_set(floor);
            for commit in &mut commits {
                commit.draft = drafts.contains(commit.sha.as_str());
            }
        }
        Some((commits, floor))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;

    match answer {
        Some((commits, floor)) => Ok(Json(json!({
            "repository": true,
            "commits": commits,
            // The floor rides along with the log rather than costing a second
            // request: a graph that cannot say which of its rows are still
            // drafts is showing half the fact. `null` where the repository
            // has published nothing to compute one against.
            "floor": floor,
        }))),
        // Not a repository, no git, an empty history: all the same answer,
        // and none of them an error.
        None => Ok(Json(json!({ "repository": false, "commits": [] }))),
    }
}

/// Parse `git log --numstat` with our record/field separators.
///
/// Split out and pure so the awkward parts — a merge with no numstat, a
/// binary file's `-`, a rename's `old => new`, a body containing blank lines
/// — are arguable in a test rather than against a real repository.
fn parse_log(text: &str) -> Vec<Commit> {
    let mut commits = Vec::new();
    for record in text.split(RECORD) {
        if record.trim().is_empty() {
            continue;
        }
        let mut parts = record.split(FIELD);
        let sha = parts.next().unwrap_or_default().trim().to_string();
        if sha.is_empty() {
            continue;
        }
        let short = parts.next().unwrap_or_default().trim().to_string();
        let parents: Vec<String> = parts
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let author = parts.next().unwrap_or_default().to_string();
        let email = parts.next().unwrap_or_default().to_string();
        let time: i64 = parts.next().unwrap_or_default().trim().parse().unwrap_or(0);
        let refs: Vec<String> = parts
            .next()
            .unwrap_or_default()
            .split(',')
            .map(|r| r.trim().trim_start_matches("HEAD -> ").to_string())
            .filter(|r| !r.is_empty())
            .collect();
        let subject = parts.next().unwrap_or_default().to_string();
        // The body and the numstat block share what is left, separated by
        // the blank line git puts between them. Everything left, not the
        // next field: the format ends with a trailing separator, so git's
        // numstat lands in a field of its own after the body — and a parser
        // that read only the body's field reported every commit as touching
        // zero lines.
        let mut rest = parts.next().unwrap_or_default().to_string();
        for tail in parts {
            rest.push('\n');
            rest.push_str(tail);
        }
        let (body, files) = split_body_and_numstat(&rest);

        let added = files.iter().filter_map(|f| f.added).sum();
        let removed = files.iter().filter_map(|f| f.removed).sum();
        commits.push(Commit {
            draft: false,
            sha,
            short,
            parents,
            author,
            email,
            time,
            subject,
            body,
            refs,
            files,
            added,
            removed,
        });
    }
    commits
}

/// The commit body and its numstat lines, told apart.
///
/// A numstat line is `<added>\t<removed>\t<path>`, and a body line is
/// anything else. Matching on the SHAPE rather than on position is what makes
/// a body containing blank lines — which is every well-written commit message
/// in this repository — parse correctly.
fn split_body_and_numstat(rest: &str) -> (String, Vec<FileChange>) {
    let mut body_lines: Vec<&str> = Vec::new();
    let mut files = Vec::new();
    let mut in_stats = false;
    for line in rest.lines() {
        if let Some(change) = parse_numstat_line(line) {
            in_stats = true;
            files.push(change);
        } else if !in_stats {
            body_lines.push(line);
        }
        // A non-stat line after the stats began is git's own padding.
    }
    (body_lines.join("\n").trim().to_string(), files)
}

fn parse_numstat_line(line: &str) -> Option<FileChange> {
    let mut fields = line.split('\t');
    let added = fields.next()?;
    let removed = fields.next()?;
    let path = fields.next()?;
    if path.is_empty() {
        return None;
    }
    // A binary file's counts are `-`, which is NOT zero: reporting zero would
    // say a change touched nothing.
    let parse = |value: &str| -> Option<u64> { value.parse().ok() };
    let added = if added == "-" {
        None
    } else {
        Some(parse(added)?)
    };
    let removed = if removed == "-" {
        None
    } else {
        Some(parse(removed)?)
    };

    // `old => new`, or the `dir/{a => b}/file` form git uses for a move
    // inside a tree.
    let (from, path, status) = if let Some((before, after)) = split_rename(path) {
        (Some(before), after, 'R')
    } else {
        (None, path.to_string(), '?')
    };
    let status = if status == 'R' {
        'R'
    } else if removed == Some(0) && added.is_some() {
        'A'
    } else if added == Some(0) && removed.is_some() {
        'D'
    } else {
        'M'
    };
    Some(FileChange {
        path,
        from,
        status,
        added,
        removed,
    })
}

/// Git's two rename spellings, both of them.
fn split_rename(path: &str) -> Option<(String, String)> {
    if let Some(open) = path.find('{')
        && let Some(close) = path[open..].find('}')
    {
        let close = open + close;
        let inner = &path[open + 1..close];
        let (before, after) = inner.split_once(" => ")?;
        let prefix = &path[..open];
        let suffix = &path[close + 1..];
        return Some((
            format!("{prefix}{before}{suffix}").replace("//", "/"),
            format!("{prefix}{after}{suffix}").replace("//", "/"),
        ));
    }
    let (before, after) = path.split_once(" => ")?;
    Some((before.to_string(), after.to_string()))
}

/// `GET /api/git/status` — the branch, and whether anything is uncommitted.
///
/// The status bar's half of this: a name and two counts, not a file list.
pub async fn status(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let answer = tokio::task::spawn_blocking(move || {
        let branch = git(&root, &["rev-parse", "--abbrev-ref", "HEAD"])
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;
        let porcelain = git(&root, &["status", "--porcelain"])
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())?;
        let mut staged = 0usize;
        let mut unstaged = 0usize;
        let mut untracked = 0usize;
        for line in porcelain.lines() {
            let mut chars = line.chars();
            let index = chars.next().unwrap_or(' ');
            let tree = chars.next().unwrap_or(' ');
            if index == '?' && tree == '?' {
                untracked += 1;
                continue;
            }
            if index != ' ' {
                staged += 1;
            }
            if tree != ' ' {
                unstaged += 1;
            }
        }
        Some((branch, staged, unstaged, untracked))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;

    Ok(Json(match answer {
        Some((branch, staged, unstaged, untracked)) => json!({
            "repository": true,
            "branch": branch,
            "staged": staged,
            "unstaged": unstaged,
            "untracked": untracked,
        }),
        None => json!({ "repository": false }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(fields: &[&str]) -> String {
        format!("{RECORD}{}", fields.join(&FIELD.to_string()))
    }

    #[test]
    fn a_commit_carries_its_summary_and_its_files() {
        let text = record(&[
            "abc123def456",
            "abc123d",
            "parent1",
            "Ada Lovelace",
            "ada@example.com",
            "1700000000",
            "HEAD -> master, origin/master",
            "Add the thing",
            "The body.\n\n12\t3\tsrc/main.rs\n0\t7\tREADME.md\n",
        ]);
        let commits = parse_log(&text);
        assert_eq!(commits.len(), 1);
        let c = &commits[0];
        assert_eq!(c.sha, "abc123def456");
        assert_eq!(c.short, "abc123d");
        assert_eq!(c.parents, vec!["parent1"]);
        assert_eq!(c.author, "Ada Lovelace");
        assert_eq!(c.time, 1_700_000_000);
        assert_eq!(c.subject, "Add the thing");
        assert_eq!(c.body, "The body.");
        assert_eq!(c.refs, vec!["master", "origin/master"]);
        assert_eq!(c.files.len(), 2);
        assert_eq!(c.added, 12);
        assert_eq!(c.removed, 10);
    }

    #[test]
    fn numstat_after_the_trailing_separator_is_counted() {
        // The live format ends with a separator after `%b`, so git's own
        // numstat block lands in a field of its own. This is what the graph
        // actually receives, and for a while it rendered every commit as
        // +0 −0.
        let text = format!(
            "{}{FIELD}\n\n4\t2\tsrc/lib.rs\n",
            record(&["a", "a", "", "N", "e", "1", "", "Subject", "The body.\n"])
        );
        let commits = parse_log(&text);
        assert_eq!(commits[0].body, "The body.");
        assert_eq!(commits[0].files.len(), 1);
        assert_eq!(commits[0].added, 4);
        assert_eq!(commits[0].removed, 2);
    }

    #[test]
    fn a_body_with_blank_lines_survives() {
        // Every well-written commit message in this repository has them, so a
        // parser that split on the first blank line would mangle most of the
        // history it is meant to display.
        let text = record(&[
            "a",
            "a",
            "",
            "N",
            "e",
            "1",
            "",
            "Subject",
            "First paragraph.\n\nSecond paragraph.\n\n1\t0\tf.rs\n",
        ]);
        let commits = parse_log(&text);
        assert_eq!(commits[0].body, "First paragraph.\n\nSecond paragraph.");
        assert_eq!(commits[0].files.len(), 1);
    }

    #[test]
    fn a_binary_file_reports_no_counts_rather_than_zero() {
        // Zero would say the change touched nothing.
        let text = record(&["a", "a", "", "N", "e", "1", "", "s", "\n-\t-\tlogo.png\n"]);
        let files = &parse_log(&text)[0].files;
        assert_eq!(files[0].added, None);
        assert_eq!(files[0].removed, None);
        assert_eq!(files[0].status, 'M');
    }

    #[test]
    fn additions_and_deletions_are_told_apart() {
        let text = record(&[
            "a",
            "a",
            "",
            "N",
            "e",
            "1",
            "",
            "s",
            "\n9\t0\tnew.rs\n0\t9\tgone.rs\n3\t4\tedited.rs\n",
        ]);
        let files = &parse_log(&text)[0].files;
        assert_eq!(files[0].status, 'A');
        assert_eq!(files[1].status, 'D');
        assert_eq!(files[2].status, 'M');
    }

    #[test]
    fn a_rename_is_one_change_with_both_names() {
        // Showing it as a delete plus an add is showing a story that did not
        // happen.
        let text = record(&[
            "a",
            "a",
            "",
            "N",
            "e",
            "1",
            "",
            "s",
            "\n1\t1\told.rs => new.rs\n",
        ]);
        let files = &parse_log(&text)[0].files;
        assert_eq!(files[0].status, 'R');
        assert_eq!(files[0].from.as_deref(), Some("old.rs"));
        assert_eq!(files[0].path, "new.rs");
    }

    #[test]
    fn a_move_inside_a_tree_uses_gits_brace_spelling() {
        let text = record(&[
            "a",
            "a",
            "",
            "N",
            "e",
            "1",
            "",
            "s",
            "\n2\t2\tsrc/{old => new}/file.rs\n",
        ]);
        let files = &parse_log(&text)[0].files;
        assert_eq!(files[0].from.as_deref(), Some("src/old/file.rs"));
        assert_eq!(files[0].path, "src/new/file.rs");
    }

    #[test]
    fn a_merge_with_no_numstat_is_still_a_commit() {
        let text = record(&["m", "m", "p1 p2", "N", "e", "1", "", "Merge branch", ""]);
        let commits = parse_log(&text);
        assert_eq!(commits[0].parents.len(), 2);
        assert!(commits[0].files.is_empty());
        assert_eq!(commits[0].added, 0);
    }

    #[test]
    fn a_root_commit_has_no_parents() {
        let text = record(&["r", "r", "", "N", "e", "1", "", "First", ""]);
        assert!(parse_log(&text)[0].parents.is_empty());
    }

    #[test]
    fn an_empty_log_is_no_commits_rather_than_a_broken_one() {
        assert!(parse_log("").is_empty());
        assert!(parse_log("\n\n").is_empty());
    }
}
