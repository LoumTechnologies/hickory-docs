// A hole: the lines a column is not showing, and the four ways to move it.
//
// A hole has two edges and each moves both ways, so there are four
// operations, not two. The arrow says which way the EDGE travels and the
// control's position says which edge it drives:
//
//   top    ↑  grow upward, swallowing the lines above
//          ↓  shrink from the top, giving those lines back
//   bottom ↑  shrink from the bottom
//          ↓  grow downward, swallowing the lines below
//
// Naming them by where the revealed text came from instead — which is how a
// diff viewer usually labels its expanders — puts the arrow backwards from
// what a hand expects when it grabs an edge.

import { foldRange, revealRange, type Range } from "./model";

/** Lines an expander moves per click. */
export const CHUNK = 10;

export interface HoleRowProps {
  from: number;
  to: number;
  max: number;
  lines: string[];
  ranges: Range[];
  onRanges: (ranges: Range[]) => void;
}

export function HoleRow({ from, to, max, lines, ranges, onRanges }: HoleRowProps) {
  const hidden = to - from + 1;
  const preview = lines.slice(from, to + 1).find((l) => l.trim().length > 2);

  return (
    <div className="hole" data-from={from} data-to={to}>
      <div className="hole-edges">
        <div className="hole-edge">
          <button
            type="button"
            className="hole-btn"
            disabled={from === 0}
            data-tip={`Hide ${Math.min(CHUNK, from)} more line(s) above`}
            aria-label={`Hide ${Math.min(CHUNK, from)} more lines above`}
            onClick={() => onRanges(foldRange(ranges, Math.max(0, from - CHUNK), from - 1, max))}
          >
            ↑
          </button>
          <button
            type="button"
            className="hole-btn"
            data-tip={`Show ${Math.min(CHUNK, hidden)} line(s) from the top`}
            aria-label={`Show ${Math.min(CHUNK, hidden)} lines from the top`}
            onClick={() => onRanges(revealRange(ranges, from, from + CHUNK - 1, max))}
          >
            ↓
          </button>
        </div>
        <div className="hole-edge">
          <button
            type="button"
            className="hole-btn"
            data-tip={`Show ${Math.min(CHUNK, hidden)} line(s) from the bottom`}
            aria-label={`Show ${Math.min(CHUNK, hidden)} lines from the bottom`}
            onClick={() => onRanges(revealRange(ranges, to - CHUNK + 1, to, max))}
          >
            ↑
          </button>
          <button
            type="button"
            className="hole-btn"
            disabled={to === max}
            data-tip={`Hide ${Math.min(CHUNK, max - to)} more line(s) below`}
            aria-label={`Hide ${Math.min(CHUNK, max - to)} more lines below`}
            onClick={() => onRanges(foldRange(ranges, to + 1, Math.min(max, to + CHUNK), max))}
          >
            ↓
          </button>
        </div>
      </div>
      <div className="hole-label">
        <span>
          <b>{hidden}</b> hidden · {from + 1}–{to + 1}
          {preview ? ` ${preview.trim().slice(0, 34)}…` : ""}
        </span>
        {hidden <= CHUNK * 3 && (
          <button
            type="button"
            className="hole-all"
            data-tip={`Show all ${hidden} hidden lines`}
            onClick={() => onRanges(revealRange(ranges, from, to, max))}
          >
            ⤢ {hidden}
          </button>
        )}
      </div>
    </div>
  );
}
