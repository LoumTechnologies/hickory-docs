<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lineage/HoleRow.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Captured reading surfaces share owned native actions](../../../../../contract.md)

Captured SHA-256: `6e8d14c09c1b208b522bb0d194e75ac37d78d397f4726a4b692b372e72f6dc3f`.

<book:entry path="apps/web/src/lineage/HoleRow.tsx">
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

import { useLineageControls } from "./useLineageControls";
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

  const moves = [
    { key: "above", label: `Hide ${Math.min(CHUNK, from)} more lines above`, enabled: from !== 0, run: () => onRanges(foldRange(ranges, Math.max(0, from - CHUNK), from - 1, max)) },
    { key: "top", label: `Show ${Math.min(CHUNK, hidden)} lines from the top`, run: () => onRanges(revealRange(ranges, from, from + CHUNK - 1, max)) },
    { key: "bottom", label: `Show ${Math.min(CHUNK, hidden)} lines from the bottom`, run: () => onRanges(revealRange(ranges, to - CHUNK + 1, to, max)) },
    { key: "below", label: `Hide ${Math.min(CHUNK, max - to)} more lines below`, enabled: to !== max, run: () => onRanges(foldRange(ranges, to + 1, Math.min(max, to + CHUNK), max)) },
    ...(hidden <= CHUNK * 3 ? [{ key: "all", label: `Show all ${hidden} hidden lines`, run: () => onRanges(revealRange(ranges, from, to, max)) }] : []),
  ];
  useLineageControls("lineage folded range", { from, to, hidden, preview: preview?.trim().slice(0, 34) ?? null }, moves);
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
            onClick={() => moves[0].run()}
          >
            ↑
          </button>
          <button
            type="button"
            className="hole-btn"
            data-tip={`Show ${Math.min(CHUNK, hidden)} line(s) from the top`}
            aria-label={`Show ${Math.min(CHUNK, hidden)} lines from the top`}
            onClick={() => moves[1].run()}
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
            onClick={() => moves[2].run()}
          >
            ↑
          </button>
          <button
            type="button"
            className="hole-btn"
            disabled={to === max}
            data-tip={`Hide ${Math.min(CHUNK, max - to)} more line(s) below`}
            aria-label={`Hide ${Math.min(CHUNK, max - to)} more lines below`}
            onClick={() => moves[3].run()}
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
            onClick={() => moves[4].run()}
          >
            ⤢ {hidden}
          </button>
        )}
      </div>
    </div>
  );
}
</book:entry>

</book:doc>
