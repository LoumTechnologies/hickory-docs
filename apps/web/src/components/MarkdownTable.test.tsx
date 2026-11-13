// Guarantee: docs/guarantees/authoring/markdown-tables-use-the-table-editor.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MarkdownTable, mountNativeTable } from "./MarkdownTable";
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
  it("fits the prose width by default, until a column is resized", () => {
    const onLayout = vi.fn();
    const { container } = render(<MarkdownTable source={source} onLayout={onLayout} />);
    expect(container.querySelector(".table-panel--fit-prose")).not.toBeNull();
    const handle = container.querySelector('[data-testid="column-resizer"]')!;
    fireEvent.mouseDown(handle, { clientX: 100 });
    fireEvent.mouseMove(window, { clientX: 130 });
    fireEvent.mouseUp(window);
    expect(container.querySelector(".table-panel--fit-prose")).toBeNull();
    expect(onLayout).toHaveBeenLastCalledWith(expect.objectContaining({ fitProse: false, widths: { "0": 134 } }));
  });

  it("honors remembered widths and the gear setting in both directions", async () => {
    const onLayout = vi.fn();
    const { container } = render(<MarkdownTable source={source} layout={{ widths: { "0": 200 } }} onLayout={onLayout} />);
    expect(container.querySelector(".table-panel--fit-prose")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Table settings" }));
    const toggle = screen.getByRole("checkbox", { name: "Wrap to prose width" });
    fireEvent.click(toggle);
    expect(container.querySelector(".table-panel--fit-prose")).not.toBeNull();
    expect(onLayout).toHaveBeenLastCalledWith(expect.objectContaining({ fitProse: true }));
    await waitFor(() => expect((toggle as HTMLInputElement).disabled).toBe(false));
    fireEvent.click(toggle);
    expect(container.querySelector(".table-panel--fit-prose")).toBeNull();
    expect(onLayout).toHaveBeenLastCalledWith(expect.objectContaining({ fitProse: false }));
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

describe("locally hosted native spreadsheet", () => {
  it("edits Markdown through a revisioned host request and preserves selection and height after acknowledgement", async () => {
    const host = document.createElement("div"); document.body.append(host);
    const messages: Record<string, unknown>[] = [];
    let bridge!: ReturnType<typeof mountNativeTable>;
    await act(async () => { bridge = mountNativeTable(host, message => messages.push(message)); bridge.update({ revision: "one", source, format: "markdown", readonly: false }); });
    try {
      fireEvent.click(screen.getByText("120"));
      fireEvent.keyDown(screen.getByLabelText("Table height"), { key: "Home" });
      fireEvent.doubleClick(screen.getByText("120"));
      const input = screen.getByRole("textbox", { name: "Cell contents" });
      fireEvent.change(input, { target: { value: "121" } }); fireEvent.keyDown(input, { key: "Enter" });
      expect(messages).toContainEqual({ kind: "change", revision: "one", source: source.replace("120", "121") });
      await act(async () => { bridge.update({ revision: "two", source: source.replace("120", "121"), format: "markdown", readonly: false }); });
      expect(screen.getByText("121")).toBeTruthy();
      expect(host.querySelector<HTMLElement>(".table-panel__scroll")?.style.height).toBe("64px");
      const before = messages.filter(event => event.kind === "change").length;
      await act(async () => { bridge.update({ revision: "three", source, format: "markdown", readonly: true }); });
      fireEvent.doubleClick(screen.getByText("120"));
      expect(messages.filter(event => event.kind === "change")).toHaveLength(before);
    } finally { await act(async () => bridge.dispose()); host.remove(); }
  });
  it("uses the native formula transport and ignores an obsolete result after source changes", async () => {
    const host = document.createElement("div"); document.body.append(host);
    const messages: Record<string, unknown>[] = [];
    let bridge!: ReturnType<typeof mountNativeTable>;
    await act(async () => { bridge = mountNativeTable(host, message => messages.push(message)); bridge.update({ revision: "one", source: "=1+1", format: "csv", readonly: false, language: "python" }); });
    try {
      await waitFor(() => expect(messages.some(event => event.operation === "evaluate")).toBe(true));
      const request = messages.find(event => event.operation === "evaluate")!;
      expect(request.rows).toEqual([["=1+1"]]);
      await act(async () => bridge.update({ revision: "two", source: "current", format: "csv", readonly: false, language: "python" }));
      await act(async () => bridge.reply(request.id as number, { values: { A1: "2" }, errors: {} }));
      expect(screen.getByRole("gridcell", { name: "current" })).toBeTruthy();
      expect(screen.queryByRole("gridcell", { name: "2" })).toBeNull();
    } finally { await act(async () => bridge.dispose()); host.remove(); }
  });
});

it("returns a failed native rendering capability to its host without writing source", async () => {
  // Contract: .hickory-source/frontends/contract.md
  const host = document.createElement("div"); document.body.append(host);
  const messages: Record<string, unknown>[] = [];
  const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
  let bridge!: ReturnType<typeof mountNativeTable>;
  await act(async () => { bridge = mountNativeTable(host, message => messages.push(message)); bridge.update({ revision: "one", source: "data", format: "csv", readonly: false }); });
  try {
    await waitFor(() => expect(messages.some(message => message.operation === "languages")).toBe(true));
    const request = messages.find(message => message.operation === "languages")!;
    await act(async () => bridge.reply(request.id as number, { languages: null }));
    await waitFor(() => expect(messages).toContainEqual({ kind: "unavailable" }));
    expect(messages.some(message => message.kind === "change")).toBe(false);
  } finally { await act(async () => bridge.dispose()); host.remove(); consoleError.mockRestore(); }
});
