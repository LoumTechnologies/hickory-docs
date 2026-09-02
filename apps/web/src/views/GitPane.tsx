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

import { useCallback, useEffect, useMemo, useState } from "react";

import { api } from "../api/client";
import type { GitBranch, GitChangeFile, GitChanges, GitDiff, GitFileChange, GitLog } from "../api/types";
import { graphWidth, laneColor, layout } from "../lib/gitGraph";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { DiffView } from "../components/DiffView";

/** Row height and lane spacing, in px. Shared by the SVG and the list, which
 * is the only way the nodes line up with the text beside them. */
const ROW = 44;
const LANE = 14;
const EXPANDED_EXTRA = 0;

export function GitPane({ onOpenFile }: { onOpenFile?: (path: string) => void }) {
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
      (answer) => live && setLog(answer),
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
      <WorkingTree tick={tick} refresh={refresh} onOpenFile={onOpenFile} head={log.commits[0]} />
      {/* The publication floor, stated rather than felt. Below it commits are
          records — someone else may be holding them; above it they are
          drafts, and re-emission may replace them. Merging moves the line,
          which is why it is computed on every read and never stored.
          docs/specs/freeform/expression-and-log.md */}
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
                  onClick={() =>
                    setOpen((current) => {
                      const next = new Set(current);
                      if (!next.delete(commit.sha)) next.add(commit.sha);
                      return next;
                    })
                  }
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
                  {commit.body && <pre className="git-commit__body">{commit.body}</pre>}
                  {commit.files.length === 0 ? (
                    <p className="muted git-commit__nofiles">
                      {commit.parents.length > 1
                        ? "A merge — its changes are in the commits it joins."
                        : "No files changed."}
                    </p>
                  ) : (
                    <ul className="git-files">
                      {commit.files.map((file) => (
                        <li key={file.path}>
                          <button
                            type="button"
                            className="git-file"
                            onClick={() => onOpenFile?.(file.path)}
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
          onClick={() => setLimit((n) => n + 120)}
        >
          Show older commits
        </button>
      )}
    </div>
  );
}

/** Horizontal centre of a lane. */
function x(lane: number): number {
  return LANE * (lane + 0.5);
}

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


// ---------------------------------------------------------------------------
// The working tree, and the verbs on it
// ---------------------------------------------------------------------------

/** What a verb answered, or what git said when it refused. */
interface Notice {
  kind: "ok" | "fail";
  text: string;
}

/** A file on one side of the index, with the status letter for that side. */
interface SideFile {
  path: string;
  from?: string;
  status: string;
}

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
    ) => {
      setBusy(verb);
      setNotice(null);
      try {
        const answer = (await action()) as { said?: string } | undefined;
        const said = answer?.said?.trim();
        setNotice({ kind: "ok", text: said ? `${done} — ${said}` : done });
        refresh();
        // A checkout, a pull, a stash or a discard rewrote files on disk:
        // every open pane and the tree need to know.
        if (filesChanged) window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
      } catch (e) {
        setNotice({ kind: "fail", text: e instanceof Error ? e.message : String(e) });
      } finally {
        setBusy(null);
      }
    },
    [refresh],
  );

  if (!changes?.repository) return null;
  const { staged, unstaged } = sides(changes.files);
  const ahead = changes.ahead ?? 0;
  const behind = changes.behind ?? 0;

  const openBranches = () => {
    if (branches) {
      setBranches(null);
      return;
    }
    api.gitBranches().then(
      (answer) => setBranches(answer.branches),
      (e: unknown) => setNotice({ kind: "fail", text: e instanceof Error ? e.message : String(e) }),
    );
  };

  const commit = (event: React.FormEvent) => {
    event.preventDefault();
    const text = message.trim();
    if (!text) return;
    void run(
      "commit",
      () => api.gitCommit(text, amend),
      amend ? "Amended the last commit" : "Committed",
    ).then(() => {
      setMessage("");
      setAmend(false);
    });
  };

  const toggleAmend = (on: boolean) => {
    setAmend(on);
    // Amending with an empty message means "keep the message"; start from
    // the one the commit has, so a reword is an edit rather than a retype.
    if (on && message.trim() === "" && head) {
      setMessage(head.body ? `${head.subject}\n\n${head.body}` : head.subject);
    }
  };

  return (
    <section className="git-work" aria-label="Working tree">
      <header className="git-work__head">
        <button
          type="button"
          className="git-branch"
          aria-expanded={branches !== null}
          onClick={openBranches}
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
            onClick={() => void run("pull", () => api.gitPull(), "Pulled", true)}
            data-tip="git pull --ff-only — never a merge, never a rebase"
          >
            Pull
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null}
            onClick={() => void run("push", () => api.gitPush(), "Pushed")}
            data-tip={changes.upstream ? "git push" : "git push -u origin <branch>"}
          >
            Push{ahead > 0 ? ` ${ahead}` : ""}
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null || changes.files.length === 0}
            onClick={() => void run("stash", () => api.gitStash("push"), "Stashed", true)}
            data-tip="git stash push --include-untracked"
          >
            Stash
          </button>
          <button
            type="button"
            className="btn btn-small"
            disabled={busy !== null}
            onClick={() => void run("stash", () => api.gitStash("pop"), "Stash popped", true)}
            data-tip="git stash pop"
          >
            Pop stash
          </button>
        </span>
      </header>

      {branches && (
        <div className="git-branches" role="group" aria-label="Branches">
          <ul className="git-branches__list">
            {branches.map((branch) => (
              <li key={branch.name}>
                <button
                  type="button"
                  className={`git-branches__item${branch.current ? " is-current" : ""}`}
                  disabled={branch.current || busy !== null}
                  onClick={() =>
                    void run("checkout", () => api.gitCheckout(branch.name), `On ${branch.name}`, true).then(
                      () => setBranches(null),
                    )
                  }
                >
                  <span className="mono">{branch.name}</span>
                  {branch.upstream && <span className="muted mono"> → {branch.upstream}</span>}
                </button>
              </li>
            ))}
          </ul>
          <form
            className="git-branches__new"
            onSubmit={(event) => {
              event.preventDefault();
              const name = newBranch.trim();
              if (!name) return;
              void run("checkout", () => api.gitCheckout(name, true), `On ${name}`, true).then(() => {
                setBranches(null);
                setNewBranch("");
              });
            }}
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
          onVerb={(path) => void run("unstage", () => api.gitUnstage({ paths: [path] }), `Unstaged ${path}`)}
          onAll={() => void run("unstage", () => api.gitUnstage({ all: true }), "Unstaged everything")}
          onPick={(path) => setSelected({ path, staged: true })}
          onOpen={onOpenFile}
        />
        <SideList
          heading="Changes"
          label="Unstaged files"
          files={unstaged}
          verb="Stage"
          allVerb="Stage all"
          busy={busy !== null}
          selected={selected && !selected.staged ? selected.path : null}
          onVerb={(path) => void run("stage", () => api.gitStage({ paths: [path] }), `Staged ${path}`)}
          onAll={() => void run("stage", () => api.gitStage({ all: true }), "Staged everything")}
          onPick={(path) => setSelected({ path, staged: false })}
          onOpen={onOpenFile}
          confirmDiscard={confirmDiscard}
          onDiscard={(path) => {
            if (confirmDiscard !== path) {
              setConfirmDiscard(path);
              return;
            }
            setConfirmDiscard(null);
            void run("discard", () => api.gitDiscard([path]), `Discarded the changes to ${path}`, true);
          }}
        />
      </div>

      <form className="git-commit-form" onSubmit={commit}>
        <textarea
          className="git-commit-form__message"
          aria-label="Commit message"
          placeholder={amend ? "New message for the last commit" : "What changed, and why"}
          value={message}
          rows={3}
          onChange={(event) => setMessage(event.target.value)}
        />
        <div className="git-commit-form__row">
          <label className="git-commit-form__amend">
            <input type="checkbox" checked={amend} onChange={(event) => toggleAmend(event.target.checked)} />{" "}
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
        <DiffView path={diff.path} diff={diff.diff} binary={diff.binary} staged={selected.staged} />
      )}
    </section>
  );
}

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
