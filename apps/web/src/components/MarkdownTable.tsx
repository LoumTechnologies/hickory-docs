import { TablePanel, type TablePanelProps } from "./TablePanel";
import { parseCsv, writeCsv } from "../lib/csv";
import { splitFormatComment, withFormatComment } from "../lib/tableFormats";
import { markdownRows, writeMarkdownTable } from "../lib/markdownTable";

/** The existing spreadsheet surface, backed by Markdown rather than CSV. */
export function MarkdownTable({ source, onChange, ...props }: TablePanelProps) {
  const { body, formats } = splitFormatComment(source);
  const csv = writeCsv({ rows: markdownRows(body), delimiter: ",", newline: "\n", trailingNewline: false });
  return <TablePanel fitProseDefault {...props} source={csv} formats={formats} delimiter="," header
    onChange={onChange ? (next, formats = "", actor) => { const text = withFormatComment(writeMarkdownTable(body, parseCsv(next, ",").rows), formats); if (actor) onChange(text, undefined, actor); else onChange(text); } : undefined} />;
}
