<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/GitPane.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f7cd01998027"></a>
## `apps/web/src/views/GitPane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Git Pane Does The Daily Loop](../../../../../guarantees/collaboration/the-git-pane-does-the-daily-loop/contract.md), [The Publication Floor Is Computed On Every Read And Shown](../../../../../guarantees/collaboration/the-publication-floor-is-computed-and-shown/contract.md), [The History Lens Reads The Repository As A Story](../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md).

<a id="f-f7cd01998027-ROW"></a>
### `ROW`

<storycode:copy id="f-f7cd01998027-ROW">
import { useGitWorkingActions } from "./uiGitWorkingActions";
import type { UiOutcome } from "../lib/uiActions";
import { useGitHistoryActions } from "./uiGitHistoryActions";
import { openCommit } from "../lib/readingViews";
// The repository, as something to read.
//
// The graph is the point, and what makes a commit graph worth drawing is that
// it answers "what came from where" in a glance that no list of commits can.
// So the lanes are real — computed by lib/gitGraph.ts and drawn as SVG beside
// the rows — rather than a decorative bullet per line.
//
// A commit expands in place. Its files were fetched with the log (one `git
// log --numstat` for everything), so expanding costs nothing and there is no
// spinner, no second request, and no moment where the row is open and empty.
// That is why the API is shaped the way it is.
//
// Above the graph: the working tree, and the verbs a person uses on it every
// day — stage, unstage, discard, commit and amend, push and pull, switch and
// create a branch, stash and pop, and a diff for any file. This pane was
// read-only for a while, on the argument that a half-built git UI — commit
// but not amend, stage but not stash — teaches a workflow it cannot finish.
// The argument was right; the answer was to finish it. Every verb is one git
// command run as itself, and git's own words are shown when it refuses.
// A merge or a rebase is a decision with conflicts in it, so pull is
// fast-forward only, and a terminal is still one click away on the tree.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { api } from "../api/client";
import type { GitBranch, GitChangeFile, GitChanges, GitDiff, GitFileChange, GitLog } from "../api/types";
import { graphWidth, laneColor, layout } from "../lib/gitGraph";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { DiffView } from "../components/DiffView";
import { BisectPane } from "./BisectPane";
import { openLiterate } from "../lib/openRepresentation";

/** Row height and lane spacing, in px. Shared by the SVG and the list, which
 * is the only way the nodes line up with the text beside them. */
const ROW = 44;
</storycode:copy>
<a id="f-f7cd01998027-LANE"></a>
### `LANE`

<storycode:copy id="f-f7cd01998027-LANE">
const LANE = 14;
</storycode:copy>
<a id="f-f7cd01998027-EXPANDED_EXTRA"></a>
### `EXPANDED_EXTRA`

<storycode:copy id="f-f7cd01998027-EXPANDED_EXTRA">
const EXPANDED_EXTRA = 0;

</storycode:copy>
<a id="f-f7cd01998027-GitPane"></a>
### `GitPane`

