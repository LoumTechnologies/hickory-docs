// Guarantee: docs/guarantees/authoring/markdown-tables-use-the-table-editor.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { MarkdownTable } from "./MarkdownTable";
import { markdownCells, markdownRows, writeMarkdownTable } from "../lib/markdownTable";
import { markdownTableBlocks } from "../editor/markdownTables";
import { parseHickDoc } from "../editor/hickDoc";
import { cardsOf } from "../editor/cards";
import { renderableBlocks } from "../elements";

const source = '| Name | Count |\n| :--- | ---: |\n| North | 120 |';
afterEach(cleanup);

describe("Markdown tables", () => {
  it("edits through the existing grid and writes Markdown with alignment intact", () => {
    const onChange = vi.fn();
    render(<MarkdownTable source={source} onChange={onChange} />);
    fireEvent.doubleClick(screen.getByText("120"));
    const input = screen.getByRole("textbox", { name: /cell/i });
    fireEvent.change(input, { target: { value: "121" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith(source.replace("120", "121"));
  });
  it("round trips quotes, commas, escaped pipes and inline code", () => {
    const text = '| Snippet | Value |\n| --- | --- |\n| `a\\|b` | "hi", there |';
    expect(markdownRows(text)[1]).toEqual(['`a|b`', '"hi", there']);
    expect(writeMarkdownTable(text, markdownRows(text))).toBe(text);
    expect(markdownCells('| one | two |')).toEqual(['one', 'two']);
  });
  it("escapes edited pipes and keeps other rows and CRLF", () => {
    const text = source.replaceAll('\n', '\r\n');
    const rows = markdownRows(text);
    rows[1][0] = 'A|B';
    expect(writeMarkdownTable(text, rows)).toBe(text.replace('North', 'A\\|B'));
  });
  it("keeps a valid delimiter row when columns or rows change", () => {
    const rows = markdownRows(source);
    rows.forEach(r => r.push('new'));
    rows.push(['South', '20', '']);
    const output = writeMarkdownTable(source, rows);
    expect(markdownRows(output)).toEqual(rows);
    expect(markdownCells(output.split('\n')[1])).toEqual([':---', '---:', '---']);
  });
  it("finds prose tables for both rendering and the source toggle", () => {
    const text = 'Intro.\n\n' + source + '\n\nAfter.';
    const structure = parseHickDoc(text);
    const [block] = markdownTableBlocks(structure, text);
    expect(text.slice(block.from, block.to)).toBe(source);
    expect(renderableBlocks(structure, text)).toHaveLength(1);
    expect(cardsOf(structure, { text }).map(c => c.kind)).toEqual(['table']);
  });
  it("excludes code, unfinished fences, indented code and verbatim hick bodies", () => {
    for (const text of ['```md\n' + source + '\n```', '```md\n' + source,
      '<hick:file path="x.md">\n' + source + '\n</hick:file>',
      source.split('\n').map(l => '    ' + l).join('\n')]) {
      expect(markdownTableBlocks(parseHickDoc(text), text)).toEqual([]);
    }
  });
  it("uses Markdown block rules, including optional outer pipes and one-cell body rows", () => {
    const text = 'Name | Count\n--- | ---\nNorth\nSouth | 20';
    const [block] = markdownTableBlocks(parseHickDoc(text), text);
    expect(text.slice(block.from, block.to)).toBe(text);
    expect(markdownRows(text)).toEqual([['Name', 'Count'], ['North'], ['South', '20']]);
    for (const invalid of ['# Heading | Text\n--- | ---', '> A | B\n> --- | ---']) {
      expect(markdownTableBlocks(parseHickDoc(invalid), invalid)).toEqual([]);
    }
    const wrapped = '<hick:doc>\n' + source + '\n</hick:doc>';
    expect(markdownTableBlocks(parseHickDoc(wrapped), wrapped)).toHaveLength(1);
  });
  it("does not consume blank lines or mistake a mismatched delimiter for a table", () => {
    const text = source + '\n\nMore | prose';
    expect(markdownTableBlocks(parseHickDoc(text), text)[0].to).toBe(source.length);
    const invalid = '| A | B |\n| --- |';
    expect(markdownTableBlocks(parseHickDoc(invalid), invalid)).toEqual([]);
  });
});
