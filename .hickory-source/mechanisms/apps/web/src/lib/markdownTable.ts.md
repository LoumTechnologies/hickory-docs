<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/markdownTable.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-104f3986928c"></a>
## `apps/web/src/lib/markdownTable.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown tables use the table editor](../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md).

<a id="f-104f3986928c-markdownCells"></a>
### `markdownCells`

Pipe-table syntax is prose; the grid edits it without changing its format.

<storycode:copy id="f-104f3986928c-markdownCells">
/** Pipe-table syntax is prose; the grid edits it without changing its format. */
export function markdownCells(line: string): string[] {
  let text = line.trim();
  if (text.startsWith("|")) text = text.slice(1);
  if (/(?<!\\)\|$/.test(text)) text = text.slice(0, -1);
  const cells: string[] = [];
  let cell = "";
  for (let i = 0; i < text.length; i++) {
    if (text[i] === "\\" && text[i + 1] === "|") { cell += "|"; i++; }
    else if (text[i] === "|") { cells.push(cell.trim()); cell = ""; }
    else cell += text[i];
  }
  cells.push(cell.trim());
  return cells;
}

</storycode:copy>
<a id="f-104f3986928c-markdownRows"></a>
### `markdownRows`

<storycode:copy id="f-104f3986928c-markdownRows">
export function markdownRows(source: string): string[][] {
  const lines = source.split(/\r?\n/);
  return [markdownCells(lines[0]), ...lines.slice(2).filter(Boolean).map(markdownCells)];
}

</storycode:copy>
<a id="f-104f3986928c-writeMarkdownTable"></a>
### `writeMarkdownTable`

Keep unchanged lines and the delimiter row byte-for-byte where possible.

<storycode:copy id="f-104f3986928c-writeMarkdownTable">
/** Keep unchanged lines and the delimiter row byte-for-byte where possible. */
export function writeMarkdownTable(source: string, rows: string[][]): string {
  const newline = source.includes("\r\n") ? "\r\n" : "\n";
  const lines = source.split(/\r?\n/);
  const old = markdownRows(source);
  const width = Math.max(1, ...rows.map(row => row.length));
  const same = (a: string[], b?: string[]) => b && a.length === b.length && a.every((v, i) => v === b[i]);
  const encode = (v: string) => v.replace(/\|/g, "\\|").replace(/\r?\n/g, "<br>");
  const output = rows.map((row, i) => same(row, old[i]) && (i !== 0 || row.length === width) ? lines[i === 0 ? 0 : i + 1] :
    `${lines[0].match(/^ */)?.[0] ?? ""}| ${Array.from({ length: width }, (_, col) => encode(row[col] ?? "")).join(" | ")} |`);
  const separators = markdownCells(lines[1]);
  output.splice(1, 0, width === separators.length ? lines[1] :
    `| ${Array.from({ length: width }, (_, col) => separators[col] ?? "---").join(" | ")} |`);
  return output.join(newline);
}
</storycode:copy>

### Assemble `apps/web/src/lib/markdownTable.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/markdownTable.ts"><storycode:paste select="#f-104f3986928c-markdownCells"/><storycode:paste select="#f-104f3986928c-markdownRows"/><storycode:paste select="#f-104f3986928c-writeMarkdownTable"/></storycode:file>

</storycode:doc>
