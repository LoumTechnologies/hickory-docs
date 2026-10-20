// One stage: its files as a tree, with one of them open in place.
//
// The tree is how you keep your bearings — a column of source with no
// surroundings tells you what a file says and never where it sits. Files
// belonging to other stages are counted at the foot and take you to the
// column that owns them, so a hidden file is a place you can reach rather
// than a number.

import { useEffect, useMemo, useRef, useState } from "react";

import { useLineageControls } from "./useLineageControls";
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

  const [otherOpen, setOtherOpen] = useState(false);
  const fold = useRef<{ from: number; to: number } | null>(null);
  const finishFold = useRef<(() => void) | null>(null);
  useEffect(() => () => { if (finishFold.current) window.removeEventListener("pointerup", finishFold.current); fold.current = null; }, [model, column.focus, column.ranges]);
  const hide = (from: number, to: number) => {
    if (![from, to].every(line => column.ranges.some(([a, b]) => line >= a && line <= b))) throw Error("Fold endpoints must be displayed lines");
    onRanges(foldRange(column.ranges, Math.min(from, to), Math.max(from, to), max));
  };
  const jump = (path: string) => {
    const owner = model.stages.findIndex(stage => stage.files.includes(path));
    onJump(path, owner === -1 ? index : owner);
  };
  const visible = file ? column.ranges.flatMap(([from, to]) => file.lines.slice(from, to + 1).map((text, offset) => ({ line: from + offset, text, hit: hits.has(from + offset), node: nodeAt(model, file.path, from + offset)?.id ?? null }))) : [];
  const nodes = [...new Set(visible.flatMap(line => line.node ? [line.node] : []))];
  useLineageControls(`lineage column ${index + 1}`, { index, stage: stage?.name, focus: column.focus, query: column.query,
    files: mine.map(path => ({ path, lines: model.files.get(path)?.lines.length, hits: hitsByFile.get(path)?.size ?? 0, executable_cells: [...model.nodes.values()].filter(node => node.file === path && node.kind === "exec").length })), visible_lines: visible.map(line => ({ ...line, relation: line.node ? relation(line.node) : null })),
    other_stages_open: otherOpen, other_files: otherOpen ? elsewhere : null, scope: "Captured displayed source lines only; folded bytes require reveal controls. Line indices are zero-based." }, [
    ...mine.filter(path => model.files.has(path)).map(path => ({ key: `file:${path}`, label: `${path === column.focus ? "Close" : "Open"} lineage file ${path}`, run: () => onOpen(path === column.focus ? null : path) })),
    { key: "query", label: `Search lineage column ${index + 1}`, enabled: !!column.focus, parameters: { query: { type: "string" as const, maxLength: 8192 } }, run: args => onQuery(args.query as string) },
    ...(visible.length ? [{ key: "fold", label: `Fold displayed lines in lineage column ${index + 1}`, parameters: { from: { type: "number" as const, minimum: 0, maximum: max, integer: true }, to: { type: "number" as const, minimum: 0, maximum: max, integer: true } }, run: (args: Record<string, import("../lib/uiActions").UiValue>) => hide(args.from as number, args.to as number) }] : []),
    ...nodes.map(id => ({ key: `node:${id}`, label: `Select lineage block ${id}`, run: () => onSelect(id) })),
    ...(elsewhere.length ? [{ key: "other", label: `${otherOpen ? "Hide" : "Show"} other lineage stages in column ${index + 1}`, run: () => setOtherOpen(!otherOpen) }] : []),
    ...(otherOpen ? elsewhere.map(path => ({ key: `jump:${path}`, label: `Go to lineage file ${path}`, run: () => jump(path) })) : []),
  ]);

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
                  <span className="sf-runs" data-tip={`${cells} executable cell${cells === 1 ? "" : "s"}`}>
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
          <details className="stage-elsewhere" open={otherOpen} onToggle={event => setOtherOpen(event.currentTarget.open)}>
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
                  data-tip={
                    owner === -1
                      ? `Open ${path} here — no stage owns it`
                      : `Go to the ${model.stages[owner].name} stage, which owns ${path}`
                  }
                  onClick={() => jump(path)}
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
          data-tip="Drag to fold these lines away"
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
    if (finishFold.current) window.removeEventListener("pointerup", finishFold.current);
    fold.current = { from: line, to: line };
    const finish = () => {
      window.removeEventListener("pointerup", finish);
      finishFold.current = null;
      if (!fold.current) return;
      const { from, to } = fold.current;
      fold.current = null;
      hide(from, to);
    };
    finishFold.current = finish;
    window.addEventListener("pointerup", finish);
  }

  function extendFold(line: number) {
    if (fold.current) fold.current.to = line;
  }
}
