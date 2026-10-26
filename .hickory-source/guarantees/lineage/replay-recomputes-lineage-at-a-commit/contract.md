<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit

For: Authors editing generated text and reviewers tracing a result to its source.

Given a `.md` document in a git repository, when a reader asks for its
lineage at an earlier commit — `hick lineage --at <commit>`, or the time
slider on the lineage view — then that commit's document is read out of git,
woven **without executing anything**, and its provenance reported exactly;
and when the old document does not parse with today's binary, then the answer
says *replay works back to the last grammar change* rather than surfacing a
parse error.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/replay-recomputes-lineage-at-a-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-f3a948bf3c55"></a>
## `apps/web/src/lineage/TimeSlider.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit](contract.md).

<a id="f-f3a948bf3c55-commit"></a>
### `commit`

<storycode:copy id="f-f3a948bf3c55-commit">
// docs/guarantees/lineage/replay-recomputes-lineage-at-a-commit.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";

import { TimeSlider, indexOf, positions } from "./TimeSlider";
import type { ReplayCommit } from "../api/types";

const commit = (over: Partial<ReplayCommit>): ReplayCommit => ({
  sha: "aaa",
  short: "aaa",
  time: 1_700_000_000,
  author: "T",
  subject: "a change",
  path: "note.md",
  ...over,
});

</storycode:copy>
<a id="f-f3a948bf3c55-commits"></a>
### `commits`

git reports newest first; a person reads time oldest → newest.

<storycode:copy id="f-f3a948bf3c55-commits">
// git reports newest first; a person reads time oldest → newest.
const commits = [
  commit({ sha: "new", short: "new", subject: "newer", time: 200 }),
  commit({ sha: "old", short: "old", subject: "older", time: 100 }),
];

</storycode:copy>
<a id="f-f3a948bf3c55-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f3a948bf3c55-expression_statement">
beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

</storycode:copy>
<a id="f-f3a948bf3c55-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f3a948bf3c55-expression_statement-2">
describe("the stops", () => {
  it("run oldest to newest and end at the working tree", () => {
    // The last position is NOT a commit and must not pretend to be one.
    const stops = positions(commits);
    expect(stops.map((c) => c?.sha ?? null)).toEqual(["old", "new", null]);
  });

  it("puts an unknown selection at the working tree rather than at nothing", () => {
    expect(indexOf(commits, null)).toBe(2);
    expect(indexOf(commits, "old")).toBe(0);
    expect(indexOf(commits, "gone")).toBe(2);
  });
});

