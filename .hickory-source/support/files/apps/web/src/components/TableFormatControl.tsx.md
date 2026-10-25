<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TableFormatControl.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A fill handle extends formulas](../../../../../../guarantees/authoring/a-fill-handle-extends-formulas/contract.md)
- [Cell formatting keeps numbers numeric](../../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md)

Captured SHA-256: `71fa3e77993e0d12660d8f4437d69eef34c3d63e78ce05a1068e476385a7d0f8`.

<book:entry path="apps/web/src/components/TableFormatControl.tsx">
import { useId, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor } from "../lib/uiActions";
import { NUMBER_FORMATS, type NumberFormat } from "../lib/tableFormats";
import { cellLabel } from "../lib/cellRef";
import { withCell, type Csv } from "../lib/csv";
import { cellsIn, type Cell, type Selection } from "../lib/tableSelection";
import { TableNumberFormat } from "./TableNumberFormat";

export function TableFormatControl({ table, selection, draft, onChange, sourceCurrent, owner }: {
  table: Csv; selection: Selection | null; draft: (Cell & { value: string }) | null;
  onChange: (table: Csv, actor?: UiActor) => void | boolean; sourceCurrent?: () => boolean; owner?: string;
}) {
  const cells = selection ? cellsIn(selection) : [];
  const values = new Set(cells.map(at => table.formats?.[cellLabel(at.column, at.row)] ?? "general"));
  const parent = `table-format:${useId()}`;
  const [error, setError] = useState("");
  const current = () => sourceCurrent?.() ?? true;
  const revision = useUiRevision(() => [table, selection, draft, current(), error]);
  const change = (format: NumberFormat, actor?: UiActor) => {
    if (!selection || !current()) { setError("Select a region in the current writable source before formatting."); return false; }
    const next = { ...table, formats: { ...table.formats } };
    if (draft) next.rows = withCell(table, draft.row, draft.column, draft.value).rows;
    for (const at of cells) next.formats[cellLabel(at.column, at.row)] = format;
    return actor ? onChange(next, actor) !== false : onChange(next) !== false;
  };
  useUiCommands([{ key: parent, command: { label: "Set table number format", parent, subject: parent, parameters: { format: { type: "string", maxLength: 64, choices: Object.keys(NUMBER_FORMATS) } }, prerequisites: [], read: () => ({ revision: revision(), enabled: !!selection && current(), reason: !selection ? "Select table cells first" : !current() ? "Owning source changed/closed/read-only" : undefined, value: { owner: owner ?? null, error: error || null, format: values.size === 1 ? [...values][0] : selection ? "mixed" : "general" } }), execute: async (args, actor) => change(args.format as NumberFormat, actor) ? { status: "completed", value: { source_write: "format handler dispatched", saved: "not established" } } : { status: "unavailable", reason: "Owning source refused the number format" } } }]);
  return <><TableNumberFormat disabled={!selection || !current()}
    value={values.size === 1 ? [...values][0] : selection ? "mixed" : "general"}
    onChange={format => { change(format); }} />{error && <span role="alert">{error}</span>}</>;
}
</book:entry>

</book:doc>
