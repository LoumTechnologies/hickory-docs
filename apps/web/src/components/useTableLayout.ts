import { useLayoutEffect, useRef, useState } from "react";
import type { TableLayout } from "./TablePanel";

/** Responsive prose tables become a fixed grid when a column is resized. */
export function useTableLayout(
  initial: TableLayout | undefined,
  onLayout: ((next: TableLayout) => void) | undefined,
  defaultFit: boolean,
  source: string,
) {
  const gridRef = useRef<HTMLTableElement | null>(null);
  const [size, setSize] = useState<TableLayout>(() => initial ?? {});
  const fitProse = size.fitProse ?? (defaultFit && !Object.keys(size.widths ?? {}).length);
  const [measured, setMeasured] = useState<{ widths: Record<string, number>; heights: Record<string, number> }>({ widths: {}, heights: {} });
  const read = () => {
    const widths: Record<string, number> = {}, heights: Record<string, number> = {};
    for (const [index, cell] of Array.from(gridRef.current?.querySelectorAll<HTMLElement>("thead .table-panel__head") ?? []).entries()) {
      const width = cell.getBoundingClientRect().width;
      if (width > 0) widths[index] = width;
    }
    for (const [index, row] of Array.from(gridRef.current?.tBodies[0]?.rows ?? []).entries()) {
      const height = row.getBoundingClientRect().height;
      if (height > 0) heights[index] = Math.ceil(height);
    }
    return { widths, heights };
  };
  useLayoutEffect(() => {
    if (!fitProse || !gridRef.current) return;
    const measure = () => {
      const next = read();
      setMeasured(previous => JSON.stringify(previous) === JSON.stringify(next) ? previous : next);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(gridRef.current);
    return () => observer.disconnect();
  }, [fitProse, source]);
  const resize = (next: TableLayout) => { setSize(next); onLayout?.(next); };
  const fixed = (): TableLayout => fitProse
    ? { ...size, ...read(), fitProse: false }
    : size;
  const columnWidth = (column: number) => fitProse
    ? (gridRef.current?.querySelectorAll("thead .table-panel__head")[column]?.getBoundingClientRect().width || measured.widths[column] || 104)
    : size.widths?.[column] ?? 104;
  const rowHeight = (row: number) => fitProse
    ? (gridRef.current?.tBodies[0]?.rows[row]?.getBoundingClientRect().height || measured.heights[row] || 24)
    : size.heights?.[row] ?? 24;
  const widen = (column: number, to: number) => {
    const next = fixed();
    resize({ ...next, widths: { ...next.widths, [column]: to } });
  };
  const heighten = (row: number, to: number) =>
    resize({ ...size, heights: { ...size.heights, [row]: to } });
  const setFitProse = (enabled: boolean) => {
    const next = enabled ? { ...size, fitProse: true } : { ...fixed(), fitProse: false };
    if (enabled) delete next.heights;
    resize(next);
  };
  return { gridRef, size, resize, fitProse, setFitProse, columnWidth, rowHeight, widen, heighten };
}
