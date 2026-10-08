import { useEffect, useRef, useState, type RefObject } from "react";
import { type Csv } from "../lib/csv";
import { fillExtent, fillTable } from "../lib/tableFill";
import { boundsOf, containsCell, type Selection } from "../lib/tableSelection";

interface FillDrag {
  table: Csv; seed: Selection; target: Selection; revision: string; formulas: string | false;
}

/** One drag owns a frozen seed and publishes exactly once, on mouseup. */
export function useTableFill({ table, selection, revision, language, gridRef, onFill }: {
  table: Csv; selection: Selection | null; revision: string; language?: string;
  gridRef: RefObject<HTMLTableElement | null>; onFill: (table: Csv, selection: Selection) => void;
}) {
  const drag = useRef<FillDrag | null>(null);
  const latest = useRef({ table, selection, revision, language, onFill });
  latest.current = { table, selection, revision, language, onFill };
  const [preview, setPreview] = useState<Selection | null>(null);
  const cancel = () => { drag.current = null; setPreview(null); };
  const over = (row: number, column: number) => {
    const current = drag.current;
    if (!current) return;
    const target = fillExtent(current.table, current.seed, { row, column });
    const a = boundsOf(target), b = boundsOf(current.target);
    if (a.top === b.top && a.bottom === b.bottom && a.left === b.left && a.right === b.right) return;
    current.target = target;
    setPreview(target);
  };
  const handlers = useRef({ over, cancel });
  handlers.current = { over, cancel };

  useEffect(() => {
    const move = (event: MouseEvent) => {
      if (!drag.current || !gridRef.current) return;
      event.preventDefault();
      const grid = gridRef.current;
      const hit = document.elementFromPoint?.(event.clientX, event.clientY)?.closest<HTMLElement>("td[data-fill-row]");
      if (hit && grid.contains(hit)) {
        handlers.current.over(Number(hit.dataset.fillRow), Number(hit.dataset.fillColumn));
        return;
      }
      // Outside the last cell, use its dimensions to preview new rows/columns.
      // At the top/left edge there is no row before 1 or column before A.
      const rowCells = Array.from(grid.querySelectorAll<HTMLElement>('td[data-fill-column="0"]'));
      const columnCells = Array.from(grid.querySelectorAll<HTMLElement>('td[data-fill-row="0"]'));
      const coordinate = (cells: HTMLElement[], point: number, axis: "x" | "y") => {
        if (!cells.length) return 0;
        for (let index = 0; index < cells.length; index++) {
          const rect = cells[index].getBoundingClientRect();
          if (point < (axis === "x" ? rect.right : rect.bottom)) return index;
        }
        const last = cells[cells.length - 1].getBoundingClientRect();
        const end = axis === "x" ? last.right : last.bottom;
        const size = axis === "x" ? last.width : last.height;
        return cells.length + Math.floor((point - end) / Math.max(1, size));
      };
      handlers.current.over(coordinate(rowCells, event.clientY, "y"), coordinate(columnCells, event.clientX, "x"));
      const viewport = grid.parentElement;
      if (viewport && typeof viewport.scrollBy === "function") {
        const box = viewport.getBoundingClientRect();
        viewport.scrollBy(event.clientX > box.right ? 24 : event.clientX < box.left ? -24 : 0,
          event.clientY > box.bottom ? 24 : event.clientY < box.top ? -24 : 0);
      }
    };
    const finish = () => {
      const current = drag.current;
      if (!current) return;
      handlers.current.cancel();
      // An external edit invalidates the frozen seed; never overwrite it.
      if (current.revision !== latest.current.revision) return;
      const a = boundsOf(current.seed), b = boundsOf(current.target);
      if (a.top === b.top && a.bottom === b.bottom && a.left === b.left && a.right === b.right) return;
      latest.current.onFill(fillTable(current.table, current.seed, current.target, current.formulas), current.target);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && drag.current) {
        event.preventDefault();
        handlers.current.cancel();
      }
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", finish);
    window.addEventListener("keydown", escape);
    window.addEventListener("blur", handlers.current.cancel);
    const onBlur = handlers.current.cancel;
    return () => {
      drag.current = null;
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", finish);
      window.removeEventListener("keydown", escape);
      window.removeEventListener("blur", onBlur);
    };
  }, [gridRef]);

  const corner = selection ? boundsOf(selection) : null;
  return {
    at: (row: number, column: number) => corner?.bottom === row && corner.right === column,
    preview,
    includes: (row: number, column: number) => !!preview && containsCell(preview, row, column) &&
      !!selection && !containsCell(selection, row, column),
    over,
    start: (event: React.MouseEvent) => {
      if (event.button !== 0 || !selection) return;
      event.preventDefault(); event.stopPropagation();
      drag.current = { table, seed: selection, target: selection, revision, formulas: language || false };
      setPreview(selection);
    },
    // A ref rather than render state allows hover handlers to distinguish a drag
    // from ordinary selection without waiting for React to rerender.
    dragging: () => drag.current !== null,
  };
}
