import { useMemo, useState } from "react";

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
  return <section className="notebook-table" aria-label="Read-only table">
    <label>Filter this page <input value={filter} onChange={e => setFilter(e.target.value)} /></label>
    <span className="muted"> Read-only · {table.rows.length} rows{table.total != null ? ` of ${table.total}` : ""}{table.has_more ? " · preview truncated" : ""}{table.columns_truncated ? " · first 256 columns" : ""}</span>
    <div className="notebook-table-scroll"><table>
      <thead><tr>{table.schema.map((col, i) => <th key={i} aria-sort={sort?.column === i ? (sort.descending ? "descending" : "ascending") : "none"}>
        <button onClick={() => setSort({ column: i, descending: sort?.column === i && !sort.descending })}>{col.name}</button><small>{col.dtype}</small>
      </th>)}</tr></thead>
      <tbody>{rows.map((row, i) => <tr key={i}>{table.schema.map((_, j) => <td key={j} title={cell(row[j])} className={row[j] === null ? "muted" : ""}>{cell(row[j])}</td>)}</tr>)}</tbody>
    </table></div>
  </section>;
}
