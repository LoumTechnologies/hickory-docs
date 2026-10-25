<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/MarkdownTable.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Cell formatting keeps numbers numeric](../../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md)
- [Markdown tables use the table editor](../../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md)

Captured SHA-256: `05a39a4dbca31dd4a3d9693b501ebe55d1c2a91d87b234ad3e449892da538af9`.

<book:entry path="apps/web/src/components/MarkdownTable.tsx">
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
</book:entry>

</book:doc>
