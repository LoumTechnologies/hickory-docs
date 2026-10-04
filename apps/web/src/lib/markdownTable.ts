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

export function markdownRows(source: string): string[][] {
  const lines = source.split(/\r?\n/);
  return [markdownCells(lines[0]), ...lines.slice(2).filter(Boolean).map(markdownCells)];
}

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
