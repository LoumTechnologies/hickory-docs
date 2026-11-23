import { cellLabel } from "./cellRef";
import { columnCount, type Csv } from "./csv";
import { copyFormulaReferences } from "./formulaReferences";
import { boundsOf, type Cell, type Selection } from "./tableSelection";

/** Extend along one axis. Diagonal drags use the larger distance outside the seed. */
export function fillSelection(seed: Selection, at: Cell, height: number, width: number, limit = 20_000): Selection {
  const bounds = boundsOf(seed);
  const row = Math.max(0, Math.min(at.row, Math.max(height, Math.floor(limit / width)) - 1));
  const column = Math.max(0, Math.min(at.column, Math.max(width, Math.floor(limit / height)) - 1));
  const vertical = row < bounds.top ? row - bounds.top : Math.max(0, row - bounds.bottom);
  const horizontal = column < bounds.left ? column - bounds.left : Math.max(0, column - bounds.right);
  return {
    kind: "cells",
    anchor: { row: Math.abs(vertical) >= Math.abs(horizontal) ? Math.min(bounds.top, row) : bounds.top,
      column: Math.abs(horizontal) > Math.abs(vertical) ? Math.min(bounds.left, column) : bounds.left },
    focus: { row: Math.abs(vertical) >= Math.abs(horizontal) ? Math.max(bounds.bottom, row) : bounds.bottom,
      column: Math.abs(horizontal) > Math.abs(vertical) ? Math.max(bounds.right, column) : bounds.right },
  };
}

function numericSeries(seed: string[]): ((offset: number) => string | undefined) | undefined {
  if (seed.length < 2 || !seed.every(value => /^-?(?:0|[1-9]\d*)(?:\.\d+)?$/.test(value))) return;
  const precision = seed.reduce((max, value) => Math.max(max, value.split(".")[1]?.length ?? 0), 0);
  if (precision > 12) return;
  const scale = 10 ** precision;
  const numbers = seed.map(value => {
    const [whole, fractional = ""] = value.split(".");
    return Number(whole + fractional.padEnd(precision, "0"));
  });
  if (!numbers.every(Number.isSafeInteger)) return;
  const step = numbers[1] - numbers[0];
  if (!Number.isSafeInteger(step) || !numbers.every((value, index) => value - numbers[0] === step * index)) return;
  return offset => {
    const advance = offset * step;
    const value = numbers[0] + advance;
    if (Number.isSafeInteger(advance) && Number.isSafeInteger(value)) return String(value / scale);
  };
}

export function fillTable(table: Csv, seed: Selection, target: Selection, formulas: boolean | string): Csv {
  const from = boundsOf(seed);
  const to = boundsOf(target);
  const rows = table.rows.map(row => [...row]);
  const formats = { ...table.formats };
  const vertical = to.left === from.left && to.right === from.right;
  const lanes = vertical ? from.right - from.left + 1 : from.bottom - from.top + 1;
  const length = vertical ? from.bottom - from.top + 1 : from.right - from.left + 1;
  const series = Array.from({ length: lanes }, (_, lane) => numericSeries(
    Array.from({ length }, (_, offset) =>
      table.rows[from.top + (vertical ? offset : lane)]?.[from.left + (vertical ? lane : offset)] ?? "",
    ),
  ));
  const modulo = (n: number, divisor: number) => ((n % divisor) + divisor) % divisor;
  for (let row = to.top; row <= to.bottom; row++) {
    while (rows.length <= row) rows.push([]);
    for (let column = to.left; column <= to.right; column++) {
      if (row >= from.top && row <= from.bottom && column >= from.left && column <= from.right) continue;
      const sourceRow = from.top + modulo(row - from.top, from.bottom - from.top + 1);
      const sourceColumn = from.left + modulo(column - from.left, from.right - from.left + 1);
      const value = table.rows[sourceRow]?.[sourceColumn] ?? "";
      while (rows[row].length <= column) rows[row].push("");
      const extended = series[vertical ? column - from.left : row - from.top]?.(vertical ? row - from.top : column - from.left);
      rows[row][column] = extended ?? (formulas ? copyFormulaReferences(value, row - sourceRow, column - sourceColumn, typeof formulas === "string" ? formulas : undefined) : value);
      const label = cellLabel(column, row);
      const format = table.formats?.[cellLabel(sourceColumn, sourceRow)];
      if (format) formats[label] = format;
      else delete formats[label];
    }
  }
  return { ...table, rows, ...(table.formats ? { formats } : {}) };
}

export function fillExtent(table: Csv, seed: Selection, at: Cell): Selection {
  return fillSelection(seed, at, Math.max(1, table.rows.length), Math.max(1, columnCount(table)));
}
