// The time slider: exact lineage at any commit.
//
// `hick lineage` answers *which span produced these bytes* exactly, and only
// for the document as it stands. Replay makes the same answer available at
// any commit, and it needs no new data model at all — each commit contains
// its document, a weave is deterministic, and weave-only checks are already
// affordable. So this is a control over `git log`, not over a store.
//
// Two things it deliberately does NOT claim:
//
//   - It gives the state *at* a commit, never the thread *between* two of
//     them. Correlating one version's spans with another's is a recorded
//     correspondence, and it is a different mechanism.
//   - Replay weaves an old document with today's binary, which asks for a
//     grammar-compatibility promise this product has not made. So the honest
//     claim is **replay works back to the last grammar change**, and the
//     boundary says so where it is reached rather than failing obscurely.
//
// See docs/specs/freeform/provenance-across-versions.md.

import { useId, useMemo } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { ReplayCommit } from "../api/types";

export interface TimeSliderProps {
  /** Newest first, as git reports them. */
  commits: readonly ReplayCommit[];
  /** The selected commit's sha, or null for "as it stands". */
  at: string | null;
  onChange: (sha: string | null) => void;
  /** Shown when the selected commit is past the grammar boundary. */
  boundary?: string | null;
  busy?: boolean;
  owner?: string;
}

/** Slider positions run oldest → newest, which is how a person reads time;
 * git reports newest first. The last position is the working tree, which is
 * not a commit and must not pretend to be one. */
export function positions(commits: readonly ReplayCommit[]): (ReplayCommit | null)[] {
  const oldestFirst: (ReplayCommit | null)[] = [...commits].reverse();
  return [...oldestFirst, null];
}

export function indexOf(commits: readonly ReplayCommit[], at: string | null): number {
  const stops = positions(commits);
  const found = stops.findIndex((c) => (c ? c.sha === at : at === null));
  return found === -1 ? stops.length - 1 : found;
}

const when = (seconds: number) => new Date(seconds * 1000).toLocaleDateString();

export function TimeSlider({ commits, at, onChange, boundary, busy, owner }: TimeSliderProps) {
  const stops = positions(commits), index = indexOf(commits, at), current = stops[index];
  const missing = at !== null && !commits.some(commit => commit.sha === at);
  const parent = `lineage-time:${useId()}`;
  const callbackRevision = useMemo(() => crypto.randomUUID(), [onChange, owner]);
  const revision = useUiRevision(() => [owner, commits, at, busy, boundary, callbackRevision]);
  const choose = (pickedIndex: number) => {
    if (!Number.isInteger(pickedIndex) || pickedIndex < 0 || pickedIndex >= stops.length) return false;
    const picked = stops[pickedIndex]; onChange(picked ? picked.sha : null); return true;
  };
  useUiCommands([
    ...(commits.length ? [{ key: `${parent}:stop`, command: { parent, subject: parent, label: "Select lineage replay stop", parameters: { index: { type: "number" as const, minimum: 0, maximum: stops.length - 1, integer: true } }, prerequisites: ["Historical replay weaves captured source with the current binary; it does not execute cells"],
      read: () => ({ revision: revision(), enabled: true }), execute: async (args: Record<string, import("../lib/uiActions").UiValue>) => { choose(args.index as number); return { status: "completed" as const, value: { selection_requested: true, index: args.index, replay_completed: false } }; } } }] : []),
    ...(at !== null ? [{ key: `${parent}:working`, command: { parent, subject: parent, label: "Show working-tree lineage", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => { choose(stops.length - 1); return { status: "completed" as const, value: { selection_requested: true, at: null, read_completed: false } }; } } }] : []),
  ]);
  useUiReading("lineage replay time", { available_commit_stops: commits.length, working_tree_index: stops.length - 1, selected_revision: at, selected_index: missing ? null : index,
    selected_commit: missing || !current ? null : { sha: current.sha, short: current.short, subject: current.subject, author: current.author, date: when(current.time) },
    missing_from_available_history: missing, busy: !!busy, boundary: boundary ?? null,
    scope: "Currently available stops and the displayed selection. Other commit subjects require selecting their stop; replay requests are not completed reads or execution." });
  const recovery = at !== null ? <button type="button" onClick={() => choose(stops.length - 1)}>Show working-tree lineage</button> : null;
  if (commits.length === 0) {
    return (
      <div>{recovery}<p className="lineage-note muted" role="status">
        No replay stops are available yet, so there is nowhere to slide to.
        Replay reads old documents out of git; it does not remember anything of
        its own.
      </p></div>
    );
  }

  return (
    <div className="time-slider">
      <label className="time-slider__control">
        <span className="time-slider__label">Replay</span>
        <input
          type="range"
          min={0}
          max={stops.length - 1}
          step={1}
          value={index}
          aria-label="Replay this project's lineage at an earlier commit"
          aria-valuetext={missing ? "Selected revision unavailable in this history" : current ? current.subject : "Working tree"}
          onChange={event => choose(Number(event.target.value))}
        />
      </label>
      {recovery}
      <p className="time-slider__stop">
        {missing ? <span>Selected revision is unavailable in this history · <code>{at}</code></span> : current === null ? (
          <>
            <strong>As it stands</strong>
            <span className="muted">
              {" "}
              — the working tree, which is not a commit.
            </span>
          </>
        ) : (
          <>
            <strong>{current.subject}</strong>
            <span className="muted">
              {" "}
              — {current.author}, {when(current.time)}{" "}
              <span className="mono" title={current.sha}>{current.short}</span>
            </span>
          </>
        )}
        {busy && <span className="muted"> · weaving…</span>}
      </p>
      {/* Not an error. The tool has reached the edge of what it can weave,
          and the document is exactly as it always was. */}
      {boundary && (
        <p className="lineage-warning time-slider__boundary" role="status">
          {boundary}
        </p>
      )}
      <p className="time-slider__caveat muted">
        Lineage here is recomputed by weaving that commit’s document — never
        executed, and never recalled from a store. It is the state <em>at</em>{" "}
        that commit; relating one version’s spans to another’s is a different
        question.
      </p>
    </div>
  );
}
