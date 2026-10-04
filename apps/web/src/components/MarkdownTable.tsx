import { TablePanel, type TablePanelProps } from "./TablePanel";
import { parseCsv, writeCsv } from "../lib/csv";
import { markdownRows, writeMarkdownTable } from "../lib/markdownTable";

/** The existing spreadsheet surface, backed by Markdown rather than CSV. */
export function MarkdownTable({ source, onChange, ...props }: TablePanelProps) {
  const csv = writeCsv({ rows: markdownRows(source), delimiter: ",", newline: "\n", trailingNewline: false });
  return <TablePanel fitProseDefault {...props} source={csv} delimiter="," header
    onChange={onChange ? next => onChange(writeMarkdownTable(source, parseCsv(next, ",").rows)) : undefined} />;
}
