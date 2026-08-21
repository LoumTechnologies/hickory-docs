// The merge, as something to read and answer.
//
// It opens whenever two versions of one file both have changes worth keeping:
// a draft restored against a file that moved on while the app was closed, and
// — once anything else needs it — any other three-way situation, because the
// shape is the same one every time.
//
// The design rule here is that the reader should be able to see how much of
// this is actually their problem before they read a word of it. Most of a
// merge is not: regions only one side changed are already decided, and saying
// so plainly ("11 changes merged, 2 need you") is the difference between a
// merge someone answers and a merge someone abandons.
//
// So:
//  - Agreed text is shown, quietly, in place, because context is what makes a
//    conflict answerable.
//  - Auto-resolved regions are shown and LABELLED with who they came from —
//    not hidden. A merge tool that silently took a side is a merge tool
//    nobody trusts twice.
//  - Conflicts are the only thing that looks like a question.
//
// The merged text is live: it updates as answers are given, and it is what
// lands in the buffer when the reader accepts. Until then the unanswered
// conflicts read as OUR text (see `mergedText`), so the reader's own unsaved
// work never disappears off the screen while they think.

import { useMemo, useState } from "react";

import {
  conflictCount,
  mergeThreeWay,
  mergeTwoWay,
  mergedText,
  type MergeRegion,
  type Resolution,
} from "../lib/merge";

export interface MergeViewProps {
  /** What is being merged, for the heading. */
  path: string;
  /** The common ancestor. Empty string means there is none, and the merge
   * falls back to a two-way comparison that has to ask about everything. */
  base: string;
  /** Our side — in the restore case, the reader's own unsaved buffer. */
  ours: string;
  /** Their side — in the restore case, the file as it is on disk now. */
  theirs: string;
  /** What each side is called, in this reader's terms. */
  oursLabel?: string;
  theirsLabel?: string;
  /** Take the merged text. */
  onAccept: (text: string) => void;
  /** Walk away and keep `ours` untouched. */
  onCancel: () => void;
}

const CHOICES: { value: Resolution; label: string; hint: string }[] = [
  { value: "ours", label: "Keep mine", hint: "Take this side and drop theirs" },
  { value: "theirs", label: "Keep theirs", hint: "Take their side and drop mine" },
  { value: "both", label: "Keep both", hint: "Mine first, then theirs" },
  { value: "base", label: "Keep neither", hint: "Go back to what was there before" },
];

export function MergeView({
  path,
  base,
  ours,
  theirs,
  oursLabel = "Your unsaved changes",
  theirsLabel = "The file on disk",
  onAccept,
  onCancel,
}: MergeViewProps) {
  // A base of "" is genuinely "no common ancestor", not "the ancestor was an
  // empty file" — a draft with no file behind it stores it that way. The
  // difference matters: three-way over an empty base would call every line of
  // both files an addition and merge them into one another.
  const threeWay = base.length > 0;
  const regions = useMemo(
    () => (threeWay ? mergeThreeWay(base, ours, theirs) : mergeTwoWay(ours, theirs)),
    [threeWay, base, ours, theirs],
  );
  const [answers, setAnswers] = useState<ReadonlyMap<number, Resolution>>(new Map());

  const total = conflictCount(regions);
  const auto = regions.filter((r) => r.kind === "resolved").length;
  const answered = [...answers.keys()].length;
  const text = mergedText(regions, answers);

  const answer = (index: number, choice: Resolution) => {
    setAnswers((current) => {
      const next = new Map(current);
      // Choosing the same answer again clears it, which is the only way back
      // to "I have not decided" without reopening the whole merge.
      if (next.get(index) === choice) next.delete(index);
      else next.set(index, choice);
      return next;
    });
  };

  let conflictIndex = -1;

  return (
    <section className="merge-view" data-testid="merge-view" aria-label={`Merge ${path}`}>
      <header className="merge-view__head">
        <h2 className="merge-view__title">
          <span className="mono">{path}</span> changed while you were away
        </h2>
        {/* The first thing the reader needs is how much of this is their
            problem. */}
        <p className="merge-view__summary" role="status">
          {total === 0
            ? `Merged cleanly — ${auto} ${auto === 1 ? "change" : "changes"} from both sides, nothing to decide.`
            : `${auto} ${auto === 1 ? "change" : "changes"} merged on their own. ${total - answered} of ${total} still ${total - answered === 1 ? "needs" : "need"} you.`}
        </p>
        {!threeWay && (
          <p className="merge-view__note">
            There is no common ancestor for these two, so every difference is a
            question — nothing can tell an addition on one side from a deletion
            on the other.
          </p>
        )}
      </header>

      <div className="merge-view__regions">
        {regions.map((region, i) => {
          if (region.kind === "conflict") conflictIndex++;
          return (
            <MergeRegionRow
              key={i}
              region={region}
              conflictIndex={region.kind === "conflict" ? conflictIndex : -1}
              chosen={region.kind === "conflict" ? answers.get(conflictIndex) : undefined}
              threeWay={threeWay}
              oursLabel={oursLabel}
              theirsLabel={theirsLabel}
              onAnswer={answer}
            />
          );
        })}
      </div>

      <footer className="merge-view__actions">
        <button type="button" className="btn" onClick={onCancel}>
          Cancel
        </button>
        <button
          type="button"
          className="btn btn-primary"
          onClick={() => onAccept(text)}
          // Deliberately NOT disabled while conflicts are unanswered. An
          // unanswered conflict already reads as our text, which is what the
          // buffer holds right now — accepting is a real, meaningful choice
          // ("everything I have not answered stays mine") and blocking it
          // would trap someone in a dialog over a region they do not care
          // about.
          data-tip={
            answered < total
              ? "Unanswered conflicts keep your side"
              : "Put the merged text in the editor"
          }
        >
          {answered < total ? "Accept, keeping mine where undecided" : "Accept merge"}
        </button>
      </footer>
    </section>
  );
}

