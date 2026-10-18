// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ReadOnlyTable, type NotebookTable } from "./ReadOnlyTable";
import { CellPanel } from "./CellPanel";
import { DatasetPanel } from "./DatasetPanel";
import { NotebookOutputs } from "./NotebookOutputs";
import { notebookOutputs, DISPLAY_PREFIX } from "../lib/notebookOutput";
import { cardsOf } from "../editor/cards";
import { parseHickDoc } from "../editor/hickDoc";

afterEach(() => { cleanup(); vi.unstubAllGlobals(); });
const table: NotebookTable = { kind: "table", schema: [{ name: "sales", dtype: "Int64" }], rows: [[20], [null], [3]], offset: 0, has_more: false };
describe("Notebook outputs", () => {
  it("gives a dataset reference a viewer action and keeps XML-style Python cells runnable", () => {
    const source = '<hick:dataset src="data.parquet" />\n\n```python exec="true" id="summary"\ndisplay(1)\n```\n';
    const cards = cardsOf(parseHickDoc(source), { text: source });
    expect(cards.map(c => c.kind)).toEqual(["dataset", "exec"]);
  });

  it("shows a completed rich table without an empty terminal underneath the source", () => {
    const { container } = render(<CellPanel block={{ kind: "exec", id: "summary", container: "default", command: "display(result)", span: [0, 10], status: "ok", transcript: [{ t: 0, kind: "cmd", data: "python3 -" }, { t: 1, kind: "out", data: `${DISPLAY_PREFIX}${JSON.stringify(table)}\n` }, { t: 2, kind: "exit", code: 0 }] }} />);
    expect(screen.getByRole("table")).toBeTruthy();
    expect(container.querySelector(".transcript")).toBeNull();
  });
  it("reopens a saved table without requiring a local execution transcript", () => {
    render(<CellPanel block={{ kind: "exec", id: "summary", container: "default", command: "display(result)", span: [0, 10], status: "unrecorded" }} recordedOutput={`${DISPLAY_PREFIX}${JSON.stringify(table)}\n`} />);
    expect(screen.getByRole("table")).toBeTruthy();
    expect(screen.getByText("Saved output")).toBeTruthy();
  });
  it("reopens ordinary printed output too", () => {
    render(<CellPanel block={{ kind: "exec", id: "summary", container: "default", command: "print('plain text')", span: [0, 10], status: "unrecorded" }} recordedOutput="plain text" />);
    expect(screen.getByText("plain text")).toBeTruthy();
  });
  it("keeps typed numbers, nulls and nested data read-only, with page filtering and numeric sorting", () => {
    const { container } = render(<ReadOnlyTable table={table} />);
    expect(screen.getByText("Int64")).toBeTruthy();
    expect(screen.getByText("null")).toBeTruthy();
    expect(container.querySelector("[contenteditable], textarea")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "sales" }));
    expect([...container.querySelectorAll("tbody td")].map(n => n.textContent)).toEqual(["3", "20", "null"]);
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "20" } });
    expect(container.querySelectorAll("tbody tr")).toHaveLength(1);
  });
  it("decodes multiple rich records after stdout chunks have been joined, preserving text", () => {
    const result = notebookOutputs(`hello\n${DISPLAY_PREFIX}${JSON.stringify(table)}\n${DISPLAY_PREFIX}{"kind":"text","text":"bye"}\n`);
    expect(result.outputs).toHaveLength(2);
    expect(result.text).toBe("hello\n");
    render(<NotebookOutputs outputs={result.outputs} />);
    expect(screen.getByText("bye")).toBeTruthy();
  });
  it("keeps invalid records visible and renders SVG as an image with no inline DOM execution", () => {
    expect(notebookOutputs(`${DISPLAY_PREFIX}broken`).text).toContain("broken");
    const data = btoa('<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"></svg>');
    const { container } = render(<NotebookOutputs outputs={[{ kind: "image", mime: "image/svg+xml", data }]} />);
    expect(container.querySelector("svg")).toBeNull();
    expect(screen.getByRole("img").getAttribute("src")).toContain("data:image/svg+xml;base64,");
  });
  it("pages through dataset previews, resolves the document path, and retries a failed request", async () => {
    const fetcher = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify({ ...table, has_more: true }))).mockResolvedValueOnce(new Response("unavailable", { status: 422 })).mockResolvedValue(new Response(JSON.stringify(table)));
    vi.stubGlobal("fetch", fetcher);
    render(<DatasetPanel src="data/sales.parquet" format="parquet" path="notes/report.md" />);
    await screen.findByText("Int64");
    expect(fetcher.mock.calls[0][0]).toContain("doc_path=notes%2Freport.md");
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await screen.findByRole("alert");
    expect(fetcher.mock.calls[1][0]).toContain("offset=100");
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    await screen.findByText("Int64");
  });
  it("loads S3 only after the reader requests it", async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify(table)));
    vi.stubGlobal("fetch", fetcher);
    render(<DatasetPanel src="s3://bucket/table" format="delta" path={null} />);
    expect(fetcher).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Load from S3" }));
    await waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
  });
});
