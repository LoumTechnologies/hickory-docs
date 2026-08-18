// Every session in the window, by project — the answer to "which terminal
// needs me?" without opening any of them.
//
// A row is a task, not a number: sessions are named, carry their branch, and
// show the last line they printed. Folding a project keeps its strongest
// claim visible (see lib/sessionGroups), so nothing hides by being tidy.

import { useState } from "react";

import type { TerminalSession } from "../api/types";
import { groupSessions } from "../lib/sessionGroups";

const STATE_LABEL: Record<TerminalSession["state"], string> = {
  "needs-you": "needs you",
  working: "working",
  idle: "idle",
  finished: "finished",
  failed: "failed",
};

export function SessionsPane({
  sessions,
  turbo,
  error,
  activeId,
  onOpen,
  onClose,
  onNew,
  onNewWorktree,
  onSetTurbo,
}: {
  sessions: readonly TerminalSession[];
  turbo: boolean;
  error: string | null;
  activeId?: string | null;
  onOpen: (id: string) => void;
  onClose: (id: string) => void;
  onNew: (monitor: boolean) => void;
  onNewWorktree: (branch: string) => void;
  onSetTurbo: (enabled: boolean) => void;
}) {
  const [folded, setFolded] = useState<ReadonlySet<string>>(new Set());
  // The worktree's branch is asked for inline rather than in a dialog: the
  // name IS the session's name, so typing it is part of starting the work.
  const [branch, setBranch] = useState<string | null>(null);
  const groups = groupSessions(sessions);

  return (
    <div className="sessions-pane">
      <div className="sessions-actions" role="toolbar" aria-label="Terminals">
        <button className="btn" onClick={() => onNew(false)} data-tip="Open a terminal here">
          New terminal
        </button>
        <button
          className="btn"
          onClick={() => setBranch((open) => (open === null ? "" : null))}
          data-tip="Open a terminal in a fresh git worktree, on its own branch"
        >
          New worktree
        </button>
        <button
          className="btn"
          onClick={() => onNew(true)}
          data-tip="Run a dev server or watcher in the dock, where it will not interrupt you"
        >
          New monitor
        </button>
        <label className="turbo-toggle" data-tip="Answer routine declared prompts automatically. Never answers a guessed one.">
          <input type="checkbox" checked={turbo} onChange={(e) => onSetTurbo(e.target.checked)} />
          Turbo
        </label>
      </div>

      {branch !== null && (
        <form
          className="worktree-form"
          onSubmit={(e) => {
            e.preventDefault();
            if (branch.trim() === "") return;
            onNewWorktree(branch.trim());
            setBranch(null);
          }}
        >
          <label htmlFor="worktree-branch">Branch</label>
          <input
            id="worktree-branch"
            value={branch}
            autoFocus
            placeholder="fix-the-parser"
            onChange={(e) => setBranch(e.target.value)}
          />
          <button className="btn btn-primary" type="submit">
            Create
          </button>
        </form>
      )}

      {error && <p className="error">{error}</p>}

      {groups.length === 0 && !error && (
        <p className="muted">No terminals yet. Open one to start a piece of work.</p>
      )}

      {groups.map((group) => {
        const isFolded = folded.has(group.cwd);
        return (
          <section className="session-group" key={group.cwd}>
            <button
              className="session-group-head"
              aria-expanded={!isFolded}
              onClick={() =>
                setFolded((current) => {
                  const next = new Set(current);
                  if (!next.delete(group.cwd)) next.add(group.cwd);
                  return next;
                })
              }
            >
              <span className={`state-dot state-${group.state}`} aria-hidden="true" />
              <strong>{group.name}</strong>
              <span className="muted">{group.sessions.length}</span>
              {/* The claim survives folding — that is the point of it. */}
              {isFolded && <span className="session-state">{STATE_LABEL[group.state]}</span>}
            </button>

            {!isFolded &&
              group.sessions.map((session) => (
                <div
                  className={`session-row${session.id === activeId ? " session-row-active" : ""}`}
                  key={session.id}
                >
                  <button className="session-open" onClick={() => onOpen(session.id)}>
                    <span className={`state-dot state-${session.state}`} aria-hidden="true" />
                    <span className="session-title">{session.title}</span>
                    {session.branch && <code className="session-branch">{session.branch}</code>}
                    {session.dirty && (
                      <span className="session-dirty" data-tip="Uncommitted changes">
                        •
                      </span>
                    )}
                    <span className="session-preview" data-tip={session.preview}>
                      {session.preview}
                    </span>
                  </button>
                  <button
                    className="session-close"
                    aria-label={`Close ${session.title}`}
                    onClick={() => onClose(session.id)}
                  >
                    ×
                  </button>
                </div>
              ))}
          </section>
        );
      })}
    </div>
  );
}
