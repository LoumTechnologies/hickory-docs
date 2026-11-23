import { useId, useMemo, useRef, useState } from "react";
import { TablePanel } from "./TablePanel";
import { writeCsv } from "../lib/csv";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";

export interface NotebookTable {
  kind: "table";
  schema: { name: string; dtype: string }[];
  rows: unknown[][];
  offset: number;
  has_more: boolean;
  total?: number | null;
  columns_truncated?: boolean;
}
function cell(value: unknown): string {
  if (value === null) return "null";
  return typeof value === "object" ? JSON.stringify(value) : String(value);
}

/** No edit callback, formula evaluation or writable controls. Values stay data. */
export function ReadOnlyTable({ table }: { table: NotebookTable }) {
  const [filter, setFilter] = useState("");
  const [filterOpen, setFilterOpen] = useState(false);
  const filterButton = useRef<HTMLButtonElement>(null);
  const [sort, setSort] = useState<{ column: number; descending: boolean } | null>(null);
  const rows = useMemo(() => {
    const result = table.rows.filter(row => row.some(v => cell(v).toLowerCase().includes(filter.toLowerCase())));
    if (sort) result.sort((a, b) => {
      const left = a[sort.column], right = b[sort.column];
      const order = typeof left === "number" && typeof right === "number" ? left - right : cell(left).localeCompare(cell(right));
      return sort.descending ? -order : order;
    });
    return result;
  }, [table, filter, sort]);
  const parent = `notebook-table:${useId()}`;
  const revision = useUiRevision(() => [table, filter, sort]);
  const sortBy = (column: number, descending: boolean) => setSort({ column, descending });
  useUiCommands([
    { key: `${parent}:filter`, command: { label: "Filter read-only table page", parent, subject: parent, parameters: { filter: { type: "string", maxLength: 16384 } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => { setFilter(args.filter as string); return { status: "completed", value: { filtered_page: true, scope: "Current loaded preview only" } }; } } },
    { key: `${parent}:sort`, command: { label: "Sort read-only table page", parent, subject: parent, parameters: { column: { type: "number", minimum: 0, maximum: 1000000, integer: true }, descending: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: table.schema.length > 0 }), execute: async args => {
      if ((args.column as number) >= table.schema.length) return { status: "unavailable", reason: "Observe a current schema column index" };
      sortBy(args.column as number, args.descending as boolean);
      return { status: "completed", value: { sorted_page: true, scope: "Current loaded preview only" } };
    } } },
    { key: `${parent}:read`, command: { label: "Read visible table page bytes", parent, subject: parent, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { visible_rows: rows.length, loaded_rows: table.rows.length, columns: table.schema.length } }), execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ schema: table.schema, rows, filter, sort, source_offset: table.offset, loaded_rows: table.rows.length, has_more: table.has_more, total: table.total ?? null, columns_truncated: table.columns_truncated ?? false, scope: "Read-only current page after local filtering/sorting; no dataset write or global sorting" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside visible table page" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    } } },
  ]);
  const source = writeCsv({ rows: [table.schema.map(col => col.name), ...rows.map(row => table.schema.map((_, column) => cell(row[column])))],
    delimiter: ",", newline: "\n", trailingNewline: false });
  return <section className="notebook-table" aria-label="Read-only table" style={{ position: "relative" }}>
    <button ref={filterButton} className="debug-icon" type="button" aria-label="Filter table" aria-expanded={filterOpen}
      style={{ position: "absolute", right: 4, top: 3, zIndex: 2 }} onClick={() => setFilterOpen(open => !open)}>
      <svg width="15" height="15" viewBox="0 0 16 16" aria-hidden="true"><path d="M2 3h12L9 8v4l-2 1V8Z" fill="none" stroke="currentColor" /></svg>
    </button>
    {filterOpen && <div role="dialog" aria-label="Filter table" className="simple-semantic-results" style={{ position: "absolute", right: 26, top: 4, zIndex: 3 }}
      onKeyDown={event => { if (event.key === "Escape") { event.stopPropagation(); setFilterOpen(false); filterButton.current?.focus(); } }}>
      <label>Filter this page <input autoFocus value={filter} onChange={e => setFilter(e.target.value)} /></label>
      <label>Sort by <select value={sort?.column ?? ""} onChange={e => e.target.value === "" ? setSort(null) : sortBy(Number(e.target.value), false)}>
        <option value="">Original order</option>
        {table.schema.map((col, column) => <option key={column} value={column}>{col.name}</option>)}
      </select></label>
      {sort && <button type="button" onClick={() => sortBy(sort.column, !sort.descending)}>{sort.descending ? "Descending" : "Ascending"}</button>}
    </div>}
    <TablePanel source={source} delimiter="," header fitProseDefault laneRight />
    {(table.has_more || table.columns_truncated) && <small>Showing {rows.length} loaded rows{table.total != null ? ` of ${table.total}` : ""}{table.columns_truncated ? " · first 256 columns" : ""}</small>}
  </section>;
}