</storycode:copy>
<a id="f-f3a948bf3c55-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-f3a948bf3c55-expression_statement-3">
describe("the slider", () => {
  it("names the commit it is showing, and says the last stop is not one", () => {
    render(<TimeSlider commits={commits} at={null} onChange={() => {}} />);
    expect(screen.getByText(/As it stands/)).toBeTruthy();
    expect(screen.getByText(/the working tree, which is not a commit/)).toBeTruthy();
  });

  it("moves to the commit at the position picked", () => {
    const onChange = vi.fn();
    render(<TimeSlider commits={commits} at={null} onChange={onChange} />);
    fireEvent.change(screen.getByRole("slider"), { target: { value: "0" } });
    expect(onChange).toHaveBeenCalledWith("old");
  });

  it("states the grammar boundary rather than showing an error", () => {
    render(
      <TimeSlider
        commits={commits}
        at="old"
        onChange={() => {}}
        boundary="replay stops here: Replay works back to the last grammar change"
      />,
    );
    expect(
      screen.getByText(/Replay works back to the last grammar change/),
    ).toBeTruthy();
  });

  it("says plainly that replay reads git and remembers nothing of its own", () => {
    render(<TimeSlider commits={[]} at={null} onChange={() => {}} />);
    expect(screen.getByText(/nowhere to slide to/)).toBeTruthy();
  });

  it("never claims to relate one version's spans to another's", () => {
    render(<TimeSlider commits={commits} at={null} onChange={() => {}} />);
    expect(screen.getByText(/relating one version/)).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-f3a948bf3c55-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-f3a948bf3c55-expression_statement-4">
it("shares native and model replay selection and retires replaced project handles", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const change = vi.fn();
  const viewFor = (owner: string, at: string | null, stops = commits) => <UiActionContext.Provider value={registry}><TimeSlider owner={owner} commits={stops} at={at} onChange={change} /></UiActionContext.Provider>;
  const view = render(viewFor("first", null));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (handle: string, args: Record<string, number> = {}) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  const pick = () => actions().find(action => action.label === "Select lineage replay stop")!.handle;
  expect(await run(pick(), { index: 0 })).toMatchObject({ status: "result", outcome: { value: { selection_requested: true, replay_completed: false } } });
  expect(change).toHaveBeenLastCalledWith("old");
  fireEvent.change(screen.getByRole("slider"), { target: { value: "1" } });
  expect(change).toHaveBeenLastCalledWith("new");
  const stale = pick();
  view.rerender(viewFor("second", "gone"));
  expect(screen.queryByText("As it stands")).toBeNull();
  expect(screen.getByText(/Selected revision is unavailable/)).toBeTruthy();
  change.mockClear();
  expect(await run(stale, { index: 0 })).not.toMatchObject({ status: "result" });
  expect(change).not.toHaveBeenCalled();
  view.rerender(viewFor("second", "gone", []));
  expect(actions().some(action => action.label === "Select lineage replay stop")).toBe(false);
  expect(await run(actions().find(action => action.label === "Show working-tree lineage")!.handle)).toMatchObject({ status: "result", outcome: { value: { at: null, read_completed: false } } });
  expect(change).toHaveBeenLastCalledWith(null);
  fireEvent.click(screen.getByRole("button", { name: "Show working-tree lineage" }));
  expect(change).toHaveBeenCalledTimes(2);
});
</storycode:copy>

### Assemble `apps/web/src/lineage/TimeSlider.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/TimeSlider.test.tsx"><storycode:paste select="#f-f3a948bf3c55-commit"/><storycode:paste select="#f-f3a948bf3c55-commits"/><storycode:paste select="#f-f3a948bf3c55-expression_statement"/><storycode:paste select="#f-f3a948bf3c55-expression_statement-2"/><storycode:paste select="#f-f3a948bf3c55-expression_statement-3"/><storycode:paste select="#f-f3a948bf3c55-expression_statement-4"/></storycode:file>

<a id="output-a23417d51224"></a>
## `crates/hickory-cli/src/replay.rs`

Related contracts: [Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit](contract.md).

<a id="f-a23417d51224-ReplayCommit"></a>
### `ReplayCommit`

Replay: exact lineage at any commit.
See `docs/specs/freeform/provenance-across-versions.md`. `hick lineage`
answers *which span produced these bytes* exactly, and only for the
document as it stands. Blame back to an older commit lands in that
commit's document, and nothing relates that document's spans to today's.
Replay needs no new data model at all: each commit contains its document,
a weave is deterministic, and weave-only checks are already affordable
(`/render` and the refactor badge never execute). So exact lineage is
**recomputable at any commit** — read the old document out of git, weave
it, map. What it cannot do is relate two versions to each other; it gives
the state *at* a commit, never the thread between commits. That thread is
a recorded correspondence, and is deliberately later.
**The one limit is stated rather than discovered.** Replay weaves an old
document with today's binary, which asks for a grammar-compatibility
promise the product has not made — `pre-launch.md` reserves the right to
break the grammar, and says document formats deserve more care than code
precisely because a `.md` file outlives any version of the tool. So the
honest claim is **replay works back to the last grammar change**, and a
commit past that boundary says so in those words instead of failing with
a parse error nobody can act on.
Nothing here checks anything out. The tree at a commit is materialized
into a scratch directory with `git archive`, so the working tree is
untouched and the up-loop never sees a file appear — and the whole tree
rather than the document alone, because a document's pastes, includes and
upstream edges read their siblings, and replaying without them would
report a lineage the old commit never had.

<storycode:copy id="f-a23417d51224-ReplayCommit">
//! Replay: exact lineage at any commit.
//!
//! See `docs/specs/freeform/provenance-across-versions.md`. `hick lineage`
//! answers *which span produced these bytes* exactly, and only for the
//! document as it stands. Blame back to an older commit lands in that
//! commit's document, and nothing relates that document's spans to today's.
//!
//! Replay needs no new data model at all: each commit contains its document,
//! a weave is deterministic, and weave-only checks are already affordable
//! (`/render` and the refactor badge never execute). So exact lineage is
//! **recomputable at any commit** — read the old document out of git, weave
//! it, map. What it cannot do is relate two versions to each other; it gives
//! the state *at* a commit, never the thread between commits. That thread is
//! a recorded correspondence, and is deliberately later.
//!
//! **The one limit is stated rather than discovered.** Replay weaves an old
//! document with today's binary, which asks for a grammar-compatibility
//! promise the product has not made — `pre-launch.md` reserves the right to
//! break the grammar, and says document formats deserve more care than code
//! precisely because a `.md` file outlives any version of the tool. So the
//! honest claim is **replay works back to the last grammar change**, and a
//! commit past that boundary says so in those words instead of failing with
//! a parse error nobody can act on.
//!
//! Nothing here checks anything out. The tree at a commit is materialized
//! into a scratch directory with `git archive`, so the working tree is
//! untouched and the up-loop never sees a file appear — and the whole tree
//! rather than the document alone, because a document's pastes, includes and
//! upstream edges read their siblings, and replaying without them would
//! report a lineage the old commit never had.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context as _, Result, bail};
use serde::Serialize;

use crate::{DocRun, ExecutorChoice, RunMode, run_doc};

/// One commit that touched a document.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReplayCommit {
    pub sha: String,
    pub short: String,
    /// Unix seconds. Formatted by the reader, in their own locale.
    pub time: i64,
    pub author: String,
    pub subject: String,
    /// The path the document had AT this commit. `--follow` means the slider
    /// walks across renames, and a replay has to read the old name.
    pub path: String,
}

</storycode:copy>
<a id="f-a23417d51224-Replay"></a>
### `Replay`

<storycode:copy id="f-a23417d51224-Replay">
/// What a replay at one commit produced.
pub enum Replay {
    /// The old document wove. Its lineage is exact, recomputed — not
    /// recalled from any store.
    Woven(Box<DocRun>),
    /// The old document does not parse with today's binary. Stated as the
    /// boundary it is rather than as a failure.
    GrammarBoundary { commit: String, detail: String },
}

</storycode:copy>
<a id="f-a23417d51224-grammar_boundary_message"></a>
### `grammar_boundary_message`

The message the boundary reads as, everywhere it is reported.

<storycode:copy id="f-a23417d51224-grammar_boundary_message">
/// The message the boundary reads as, everywhere it is reported.
pub fn grammar_boundary_message(commit: &str, path: &str, detail: &str) -> String {
    format!(
        "replay stops here: {path} at {commit} does not parse with this \
         version of hick, which means the grammar changed between that \
         commit and now.\n  \
         Replay works back to the last grammar change — the document is \
         still exactly as it was, and `git show {commit}:{path}` reads it.\n  \
         Nothing is wrong with the commit; this version of the tool cannot \
         weave it.\n  \
         (parser said: {detail})"
    )
}

</storycode:copy>
<a id="f-a23417d51224-FIELD"></a>
### `FIELD`

<storycode:copy id="f-a23417d51224-FIELD">
// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

const FIELD: char = '\u{1f}';
</storycode:copy>
<a id="f-a23417d51224-RECORD"></a>
### `RECORD`

<storycode:copy id="f-a23417d51224-RECORD">
const RECORD: char = '\u{1e}';

</storycode:copy>
<a id="f-a23417d51224-history"></a>
### `history`

Every commit that touched `rel` (repository-relative), newest first.
A directory that is not a repository, a machine with no `git`, and a file
with no history are all the same answer — an empty list — because opening
a folder of notes that is not under version control is entirely normal and
the slider simply has nowhere to go.

<storycode:copy id="f-a23417d51224-history">
/// Every commit that touched `rel` (repository-relative), newest first.
///
/// A directory that is not a repository, a machine with no `git`, and a file
/// with no history are all the same answer — an empty list — because opening
/// a folder of notes that is not under version control is entirely normal and
/// the slider simply has nowhere to go.
pub fn history(root: &Path, rel: &str, limit: usize) -> Vec<ReplayCommit> {
    let format = format!("{RECORD}%H{FIELD}%h{FIELD}%at{FIELD}%an{FIELD}%s");
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "log",
            &format!("-n{limit}"),
            // Renames as renames: a document that moved is one history, and
            // a slider that stopped at the rename would say the document was
            // born there.
            "--follow",
            "--name-only",
            &format!("--pretty=format:{format}"),
            "--",
            rel,
        ])
        .output();
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    parse_history(&String::from_utf8_lossy(&out.stdout), rel)
}

