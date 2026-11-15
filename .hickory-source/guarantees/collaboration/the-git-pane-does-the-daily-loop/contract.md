<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Git Pane Does The Daily Loop

For: Engineers using local sessions or pairing their own machines.

Given a folder that is a git repository, when the Git pane is open, then
above the commit graph it shows the working tree — the branch, its upstream
and how far apart they are, every changed file on the staged and unstaged
sides — and offers, each as one git command run as itself: stage and unstage
a file or everything, discard a file's changes after a second click, show a
file's diff, commit what is staged with a message, amend the last commit,
push (setting an upstream the first time), pull fast-forward only, switch to
or create a branch, and stash and pop. When git refuses, git's own words are
shown. Amending is refused when `HEAD` is below the publication floor, since
that commit is a record somebody else may hold. Nothing prompts for a
credential; a push that needs one fails saying so rather than hanging.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-ed96f162b5e6"></a>
## `apps/web/src/components/DiffView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Git Pane Does The Daily Loop](contract.md).

<a id="f-ed96f162b5e6-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md

<storycode:copy id="f-ed96f162b5e6-expression_statement">
// Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { DiffView, diffLineKind } from "./DiffView";

afterEach(cleanup);

</storycode:copy>
<a id="f-ed96f162b5e6-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ed96f162b5e6-expression_statement-2">
describe("a diff's lines", () => {
  it("are told apart by their first character", () => {
    expect(diffLineKind("@@ -1,2 +1,3 @@")).toBe("hunk");
    expect(diffLineKind("+added")).toBe("add");
    expect(diffLineKind("-gone")).toBe("del");
    expect(diffLineKind(" same")).toBe("ctx");
    expect(diffLineKind("diff --git a/x b/x")).toBe("meta");
    expect(diffLineKind("+++ b/x")).toBe("meta");
    expect(diffLineKind("--- a/x")).toBe("meta");
  });

  it("render with a class per kind, so a reader sees what changed", () => {
    render(
      <DiffView
        path="a.txt"
        staged={false}
        binary={false}
        diff={"diff --git a/a.txt b/a.txt\n@@ -1 +1,2 @@\n one\n+two\n"}
      />,
    );
    expect(document.querySelectorAll(".diff-line--add")).toHaveLength(1);
    expect(document.querySelectorAll(".diff-line--hunk")).toHaveLength(1);
    expect(screen.getByText("unstaged")).toBeTruthy();
  });

  it("says a binary file has no lines rather than showing an empty diff", () => {
    render(<DiffView path="logo.png" staged={false} binary diff="Binary files differ\n" />);
    expect(screen.getByText(/binary file/i)).toBeTruthy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/components/DiffView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/DiffView.test.tsx"><storycode:paste select="#f-ed96f162b5e6-expression_statement"/><storycode:paste select="#f-ed96f162b5e6-expression_statement-2"/></storycode:file>

<a id="output-2d0324b58c4d"></a>
## `apps/web/src/components/DiffView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Git Pane Does The Daily Loop](contract.md), [Patch layout exposes endpoint breakpoints](../../literate-projects/patch-layout-breakpoints/contract.md), [Patch elements use the shared diff renderer](../../literate-projects/patch-view-uses-the-shared-diff-renderer/contract.md).

<a id="f-2d0324b58c4d-DiffLineKind"></a>
### `DiffLineKind`

<storycode:copy id="f-2d0324b58c4d-DiffLineKind">
import { useUiReading } from "../lib/uiReading";
import "./DiffView.css";
import type { ReactNode } from "react";

// A diff, as git prints it, with the lines told apart.
//
// Unified diff and nothing cleverer: side-by-side needs a wide pane and a
// merge view needs three versions, and neither is what "what did I change in
// this file" asks for. The hunk headers, the additions and the removals are
// coloured; the file headers are dimmed rather than hidden, because `---`
// and `+++` are how a rename or a mode change announces itself.

export type DiffLineKind = "meta" | "hunk" | "add" | "del" | "ctx";

</storycode:copy>
<a id="f-2d0324b58c4d-diffLineKind"></a>
### `diffLineKind`

Which kind of line this is, by its first character — the way every diff
reader since 1990 has decided it.

<storycode:copy id="f-2d0324b58c4d-diffLineKind">
/** Which kind of line this is, by its first character — the way every diff
 * reader since 1990 has decided it. */
export function diffLineKind(line: string): DiffLineKind {
  if (line.startsWith("@@")) return "hunk";
  if (
    line.startsWith("diff ") ||
    line.startsWith("index ") ||
    line.startsWith("--- ") ||
    line.startsWith("+++ ") ||
    line.startsWith("new file") ||
    line.startsWith("deleted file") ||
    line.startsWith("similarity") ||
    line.startsWith("rename ") ||
    line.startsWith("old mode") ||
    line.startsWith("new mode") ||
    line.startsWith("Binary files")
  ) {
    return "meta";
  }
  if (line.startsWith("+")) return "add";
  if (line.startsWith("-")) return "del";
  return "ctx";
}

</storycode:copy>
<a id="f-2d0324b58c4d-DiffView"></a>
### `DiffView`

<storycode:copy id="f-2d0324b58c4d-DiffView">
export function DiffView({
  path,
  diff,
  binary,
  staged,
  label,
  gutter,
}: {
  path: string;
  diff: string;
  binary: boolean;
  staged: boolean;
  label?: string;
  gutter?: (side: "old" | "new", line: number) => ReactNode;
}) {
  useUiReading("rendered diff", { path, label: label ?? (staged ? "staged" : "unstaged"), binary, diff: binary ? null : diff, scope: "Displayed unified diff; no binary contents or file edit baseline" }, [path, diff, binary, staged, label]);
  const lines = diff.replace(/\n$/, "").split("\n");
  let oldLine = 0, newLine = 0;
  return (
    <section className="diff-view" aria-label={`Diff of ${path}`}>
      <header className="diff-view__head">
        <span className="mono diff-view__path">{path}</span>
        <span className="muted">{label ?? (staged ? "staged" : "unstaged")}</span>
      </header>
      {binary ? (
        <p className="muted diff-view__binary">A binary file; there are no lines to show.</p>
      ) : diff.trim() === "" ? (
        <p className="muted diff-view__binary">No difference.</p>
      ) : (
        <pre className="diff-view__body">
          {lines.map((line, index) => {
            const kind = diffLineKind(line);
            const hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line);
            if (hunk) { oldLine = Number(hunk[1]) - 1; newLine = Number(hunk[2]) - 1; }
            const code = kind === "add" || kind === "del" || (kind === "ctx" && line.startsWith(" "));
            const oldAt = oldLine, newAt = newLine;
            if (code) { if (kind !== "add") oldLine++; if (kind !== "del") newLine++; }
            return <span key={index} className={`diff-line diff-line--${kind}`}>
              {gutter && <span className="diff-breakpoints"><span>{code && kind !== "add" ? gutter("old", oldAt) : null}</span><span>{code && kind !== "del" ? gutter("new", newAt) : null}</span></span>}
              {line}{"\n"}
            </span>;
          })}
        </pre>
      )}
    </section>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/DiffView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/DiffView.tsx"><storycode:paste select="#f-2d0324b58c4d-DiffLineKind"/><storycode:paste select="#f-2d0324b58c4d-diffLineKind"/><storycode:paste select="#f-2d0324b58c4d-DiffView"/></storycode:file>

<a id="output-900059e12d12"></a>
## `crates/hickory-cli/src/serve/git_ops.rs`

Related contracts: [The Git Pane Does The Daily Loop](contract.md).

<a id="f-900059e12d12-git"></a>
### `git`

The repository, as something to change.
`git.rs` reads history and argued, for a while, that writing it belonged in
a terminal where the exact command is visible. The argument was right about
a HALF-built git UI — one that can commit but not amend, stage but not
stash — and wrong that the answer was none. What a person leaving an IDE
does every day is the loop below, and every verb in it is here: see what
changed, read a diff, stage and unstage, discard, commit and amend, push
and pull, switch and create a branch, stash and pop. Each one is one git
command, run as itself, with git's own words shown when it refuses.
Three rules hold across all of them:
* **Nothing prompts.** `GIT_TERMINAL_PROMPT=0` and an ssh in batch mode,
so a push that needs a credential fails saying so rather than hanging a
request forever waiting for a password nobody can type.
* **Nothing rewrites below the publication floor.** Amend is refused when
`HEAD` is a record somebody else may hold — the same line `hick emit`
draws. See docs/specs/freeform/expression-and-log.md.
* **Pull is fast-forward only.** A merge or a rebase is a decision with
conflicts in it, and a button is the wrong place to make one.

<storycode:copy id="f-900059e12d12-git">
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
        .arg("--literal-pathspecs")
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

</storycode:copy>
<a id="f-900059e12d12-git_ok"></a>
### `git_ok`

Run git and turn a non-zero exit into git's own explanation.

<storycode:copy id="f-900059e12d12-git_ok">
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

</storycode:copy>
<a id="f-900059e12d12-is_repository"></a>
### `is_repository`

<storycode:copy id="f-900059e12d12-is_repository">
fn is_repository(dir: &Path) -> bool {
    git(dir, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

</storycode:copy>
<a id="f-900059e12d12-not_a_repository"></a>
### `not_a_repository`

<storycode:copy id="f-900059e12d12-not_a_repository">
fn not_a_repository() -> ApiError {
    ApiError::unprocessable(
        "this folder is not a git repository. Open a terminal on the folder's row \
         in the tree and run `git init` if you want one.",
    )
}

</storycode:copy>
<a id="f-900059e12d12-checked_path"></a>
### `checked_path`

A path the browser sent, kept inside the repository.

<storycode:copy id="f-900059e12d12-checked_path">
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

</storycode:copy>
<a id="f-900059e12d12-paths_of"></a>
### `paths_of`

<storycode:copy id="f-900059e12d12-paths_of">
fn paths_of(paths: &[String]) -> Result<Vec<&str>, ApiError> {
    if paths.is_empty() {
        return Err(ApiError::bad_request("no paths were given"));
    }
    paths.iter().map(|p| checked_path(p)).collect()
}

</storycode:copy>
<a id="f-900059e12d12-parse_status_line"></a>
### `parse_status_line`

Review actions need exact path identities. Use Git’s NUL-delimited porcelain format, where a rename has a separate old-path record and filenames are never display-quoted. Consume the old path even when the new path is outside this workspace; otherwise the next record becomes misaligned. Strip only the repository-to-workspace prefix, preserving newlines, Unicode, quotes and literal arrow text. Refuse truncated records instead of inventing a path. The process adapter disables pathspec expansion so an ordinary filename containing Git magic cannot select other files.

Reading the working tree
One line of `git status --porcelain`, decoded.

<storycode:copy id="f-900059e12d12-parse_status_line">
// ---------------------------------------------------------------------------
// Reading the working tree
// ---------------------------------------------------------------------------

/// Decode one raw NUL-delimited porcelain record without interpreting its path.
fn parse_status_line(record: &str) -> Option<Value> {
    let bytes = record.as_bytes();
    if bytes.len() < 4 || bytes[2] != b' ' || !bytes[0].is_ascii() || !bytes[1].is_ascii() {
        return None;
    }
    Some(json!({"index": &record[..1], "tree": &record[1..2], "path": &record[3..]}))
}
fn parse_status_zero(text: &str, prefix: &str) -> Result<(String, Vec<Value>), String> {
    if !text.ends_with('\0') {
        return Err("Git returned incomplete status records. Refresh repository changes.".into());
    }
    let mut records = text.split_terminator('\0');
    let header = records
        .next()
        .filter(|value| value.starts_with("## "))
        .ok_or("Git returned no branch header.")?
        .to_owned();
    let mut files = Vec::new();
    while let Some(record) = records.next() {
        let mut item =
            parse_status_line(record).ok_or("Git returned a malformed change record.")?;
        let path = item["path"].as_str().unwrap().to_owned();
        let rename = [item["index"].as_str(), item["tree"].as_str()]
            .into_iter()
            .any(|value| matches!(value, Some("R" | "C")));
        let from = if rename {
            Some(
                records
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or("Git returned an incomplete rename record.")?,
            )
        } else {
            None
        };
        let Some(local) = path.strip_prefix(prefix) else {
            continue;
        };
        if local.is_empty() {
            continue;
        }
        item["path"] = json!(local);
        if let Some(from) = from {
            item["from"] = json!(from.strip_prefix(prefix).unwrap_or(from));
            if !from.starts_with(prefix) {
                item["from_outside_workspace"] = json!(true);
            }
        }
        files.push(item);
    }
    Ok((header, files))
}

</storycode:copy>
<a id="f-900059e12d12-parse_branch_line"></a>
### `parse_branch_line`

<storycode:copy id="f-900059e12d12-parse_branch_line">
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

</storycode:copy>
<a id="f-900059e12d12-changes"></a>
### `changes`

Read saved Git state independently of editor drafts. Reject undecodable paths before JSON serialization, retain staged and working-tree columns separately, and translate repository-root paths to this workspace. This is one captured response; a later diff or mutation must establish its own freshness.

`GET /api/git/changes` — the working tree: every changed file with its
index and tree state, the branch, its upstream, and how far apart they are.

<storycode:copy id="f-900059e12d12-changes">
/// `GET /api/git/changes` — the working tree: every changed file with its
/// index and tree state, the branch, its upstream, and how far apart they are.
pub async fn changes(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        if !is_repository(&root) {
            return Ok(Json(json!({ "repository": false, "files": [] })));
        }
        let output = git(
            &root,
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "-b",
                "--untracked-files=all",
            ],
        )?;
        if !output.status.success() {
            return Err(ApiError::unprocessable(
                "Git status refused. Refresh the repository and retry.",
            ));
        }
        let text = String::from_utf8(output.stdout).map_err(|_| {
            ApiError::unprocessable(
                "Git paths are not valid UTF-8. Rename those paths before reviewing them here.",
            )
        })?;
        let prefix = git_ok(&root, &["rev-parse", "--show-prefix"])?;
        let prefix = prefix.strip_suffix('\n').unwrap_or(&prefix);
        let (header, files) = parse_status_zero(&text, prefix).map_err(ApiError::unprocessable)?;
        let (branch, upstream, ahead, behind) = parse_branch_line(&header);
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

</storycode:copy>
<a id="f-900059e12d12-DiffParams"></a>
### `DiffParams`

<storycode:copy id="f-900059e12d12-DiffParams">
#[derive(Deserialize)]
pub struct DiffParams {
    pub path: String,
    #[serde(default)]
    pub staged: bool,
}

</storycode:copy>
<a id="f-900059e12d12-diff"></a>
### `diff`

An untracked file has no staged diff. Return an empty staged view instead of displaying its working-tree contents as if they were in the index. Disable external diff and text-conversion drivers while reading literal file paths. A failed read must not become an empty successful diff: only the expected nonempty no-index difference accepts Git’s status 1. The [simple review engine witness](../../../frontends/git-review-browser.md) removes a file between listing and comparison and checks the explicit failure while retaining an unsaved draft.

`GET /api/git/diff?path=&staged=` — one file's diff, as git prints it.
An untracked file is shown as wholly added.

<storycode:copy id="f-900059e12d12-diff">
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
            let mut args = vec!["diff", "--no-color", "--no-ext-diff", "--no-textconv"];
            if params.staged {
                args.push("--cached");
            }
            args.extend(["--", path.as_str()]);
            git(&root, &args)?
        } else {
            if params.staged {
                return Ok(Json(json!({"path": path, "diff": "", "binary": false})));
            }
            // `--no-index` exits 1 when the files differ, which they do.
            let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
            git(
                &root,
                &[
                    "diff",
                    "--no-color",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-index",
                    "--",
                    null,
                    &path,
                ],
            )?
        };
        if !out.status.success()
            && (tracked || out.status.code() != Some(1) || out.stdout.is_empty())
        {
            return Err(ApiError::unprocessable(format!(
                "Git diff refused: {}. Refresh repository changes.",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let binary = text.lines().any(|l| l.starts_with("Binary files"));
        Ok(Json(
            json!({ "path": path, "diff": text, "binary": binary }),
        ))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?
}

</storycode:copy>
<a id="f-900059e12d12-PathsBody"></a>
### `PathsBody`

<storycode:copy id="f-900059e12d12-PathsBody">
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

</storycode:copy>
<a id="f-900059e12d12-on_repo"></a>
### `on_repo`

<storycode:copy id="f-900059e12d12-on_repo">
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

</storycode:copy>
<a id="f-900059e12d12-stage"></a>
### `stage`

<storycode:copy id="f-900059e12d12-stage">
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

</storycode:copy>
<a id="f-900059e12d12-unstage"></a>
### `unstage`

`POST /api/git/unstage` — take a file out of the index, leaving the
working tree alone.

<storycode:copy id="f-900059e12d12-unstage">
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

</storycode:copy>
<a id="f-900059e12d12-discard"></a>
### `discard`

`POST /api/git/discard` — throw the working-tree changes to these files
away: a tracked file goes back to the index, an untracked one is deleted.
The pane confirms first; this does not, because it is the pane's verb.

<storycode:copy id="f-900059e12d12-discard">
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

</storycode:copy>
<a id="f-900059e12d12-CommitBody"></a>
### `CommitBody`

<storycode:copy id="f-900059e12d12-CommitBody">
#[derive(Deserialize)]
pub struct CommitBody {
    pub message: String,
    #[serde(default)]
    pub amend: bool,
}

</storycode:copy>
<a id="f-900059e12d12-commit"></a>
### `commit`

`POST /api/git/commit` — commit what is staged, or amend `HEAD` — never
a `HEAD` below the publication floor, which is a record.

<storycode:copy id="f-900059e12d12-commit">
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

</storycode:copy>
<a id="f-900059e12d12-push"></a>
### `push`

`POST /api/git/push` — push the branch, setting an upstream the first
time. Nothing is forced, ever.

<storycode:copy id="f-900059e12d12-push">
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

</storycode:copy>
<a id="f-900059e12d12-pull"></a>
### `pull`

`POST /api/git/pull` — fast-forward only. A merge or a rebase is a
decision with conflicts in it, not a button.

<storycode:copy id="f-900059e12d12-pull">
/// `POST /api/git/pull` — fast-forward only. A merge or a rebase is a
/// decision with conflicts in it, not a button.
pub async fn pull(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    on_repo(state, move |root| {
        let said = git_ok(root, &["pull", "--ff-only"])?;
        Ok(json!({ "ok": true, "said": said.trim() }))
    })
    .await
}

</storycode:copy>
<a id="f-900059e12d12-branches"></a>
### `branches`

`GET /api/git/branches` — local branches, each with its upstream.

<storycode:copy id="f-900059e12d12-branches">
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

</storycode:copy>
<a id="f-900059e12d12-CheckoutBody"></a>
### `CheckoutBody`

<storycode:copy id="f-900059e12d12-CheckoutBody">
#[derive(Deserialize)]
pub struct CheckoutBody {
    pub branch: String,
    #[serde(default)]
    pub create: bool,
}

</storycode:copy>
<a id="f-900059e12d12-checkout"></a>
### `checkout`

`POST /api/git/checkout` — switch to a branch, or create it here.

<storycode:copy id="f-900059e12d12-checkout">
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

</storycode:copy>
<a id="f-900059e12d12-StashBody"></a>
### `StashBody`

<storycode:copy id="f-900059e12d12-StashBody">
#[derive(Deserialize)]
pub struct StashBody {
    /// `push` or `pop`.
    pub action: String,
}

</storycode:copy>
<a id="f-900059e12d12-stash"></a>
### `stash`

`POST /api/git/stash` — put the working tree aside, or take it back.

<storycode:copy id="f-900059e12d12-stash">
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

</storycode:copy>
<a id="f-900059e12d12-tests"></a>
### `tests`

The focused fixture creates a temporary repository, stages a rename containing quoted and newline paths, then verifies workspace-relative identities and a literal magic-looking filename. Parsing cases separately reject incomplete rename and status records.

<storycode:copy id="f-900059e12d12-tests">
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
        let (_, items) = parse_status_zero("## main\0R  new.rs\0old.rs\0", "").unwrap();
        let item = &items[0];
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
    #[test]
    fn zero_records_preserve_literal_paths_and_workspace_relative_renames() {
        let (_, files) = parse_status_zero("## main\0 M sub/a -> b\n\"é.md\0R  sub/new\nname\0sub/old -> name\0?? other/outside\0?? sub/:(glob)*.md\0", "sub/").unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0]["path"], "a -> b\n\"é.md");
        assert_eq!(files[1]["path"], "new\nname");
        assert_eq!(files[1]["from"], "old -> name");
        assert_eq!(files[2]["path"], ":(glob)*.md");
        assert!(parse_status_zero("## main\0R  target\0", "").is_err());
        assert!(parse_status_zero("## main\0?? truncated", "").is_err());
    }
    #[test]
    fn real_git_status_preserves_quoted_paths_and_literal_pathspecs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git_ok(root, &["init", "-q"]).unwrap();
        std::fs::create_dir(root.join("sub")).unwrap();
        let names = ["quoted \"é\n -> file.md", ":(glob)*.md", "other.md"];
        for name in names {
            std::fs::write(root.join("sub").join(name), "before\n").unwrap();
        }
        git_ok(root, &["add", "--", "sub"]).unwrap();
        git_ok(
            root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "base",
            ],
        )
        .unwrap();
        std::fs::rename(
            root.join("sub").join(names[0]),
            root.join("sub/new\nname.md"),
        )
        .unwrap();
        git_ok(root, &["add", "--", "sub"]).unwrap();
        for name in &names[1..] {
            std::fs::write(root.join("sub").join(name), "changed\n").unwrap();
        }
        let folder = root.join("sub");
        let prefix = git_ok(&folder, &["rev-parse", "--show-prefix"]).unwrap();
        let raw = git_ok(
            &folder,
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "-b",
                "--untracked-files=all",
            ],
        )
        .unwrap();
        let (_, files) = parse_status_zero(&raw, prefix.strip_suffix('\n').unwrap()).unwrap();
        assert!(
            files
                .iter()
                .any(|file| file["path"] == "new\nname.md" && file["from"] == names[0])
        );
        let literal = git_ok(
            &folder,
            &["diff", "--no-ext-diff", "--no-textconv", "--", names[1]],
        )
        .unwrap();
        assert!(literal.contains(":(glob)*.md"));
        assert!(!literal.contains("other.md"));
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/git_ops.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/git_ops.rs"><storycode:paste select="#f-900059e12d12-git"/><storycode:paste select="#f-900059e12d12-git_ok"/><storycode:paste select="#f-900059e12d12-is_repository"/><storycode:paste select="#f-900059e12d12-not_a_repository"/><storycode:paste select="#f-900059e12d12-checked_path"/><storycode:paste select="#f-900059e12d12-paths_of"/><storycode:paste select="#f-900059e12d12-parse_status_line"/><storycode:paste select="#f-900059e12d12-parse_branch_line"/><storycode:paste select="#f-900059e12d12-changes"/><storycode:paste select="#f-900059e12d12-DiffParams"/><storycode:paste select="#f-900059e12d12-diff"/><storycode:paste select="#f-900059e12d12-PathsBody"/><storycode:paste select="#f-900059e12d12-on_repo"/><storycode:paste select="#f-900059e12d12-stage"/><storycode:paste select="#f-900059e12d12-unstage"/><storycode:paste select="#f-900059e12d12-discard"/><storycode:paste select="#f-900059e12d12-CommitBody"/><storycode:paste select="#f-900059e12d12-commit"/><storycode:paste select="#f-900059e12d12-push"/><storycode:paste select="#f-900059e12d12-pull"/><storycode:paste select="#f-900059e12d12-branches"/><storycode:paste select="#f-900059e12d12-CheckoutBody"/><storycode:paste select="#f-900059e12d12-checkout"/><storycode:paste select="#f-900059e12d12-StashBody"/><storycode:paste select="#f-900059e12d12-stash"/><storycode:paste select="#f-900059e12d12-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/components/DiffView.tsx](contract.md#output-2d0324b58c4d)
- [apps/web/src/views/GitPane.tsx](../../../mechanisms/apps/web/src/views/GitPane.tsx.md#output-f7cd01998027)
- [crates/hickory-cli/src/serve/git_ops.rs](contract.md#output-900059e12d12)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)

### Tests and fixtures

- [apps/web/src/components/DiffView.test.tsx](contract.md#output-ed96f162b5e6)
- [apps/web/src/views/GitPane.test.tsx](../../../mechanisms/apps/web/src/views/GitPane.tsx.md#output-0e5613784fdf)
- [crates/hickory-cli/tests/git_ops.rs](../../../mechanisms/crates/hickory-cli/tests/git_ops.rs.md#output-34c22c3c39d6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