<storycode:copy id="f-f7cd01998027-GitPane">
export function GitPane({
  onOpenFile,
  onOpenStory,
  onOpenLiterate,
}: {
  onOpenFile?: (path: string) => void;
  /** The same repository, read as a story: the history lens. */
  onOpenStory?: () => void;
  onOpenLiterate?: () => void;
}) {
  const [log, setLog] = useState<GitLog | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const [limit, setLimit] = useState(120);
  // Re-read everything after a verb, and when the files change under us.
  const [tick, setTick] = useState(0);
  const refresh = useCallback(() => setTick((n) => n + 1), []);

  useEffect(() => {
    let live = true;
    api.gitLog(limit).then(
      (answer) => { if (live) { setLog(answer); setError(null); } },
      (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [limit, tick]);

  useEffect(() => {
    window.addEventListener(FILES_CHANGED_EVENT, refresh);
    window.addEventListener("focus", refresh);
    return () => {
      window.removeEventListener(FILES_CHANGED_EVENT, refresh);
      window.removeEventListener("focus", refresh);
    };
  }, [refresh]);

  const historyAction = useGitHistoryActions({ log, error, open, limit,
    toggle: sha => setOpen(current => { const next = new Set(current); if (!next.delete(sha)) next.add(sha); return next; }),
    readCommit: openCommit, openFile: onOpenFile, story: onOpenStory, literate: onOpenLiterate,
    more: () => setLimit(n => n + 120), refresh,
  });

  const rows = useMemo(() => layout(log?.commits ?? []), [log]);
  const width = graphWidth(rows);

  if (error) return <p className="error git-pane__error">{error}</p>;
  if (!log) return <p className="muted git-pane__loading">Reading history…</p>;
  if (!log.repository) {
    return (
      <div className="git-pane git-pane--none">
        <p className="muted">
          This folder is not a git repository — which is an entirely normal
          thing for a folder of notes to be.
        </p>
        <p className="muted">
          Open a terminal on the folder’s row in the tree and run{" "}
          <span className="mono">git init</span> if you want one.
        </p>
      </div>
    );
  }
  if (log.commits.length === 0) {
    return <p className="muted git-pane__loading">No commits yet.</p>;
  }

  const graphPx = (width + 1) * LANE;

  const floor = log.floor;

  return (
    <div className="git-pane">
      <BisectPane />
      {onOpenLiterate && <button className="btn" onClick={() => void historyAction("literate")}>Literate views</button>}
      <WorkingTree tick={tick} refresh={refresh} onOpenFile={onOpenFile} head={log.commits[0]} />
      {/* The publication floor, stated rather than felt. Below it commits are
          records — someone else may be holding them; above it they are
          drafts, and re-emission may replace them. Merging moves the line,
          which is why it is computed on every read and never stored.
          docs/specs/freeform/expression-and-log.md */}
      {onOpenStory && (
        <p className="git-pane__story">
          <button
            type="button"
            className="btn"
            onClick={() => void historyAction("story")}
            data-tip="The same commits, oldest first, drawn as cards. A lens: read-only, and on disk nowhere."
          >
            Read as a story
          </button>
        </p>
      )}
      {floor && (
        <p className="git-floor" role="status">
          <span className="git-floor__count">
            {floor.drafts.length === 0
              ? "No drafts"
              : `${floor.drafts.length} draft${floor.drafts.length === 1 ? "" : "s"}`}
          </span>{" "}
          {floor.summary}
        </p>
      )}
      <ol className="git-log" style={{ ["--git-graph" as string]: `${graphPx}px` }}>
        {log.commits.map((commit, index) => {
          const row = rows[index];
          const expanded = open.has(commit.sha);
          return (
            <li
              key={commit.sha}
              className={`git-commit${commit.draft ? " git-commit--draft" : ""}`}
            >
              <div className="git-commit__line">
                {/* One SVG per row, so a row can grow when it is expanded
                    without the graph above it having to be redrawn. */}
                <svg
                  className="git-commit__graph"
                  width={graphPx}
                  height={ROW}
                  aria-hidden="true"
                >
                  {row.through.map((line, i) => (
                    <path
                      key={i}
                      className={`git-edge git-edge--c${laneColor(line.from)}`}
                      d={edgePath(line.from, line.to)}
                    />
                  ))}
                  <circle
                    className={`git-node git-node--c${laneColor(row.lane)}${
                      commit.parents.length > 1 ? " git-node--merge" : ""
                    }`}
                    cx={x(row.lane)}
                    cy={ROW / 2}
                    r={commit.parents.length > 1 ? 5 : 4}
                  />
                </svg>

                <button
                  type="button"
                  className="git-commit__summary"
                  aria-expanded={expanded}
                  onClick={() => void historyAction(`toggle:${commit.sha}`)}
                >
                  <span className="git-commit__subject">{commit.subject}</span>
                  {commit.draft && (
                    <span
                      className="git-draft"
                      data-tip="Above the publication floor: still a draft, because nobody else can be holding it yet. Below the floor a commit is a record and nothing may re-produce it."
                    >
                      draft
                    </span>
                  )}
                  <span className="git-commit__meta">
                    {commit.refs.map((ref) => (
                      <span key={ref} className="git-ref">
                        {ref}
                      </span>
                    ))}
                    <span className="git-commit__author">{commit.author}</span>
                    <span className="git-commit__when">{when(commit.time)}</span>
                    <span className="git-commit__sha mono">{commit.short}</span>
                    <Churn added={commit.added} removed={commit.removed} files={commit.files.length} />
                  </span>
                </button>
              </div>

              {expanded && (
                <div className="git-commit__detail">
                  <button className="btn" onClick={() => void historyAction(`commit:${commit.sha}`)}>Read commit</button>
                  {commit.body && <pre className="git-commit__body">{commit.body}</pre>}
                  {commit.files.length === 0 ? (
                    <p className="muted git-commit__nofiles">
                      {commit.parents.length > 1
                        ? "A merge — its changes are in the commits it joins."
                        : "No files changed."}
                    </p>
                  ) : (
                    <ul className="git-files">
                      {commit.files.map((file, fileIndex) => (
                        <li key={file.path}>
                          <button
                            type="button"
                            className="git-file"
                            onClick={() => void historyAction(`file:${commit.sha}:${fileIndex}`)}
                            data-tip={`Open ${file.path}`}
                          >
                            <span className={`git-file__status git-file__status--${file.status}`}>
                              {file.status}
                            </span>
                            <span className="git-file__path mono">
                              {file.from && (
                                <span className="git-file__from">{file.from} → </span>
                              )}
                              {file.path}
                            </span>
                            <FileChurn file={file} />
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </li>
          );
        })}
      </ol>
      {log.commits.length >= limit && (
        <button
          type="button"
          className="btn btn-small git-pane__more"
          onClick={() => void historyAction("more")}
        >
          Show older commits
        </button>
      )}
    </div>
  );
}

</storycode:copy>
<a id="f-f7cd01998027-x"></a>
### `x`

<storycode:copy id="f-f7cd01998027-x">
/** Horizontal centre of a lane. */
function x(lane: number): number {
  return LANE * (lane + 0.5);
}

</storycode:copy>
<a id="f-f7cd01998027-edgePath"></a>
### `edgePath`

The line from one lane to another across a row.
A straight run is a straight line; a change of lane is a curve, because a
dog-leg of two right angles reads as two separate lines meeting rather than
as one line moving.

<storycode:copy id="f-f7cd01998027-edgePath">
/**
 * The line from one lane to another across a row.
 *
 * A straight run is a straight line; a change of lane is a curve, because a
 * dog-leg of two right angles reads as two separate lines meeting rather than
 * as one line moving.
 */
function edgePath(from: number, to: number): string {
  const x0 = x(from);
  const x1 = x(to);
  if (from === to) return `M ${x0} 0 L ${x0} ${ROW + EXPANDED_EXTRA}`;
  const mid = ROW / 2;
  return `M ${x0} 0 C ${x0} ${mid}, ${x1} ${mid}, ${x1} ${ROW}`;
}

</storycode:copy>
<a id="f-f7cd01998027-when"></a>
### `when`

<storycode:copy id="f-f7cd01998027-when">
/** A date the way a person reads one in a log. */
export function when(seconds: number, now: number = Date.now()): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return "";
  const days = Math.floor((now - seconds * 1000) / 86_400_000);
  if (days < 0) return "just now";
  if (days === 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 30) return `${days}d ago`;
  if (days < 365) return `${Math.floor(days / 30)}mo ago`;
  return new Date(seconds * 1000).getFullYear().toString();
}

</storycode:copy>
<a id="f-f7cd01998027-Churn"></a>
### `Churn`

The size of a change, as two numbers and a file count.

<storycode:copy id="f-f7cd01998027-Churn">
/** The size of a change, as two numbers and a file count. */
function Churn({ added, removed, files }: { added: number; removed: number; files: number }) {
  return (
    <span
      className="git-churn"
      data-tip={`${files} file${files === 1 ? "" : "s"}, +${added} −${removed}`}
    >
      <span className="git-churn__files">{files}</span>
      <span className="git-churn__added">+{added}</span>
      <span className="git-churn__removed">−{removed}</span>
    </span>
  );
}

</storycode:copy>
<a id="f-f7cd01998027-FileChurn"></a>
### `FileChurn`

One file's counts. A binary file has none, and says so rather than
showing zeroes — which would claim the change touched nothing.

<storycode:copy id="f-f7cd01998027-FileChurn">
/** One file's counts. A binary file has none, and says so rather than
 * showing zeroes — which would claim the change touched nothing. */
function FileChurn({ file }: { file: GitFileChange }) {
  if (file.added === undefined || file.removed === undefined) {
    return <span className="git-churn git-churn--binary">binary</span>;
  }
  return (
    <span className="git-churn">
      <span className="git-churn__added">+{file.added}</span>
      <span className="git-churn__removed">−{file.removed}</span>
    </span>
  );
}


</storycode:copy>
<a id="f-f7cd01998027-Notice"></a>
### `Notice`

The working tree, and the verbs on it
What a verb answered, or what git said when it refused.

<storycode:copy id="f-f7cd01998027-Notice">
// ---------------------------------------------------------------------------
// The working tree, and the verbs on it
// ---------------------------------------------------------------------------

/** What a verb answered, or what git said when it refused. */
interface Notice {
  kind: "ok" | "fail";
  text: string;
}

</storycode:copy>
<a id="f-f7cd01998027-SideFile"></a>
### `SideFile`

A file on one side of the index, with the status letter for that side.

<storycode:copy id="f-f7cd01998027-SideFile">
/** A file on one side of the index, with the status letter for that side. */
interface SideFile {
  path: string;
  from?: string;
  status: string;
}

</storycode:copy>
<a id="f-f7cd01998027-sides"></a>
### `sides`

Split git's two columns into the two lists a person stages between.

<storycode:copy id="f-f7cd01998027-sides">
/** Split git's two columns into the two lists a person stages between. */
export function sides(files: readonly GitChangeFile[]): { staged: SideFile[]; unstaged: SideFile[] } {
  const staged: SideFile[] = [];
  const unstaged: SideFile[] = [];
  for (const file of files) {
    if (file.index === "?") {
      unstaged.push({ path: file.path, status: "A" });
      continue;
    }
    if (file.index !== " ") staged.push({ path: file.path, from: file.from, status: file.index });
    if (file.tree !== " ") unstaged.push({ path: file.path, status: file.tree });
  }
  return { staged, unstaged };
}

</storycode:copy>
<a id="f-f7cd01998027-WorkingTree"></a>
### `WorkingTree`

<storycode:copy id="f-f7cd01998027-WorkingTree">
function WorkingTree({
  tick,
  refresh,
  onOpenFile,
  head,
}: {
  tick: number;
  refresh: () => void;
  onOpenFile?: (path: string) => void;
  head?: { subject: string; body: string };
}) {
  const [changes, setChanges] = useState<GitChanges | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const inFlight = useRef(false);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [selected, setSelected] = useState<{ path: string; staged: boolean } | null>(null);
  const [diff, setDiff] = useState<GitDiff | null>(null);
  const [message, setMessage] = useState("");
  const [amend, setAmend] = useState(false);
  const [branches, setBranches] = useState<GitBranch[] | null>(null);
  const [newBranch, setNewBranch] = useState("");
  // Two clicks to discard: the first turns the button into the question.
  const [confirmDiscard, setConfirmDiscard] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    api.gitChanges().then(
      (answer) => live && setChanges(answer),
      (e: unknown) =>
        live && setNotice({ kind: "fail", text: e instanceof Error ? e.message : String(e) }),
    );
    return () => {
      live = false;
    };
  }, [tick]);

  // The selected file's diff, re-read with everything else.
  useEffect(() => {
    if (!selected) {
      setDiff(null);
      return;
    }
    let live = true;
    setDiff(null);
    api.gitDiff(selected.path, selected.staged).then(
      (answer) => live && setDiff(answer),
      () => live && setDiff(null),
    );
    return () => {
      live = false;
    };
  }, [selected, tick]);

  /** Run one verb: say what happened, then re-read the repository. */
  const run = useCallback(
    async (
      verb: string,
      action: () => Promise<{ said?: string } | unknown>,
      done: string,
      filesChanged = false,
    ): Promise<UiOutcome> => {
      if (inFlight.current) return { status: "unavailable", reason: "Another Git operation is pending" };
      inFlight.current = true;
      setBusy(verb);
      setNotice(null);
      try {
        const answer = (await action()) as { said?: string; ok?: boolean } | undefined;
        if (answer?.ok === false) throw new Error(answer.said || "Git refused the operation");
        const said = answer?.said?.trim();
        setNotice({ kind: "ok", text: said ? `${done} — ${said}` : done });
        refresh();
        // A checkout, a pull, a stash or a discard rewrote files on disk:
        // every open pane and the tree need to know.
        if (filesChanged) window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
        if (verb === "commit") { setMessage(""); setAmend(false); }
        if (verb === "checkout") { setBranches(null); setNewBranch(""); }
        return { status: "completed", value: { verb, reply: JSON.stringify(answer ?? null), scope: "Actual host reply; not independent verification" } };
      } catch (e) {
        const reason = e instanceof Error ? e.message : String(e);
        setNotice({ kind: "fail", text: reason });
        return { status: "unavailable", reason };
      } finally {
        inFlight.current = false;
        setBusy(null);
      }
    },
    [refresh],
  );

  const { staged, unstaged } = sides(changes?.files ?? []);
  const ahead = changes?.ahead ?? 0;
  const behind = changes?.behind ?? 0;

  const openBranches = async (): Promise<UiOutcome> => {
    if (branches) { setBranches(null); return { status: "completed", value: { open: false } }; }
    return run("branches", async () => { const answer = await api.gitBranches(); setBranches(answer.branches); return answer; }, "Read branches");
  };

  const toggleAmend = (on: boolean) => {
    setAmend(on);
    // Amending with an empty message means "keep the message"; start from
    // the one the commit has, so a reword is an edit rather than a retype.
    if (on && message.trim() === "" && head) {
      setMessage(head.body ? `${head.subject}\n\n${head.body}` : head.subject);
    }
  };

  const workingAction = useGitWorkingActions({ changes, busy, notice, selected, diff, message, amend, branches, newBranch, confirmDiscard,
    staged: staged.map(f => f.path), unstaged: unstaged.map(f => f.path), run, openBranches,
    setMessage, setAmend: toggleAmend, setNewBranch, setDiscard: setConfirmDiscard,
    pick: (path, staged) => { setDiff(null); setSelected({ path, staged }); }, openFile: onOpenFile,
    operations: { pull: () => api.gitPull(), push: () => api.gitPush(), stash: pop => api.gitStash(pop ? "pop" : "push"),
      stage: paths => api.gitStage(paths ? { paths } : { all: true }), unstage: paths => api.gitUnstage(paths ? { paths } : { all: true }),
      discard: path => api.gitDiscard([path]), checkout: (name, create) => create ? api.gitCheckout(name, true) : api.gitCheckout(name), commit: () => api.gitCommit(message.trim(), amend),
      review: () => selected ? openLiterate({ kind: "files", paths: [selected.path] }, selected.staged ? "HEAD" : "INDEX", selected.staged ? "INDEX" : undefined) : Promise.reject(new Error("No selected diff")),
    },
  });
  const sideKey = (path: string, stagedSide: boolean) => `${stagedSide ? "staged" : "unstaged"}:${(stagedSide ? staged : unstaged).findIndex(f => f.path === path)}`;
  if (!changes?.repository) return null;

  return (
    <section className="git-work" aria-label="Working tree">
      <header className="git-work__head">
        <button
          type="button"
          className="git-branch"
          aria-expanded={branches !== null}
          disabled={busy !== null}
          onClick={() => void workingAction("branches")}
          data-tip="Switch or create a branch"
        >
          <span className="git-branch__icon" aria-hidden="true">⎇</span>
          <span className="mono">{changes.branch}</span>
          <span aria-hidden="true">▾</span>
        </button>
        <span className="muted git-work__upstream">
          {changes.upstream ? (
            <>
              <span className="mono">{changes.upstream}</span>
              {ahead > 0 && <span className="git-count" data-tip={`${ahead} to push`}>↑{ahead}</span>}
              {behind > 0 && <span className="git-count" data-tip={`${behind} to pull`}>↓{behind}</span>}
            </>
          ) : (
            "no upstream yet"
          )}
        </span>
        <span className="git-work__actions">
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null}
            onClick={() => void workingAction("pull")}
            data-tip="git pull --ff-only — never a merge, never a rebase"
          >
            Pull
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null}
            onClick={() => void workingAction("push")}
            data-tip={changes.upstream ? "git push" : "git push -u origin <branch>"}
          >
            Push{ahead > 0 ? ` ${ahead}` : ""}
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null || changes.files.length === 0}
            onClick={() => void workingAction("stash")}
            data-tip="git stash push --include-untracked"
          >
            Stash
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null}
            onClick={() => void workingAction("pop")}
            data-tip="git stash pop"
          >
            Pop stash
          </button>
        </span>
      </header>

      {branches && (
        <div className="git-branches" role="group" aria-label="Branches">
          <ul className="git-branches__list">
            {branches.map((branch, index) => (
              <li key={branch.name}>
                <button
                  type="button"
                  className={`git-branches__item${branch.current ? " is-current" : ""}`}
                  disabled={branch.current || busy !== null}
                  onClick={() => void workingAction(`checkout:${index}`)}
                >
                  <span className="mono">{branch.name}</span>
                  {branch.upstream && <span className="muted mono"> → {branch.upstream}</span>}
                </button>
              </li>
            ))}
          </ul>
          <form
            className="git-branches__new"
            onSubmit={event => { event.preventDefault(); void workingAction("branch-create"); }}
          >
            <input
              className="git-branches__input mono"
              placeholder="new branch name"
              aria-label="New branch name"
              value={newBranch}
              onChange={(event) => setNewBranch(event.target.value)}
            />
            <button type="submit" className="btn btn-small" disabled={!newBranch.trim() || busy !== null}>
              Create and switch
            </button>
          </form>
        </div>
      )}

      {notice && (
        <p className={`git-notice git-notice--${notice.kind}`} role={notice.kind === "fail" ? "alert" : "status"}>
          {notice.text}
        </p>
      )}

      <div className="git-work__lists">
        <SideList
          heading="Staged"
          label="Staged files"
          files={staged}
          verb="Unstage"
          allVerb="Unstage all"
          busy={busy !== null}
          selected={selected?.staged ? selected.path : null}
          onVerb={path => void workingAction(`unstage:${staged.findIndex(f => f.path === path)}`)}
          onAll={() => void workingAction("unstage:all")}
          onPick={path => void workingAction(`pick:${sideKey(path, true)}`)}
          onOpen={onOpenFile ? path => void workingAction(`open:${sideKey(path, true)}`) : undefined}
        />
        <SideList
          heading="Changes"
          label="Unstaged files"
          files={unstaged}
          verb="Stage"
          allVerb="Stage all"
          busy={busy !== null}
          selected={selected && !selected.staged ? selected.path : null}
          onVerb={path => void workingAction(`stage:${unstaged.findIndex(f => f.path === path)}`)}
          onAll={() => void workingAction("stage:all")}
          onPick={path => void workingAction(`pick:${sideKey(path, false)}`)}
          onOpen={onOpenFile ? path => void workingAction(`open:${sideKey(path, false)}`) : undefined}
          confirmDiscard={confirmDiscard}
          onDiscard={path => void workingAction(`discard:${unstaged.findIndex(f => f.path === path)}`)}
        />
      </div>

      <form className="git-commit-form" onSubmit={event => { event.preventDefault(); void workingAction("commit"); }}>
        <textarea
          className="git-commit-form__message"
          aria-label="Commit message"
          placeholder={amend ? "New message for the last commit" : "What changed, and why"}
          value={message}
          rows={3}
          disabled={busy !== null}
          onChange={(event) => setMessage(event.target.value)}
        />
        <div className="git-commit-form__row">
          <label className="git-commit-form__amend">
            <input type="checkbox" checked={amend} disabled={busy !== null} onChange={event => void workingAction("amend", { enabled: event.target.checked })} />{" "}
            Amend last commit
          </label>
          <button
            type="submit"
            className="btn btn-primary btn-small"
            disabled={busy !== null || !message.trim() || (!amend && staged.length === 0)}
            data-tip={
              !amend && staged.length === 0 ? "Stage something first" : amend ? "git commit --amend" : "git commit"
            }
          >
            {amend ? "Amend" : `Commit${staged.length > 0 ? ` ${staged.length}` : ""}`}
          </button>
        </div>
      </form>

      {selected && diff && (
        <button className="btn" onClick={() => void workingAction("review")}>Review in the main editor</button>
      )}
      {selected && diff && (
        <DiffView path={diff.path} diff={diff.diff} binary={diff.binary} staged={selected.staged} />
      )}
    </section>
  );
}

</storycode:copy>
<a id="f-f7cd01998027-SideList"></a>
### `SideList`

<storycode:copy id="f-f7cd01998027-SideList">
function SideList({
  heading,
  label,
  files,
  verb,
  allVerb,
  busy,
  selected,
  onVerb,
  onAll,
  onPick,
  onOpen,
  confirmDiscard,
  onDiscard,
}: {
  heading: string;
  /** The accessible name, distinct from the heading a reader sees. */
  label: string;
  files: SideFile[];
  verb: string;
  allVerb: string;
  busy: boolean;
  selected: string | null;
  onVerb: (path: string) => void;
  onAll: () => void;
  onPick: (path: string) => void;
  onOpen?: (path: string) => void;
  confirmDiscard?: string | null;
  onDiscard?: (path: string) => void;
}) {
  return (
    <section className="git-side" aria-label={label}>
      <header className="git-side__head">
        <span className="git-side__title">
          {heading} <span className="muted">{files.length}</span>
        </span>
        {files.length > 0 && (
          <button type="button" className="btn-link" disabled={busy} onClick={onAll}>
            {allVerb}
          </button>
        )}
      </header>
      {files.length === 0 ? (
        <p className="muted git-side__empty">Nothing here.</p>
      ) : (
        <ul className="git-files git-side__files">
          {files.map((file) => (
            <li key={file.path} className={`git-side__row${selected === file.path ? " is-selected" : ""}`}>
              <button
                type="button"
                className="git-file"
                onClick={() => onPick(file.path)}
                onDoubleClick={() => onOpen?.(file.path)}
                data-tip={`Show the diff of ${file.path}; double-click opens it`}
              >
                <span className={`git-file__status git-file__status--${file.status}`}>{file.status}</span>
                <span className="git-file__path mono">
                  {file.from && <span className="git-file__from">{file.from} → </span>}
                  {file.path}
                </span>
              </button>
              <span className="git-side__verbs">
                {onDiscard && (
                  <button
                    type="button"
                    className={`btn-link git-side__discard${confirmDiscard === file.path ? " is-armed" : ""}`}
                    disabled={busy}
                    onClick={() => onDiscard(file.path)}
                    data-tip="Throw these changes away: a tracked file goes back to what is staged, an untracked one is deleted"
                  >
                    {confirmDiscard === file.path ? "Discard?" : "Discard"}
                  </button>
                )}
                <button type="button" className="btn-link" disabled={busy} onClick={() => onVerb(file.path)}>
                  {verb}
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
</storycode:copy>

### Assemble `apps/web/src/views/GitPane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/GitPane.tsx"><storycode:paste select="#f-f7cd01998027-ROW"/><storycode:paste select="#f-f7cd01998027-LANE"/><storycode:paste select="#f-f7cd01998027-EXPANDED_EXTRA"/><storycode:paste select="#f-f7cd01998027-GitPane"/><storycode:paste select="#f-f7cd01998027-x"/><storycode:paste select="#f-f7cd01998027-edgePath"/><storycode:paste select="#f-f7cd01998027-when"/><storycode:paste select="#f-f7cd01998027-Churn"/><storycode:paste select="#f-f7cd01998027-FileChurn"/><storycode:paste select="#f-f7cd01998027-Notice"/><storycode:paste select="#f-f7cd01998027-SideFile"/><storycode:paste select="#f-f7cd01998027-sides"/><storycode:paste select="#f-f7cd01998027-WorkingTree"/><storycode:paste select="#f-f7cd01998027-SideList"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-0e5613784fdf"></a>
## `apps/web/src/views/GitPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [The Git Pane Does The Daily Loop](../../../../../guarantees/collaboration/the-git-pane-does-the-daily-loop/contract.md), [The Publication Floor Is Computed On Every Read And Shown](../../../../../guarantees/collaboration/the-publication-floor-is-computed-and-shown/contract.md).

<a id="f-0e5613784fdf-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import { GitPane, sides, when } from "./GitPane";
import { bisects } from "../api/representations";
import { api } from "../api/client";
import type { GitChanges, GitCommit, GitLog } from "../api/types";

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(bisects, "list").mockResolvedValue({sessions:[]});
  // The working tree is read beside the log; the history tests are about the
  // graph, so they see a repository with nothing to commit.
  vi.spyOn(api, "gitChanges").mockResolvedValue({
    repository: true,
    branch: "master",
    upstream: null,
    ahead: 0,
    behind: 0,
    files: [],
  });
});
</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-2">
afterEach(() => {
  cleanup();
});

</storycode:copy>
<a id="f-0e5613784fdf-commit"></a>
### `commit`

<storycode:copy id="f-0e5613784fdf-commit">
const commit = (over: Partial<GitCommit> = {}): GitCommit => ({
  sha: "abc123def",
  short: "abc123d",
  parents: ["parent"],
  author: "Ada Lovelace",
  email: "ada@example.com",
  time: Math.floor(Date.now() / 1000) - 86_400,
  subject: "Add the thing",
  body: "",
  refs: [],
  files: [{ path: "src/main.rs", status: "M", added: 12, removed: 3 }],
  added: 12,
  removed: 3,
  ...over,
});

</storycode:copy>
<a id="f-0e5613784fdf-serve"></a>
### `serve`

<storycode:copy id="f-0e5613784fdf-serve">
const serve = (log: Partial<GitLog>) =>
  vi.spyOn(api, "gitLog").mockResolvedValue({
    repository: true,
    commits: [commit()],
    floor: null,
    ...log,
  });

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-3">
describe("a folder with no repository", () => {
  it("says so plainly, because notes folders often are not one", () => {
    serve({ repository: false, commits: [] });
    render(<GitPane />);
    return waitFor(() =>
      expect(screen.getByText(/not a git repository/i)).toBeTruthy(),
    );
  });

  it("says what to run if you want one", () => {
    serve({ repository: false, commits: [] });
    render(<GitPane />);
    return waitFor(() => expect(screen.getByText("git init")).toBeTruthy());
  });
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-4">
describe("what a commit's node says without being opened", () => {
  it("carries the subject, the author, the hash and the churn", async () => {
    serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    expect(screen.getByText("Ada Lovelace")).toBeTruthy();
    expect(screen.getByText("abc123d")).toBeTruthy();
    expect(screen.getByText("+12")).toBeTruthy();
    expect(screen.getByText("−3")).toBeTruthy();
  });

  it("shows the branch and tag names pointing at it", async () => {
    serve({ commits: [commit({ refs: ["master", "v1.0"] })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("master")).toBeTruthy());
    expect(screen.getByText("v1.0")).toBeTruthy();
  });

  it("draws a node per commit in the graph", async () => {
    serve({ commits: [commit({ sha: "a", parents: ["b"] }), commit({ sha: "b", parents: [] })] });
    render(<GitPane />);
    await waitFor(() =>
      expect(document.querySelectorAll(".git-node")).toHaveLength(2),
    );
  });

  it("marks a merge differently from an ordinary commit", async () => {
    serve({ commits: [commit({ parents: ["a", "b"] })] });
    render(<GitPane />);
    await waitFor(() =>
      expect(document.querySelector(".git-node--merge")).toBeTruthy(),
    );
  });
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-5">
describe("expanding a commit", () => {
  it("shows its files with no second request", async () => {
    // The files came with the log — one git invocation for everything — so
    // there is no spinner and no moment where the row is open and empty.
    const gitLog = serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    expect(screen.getByText("src/main.rs")).toBeTruthy();
    expect(gitLog).toHaveBeenCalledTimes(1);
  });

  it("shows a rename as one change with both names", async () => {
    serve({
      commits: [
        commit({ files: [{ path: "new.rs", from: "old.rs", status: "R", added: 1, removed: 1 }] }),
      ],
    });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    const row = screen.getByText("new.rs").closest("button")!;
    expect(within(row).getByText(/old\.rs/)).toBeTruthy();
    expect(within(row).getByText("R")).toBeTruthy();
  });

  it("says 'binary' rather than showing zeroes for a binary file", async () => {
    // Zero would claim the change touched nothing.
    serve({ commits: [commit({ files: [{ path: "logo.png", status: "M" }] })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    expect(screen.getByText("binary")).toBeTruthy();
  });

  it("explains an empty merge rather than showing nothing", async () => {
    serve({ commits: [commit({ parents: ["a", "b"], files: [], added: 0, removed: 0 })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    expect(screen.getByText(/changes are in the commits it joins/i)).toBeTruthy();
  });

  it("opens a file when its row is clicked", async () => {
    const onOpenFile = vi.fn();
    serve({});
    render(<GitPane onOpenFile={onOpenFile} />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    fireEvent.click(screen.getByText("src/main.rs").closest("button")!);
    expect(onOpenFile).toHaveBeenCalledWith("src/main.rs");
  });

  it("closes again", async () => {
    serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    const summary = screen.getByRole("button", { expanded: false, name: /Add the thing/ });
    fireEvent.click(summary);
    expect(screen.queryByText("src/main.rs")).toBeTruthy();
    fireEvent.click(summary);
    expect(screen.queryByText("src/main.rs")).toBeNull();
  });
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-6">
describe("how a commit's age reads", () => {
  const now = Date.UTC(2026, 7, 21);
  const at = (days: number) => Math.floor((now - days * 86_400_000) / 1000);

  it("is relative while relative means something", () => {
    expect(when(at(0), now)).toBe("today");
    expect(when(at(1), now)).toBe("yesterday");
    expect(when(at(5), now)).toBe("5d ago");
    expect(when(at(60), now)).toBe("2mo ago");
  });

  it("becomes a year past that", () => {
    expect(when(at(500), now)).toBe("2025");
  });
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-7">
// docs/guarantees/collaboration/the-publication-floor-is-computed-and-shown.md
describe("the publication floor", () => {
  it("marks the commits above it as drafts and says why", () => {
    serve({
      commits: [
        commit({ sha: "aaa", short: "aaa", subject: "still a draft", draft: true }),
        commit({ sha: "bbb", short: "bbb", subject: "published", parents: ["ccc"] }),
      ],
      floor: {
        published_ref: "origin/master",
        sha: "bbb",
        source: "upstream",
        drafts: ["aaa"],
        summary:
          "1 commit(s) above the floor at origin/master: still drafts, because nobody else can be holding them yet. Everything below is a record.",
      },
    });
    render(<GitPane />);
    return waitFor(() => {
      expect(screen.getByText(/1 draft$/)).toBeTruthy();
      expect(screen.getByText(/nobody else can be holding them/)).toBeTruthy();
      // One badge, on the one commit above the floor.
      expect(screen.getAllByText("draft")).toHaveLength(1);
    });
  });

  it("says nothing where the repository has published nothing to compare against", () => {
    // A folder with no remote has published nothing; a floor bar claiming a
    // line exists would be an invented fact.
    serve({ floor: null });
    render(<GitPane />);
    return waitFor(() => expect(screen.queryByText(/above the floor/)).toBeNull());
  });
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-8"></a>
### `expression_statement`

Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md

<storycode:copy id="f-0e5613784fdf-expression_statement-8">
// Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md
describe("the working tree", () => {
  const serveChanges = (over: Partial<GitChanges> = {}) =>
    vi.spyOn(api, "gitChanges").mockResolvedValue({
      repository: true,
      branch: "feature",
      upstream: "origin/feature",
      ahead: 2,
      behind: 0,
      files: [
        { path: "src/main.rs", index: "M", tree: " " },
        { path: "README.md", index: " ", tree: "M" },
        { path: "notes.md", index: "?", tree: "?" },
      ],
      ...over,
    });

  it("splits git's two columns into the two lists a person stages between", () => {
    const { staged, unstaged } = sides([
      { path: "both.rs", index: "M", tree: "M" },
      { path: "new.md", index: "?", tree: "?" },
      { path: "moved.rs", from: "old.rs", index: "R", tree: " " },
    ]);
    expect(staged.map((f) => f.path)).toEqual(["both.rs", "moved.rs"]);
    expect(unstaged.map((f) => f.path)).toEqual(["both.rs", "new.md"]);
    // An untracked file reads as an addition, not as a question mark.
    expect(unstaged[1].status).toBe("A");
    expect(staged[1].from).toBe("old.rs");
  });

  it("shows the branch, how far ahead it is, and the files on each side", async () => {
    serve({});
    serveChanges();
    render(<GitPane />);
    const changes = await screen.findByRole("region", { name: "Working tree" });
    expect(within(changes).getByText("feature")).toBeTruthy();
    expect(within(changes).getByText("↑2")).toBeTruthy();
    const staged = within(changes).getByRole("region", { name: "Staged files" });
    expect(within(staged).getByText("src/main.rs")).toBeTruthy();
    const unstaged = within(changes).getByRole("region", { name: "Unstaged files" });
    expect(within(unstaged).getByText("README.md")).toBeTruthy();
    expect(within(unstaged).getByText("notes.md")).toBeTruthy();
  });

  it("stages a file with one click and re-reads the repository", async () => {
    serve({});
    const changes = serveChanges();
    const stage = vi.spyOn(api, "gitStage").mockResolvedValue({ ok: true });
    render(<GitPane />);
    const unstaged = await screen.findByRole("region", { name: "Unstaged files" });
    const row = within(unstaged).getByText("README.md").closest("li")!;
    fireEvent.click(within(row).getByRole("button", { name: "Stage" }));
    await waitFor(() => expect(stage).toHaveBeenCalledWith({ paths: ["README.md"] }));
    await waitFor(() => expect(changes.mock.calls.length).toBeGreaterThan(1));
  });

  it("shows a file's diff when its row is clicked", async () => {
    serve({});
    serveChanges();
    vi.spyOn(api, "gitDiff").mockResolvedValue({
      path: "README.md",
      diff: "@@ -1 +1,2 @@\n one\n+two\n",
      binary: false,
    });
    render(<GitPane />);
    const unstaged = await screen.findByRole("region", { name: "Unstaged files" });
    fireEvent.click(within(unstaged).getByText("README.md"));
    await waitFor(() => expect(screen.getByRole("region", { name: "Diff of README.md" })).toBeTruthy());
    expect(document.querySelectorAll(".diff-line--add")).toHaveLength(1);
  });

  it("commits what is staged with the message typed, and nothing without one", async () => {
    serve({});
    serveChanges();
    const commit = vi.spyOn(api, "gitCommit").mockResolvedValue({ sha: "s", short: "s", subject: "Do it" });
    render(<GitPane />);
    await screen.findByRole("region", { name: "Working tree" });
    const button = screen.getByRole("button", { name: /^Commit/ });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(screen.getByLabelText("Commit message"), { target: { value: "Do it" } });
    expect((button as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(button);
    await waitFor(() => expect(commit).toHaveBeenCalledWith("Do it", false));
    await waitFor(() => expect(screen.getByRole("status").textContent).toMatch(/Committed/));
  });

  it("asks twice before discarding, and shows git's own words when it refuses", async () => {
    serve({});
    serveChanges();
    const discard = vi
      .spyOn(api, "gitDiscard")
      .mockRejectedValue(new Error("git checkout refused: pathspec 'README.md' did not match"));
    render(<GitPane />);
    const unstaged = await screen.findByRole("region", { name: "Unstaged files" });
    const row = within(unstaged).getByText("README.md").closest("li")!;
    const button = within(row).getByRole("button", { name: "Discard" });
    fireEvent.click(button);
    expect(discard).not.toHaveBeenCalled();
    expect(within(row).getByRole("button", { name: "Discard?" })).toBeTruthy();
    fireEvent.click(within(row).getByRole("button", { name: "Discard?" }));
    await waitFor(() => expect(discard).toHaveBeenCalledWith(["README.md"]));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/pathspec/));
  });

  it("switches branches from the branch menu", async () => {
    serve({});
    serveChanges();
    vi.spyOn(api, "gitBranches").mockResolvedValue({
      branches: [
        { name: "feature", upstream: "origin/feature", current: true },
        { name: "master", upstream: "origin/master", current: false },
      ],
    });
    const checkout = vi.spyOn(api, "gitCheckout").mockResolvedValue({ ok: true, branch: "master" });
    render(<GitPane />);
    await screen.findByRole("region", { name: "Working tree" });
    fireEvent.click(screen.getByRole("button", { name: /feature/ }));
    const menu = await screen.findByRole("group", { name: "Branches" });
    fireEvent.click(within(menu).getByRole("button", { name: /master/ }));
    await waitFor(() => expect(checkout).toHaveBeenCalledWith("master"));
  });

  it("offers no verbs for a folder that is not a repository", async () => {
    serve({ repository: false, commits: [] });
    vi.spyOn(api, "gitChanges").mockResolvedValue({ repository: false, files: [] });
    render(<GitPane />);
    await screen.findByText(/not a git repository/i);
    expect(screen.queryByRole("region", { name: "Working tree" })).toBeNull();
  });
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-9">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains the commit draft when the host refuses the commit", async () => {
  serve({});
  vi.spyOn(api, "gitChanges").mockResolvedValue({ repository: true, branch: "master", files: [{ path: "a.py", index: "M", tree: " " }] });
  vi.spyOn(api, "gitCommit").mockRejectedValue(new Error("hook refused"));
  render(<GitPane />);
  await screen.findByRole("button", { name: "Commit 1" });
  fireEvent.change(screen.getByRole("textbox", { name: "Commit message" }), { target: { value: "keep this message" } });
  fireEvent.click(screen.getByRole("button", { name: "Commit 1" }));
  await screen.findByText("hook refused");
  expect((screen.getByRole("textbox", { name: "Commit message" }) as HTMLTextAreaElement).value).toBe("keep this message");
});

</storycode:copy>
<a id="f-0e5613784fdf-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-0e5613784fdf-expression_statement-10">
it("lets an agent stage and inspect actual replies without impersonating discard or publication confirmation", async () => {
  serve({});
  vi.spyOn(api, "gitChanges").mockResolvedValue({ repository: true, branch: "master", files: [{ path: "a.py", index: " ", tree: "M" }] });
  const stage = vi.spyOn(api, "gitStage").mockResolvedValue({ ok: true, said: "staged by host" });
  const discard = vi.spyOn(api, "gitDiscard").mockResolvedValue({ ok: true });
  const push = vi.spyOn(api, "gitPush").mockResolvedValue({ ok: true });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><GitPane /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "Stage all" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  expect(await run("Discard Git changes a.py")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(await run("Push Git branch")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(discard).not.toHaveBeenCalled(); expect(push).not.toHaveBeenCalled();
  const result = await run("Stage Git file a.py");
  expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { reply: JSON.stringify({ ok: true, said: "staged by host" }) } } });
  expect(stage).toHaveBeenCalledWith({ paths: ["a.py"] });
});
</storycode:copy>

### Assemble `apps/web/src/views/GitPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/GitPane.test.tsx"><storycode:paste select="#f-0e5613784fdf-expression_statement"/><storycode:paste select="#f-0e5613784fdf-expression_statement-2"/><storycode:paste select="#f-0e5613784fdf-commit"/><storycode:paste select="#f-0e5613784fdf-serve"/><storycode:paste select="#f-0e5613784fdf-expression_statement-3"/><storycode:paste select="#f-0e5613784fdf-expression_statement-4"/><storycode:paste select="#f-0e5613784fdf-expression_statement-5"/><storycode:paste select="#f-0e5613784fdf-expression_statement-6"/><storycode:paste select="#f-0e5613784fdf-expression_statement-7"/><storycode:paste select="#f-0e5613784fdf-expression_statement-8"/><storycode:paste select="#f-0e5613784fdf-expression_statement-9"/><storycode:paste select="#f-0e5613784fdf-expression_statement-10"/></storycode:file>


</storycode:doc>
