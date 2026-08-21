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
// Read-only, deliberately. Committing, checking out and staging are things
// people rightly want to do where the exact command is visible — and this app
// has a terminal on every row of the file tree, in the directory the work is
// in. A half-built git UI that can commit but not amend, or stage but not
// stash, is worse than none: it teaches a workflow it cannot finish.

import { useEffect, useMemo, useState } from "react";

import { api } from "../api/client";
import type { GitCommit, GitFileChange } from "../api/types";
import { graphWidth, laneColor, layout } from "../lib/gitGraph";

/** Row height and lane spacing, in px. Shared by the SVG and the list, which
 * is the only way the nodes line up with the text beside them. */
const ROW = 44;
const LANE = 14;
const EXPANDED_EXTRA = 0;

export function GitPane({ onOpenFile }: { onOpenFile?: (path: string) => void }) {
  const [log, setLog] = useState<{ repository: boolean; commits: GitCommit[] } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const [limit, setLimit] = useState(120);

  useEffect(() => {
    let live = true;
    api.gitLog(limit).then(
      (answer) => live && setLog(answer),
      (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [limit]);

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

  return (
    <div className="git-pane">
      <ol className="git-log" style={{ ["--git-graph" as string]: `${graphPx}px` }}>
        {log.commits.map((commit, index) => {
          const row = rows[index];
          const expanded = open.has(commit.sha);
          return (
            <li key={commit.sha} className="git-commit">
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
