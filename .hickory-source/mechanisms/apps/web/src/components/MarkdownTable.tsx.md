<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MarkdownTable.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-04adf2e0e8b7"></a>
## `apps/web/src/components/MarkdownTable.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md), [Markdown tables use the table editor](../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md).

<a id="f-04adf2e0e8b7-MarkdownTable"></a>
### `MarkdownTable`

<storycode:copy id="f-04adf2e0e8b7-MarkdownTable">
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
</storycode:copy>

### Assemble `apps/web/src/components/MarkdownTable.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MarkdownTable.tsx"><storycode:paste select="#f-04adf2e0e8b7-MarkdownTable"/></storycode:file>

</storycode:doc>