</storycode:copy>
<a id="f-a23417d51224-parse_history"></a>
### `parse_history`

Split out and pure so the awkward parts are arguable in a test.

<storycode:copy id="f-a23417d51224-parse_history">
/// Split out and pure so the awkward parts are arguable in a test.
fn parse_history(text: &str, fallback_path: &str) -> Vec<ReplayCommit> {
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
        let time: i64 = parts.next().unwrap_or_default().trim().parse().unwrap_or(0);
        let author = parts.next().unwrap_or_default().to_string();
        let rest = parts.collect::<Vec<_>>().join(&FIELD.to_string());
        // `--name-only` appends the paths after the subject, separated by a
        // blank line. The FIRST of them is the name the document had at this
        // commit, which is what a replay must read.
        let mut lines = rest.lines();
        let subject = lines.next().unwrap_or_default().trim().to_string();
        let path = lines
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or(fallback_path)
            .to_string();
        commits.push(ReplayCommit {
            sha,
            short,
            time,
            author,
            subject,
            path,
        });
    }
    commits
}

</storycode:copy>
<a id="f-a23417d51224-materialize"></a>
### `materialize`

Replay
Materialize the tree at `commit` into `dest`, without touching the working
tree. `git archive` reads objects; nothing is checked out and no worktree
is added.

