import type React from "react";
// The status bar: the one line that is always true.
//
// A status bar earns its permanent row by answering questions you would
// otherwise have to go and look for, and it loses that row the moment it
// starts showing things nobody reads. So the rule here is that everything on
// it is either a COUNT that changes, or the identity of what you are looking
// at — and nothing is on it merely because there was space.
//
// Left is the state of the work: how much is wrong, and what is running.
// Right is the state of the view: where the caret is, what the file is, how
// far it is zoomed. That split is the one every editor uses, and it is worth
// keeping because it means a glance to one side answers one kind of question.

import { useEffect, useState } from "react";
import { focusedEditor } from "../editor/activeEditor";
import { problemsLabel, type ProblemCounts } from "../lib/problems";
import { zoomLabel } from "../lib/zoom";

export interface StatusBarProps {
  problems: ProblemCounts;
  /** Terminals that want something from a person, right now. */
  needsAttention: number;
  /** The branch, and how much is uncommitted. Absent for a folder that is not
   * a repository, which is entirely normal for a folder of notes. */
  git?: { branch: string; dirty: number } | null;
  /** Show the commit graph. */
  onGit?: () => void;
  /** The focused file's path, or null when nothing is focused. */
  path: string | null;
  /** Controls the workspace wants at hand on the right — the provenance
   * layer toggles. */
  extra?: React.ReactNode;
  /** Whole-window zoom. Shown only when it is not 100%: a status bar that
   * always says "100%" has spent a slot on a constant. */
  zoom: number;
  /** Go to the next problem. */
  onProblems?: () => void;
  /** Walk the attention queue — the same verb ⌘J has. */
  onAttention?: () => void;
}

export function StatusBar({
  problems,
  needsAttention,
  path,
  zoom,
  git,
  onProblems,
  onAttention,
  onGit,
  extra,
}: StatusBarProps) {
  const clean = problems.errors === 0 && problems.warnings === 0;
  return (
    <footer className="status-bar" role="contentinfo">
      <div className="status-bar__left">
        {/* Leftmost, the way every editor puts it — and only when there IS a
            branch. A permanent "no repository" would spend a slot saying
            nothing. */}
        {git && (
          <button
            type="button"
            className="status-bar__item"
            onClick={onGit}
            data-tip={
              git.dirty > 0
                ? `On ${git.branch} — ${git.dirty} file${git.dirty === 1 ? "" : "s"} changed. Show history.`
                : `On ${git.branch}, nothing uncommitted. Show history.`
            }
            aria-label={`Branch ${git.branch}${git.dirty > 0 ? `, ${git.dirty} changed` : ""}`}
          >
            <span className="status-bar__glyph" aria-hidden>
              ⑂
            </span>
            {git.branch}
            {git.dirty > 0 && (
              <span className="status-bar__dirty">{git.dirty}</span>
            )}
          </button>
        )}
        <button
          type="button"
          className={`status-bar__item${clean ? "" : " status-bar__item--loud"}`}
          onClick={onProblems}
          data-tip={
            clean ? "Nothing is wrong right now" : "Go to the next problem"
          }
          aria-label={problemsLabel(problems)}
        >
          <span className="status-bar__glyph" aria-hidden>
            ✕
          </span>
          {problems.errors}
          <span className="status-bar__glyph" aria-hidden>
            ⚠
          </span>
          {problems.warnings}
        </button>

        {/* Only when there is something to say. A permanent "0 waiting" is a
            slot spent on a constant. */}
        {needsAttention > 0 && (
          <button
            type="button"
            className="status-bar__item status-bar__item--attention"
            onClick={onAttention}
            data-tip="Go to the next terminal that needs you"
            aria-label={`${needsAttention} terminal${needsAttention === 1 ? "" : "s"} needing you`}
          >
            <span className="status-bar__glyph" aria-hidden>
              ▮
            </span>
            {needsAttention} waiting
          </button>
        )}
      </div>

      <div className="status-bar__right">
        {extra}
        <CaretPosition />
        {path && (
          <span
            className="status-bar__item status-bar__path mono"
            data-tip={path}
          >
            {path}
          </span>
        )}
        {zoom !== 1 && (
          <span
            className="status-bar__item"
            data-tip="Window zoom — ⌘0 for actual size"
          >
            {zoomLabel(zoom)}
          </span>
        )}
      </div>
    </footer>
  );
}

/**
 * Where the caret is, polled from the focused editor.
 *
 * Its own component, on purpose: this is the only thing in the window that
 * changes on every keystroke, and it is a few characters of text. Held in the
 * workspace's state it re-rendered every pane, rail and ruler in the window
 * eight times a second while you typed; held here it re-renders this span.
 * Polled on a frame rather than hooked into every editor: there is no one
 * editor to hook, panes come and go, and a status bar that is one frame
 * behind the caret is indistinguishable from one that is not.
 */
function CaretPosition() {
  const [caret, setCaret] = useState<{ line: number; column: number } | null>(
    null,
  );
  useEffect(() => {
    let frame: number | null = null;
    const read = () => {
      frame = null;
      const view = focusedEditor();
      if (!view || !view.dom.isConnected) {
        setCaret((current) => (current === null ? current : null));
        return;
      }
      const head = view.state.selection.main.head;
      const line = view.state.doc.lineAt(head);
      const next = { line: line.number, column: head - line.from + 1 };
      // Same place, same object, no render.
      setCaret((current) =>
        current && current.line === next.line && current.column === next.column
          ? current
          : next,
      );
    };
    const tick = () => {
      if (frame === null) frame = requestAnimationFrame(read);
    };
    const timer = window.setInterval(tick, 120);
    tick();
    return () => {
      window.clearInterval(timer);
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, []);
  if (!caret) return null;
  return (
    <span className="status-bar__item" data-tip="Line and column of the caret">
      Ln {caret.line}, Col {caret.column}
    </span>
  );
}
