<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Markdown tables use the table editor

For: Authors editing executable documents and the files those documents produce.

Given a top-level Markdown pipe table in a document's prose,
when the document opens,
then the table uses the same grid as a CSV table, with the first row as its
header and a rail action to reveal its Markdown source.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/markdown-tables-use-the-table-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

We already adapt Markdown rows into the CSV grid and serialize edits back into
pipe syntax. Keep that shared surface: the document-editor witness below checks
that an edit follows a table moved by prose, preserves alignment, and participates
in ordinary undo before revealing source. The settings witness waits for the
shared checkbox callback to finish before toggling again; a pending control
cannot accept a second click.


<a id="output-66a333e642b4"></a>
## `apps/web/src/components/MarkdownTable.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown tables use the table editor](contract.md).

<a id="f-66a333e642b4-source"></a>
### `source`

<storycode:copy id="f-66a333e642b4-source">
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
</storycode:copy>
<a id="f-66a333e642b4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-66a333e642b4-expression_statement">
afterEach(cleanup);

</storycode:copy>
<a id="f-66a333e642b4-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-66a333e642b4-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/components/MarkdownTable.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MarkdownTable.test.tsx"><storycode:paste select="#f-66a333e642b4-source"/><storycode:paste select="#f-66a333e642b4-expression_statement"/><storycode:paste select="#f-66a333e642b4-expression_statement-2"/></storycode:file>

<a id="output-32a3bcb3ccd7"></a>
## `apps/web/src/editor/markdownTables.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown tables use the table editor](contract.md).

<a id="f-32a3bcb3ccd7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-32a3bcb3ccd7-expression_statement">
// Guarantee: docs/guarantees/authoring/markdown-tables-use-the-table-editor.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { DocumentEditor } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { showBlockSource } from "./rendered";
import { undo } from "@codemirror/commands";

afterEach(cleanup);

</storycode:copy>
<a id="f-32a3bcb3ccd7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-32a3bcb3ccd7-expression_statement-2">
it("opens a Markdown table as a grid, keeps edits after prose moves it, and reveals source", async () => {
  const onChange = vi.fn();
  const source = 'Intro.\n\n| Name | Count |\n| :--- | ---: |\n| North | 120 |\n\nAfter.';
  const { container } = render(<DocumentEditor docId="markdown-grid" initialSource={source}
    realtime={new LocalRealtime()} execBlocks={[]} runningCells={new Set()}
    onRunCell={() => undefined} onChange={onChange} />);
  await waitFor(() => expect(screen.getByTestId("table-panel")).toBeTruthy());
  const view = EditorView.findFromDOM(container.querySelector('.cm-editor')!)!;
  view.dispatch({ changes: { from: 0, insert: '# Notes\n' } });
  fireEvent.doubleClick(screen.getByText('120'));
  const input = screen.getByRole('textbox', { name: 'Cell contents' });
  fireEvent.change(input, { target: { value: '121' } });
  fireEvent.keyDown(input, { key: 'Enter' });
  await waitFor(() => expect(view.state.doc.toString()).toContain('| North | 121 |'));
  expect(view.state.doc.toString()).toBe('# Notes\n' + source.replace('120', '121'));
  expect(onChange).toHaveBeenLastCalledWith(view.state.doc.toString());
  expect(screen.getByTestId('table-panel')).toBeTruthy();
  // Editing the header replaces the table's first bytes too, and must keep
  // its rendered state and mounted grid rather than discard the fold.
  fireEvent.doubleClick(screen.getByText('Name'));
  const header = screen.getByRole('textbox', { name: 'Cell contents' });
  fireEvent.change(header, { target: { value: 'Region' } });
  fireEvent.keyDown(header, { key: 'Enter' });
  await waitFor(() => expect(view.state.doc.toString()).toContain('| Region | Count |'));
  expect(screen.getByTestId('table-panel')).toBeTruthy();
  // The shared grid commits a document transaction, so ordinary editor undo
  // restores the header without losing the table viewer.
  expect(undo(view)).toBe(true);
  await waitFor(() => expect(view.state.doc.toString()).toContain('| Name | Count |'));
  expect(view.state.doc.toString()).toBe('# Notes\n' + source.replace('120', '121'));
  expect(screen.getByTestId('table-panel')).toBeTruthy();
  const at = view.state.doc.toString().indexOf('| Name');
  view.dispatch({ effects: showBlockSource.of(at) });
  await waitFor(() => expect(screen.queryByTestId('table-panel')).toBeNull());
  expect(container.querySelector('.cm-content')?.textContent).toContain('| Name | Count |');
});

it("keeps the Markdown grid viewable without accepting edits in a read-only document", async () => {
  const source = '| Name | Count |\n| --- | --- |\n| North | 120 |';
  const onChange = vi.fn();
  const { container } = render(<DocumentEditor docId="markdown-readonly" initialSource={source}
    realtime={new LocalRealtime()} execBlocks={[]} runningCells={new Set()}
    readOnly onRunCell={() => undefined} onChange={onChange} />);
  await waitFor(() => expect(screen.getByTestId("table-panel")).toBeTruthy());
  onChange.mockClear(); // Initial room hydration reports the unchanged source.
  fireEvent.doubleClick(screen.getByText('120'));
  expect((screen.getByRole('textbox', { name: 'Cell contents' }) as HTMLInputElement).disabled).toBe(true);
  const view = EditorView.findFromDOM(container.querySelector('.cm-editor')!)!;
  expect(view.state.doc.toString()).toBe(source);
  expect(onChange).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/editor/markdownTables.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/markdownTables.test.tsx"><storycode:paste select="#f-32a3bcb3ccd7-expression_statement"/><storycode:paste select="#f-32a3bcb3ccd7-expression_statement-2"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/MarkdownTable.tsx](../../../mechanisms/apps/web/src/components/MarkdownTable.tsx.md#output-04adf2e0e8b7)
- [apps/web/src/editor/cards.ts](../../../mechanisms/apps/web/src/editor/cards.ts.md#output-84385185ee44)
- [apps/web/src/editor/markdownTables.ts](../../../mechanisms/apps/web/src/editor/markdownTables.ts.md#output-6f87bd812952)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/elements/index.ts](../../../mechanisms/apps/web/src/elements/index.ts.md#output-9b7b2fa9b3c3)
- [apps/web/src/lib/markdownTable.ts](../../../mechanisms/apps/web/src/lib/markdownTable.ts.md#output-104f3986928c)

### Tests and fixtures

- [apps/web/src/components/MarkdownTable.test.tsx](contract.md#output-66a333e642b4)
- [apps/web/src/editor/markdownTables.test.tsx](contract.md#output-32a3bcb3ccd7)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