<storycode:copy id="f-a23417d51224-materialize">
// ---------------------------------------------------------------------------
// Replay
// ---------------------------------------------------------------------------

/// Materialize the tree at `commit` into `dest`, without touching the working
/// tree. `git archive` reads objects; nothing is checked out and no worktree
/// is added.
fn materialize(root: &Path, commit: &str, dest: &Path) -> Result<()> {
    let archive = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["archive", "--format=tar", commit])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run `git archive` (is git installed?)")?;
    if !archive.status.success() {
        bail!(
            "could not read the tree at {commit}: {}",
            String::from_utf8_lossy(&archive.stderr).trim()
        );
    }
    tar::Archive::new(archive.stdout.as_slice())
        .unpack(dest)
        .with_context(|| format!("could not unpack the tree at {commit}"))?;
    Ok(())
}

</storycode:copy>
<a id="f-a23417d51224-replay_at"></a>
### `replay_at`

Weave the document `rel` as it stood at `commit`, and return its run.
**Weave-only, never execute.** A replay of last March must not run last
March's commands: they would run against today's machine, today's network
and today's containers, and whatever came out would be neither what
happened then nor what happens now. Cached transcripts where present,
never-run everywhere else — the same mode `/render` and the refactor badge
use.

<storycode:copy id="f-a23417d51224-replay_at">
/// Weave the document `rel` as it stood at `commit`, and return its run.
///
/// **Weave-only, never execute.** A replay of last March must not run last
/// March's commands: they would run against today's machine, today's network
/// and today's containers, and whatever came out would be neither what
/// happened then nor what happens now. Cached transcripts where present,
/// never-run everywhere else — the same mode `/render` and the refactor badge
/// use.
pub async fn replay_at(root: &Path, rel: &str, commit: &str) -> Result<Replay> {
    let scratch = tempfile::tempdir().context("could not create a scratch directory")?;
    materialize(root, commit, scratch.path())?;

    let doc_path = scratch.path().join(rel);
    if !doc_path.is_file() {
        bail!(
            "{rel} does not exist at {commit}.\n  \
             The slider walks the commits that touched this document, so this \
             usually means the path was different then — `git log --follow -- \
             {rel}` names the commits and the names it had."
        );
    }

    // Parse first, and separately, so the grammar boundary is REPORTED as
    // itself. Weaving would report the same failure wrapped in pipeline
    // language, which is exactly the obscure parse failure this must not be.
    let source = std::fs::read_to_string(&doc_path)
        .with_context(|| format!("could not read {rel} at {commit}"))?;
    if let Err(e) = hick_lang::parse(&source) {
        return Ok(Replay::GrammarBoundary {
            commit: commit.to_string(),
            detail: e.to_string(),
        });
    }

    match run_doc(&doc_path, &[], RunMode::Weave, ExecutorChoice::Local).await {
        Ok(run) => Ok(Replay::Woven(Box::new(run))),
        // A document that parses but will not weave is a different fact from
        // a grammar change, and saying so is the whole point of splitting the
        // two: this one is about the commit, not about the tool.
        Err(e) => bail!(
            "{rel} at {commit} parses but does not weave with this version of \
             hick ({e:#}).\n  \
             This is not the grammar boundary — the document is readable. It \
             usually means the commit referred to something outside its own \
             tree, or to a feature this build does not have."
        ),
    }
}

