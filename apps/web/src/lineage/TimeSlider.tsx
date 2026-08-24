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

export function TimeSlider({ commits, at, onChange, boundary, busy }: TimeSliderProps) {
  if (commits.length === 0) {
    return (
      <p className="lineage-note muted" role="status">
        No commits touch these documents yet, so there is nowhere to slide to.
        Replay reads old documents out of git; it does not remember anything of
        its own.
      </p>
    );
  }

  const stops = positions(commits);
  const index = indexOf(commits, at);
  const current = stops[index];

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
          onChange={(e) => {
            const picked = stops[Number(e.target.value)];
            onChange(picked ? picked.sha : null);
          }}
        />
      </label>
      <p className="time-slider__stop">
        {current === null ? (
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
              <span className="mono">{current.short}</span>
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
