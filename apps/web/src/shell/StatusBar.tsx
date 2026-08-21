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

import { problemsLabel, type ProblemCounts } from "../lib/problems";
import { zoomLabel } from "../lib/zoom";

export interface StatusBarProps {
  problems: ProblemCounts;
  /** Terminals that want something from a person, right now. */
  needsAttention: number;
  /** The focused file's path, or null when nothing is focused. */
  path: string | null;
  /** 1-based caret position in the focused editor. */
  caret: { line: number; column: number } | null;
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
  caret,
  zoom,
  onProblems,
  onAttention,
}: StatusBarProps) {
  const clean = problems.errors === 0 && problems.warnings === 0;
  return (
    <footer className="status-bar" role="contentinfo">
      <div className="status-bar__left">
        <button
          type="button"
          className={`status-bar__item${clean ? "" : " status-bar__item--loud"}`}
          onClick={onProblems}
          data-tip={clean ? "Nothing is wrong right now" : "Go to the next problem"}
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
        {caret && (
          <span className="status-bar__item" data-tip="Line and column of the caret">
            Ln {caret.line}, Col {caret.column}
          </span>
        )}
        {path && (
          <span className="status-bar__item status-bar__path mono" data-tip={path}>
            {path}
          </span>
        )}
        {zoom !== 1 && (
          <span className="status-bar__item" data-tip="Window zoom — ⌘0 for actual size">
            {zoomLabel(zoom)}
          </span>
        )}
      </div>
    </footer>
  );
}
