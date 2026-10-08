import { cellLabel } from "../lib/cellRef";
import { withCell, type Csv } from "../lib/csv";
import { cellsIn, type Cell, type Selection } from "../lib/tableSelection";
import { TableNumberFormat } from "./TableNumberFormat";

export function TableFormatControl({ table, selection, draft, onChange }: {
  table: Csv; selection: Selection | null; draft: (Cell & { value: string }) | null;
  onChange: (table: Csv) => void;
}) {
  const cells = selection ? cellsIn(selection) : [];
  const values = new Set(cells.map(at => table.formats?.[cellLabel(at.column, at.row)] ?? "general"));
  return <TableNumberFormat disabled={!selection}
    value={values.size === 1 ? [...values][0] : selection ? "mixed" : "general"}
    onChange={format => {
      if (!selection) return;
      const next = { ...table, formats: { ...table.formats } };
      if (draft) next.rows = withCell(table, draft.row, draft.column, draft.value).rows;
      for (const at of cells) next.formats[cellLabel(at.column, at.row)] = format;
      onChange(next);
    }} />;
}
