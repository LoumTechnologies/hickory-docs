// The lineage browser: one column per stage, each the stage's files with one
// of them open in place.
//
// The shape is the answer to a question the two-pane view could not answer —
// "why does this say what it says?" — where the answer usually lives several
// documents away. Everything here follows from that:
//
//  * A column is a STAGE, named for the stage, so opening one of its files
//    never makes the column look like it has become something else.
//  * Holes work like a pull request. Both edges of one move both ways, and
//    the gutter folds. You keep exactly the lines you care about.
//  * Selecting a block reveals what it came from and what it feeds, wherever
//    those live — opening the hole or the file that was hiding them — and
//    never scrolls the column you clicked in.
//  * A link that cannot reach its end draws to the hole or the closed file
//    that swallowed it, and says how many ended there. It is never simply
//    absent, because absence reads as "no such relationship".

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { indexOf } from "./bm25";
import { LinkLayer } from "./LinkLayer";
import { StageColumn } from "./StageColumn";
import {
  LINK_KINDS,
  fullRange,
  rangesForLines,
  stageOf,
  walk,
  type LinkKind,
  type LineageModel,
  type Range,
} from "./model";

export interface ColumnState {
  /** Which file this column has open, or null for the tree alone. */
  focus: string | null;
  ranges: Range[];
  query: string;
}

export interface LineageColumnsProps {
  model: LineageModel;
  /** Files to open when the browser starts, one column each. */
  initialFocus?: string[];
  /** Called when a block is selected, for panes outside this component. */
  onSelect?: (nodeId: string | null) => void;
  /** Hide the toolbar when the host provides its own. */
  compact?: boolean;
}