</storycode:copy>
<a id="f-a23417d51224-git_root"></a>
### `git_root`

The repository root for `dir`, or `None` when it is not a work tree.

<storycode:copy id="f-a23417d51224-git_root">
/// The repository root for `dir`, or `None` when it is not a work tree.
pub fn git_root(dir: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

</storycode:copy>
<a id="f-a23417d51224-tests"></a>
### `tests`

<storycode:copy id="f-a23417d51224-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn record(fields: &[&str]) -> String {
        format!("{RECORD}{}", fields.join(&FIELD.to_string()))
    }

    #[test]
    fn a_commit_carries_the_name_the_document_had_then() {
        // The whole reason for `--follow`: a slider that stopped at a rename
        // would say the document was born there.
        let text = record(&[
            "abc123",
            "abc123d",
            "1700000000",
            "Ada Lovelace",
            "Rename the document\n\ndocs/old-name.md\n",
        ]);
        let commits = parse_history(&text, "docs/new-name.md");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].sha, "abc123");
        assert_eq!(commits[0].subject, "Rename the document");
        assert_eq!(commits[0].path, "docs/old-name.md");
        assert_eq!(commits[0].time, 1_700_000_000);
    }

    #[test]
    fn a_commit_with_no_name_block_keeps_the_path_asked_about() {
        let text = record(&["a", "a", "1", "N", "Subject"]);
        let commits = parse_history(&text, "app.md");
        assert_eq!(commits[0].path, "app.md");
    }

    #[test]
    fn an_empty_history_is_no_commits_rather_than_a_broken_one() {
        assert!(parse_history("", "app.md").is_empty());
        assert!(parse_history("\n\n", "app.md").is_empty());
    }

    #[test]
    fn the_boundary_says_replay_works_back_to_the_last_grammar_change() {
        // The exact words matter: this is the sentence that tells a reader
        // the tool is at its limit rather than that their document is broken.
        let msg = grammar_boundary_message("abc123", "app.md", "unexpected tag");
        assert!(msg.contains("Replay works back to the last grammar change"));
        assert!(msg.contains("git show abc123:app.md"));
        assert!(msg.contains("Nothing is wrong with the commit"));
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/replay.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/replay.rs"><storycode:paste select="#f-a23417d51224-ReplayCommit"/><storycode:paste select="#f-a23417d51224-Replay"/><storycode:paste select="#f-a23417d51224-grammar_boundary_message"/><storycode:paste select="#f-a23417d51224-FIELD"/><storycode:paste select="#f-a23417d51224-RECORD"/><storycode:paste select="#f-a23417d51224-history"/><storycode:paste select="#f-a23417d51224-parse_history"/><storycode:paste select="#f-a23417d51224-materialize"/><storycode:paste select="#f-a23417d51224-replay_at"/><storycode:paste select="#f-a23417d51224-git_root"/><storycode:paste select="#f-a23417d51224-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lineage/TimeSlider.tsx](../../../mechanisms/apps/web/src/lineage/TimeSlider.tsx.md#output-dca5a7314b59)
- [apps/web/src/views/LineageView.tsx](../../../mechanisms/apps/web/src/views/LineageView.tsx.md#output-f7cde247eee4)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/replay.rs](contract.md#output-a23417d51224)
- [crates/hickory-cli/src/serve/history.rs](../../../mechanisms/crates/hickory-cli/src/serve/history.rs.md#output-be949b7ea526)

### Tests and fixtures

- [apps/web/src/lineage/TimeSlider.test.tsx](contract.md#output-f3a948bf3c55)
- [crates/hickory-cli/tests/replay_and_floor.rs](../../../mechanisms/crates/hickory-cli/tests/replay_and_floor.rs.md#output-652493fd98fd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