function MergeRegionRow({
  region,
  conflictIndex,
  chosen,
  threeWay,
  oursLabel,
  theirsLabel,
  onAnswer,
}: {
  region: MergeRegion;
  conflictIndex: number;
  chosen: Resolution | undefined;
  threeWay: boolean;
  oursLabel: string;
  theirsLabel: string;
  onAnswer: (index: number, choice: Resolution) => void;
}) {
  if (region.kind === "stable") {
    if (!region.text) return null;
    return (
      <pre className="merge-region merge-region--stable">
        <code>{region.text}</code>
      </pre>
    );
  }

  if (region.kind === "resolved") {
    // Shown, not hidden. A merge tool that silently took a side is a merge
    // tool nobody trusts twice.
    const who =
      region.from === "both"
        ? "both sides made this same change"
        : region.from === "ours"
          ? `only ${oursLabel.toLowerCase()} touched this`
          : `only ${theirsLabel.toLowerCase()} changed this`;
    return (
      <div className="merge-region merge-region--resolved">
        <p className="merge-region__label">Merged — {who}</p>
        <pre>
          <code>{region.text}</code>
        </pre>
      </div>
    );
  }

  const choices = threeWay ? CHOICES : CHOICES.filter((c) => c.value !== "base");
  return (
    <div
      className={`merge-region merge-region--conflict${chosen ? " merge-region--answered" : ""}`}
      role="group"
      aria-label={`Conflict ${conflictIndex + 1}`}
    >
      <div className="merge-region__sides">
        <div
          className={`merge-side merge-side--ours${chosen === "ours" || chosen === "both" ? " merge-side--taken" : ""}`}
        >
          <p className="merge-region__label">{oursLabel}</p>
          <pre>
            <code>{region.ours || <em className="muted">(nothing)</em>}</code>
          </pre>
        </div>
        <div
          className={`merge-side merge-side--theirs${chosen === "theirs" || chosen === "both" ? " merge-side--taken" : ""}`}
        >
          <p className="merge-region__label">{theirsLabel}</p>
          <pre>
            <code>{region.theirs || <em className="muted">(nothing)</em>}</code>
          </pre>
        </div>
      </div>
      <div className="merge-region__choices" role="toolbar">
        {choices.map((choice) => (
          <button
            key={choice.value}
            type="button"
            className={`btn btn-small${chosen === choice.value ? " btn-primary" : ""}`}
            aria-pressed={chosen === choice.value}
            data-tip={choice.hint}
            onClick={() => onAnswer(conflictIndex, choice.value)}
          >
            {choice.label}
          </button>
        ))}
      </div>
    </div>
  );
}