export function LineageColumns({ model, initialFocus, onSelect, compact }: LineageColumnsProps) {
  const startingFocus = useMemo(
    () => initialFocus ?? model.stages.map((s) => s.doc),
    [initialFocus, model],
  );

  const [columns, setColumns] = useState<ColumnState[]>(() =>
    startingFocus.map((focus) => ({
      focus,
      ranges: model.files.has(focus) ? fullRange(model.files.get(focus)!) : [],
      query: "",
    })),
  );
  const [selected, setSelected] = useState<string | null>(null);
  const [globalQuery, setGlobalQuery] = useState("");
  const [kinds, setKinds] = useState<Set<LinkKind>>(() => new Set(Object.keys(LINK_KINDS) as LinkKind[]));
  const [linksOn, setLinksOn] = useState(true);
  const railRef = useRef<HTMLDivElement>(null);
  const [tick, setTick] = useState(0);

  // One ranked index for the whole model: term rarity only means anything
  // against every file, not one at a time.
  const index = useMemo(() => indexOf(model), [model]);
  const hitsFor = useCallback(
    (query: string) => index.searchByFile(query),
    [index],
  );

  // Re-measure the links after anything that can move an endpoint.
  const remeasure = useCallback(() => setTick((n) => n + 1), []);
  useEffect(() => {
    const rail = railRef.current;
    if (!rail) return;
    // Guarded like the editor's own observer: a missing ResizeObserver is a
    // stale layout, never a blank component.
    if (typeof ResizeObserver === "undefined") {
      window.addEventListener("resize", remeasure);
      return () => window.removeEventListener("resize", remeasure);
    }
    const observer = new ResizeObserver(remeasure);
    observer.observe(rail);
    window.addEventListener("resize", remeasure);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", remeasure);
    };
  }, [remeasure]);

  const update = useCallback((index: number, patch: Partial<ColumnState>) => {
    setColumns((prev) => prev.map((col, i) => (i === index ? { ...col, ...patch } : col)));
  }, []);

  const queryFor = useCallback(
    (col: ColumnState) => (col.query.trim() || globalQuery.trim()).toLowerCase(),
    [globalQuery],
  );

  const openFile = useCallback(
    (columnIndex: number, path: string | null) => {
      if (!path) {
        update(columnIndex, { focus: null, ranges: [], query: "" });
        return;
      }
      const file = model.files.get(path);
      if (!file) return;
      const query = globalQuery.trim();
      update(columnIndex, {
        focus: path,
        query: "",
        ranges: query ? rangesForLines(file, hitsFor(query).get(path) ?? []) : fullRange(file),
      });
    },
    [globalQuery, hitsFor, model, update],
  );

  /**
   * Select a block and make everything it touches visible.
   *
   * The column under the cursor is left exactly as it is — scrolling away
   * what somebody just pointed at is the tool arguing with them. Other
   * columns open the file (in the column that owns its STAGE, so a selection
   * never drags a column off its subject) and reveal the lines.
   */
  const select = useCallback(
    (id: string | null) => {
      const next = id === selected ? null : id;
      setSelected(next);
      onSelect?.(next);
      if (!next) return;
      const related = new Set([next, ...walk(model.links, kinds, next, "up"), ...walk(model.links, kinds, next, "down")]);

      setColumns((prev) => {
        const cols = prev.map((c) => ({ ...c }));
        for (const rid of related) {
          const node = model.nodes.get(rid);
          if (!node) continue;
          let index = cols.findIndex((c) => c.focus === node.file);
          if (index === -1) {
            // Retarget the column that owns this file's STAGE — but never one
            // already showing part of the same lineage. Both ends of a link
            // can live in one stage (a fragment and the file it weaves), and
            // a column that closed the document to show its output would have
            // answered the question by destroying it.
            const stage = stageOf(model, node.file);
            index = cols.findIndex(
              (c) =>
                c.focus &&
                stageOf(model, c.focus)?.doc === stage?.doc &&
                ![...related].some((r) => model.nodes.get(r)?.file === c.focus),
            );
            // Nowhere to put it: the link docks to the closed file's row and
            // says so, which is a smaller loss than losing the file you are
            // reading.
            if (index === -1) continue;
            const file = model.files.get(node.file)!;
            cols[index] = { focus: node.file, query: "", ranges: fullRange(file) };
          }
          const file = model.files.get(node.file);
          if (!file) continue;
          const max = file.lines.length - 1;
          const ranges = cols[index].ranges.length ? cols[index].ranges : [];
          cols[index] = {
            ...cols[index],
            ranges: mergeReveal(ranges, node.startLine, node.endLine, max),
          };
        }
        return cols;
      });
      requestAnimationFrame(remeasure);
    },
    [kinds, model, onSelect, remeasure, selected],
  );

  const relation = useCallback(
    (id: string): "self" | "up" | "down" | null => {
      if (!selected) return null;
      if (id === selected) return "self";
      if (walk(model.links, kinds, selected, "up").has(id)) return "up";
      if (walk(model.links, kinds, selected, "down").has(id)) return "down";
      return null;
    },
    [kinds, model, selected],
  );

  const totalHits = useMemo(() => {
    const q = globalQuery.trim();
    if (!q) return null;
    // Counted over every match, not the capped fold: the toolbar reports how
    // much there is, the fold shows the best of it.
    return index.countByFile(q);
  }, [globalQuery, index]);

  return (
    <div className="lineage">
      {!compact && (
        <div className="lineage-bar">
          <div className="lineage-kinds">
            <span className="lineage-kinds-label">Links</span>
            {(Object.keys(LINK_KINDS) as LinkKind[]).map((kind) => (
              <button
                key={kind}
                type="button"
                className={`lineage-kind k-${kind}`}
                aria-pressed={kinds.has(kind)}
                data-tip={LINK_KINDS[kind].detail}
                onClick={() => {
                  const next = new Set(kinds);
                  if (next.has(kind)) next.delete(kind);
                  else next.add(kind);
                  setKinds(next);
                  requestAnimationFrame(remeasure);
                }}
              >
                <i />
                {LINK_KINDS[kind].label}
              </button>
            ))}
            <button
              type="button"
              className="btn btn-quiet"
              aria-pressed={linksOn}
              onClick={() => {
                setLinksOn(!linksOn);
                requestAnimationFrame(remeasure);
              }}
            >
              {linksOn ? "Links on" : "Links off"}
            </button>
          </div>
          <label className="lineage-search">
            <input
              type="search"
              value={globalQuery}
              placeholder="Search every stage…"
              aria-label="Search every stage"
              onChange={(e) => {
                const q = e.target.value;
                setGlobalQuery(q);
                const hits = q.trim() ? hitsFor(q) : null;
                setColumns((prev) =>
                  prev.map((col) => {
                    if (!col.focus || col.query.trim()) return col;
                    const file = model.files.get(col.focus)!;
                    return {
                      ...col,
                      ranges: hits
                        ? rangesForLines(file, hits.get(col.focus) ?? [])
                        : fullRange(file),
                    };
                  }),
                );
                requestAnimationFrame(remeasure);
              }}
            />
            {totalHits && (
              <span className="lineage-count">
                {totalHits.lines} in {totalHits.files} file{totalHits.files === 1 ? "" : "s"}
              </span>
            )}
          </label>
        </div>
      )}

      <div className="lineage-rail" ref={railRef}>
        <div className="lineage-columns" onScroll={remeasure}>
          {columns.map((col, index) => (
            <StageColumn
              key={index}
              index={index}
              model={model}
              column={col}
              relation={relation}
              hits={hitsFor(queryFor(col))}
              onOpen={(path) => {
                openFile(index, path);
                requestAnimationFrame(remeasure);
              }}
              onQuery={(q) => {
                const file = col.focus ? model.files.get(col.focus) : null;
                update(index, {
                  query: q,
                  ranges: file
                    ? q.trim()
                      ? rangesForLines(file, hitsFor(q).get(file.path) ?? [])
                      : fullRange(file)
                    : col.ranges,
                });
                requestAnimationFrame(remeasure);
              }}
              onRanges={(ranges) => {
                update(index, { ranges });
                requestAnimationFrame(remeasure);
              }}
              onSelect={select}
              onScroll={remeasure}
              onJump={(path, target) => {
                openFile(target, path);
                requestAnimationFrame(() => {
                  const rail = railRef.current?.querySelector(".lineage-columns");
                  const el = rail?.querySelector(`[data-column="${target}"]`);
                  el?.scrollIntoView({ inline: "start", block: "nearest", behavior: "smooth" });
                  el?.classList.add("flash");
                  setTimeout(() => el?.classList.remove("flash"), 900);
                  remeasure();
                });
              }}
            />
          ))}
        </div>
        <LinkLayer
          model={model}
          kinds={kinds}
          enabled={linksOn}
          selected={selected}
          relation={relation}
          railRef={railRef}
          tick={tick}
        />
      </div>
    </div>
  );
}

/** Reveal a range, treating an empty set as "nothing is showing yet". */
function mergeReveal(ranges: Range[], from: number, to: number, max: number): Range[] {
  const merged: Range[] = [...ranges, [from, to]];
  return merged
    .map(([a, b]): Range => [Math.max(0, a), Math.min(max, b)])
    .filter(([a, b]) => a <= b)
    .sort((x, y) => x[0] - y[0])
    .reduce<Range[]>((out, range) => {
      const last = out[out.length - 1];
      if (last && range[0] <= last[1] + 1) last[1] = Math.max(last[1], range[1]);
      else out.push([...range]);
      return out;
    }, []);
}
