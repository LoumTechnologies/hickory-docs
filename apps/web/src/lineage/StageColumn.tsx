// One stage: its files as a tree, with one of them open in place.
//
// The tree is how you keep your bearings — a column of source with no
// surroundings tells you what a file says and never where it sits. Files
// belonging to other stages are counted at the foot and take you to the
// column that owns them, so a hidden file is a place you can reach rather
// than a number.

import { useMemo } from "react";

import { HoleRow } from "./HoleRow";
import {
  foldRange,
  holesOf,
  nodeAt,
  stageOf,
  type LineageModel,
  type Range,
} from "./model";
import type { ColumnState } from "./LineageColumns";

export interface StageColumnProps {
  index: number;
  model: LineageModel;
  column: ColumnState;
  relation: (id: string) => "self" | "up" | "down" | null;
  /** Ranked hits per file, from the shared index. */
  hits: Map<string, Set<number>>;
  onOpen: (path: string | null) => void;
  onQuery: (query: string) => void;
  onRanges: (ranges: Range[]) => void;
  onSelect: (id: string) => void;
  onScroll: () => void;
  onJump: (path: string, targetColumn: number) => void;
}

export function StageColumn(props: StageColumnProps) {
  const {
    index,
    model,
    column,
    relation,
    hits: hitsByFile,
    onOpen,
    onQuery,
    onRanges,
    onSelect,
    onScroll,
    onJump,
  } = props;

  const stage = useMemo(
    () => (column.focus ? stageOf(model, column.focus) : model.stages[index]),
    [column.focus, index, model],
  );
  const mine = stage?.files ?? [];
  const elsewhere = useMemo(
    () => [...model.files.keys()].filter((p) => !mine.includes(p)),
    [mine, model],
  );

  const file = column.focus ? model.files.get(column.focus) : null;
  const max = file ? file.lines.length - 1 : 0;
  const hits = useMemo(
    () => (file ? (hitsByFile.get(file.path) ?? new Set<number>()) : new Set<number>()),
    [file, hitsByFile],
  );

  return (
    <section className="stage-col" data-column={index} data-stage={stage?.doc ?? ""}>
      <header className="stage-head">
        <div className="stage-title">
          {/* Named for the stage, never for whichever file is open: opening a
              generated file must not look like the column became that file. */}
          <span className="stage-name">{stage?.name ?? "—"}</span>
          <span className="stage-kind">stage</span>
        </div>
        <div className="stage-open">
          {column.focus
            ? `${column.focus} · ${column.ranges.reduce((n, [a, b]) => n + (b - a + 1), 0)}/${
                (file?.lines.length ?? 0)
              } lines`
            : "nothing open"}
        </div>
        <input
          className="stage-search"
          type="search"
          value={column.query}
          disabled={!column.focus}
          placeholder={column.focus ? "Search this file…" : "Open a file to search it"}
          aria-label={`Search ${column.focus ?? "this stage"}`}
          onChange={(e) => onQuery(e.target.value)}
        />
      </header>

      <div className="stage-body" onScroll={onScroll}>
        {mine.map((path) => {
          const open = path === column.focus;
          const model_file = model.files.get(path);
          if (!model_file) return null;
          const fileHits = hitsByFile.get(path)?.size ?? 0;
          const cells = [...model.nodes.values()].filter(
            (n) => n.file === path && n.kind === "exec",
          ).length;
          return (
            <div key={path}>
              <button
                type="button"
                className={`stage-file${open ? " open" : ""}`}
                data-treepath={path}
                aria-expanded={open}
                onClick={() => onOpen(open ? null : path)}
              >
                {/* Decorative: `aria-expanded` on the button already says
                    open or closed, and letting the glyph into the accessible
                    name buries the file name a screen reader announces. */}
                <span className="sf-mark" aria-hidden="true">
                  {open ? "▾" : "▸"}
                </span>
                {/* The open row carries the RELATIVE PATH and sticks to the top
                    of the scroller: a name alone is not a location, and
                    scrolling into a long file otherwise takes every clue about
                    where it lives with it. */}
                <span className="sf-name">{open ? path : path.split("/").pop()}</span>
                {fileHits > 0 && <span className="sf-hits">{fileHits}</span>}
                {/* Any stage can run things — saying so per file keeps
                    execution from looking like the last stage's privilege. */}
                {cells > 0 && (
                  <span className="sf-runs" title={`${cells} executable cell${cells === 1 ? "" : "s"}`}>
                    ▶ {cells}
                  </span>
                )}
                <span className="sf-lines">{model_file.lines.length}L</span>
              </button>

              {open && file && (
                <div className="stage-file-body">
                  {renderBody()}
                </div>
              )}
            </div>
          );
        })}

        {elsewhere.length > 0 && (
          <details className="stage-elsewhere">
            <summary>
              ⋯ {elsewhere.length} more file{elsewhere.length === 1 ? "" : "s"} in other stages
            </summary>
            {elsewhere.map((path) => {
              const owner = model.stages.findIndex((s) => s.files.includes(path));
              return (
                <button
                  key={path}
                  type="button"
                  className="stage-elsewhere-entry"
                  title={
                    owner === -1
                      ? `Open ${path} here — no stage owns it`
                      : `Go to the ${model.stages[owner].name} stage, which owns ${path}`
                  }
                  onClick={() => onJump(path, owner === -1 ? index : owner)}
                >
                  <span className="sf-name">{path}</span>
                  <span className="sf-where">
                    {owner === -1 ? "no stage" : model.stages[owner].name}
                  </span>
                </button>
              );
            })}
          </details>
        )}
      </div>
    </section>
  );

  function renderBody() {
    if (!file) return null;
    const out: JSX.Element[] = [];
    const holes = holesOf(column.ranges, max);
    let holeIndex = 0;
    let cursor = 0;

    const pushHole = (hole: Range) => {
      out.push(
        <HoleRow
          key={`hole-${holeIndex++}`}
          from={hole[0]}
          to={hole[1]}
          max={max}
          lines={file.lines}
          onRanges={(next) => onRanges(next)}
          ranges={column.ranges}
        />,
      );
    };

    for (const [a, b] of column.ranges) {
      const hole = holes.find((h) => h[1] === a - 1);
      if (hole && hole[0] === cursor) pushHole(hole);
      for (let i = a; i <= b; i++) out.push(line(i));
      cursor = b + 1;
    }
    const tail = holes.find((h) => h[0] === cursor);
    if (tail) pushHole(tail);
    if (!column.ranges.length && holes.length) pushHole(holes[0]);
    return out;
  }

  function line(i: number) {
    const node = column.focus ? nodeAt(model, column.focus, i) : undefined;
    const rel = node ? relation(node.id) : null;
    return (
      <div
        key={i}
        className="lin-row"
        data-blockid={node?.id ?? undefined}
        data-lineid={node && node.startLine === i ? node.id : undefined}
        data-rel={rel ?? undefined}
        data-hit={hits.has(i) ? "1" : undefined}
      >
        <span
          className="lin-gutter"
          title="Drag to fold these lines away"
          onPointerDown={(e) => {
            e.preventDefault();
            (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
            startFold(i);
          }}
          onPointerEnter={() => extendFold(i)}
        >
          {i + 1}
        </span>
        <span
          className="lin-text"
          role={node ? "button" : undefined}
          tabIndex={node ? 0 : undefined}
          onClick={() => node && onSelect(node.id)}
          onKeyDown={(e) => {
            if (node && (e.key === "Enter" || e.key === " ")) {
              e.preventDefault();
              onSelect(node.id);
            }
          }}
        >
          {file!.lines[i] || " "}
        </span>
      </div>
    );
  }

  // Folding is a drag over the gutter, so the unit is whatever the reader
  // decides it is rather than one the tool picked for them.
  function startFold(line: number) {
    foldState = { from: line, to: line };
    const finish = () => {
      window.removeEventListener("pointerup", finish);
      if (!foldState) return;
      const { from, to } = foldState;
      foldState = null;
      onRanges(foldRange(column.ranges, Math.min(from, to), Math.max(from, to), max));
    };
    window.addEventListener("pointerup", finish);
  }

  function extendFold(line: number) {
    if (foldState) foldState.to = line;
  }
}

let foldState: { from: number; to: number } | null = null;
