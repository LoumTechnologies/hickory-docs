// Contract: .hickory-source/frontends/contract.md
import { createElement, Fragment } from "react";
import { within } from "@testing-library/react";
import { readFileSync } from "node:fs";
import katex from "katex";
import { RichOwners } from "./RichInline";
import { breakpointLine, patchDebugTargets } from "../debug/cmDebug";
import { parseHickDoc, proseFences } from "../editor/hickDoc";
// Vitest stubs CSS by default; supply the actual local asset to exercise its shadow-root placement.
vi.mock("@xyflow/react/dist/style.css?inline", async () => ({ default: new TextDecoder().decode((await import("node:fs")).readFileSync("node_modules/@xyflow/react/dist/style.css")) }));
import * as hickParser from "../editor/hickLang";
import { render, waitFor, act, fireEvent } from "@testing-library/react";
import { smartEnter } from "./SmartEnter";
import enterCases from "../../../simple-contract/enter-cases.json";
import { EditorView } from "@codemirror/view";
import { EditorSelection, EditorState } from "@codemirror/state";
import { foldedRanges } from "@codemirror/language";
import { undo, isolateHistory } from "@codemirror/commands";
import { currentCompletions } from "@codemirror/autocomplete";
import { Document, simpleWordOffset, simpleWordMove, simpleFoldRanges, openFind, simpleIndent, simpleTab } from "./Document";
vi.mock("../debug/launchProfiles", () => ({ launchWithProfiles: vi.fn(async (_query, start, options) => { if (options.isCurrent()) start({ id: "test", fingerprint: "test" }); }) }));
import { launchWithProfiles } from "../debug/launchProfiles";
import { IDLE_SESSION } from "../debug/useDebugger";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
vi.mock("../debug/plainDebugHosts", () => ({ useFollowingDebugSession: vi.fn(() => null), PlainDebugHosts: () => null }));
import { MarkdownMessage } from "./Agent";
import { App } from "./App";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { LspClient } from "../lsp/client";
import { createLspChannel, decodeLspFrame, encodeLspFrame } from "../lsp/channel";
vi.mock("../lsp/useLsp", () => ({ useWorkspaceLsp: vi.fn(() => ({ client: null, uri: "hick:///sample.ts", ready: false, diagnostics: [] })) }));
import { describe, expect, it, vi } from "vitest";
import fixture from "../../../simple-contract/layout-cases.json";
import { activate, groups, initialLayout, move, remove, split, dock, dropEdge, isGroup, type Edge, type Layout } from "./layout";
import { TerminalSession } from "./Terminal";
import { Buffer, Buffers } from "./buffers";
import { api } from "../api/client";
vi.mock("../api/client", () => ({ api: { files: vi.fn(), doc: vi.fn(), file: vi.fn(), saveFile: vi.fn(), saveDoc: vi.fn(), fileOp: vi.fn(), openFile: vi.fn(), createDoc: vi.fn(), openTerminal: vi.fn(), terminalInput: vi.fn(), agent: vi.fn() } }));
describe("shared frontend layout contract", () => {
  it("constructs patch fences and offers source-mapped patch debugging in Simple", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const source = "<hick:project/>\n<hick:file id=\"base\" path=\"x.py\">\nprint(1)\n</hick:file>\n```patch id=\"change\" after=\"base\"\n--- a/x.py\n+++ b/x.py\n@@ -1 +1 @@\n-print(1)\n+print(\"café 😀\")\n```\n```python path=\"other.py\"\nprint(3)\n```\n```diff\n--- a/fake.py\n+++ b/fake.py\n@@ -1 +1 @@\n-a\n+b\n```";
    expect(proseFences(parseHickDoc(source), source).map(fence => fence.info)).toEqual(["diff"]);
    const workbook = new TextDecoder().decode(readFileSync("../../examples/simple-ui/workbook.md"));
    const demo = hickParser.materializeProject("workbook.md", { "workbook.md": workbook });
    expect(new TextDecoder().decode(new Uint8Array(demo.new.find(entry => entry.path === "generated/coupon.py")!.bytes))).toContain("Coupon receipt:");
    const project = hickParser.materializeProject("project.md", { "project.md": source });
    expect(new TextDecoder().decode(new Uint8Array(project.new.find(entry => entry.path === "x.py")!.bytes))).toBe('print("café 😀")\n');
    const buffer = new Buffer({ path: "patch.md", name: "patch.md", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      expect(patchDebugTargets(view.state).map(target => target.path)).toEqual(["x.py"]);
      expect(breakpointLine(view.state, 9)).toBe(9);
      expect(breakpointLine(view.state, 8)).toBeNull();
      expect(breakpointLine(view.state, 19)).toBeNull();
      expect(breakpointLine(view.state, 12)).toBe(12);
      fireEvent.click(screen.getAllByRole("button", { name: "Debug x.py" }).at(-1)!);
      await waitFor(() => expect(launchWithProfiles).toHaveBeenCalledWith(expect.objectContaining({ program: "x.py", expected_document: source }), expect.any(Function), expect.any(Object)));
      expect(buffer.text).toBe(source);
      expect(buffer.dirty).toBe(false);
    } finally { screen.unmount(); }
  });
  it("highlights ordinary fenced Python in the retained Simple source editor", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const source = "Prose\n```python\ndef coupon():\n    return \"café 😀\"\n```";
    const buffers = new Buffers(() => {}), buffer = new Buffer({ path: "fenced.md", name: "fenced.md", dir: false }, source, "hash", () => {});
    buffers.values.set(buffer.file.path, buffer);
    const screen = render(createElement(Fragment, null, createElement(Document, { buffer }), createElement(RichOwners, { buffers })));
    try {
      await waitFor(() => expect(screen.container.querySelector(".cm-content .tok-keyword")?.textContent).toBe("def"), { timeout: 10000 });
      expect(screen.container.querySelector(".cm-content .tok-string")?.textContent).toContain("café");
      expect(buffer.text).toBe(source);
      expect(buffer.dirty).toBe(false);
      expect(screen.queryByRole("button", { name: /Debug .*coupon/ })).toBeNull();
    } finally { screen.unmount(); }
  }, 20000);
  it("includes graph canvas styles in rendered Simple shadow roots", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const buffers = new Buffers(() => {});
    const source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:diagram renderer="graph">{ "topology": { "nodes": [{"id":"a"}], "edges": [] } }</h:diagram>\n</h:doc>';
    const buffer = new Buffer({ path: "diagram.md", name: "diagram.md", dir: false }, source, "hash", () => {});
    buffers.values.set(buffer.file.path, buffer);
    const screen = render(createElement(Fragment, null, createElement(Document, { buffer }), createElement(RichOwners, { buffers })));
    try {
      await waitFor(() => {
        const root = [...screen.container.querySelectorAll("*")].map(node => node.shadowRoot).find(root => root?.querySelector("style"));
        expect(root?.querySelector("style")?.textContent).toContain(".react-flow__renderer");
      }, { timeout: 10000 });
      expect(buffer.text).toBe(source);
      expect(buffer.dirty).toBe(false);
    } finally { screen.unmount(); }
  }, 20000);
  it("renders native graph previews safely and rejects obsolete Mermaid results", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const html = new TextDecoder().decode(readFileSync("../apple-simple/Sources/Resources/math.html"));
    const document = new DOMParser().parseFromString(html, "text/html");
    const scope: any = {};
    const pending: ((value: { svg: string }) => void)[] = [];
    const mermaid = { initialize: vi.fn(), render: vi.fn(() => new Promise(resolve => pending.push(resolve as any))) };
    scope.mermaid = mermaid;
    const script = html.match(/<script nonce="hickory-math">([\s\S]*?)<\/script>/)![1];
    new Function("document", "globalThis", "mermaid", script)(document, scope, mermaid);
    await scope.renderDiagram(JSON.stringify({ topology: { nodes: [{ id: "a", label: "<img src=x onerror=alert(1)>" }, { id: "b" }], edges: [{ from: "a", to: "b" }] } }), "graph");
    expect(document.querySelectorAll("svg rect")).toHaveLength(2);
    expect(document.querySelector("img")).toBeNull();
    expect(document.querySelector("svg text")?.textContent).toContain("<img");
    await scope.renderDiagram("broken", "graph");
    expect(document.getElementById("status")?.textContent).not.toBe("");
    const old = scope.renderDiagram("graph TD; A-->B", "mermaid");
    const fresh = scope.renderDiagram("graph TD; X-->Y", "mermaid");
    pending[1]({ svg: "<svg><text>fresh</text></svg>" }); await fresh;
    pending[0]({ svg: "<svg><text>old</text></svg>" }); await old;
    expect(document.getElementById("formula")?.textContent).toBe("fresh");
    expect(mermaid.initialize).toHaveBeenCalledWith(expect.objectContaining({ securityLevel: "strict" }));
  });
  it("folds Markdown headings through peers and excludes literal bodies", () => {
    // Contract: .hickory-source/frontends/contract.md
    const source = "# A\nintro 😀\n## B\nbody\n```python\n# comment\n```\n<h:file path=\"x.py\">\n# literal\n</h:file>\n## C\nlast\nNext\n====\nend";
    const folds = simpleFoldRanges(source, true);
    expect(folds.map(fold => fold.line)).toEqual([0, 2, 10, 12]);
    expect(folds[0].to).toBe(source.indexOf("Next") - 1);
    expect(source.slice(folds[1].from, folds[1].to)).toContain("# literal");
    expect(simpleFoldRanges("# comment\nbody")).toEqual([]);
    expect(simpleFoldRanges("```python\n# comment\nbody", true)).toEqual([]);
  });
  it("applies the shared smart Enter language, context and mode fixtures", () => {
    // Contract: .hickory-source/frontends/contract.md
    for (const item of enterCases.cases) {
      const from = item.before.indexOf("¦"), source = item.before.replace("¦", "");
      const plan = smartEnter(source, from, from, item.path, item.unit, item.mode as "smart" | "copy" | "none");
      const updated = source.slice(0, plan.from) + plan.text + source.slice(plan.to);
      expect.soft(updated.slice(0, plan.caret) + "¦" + updated.slice(plan.caret), item.name).toBe(item.after);
    }
    const long = "x\n".repeat(32769) + "if ok:";
    expect(smartEnter(long, long.length, long.length, "x.py", "    ", "smart").text).toBe("\n");
    const line = "x".repeat(4097) + " {";
    expect(smartEnter(line, line.length, line.length, "x.js", "    ", "smart").text).toBe("\n");
  });
  it("uses live Enter settings in the mounted document with one-step undo", () => {
    // Contract: .hickory-source/frontends/contract.md
    const old = localStorage.getItem("editor.enterIndent"), source = "class C:\n    def f():";
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [] });
    const buffer = new Buffer({ path: "enter.py", name: "enter.py", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Fragment, null, createElement(App), createElement(Document, { buffer })));
    try {
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      for (const [mode, indent] of [["smart", "        "], ["copy", "    "], ["none", ""]]) {
        fireEvent.change(screen.getByRole("combobox", { name: "Enter indentation", hidden: true }), { target: { value: mode } });
        expect(localStorage.getItem("editor.enterIndent")).toBe(mode);
        act(() => view.dispatch({ selection: { anchor: source.length } }));
        fireEvent.keyDown(view.contentDOM, { key: "Enter", code: "Enter" });
        expect(buffer.text).toBe(source + "\n" + indent);
        expect(view.state.selection.main.head).toBe(buffer.text.length);
        act(() => { expect(undo(view)).toBe(true); });
        expect(buffer.text).toBe(source);
      }
      fireEvent.change(screen.getByRole("combobox", { name: "Enter indentation", hidden: true }), { target: { value: "smart" } });
      act(() => view.dispatch({ selection: EditorSelection.create([EditorSelection.cursor(source.indexOf("\n")), EditorSelection.cursor(source.length)]) }));
      fireEvent.keyDown(view.contentDOM, { key: "Enter", code: "Enter" });
      expect(buffer.text).toBe("class C:\n\n    def f():\n    ");
      act(() => { expect(undo(view)).toBe(true); });
      expect(buffer.text).toBe(source);
    } finally { screen.unmount(); if (old === null) localStorage.removeItem("editor.enterIndent"); else localStorage.setItem("editor.enterIndent", old); }
  });
  it("moves through whole identifiers or word pieces without changing read-only text", () => {
    // Contract: .hickory-source/frontends/contract.md
    const source = "THIS_IS_ONE_WORD HTTPServer42 café😀";
    for (const [offset, forward, pieces, expected] of [[0, true, false, 16], [0, true, true, 4], [4, true, true, 7], [7, false, true, 5], [17, true, true, 21], [21, true, true, 27], [27, true, true, 29], [30, true, true, 34]] as const) {
      expect(simpleWordOffset(source, offset, forward, pieces)).toBe(expected);
    }
    expect(simpleWordOffset("E\u0301clairCase", 0, true, true)).toBe(7);
    expect(simpleWordOffset("E\u0301clairCase", 7, false, true)).toBe(0);
    expect(simpleWordOffset("a\r\nb", 3, false, true)).toBe(0);
    expect(simpleWordOffset("", 0, false, true)).toBe(0);
    const view = new EditorView({ doc: source, parent: document.body, extensions: [EditorState.readOnly.of(true)] });
    expect(simpleWordMove(view, true, false, "native")).toBe(false);
    simpleWordMove(view, true, true, "pieces"); expect(view.state.selection.main.head).toBe(4);
    simpleWordMove(view, true, true, "pieces"); expect(view.state.selection.main.head).toBe(7);
    simpleWordMove(view, false, true, "pieces"); expect(view.state.selection.main.head).toBe(5);
    expect(view.state.selection.main.anchor).toBe(0);
    expect(view.state.doc.toString()).toBe(source);
    view.destroy();
  });
  it("persists independent arrow settings and applies them to an existing editor", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const control = localStorage.getItem("editor.controlArrow"), alt = localStorage.getItem("editor.altArrow");
    localStorage.removeItem("editor.controlArrow"); localStorage.removeItem("editor.altArrow");
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [] });
    const screen = render(createElement(App));
    try {
      fireEvent.click(screen.getByRole("button", { name: "New Tab" }));
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ changes: { from: 0, insert: "THIS_IS_ONE_WORD" }, selection: { anchor: 0 } }));
      fireEvent.change(screen.getByRole("combobox", { name: "Alt/Option + Left/Right", hidden: true }), { target: { value: "pieces" } });
      expect(localStorage.getItem("editor.altArrow")).toBe("pieces");
      expect(localStorage.getItem("editor.controlArrow")).toBeNull();
      fireEvent.keyDown(view.contentDOM, { key: "ArrowRight", code: "ArrowRight", altKey: true });
      expect(view.state.selection.main.head).toBe(4);
      fireEvent.keyDown(view.contentDOM, { key: "ArrowRight", code: "ArrowRight", altKey: true, shiftKey: true });
      expect(view.state.selection.main.anchor).toBe(4); expect(view.state.selection.main.head).toBe(7);
      fireEvent.change(screen.getByRole("combobox", { name: "Ctrl + Left/Right", hidden: true }), { target: { value: "word" } });
      act(() => view.dispatch({ selection: { anchor: 0 } }));
      fireEvent.keyDown(view.contentDOM, { key: "ArrowRight", code: "ArrowRight", ctrlKey: true });
      expect(view.state.selection.main.head).toBe(16);
      expect(view.state.doc.toString()).toBe("THIS_IS_ONE_WORD");
    } finally {
      screen.unmount();
      for (const [key, value] of [["editor.controlArrow", control], ["editor.altArrow", alt]]) { if (value === null) localStorage.removeItem(key!); else localStorage.setItem(key!, value!); }
    }
  });
  it("detects indentation and advances partial lines to the next stop", () => {
    // Contract: .hickory-source/frontends/contract.md
    expect(simpleIndent("root\n    one\n        two\n")).toBe("    ");
    expect(simpleIndent("root\n  one\n    two\n")).toBe("  ");
    expect(simpleIndent("root\n\tone\n\t\ttwo\n")).toBe("\t");
    expect(simpleIndent("\n   \n")).toBe("    ");
    const view = new EditorView({ doc: "root\n    child\n ", parent: document.body });
    view.dispatch({ selection: { anchor: view.state.doc.length } });
    simpleTab(view); simpleTab(view);
    expect(view.state.doc.toString().split("\n").at(-1)).toBe("        ");
    view.destroy();
  });
  for (const scenario of fixture.cases) it(scenario.name, () => {
    let layout = initialLayout();
    for (const step of scenario.steps) {
      const group = "target" in step ? step.target! : "group" in step ? groups(layout)[step.group!].id : "";
      if (step.op === "open") layout = activate(layout, group, step.tab!);
      if (step.op === "move") layout = move(layout, step.tab!, group, "before" in step ? step.before : undefined);
      if (step.op === "dock" && "edge" in step && "id" in step) layout = dock(layout, step.tab!, group, step.edge as Edge, step.id!);
      if (step.op === "remove") layout = remove(layout, step.tab!);
      if (step.op === "split" && "axis" in step && "id" in step) layout = split(layout, group, step.axis as "vertical", step.id!);
    }
    expect(groups(layout).map(group => group.tabs)).toEqual(scenario.expected);
    const axes = (node: Layout): string[] => isGroup(node) ? [] : [node.axis, ...axes(node.first), ...axes(node.second)];
    if ("axes" in scenario) expect(axes(layout)).toEqual(scenario.axes);
    for (const group of groups(layout)) expect(group.active === null || group.tabs.includes(group.active)).toBe(true);
  });
  for (const point of fixture.edgeCases) it(`classifies pane edge at ${point.x},${point.y}`, () => {
    expect(dropEdge(point.x, point.y)).toBe(point.edge);
  });
  it("refuses an unknown move target without removing the document", () => {
    const layout = activate(initialLayout(), "document-group", "notes.md");
    expect(move(layout, "notes.md", "missing")).toBe(layout);
  });
});


// Contract: .hickory-source/frontends/contract.md — logical line numbers are immediate.
describe("simple inline rich editing", () => {
  it("opens CSV as a keyboard-navigable grid and restores source without replacing its editor", async () => {
    const source = 'Name,Note\nCafé,"a,b"\n';
    const buffer = new Buffer({ path: "stock.csv", name: "stock.csv", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      const table = await screen.findByTestId("table-panel");
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      const grid = within(table);
      fireEvent.click(grid.getByRole("gridcell", { name: "Café" }));
      fireEvent.keyDown(grid.getByRole("gridcell", { name: "Café" }), { key: "ArrowRight" });
      const input = grid.getByRole("textbox", { name: "Cell contents" });
      fireEvent.change(input, { target: { value: 'two,"quotes"' } });
      fireEvent.keyDown(input, { key: "Enter" });
      await waitFor(() => expect(buffer.text).toContain('"two,""quotes"""'));
      fireEvent.click(screen.getByRole("button", { name: "Edit source text" }));
      expect(screen.queryByTestId("table-panel")).toBeNull();
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      act(() => { undo(view); });
      expect(buffer.text).toBe(source);
      fireEvent.click(screen.getByRole("button", { name: "Edit table as grid" }));
      expect(await screen.findByTestId("table-panel")).toBeTruthy();
    } finally { screen.unmount(); }
  });
  it("keeps source and undo usable while the parser needed by table grids is loading", async () => {
    let ready!: () => void;
    const pending = new Promise<void>(resolve => { ready = resolve; });
    const readiness = vi.spyOn(hickParser, "hickLangReady").mockReturnValue(false);
    const loading = vi.spyOn(hickParser, "loadHickLang").mockReturnValue(pending);
    const source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\nBefore\n\n| A | B |\n| --- | --- |\n| café 😀 | 120 |\n\nAfter\n</h:doc>';
    const buffers = new Buffers(() => {}), buffer = new Buffer({ path: "pending-grid.md", name: "pending-grid.md", dir: false }, source, "hash", () => {});
    buffers.values.set(buffer.file.path, buffer);
    const screen = render(createElement(Fragment, null, createElement(Document, { buffer }), createElement(RichOwners, { buffers })));
    try {
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      expect(view.state.doc.toString()).toBe(source);
      act(() => { view.dispatch({ changes: { from: 0, insert: "Note\n" } }); });
      expect(buffer.text).toBe("Note\n" + source);
      await act(async () => { ready(); await pending; });
      await waitFor(() => expect([...screen.container.querySelectorAll("*")].some(node => node.shadowRoot?.querySelector('[data-testid="table-panel"]'))).toBe(true), { timeout: 10000 });
      act(() => { undo(view); });
      expect(buffer.text).toBe(source);
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
    } finally { screen.unmount(); readiness.mockRestore(); loading.mockRestore(); }
  }, 15000);

  it.each([false, true])("uses the established table grid in place and keeps source edits undoable (literate=%s)", async literate => {
    const prose = "Intro.\n\n| Name | Count |\n| --- | --- |\n| North | 120 |\n\nAfter.";
    const source = literate ? `<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n${prose}\n</h:doc>` : prose;
    const buffers = new Buffers(() => {});
    const buffer = new Buffer({ path: "grid.md", name: "grid.md", dir: false }, source, "hash", () => {});
    buffers.values.set(buffer.file.path, buffer);
    const screen = render(createElement(Fragment, null, createElement(Document, { buffer }), createElement(RichOwners, { buffers })));
    try {
      const root = () => Array.from(screen.container.querySelectorAll("*")).map(node => node.shadowRoot).find(shadow => shadow?.querySelector('[data-testid="table-panel"]'));
      await waitFor(() => expect(root(), buffer.error).toBeTruthy(), { timeout: 10000 });
      const table = within(root()! as unknown as HTMLElement);
      fireEvent.doubleClick(table.getByText("120"));
      const input = table.getByRole("textbox", { name: "Cell contents" });
      fireEvent.change(input, { target: { value: "121" } }); fireEvent.keyDown(input, { key: "Enter" });
      await waitFor(() => expect(buffer.text).toBe(source.replace("120", "121")));
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => { undo(view); }); await waitFor(() => expect(buffer.text).toBe(source));
      fireEvent.click(table.getByRole("button", { name: "Edit source text" }));
      await waitFor(() => expect(root()).toBeUndefined()); expect(view.state.doc.toString()).toBe(source);
      fireEvent.click(screen.getByRole("button", { name: "Edit tables as grids" }));
      await waitFor(() => expect(root()).toBeTruthy());
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      expect(view.state.doc.toString()).toBe(source);
    } finally { screen.unmount(); }
  }, 15000);
  it("shows cached stdout when timed transcript events are absent", async () => {
    const { execView } = await import("../elements/exec/view");
    const slot = { span: [0, 20], index: 0, at: 0, text: "python3 orders.py", outputOnly: true } as import("../editor/rendered").RenderedSlot;
    const context = { execBlocks: [{ kind: "exec", id: "python:1", container: "python", command: slot.text, span: [0, 20], status: "ok", recorded_output: "Order: $18.80\nItems: 5\n" }], runningCells: new Set<string>(), replaying: [] } as unknown as import("../elements/types").SlotContext;
    const screen = render(execView.render(slot, context));
    try { expect(screen.getByText(/Order: \$18.80/)).toBeTruthy(); } finally { screen.unmount(); }
  });
  it("typesets local math without trusting HTML or URL macros", () => {
    const html = new TextDecoder().decode(readFileSync("../apple-simple/Sources/Resources/math.html"));
    const script = /<script nonce="hickory-math">([\s\S]*?)<\/script>/.exec(html)![1];
    const host = document.createElement("div"); host.innerHTML = '<div id="formula"></div><div id="status"></div>';
    const page = { getElementById: (id: string) => host.querySelector(`#${id}`) };
    const renderer = new Function("katex", "document", script + ";return globalThis.renderMath;")(katex, page) as (text: string) => void;
    try {
      renderer("x^2 + 1"); expect(host.querySelector(".katex")).not.toBeNull();
      renderer("\\href{javascript:alert(1)}{click}"); expect(host.querySelector("a")).toBeNull();
      renderer('</script><img src=x onerror=alert(1)>'); expect(host.querySelector("img,script")).toBeNull();
      renderer("x".repeat(4001)); expect(host.querySelector("#status")!.textContent).toContain("too large");
    } finally { delete (globalThis as unknown as { renderMath?: unknown }).renderMath; }
  });
});

describe("simple search and replace", () => {
  it("uses the real search panel for regex replace-all and one-step undo", async () => {
    const source = "😀 name=12\nname=34\n";
    const buffer = new Buffer({ path: "search.txt", name: "search.txt", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      act(() => openFind(buffer));
      fireEvent.change(await screen.findByLabelText("Find"), { target: { value: "name=(\\d+)" } });
      fireEvent.click(screen.getByLabelText("regexp"));
      fireEvent.change(screen.getByLabelText("Replace"), { target: { value: "$1" } });
      fireEvent.click(screen.getByRole("button", { name: "replace all" }));
      expect(buffer.text).toBe("😀 12\n34\n");
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => { undo(view); });
      expect(buffer.text).toBe(source);
      expect(view.state.doc.toString()).toBe(source);
    } finally { screen.unmount(); }
  });
  it("keeps backslashes literal and hides replacement controls in read-only documents", async () => {
    const source = "literal \\n and newline\n";
    const buffer = new Buffer({ path: "readonly.txt", name: "readonly.txt", dir: false, generated_by: "owner" }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      act(() => openFind(buffer));
      fireEvent.change(await screen.findByLabelText("Find"), { target: { value: "\\n" } });
      fireEvent.click(screen.getByRole("button", { name: "next" }));
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      expect(view.state.sliceDoc(view.state.selection.main.from, view.state.selection.main.to)).toBe("\\n");
      expect(screen.queryByLabelText("Replace")).toBeNull();
      expect(buffer.text).toBe(source); expect(buffer.dirty).toBe(false);
    } finally { screen.unmount(); }
  });
});

describe("simple folding", () => {
  it("finds nested indentation regions with UTF-16 offsets and excludes blank tails", () => {
    const source = "😀 header\n\tchild\n        nested\n\nnext\n";
    const ranges = simpleFoldRanges(source);
    expect(ranges.map(range => range.line)).toEqual([0, 1]);
    expect(ranges.map(range => source.slice(range.from, range.to))).toEqual(["\n\tchild\n        nested", "\n        nested"]);
    expect(simpleFoldRanges("one\ntwo\n")).toEqual([]);
  });
  it("folds and unfolds from the left plus/minus gutter without changing source, dirty state or undo", async () => {
    const source = "function main() {\n  first()\n  second()\n}\n";
    const buffer = new Buffer({ path: "fold.txt", name: "fold.txt", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      const gutter = screen.container.querySelector(".cm-foldGutter")!;
      expect(gutter.closest(".cm-gutters")).not.toBeNull();
      expect(screen.queryByRole("button", { name: "Fold line 1" })).toBeNull();
      const marker = () => gutter.querySelector(".cm-gutterElement:not([style*=\"visibility\"]) span[title]")!;
      await waitFor(() => expect(marker()?.textContent).toBe("−"));
      fireEvent.click(marker());
      await waitFor(() => expect(foldedRanges(view.state).size).toBe(1));
      expect(view.state.doc.toString()).toBe(source); expect(buffer.dirty).toBe(false);
      await waitFor(() => expect(marker().textContent).toBe("+"));
      fireEvent.click(marker());
      await waitFor(() => expect(foldedRanges(view.state).size).toBe(0));
      act(() => view.dispatch({ changes: { from: source.length, insert: "typed" } }));
      act(() => { undo(view); });
      expect(view.state.doc.toString()).toBe(source);
    } finally { screen.unmount(); }
  });
});

describe("simple line numbers", () => {
  it("numbers empty and trailing lines, updating through edits and undo", async () => {
    const buffer = new Buffer({ path: "numbers.txt", name: "numbers.txt", dir: false }, "", "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      const numbers = () => Array.from(screen.container.querySelectorAll(".cm-lineNumbers .cm-gutterElement"))
        .filter(node => (node as HTMLElement).style.visibility !== "hidden").map(node => node.textContent);
      await waitFor(() => expect(numbers()).toEqual(["1"]));
      act(() => view.dispatch({ changes: { from: 0, insert: "first\nsecond\n" } }));
      await waitFor(() => expect(numbers()).toEqual(["1", "2", "3"]));
      act(() => { undo(view); });
      await waitFor(() => expect(numbers()).toEqual(["1"]));
    } finally { screen.unmount(); }
  });
});

describe("simple inline DAP debugging", () => {
  it("launches the generated target from a literate file block", async () => {
    const start = vi.fn();
    vi.mocked(useFollowingDebugSession).mockReturnValue({ ...IDLE_SESSION, start });
    const text = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="hello.py">\nprint(1)\n</h:file>\n</h:doc>\n';
    const screen = render(createElement(Document, { buffer: new Buffer({ path: "source.md", name: "source.md", dir: false }, text, "hash", () => {}) }));
    try {
      fireEvent.click(await screen.findByRole("button", { name: "Debug hello.py" }));
      await waitFor(() => expect(start).toHaveBeenCalledWith("hello.py", {}, undefined, { id: "test", fingerprint: "test" }));
      expect(launchWithProfiles).toHaveBeenLastCalledWith(expect.objectContaining({ program: "hello.py" }), expect.any(Function), expect.objectContaining({ preferProgram: true }));
    } finally { screen.unmount(); vi.mocked(useFollowingDebugSession).mockReturnValue(null); }
  });
  it("attaches paused values and controls without replacing the editor or its undo", async () => {
    const buffer = new Buffer({ path: "sample.py", name: "sample.py", dir: false }, "value = 1\nprint(value)", "hash", () => {});
    const toggleBreakpoint = vi.fn(), step = vi.fn(), stop = vi.fn();
    const screen = render(createElement(Document, { buffer }));
    const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
    try {
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: "\n" } }));
      act(() => { undo(view); });
      vi.mocked(useFollowingDebugSession).mockReturnValue({ ...IDLE_SESSION, status: "paused", pausedLine: 1,
        variables: [{ name: "value", value: "1", type: "int", variables_reference: 0 }], toggleBreakpoint, step, stop });
      screen.rerender(createElement(Document, { buffer }));
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      await waitFor(() => expect(screen.container.querySelector(".cm-debug-value")?.textContent).toContain("value = 1"));
      fireEvent.click(screen.getByRole("button", { name: "Step over" })); expect(step).toHaveBeenCalledWith("over");
      fireEvent.click(screen.getByRole("button", { name: "Stop" })); expect(stop).toHaveBeenCalledOnce();
      act(() => view.dispatch({ changes: { from: 0, insert: "# changed\n" } }));
      await waitFor(() => expect(screen.container.querySelector(".cm-debug-value")).toBeNull());
      act(() => { undo(view); }); expect(buffer.text).toBe("value = 1\nprint(value)");
    } finally { screen.unmount(); vi.mocked(useFollowingDebugSession).mockReturnValue(null); }
  });
  it("refuses launching unsaved text and offers no launch for prose", async () => {
    const start = vi.fn();
    vi.mocked(useFollowingDebugSession).mockReturnValue({ ...IDLE_SESSION, start });
    const buffer = new Buffer({ path: "sample.py", name: "sample.py", dir: false }, "print(1)", "hash", () => {});
    buffer.edit("print(2)");
    const screen = render(createElement(Document, { buffer }));
    try {
      fireEvent.click(await screen.findByRole("button", { name: "Debug sample.py" }));
      await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("Save this file"));
      expect(start).not.toHaveBeenCalled();
      screen.rerender(createElement(Document, { buffer: new Buffer({ path: "notes.txt", name: "notes.txt", dir: false }, "prose", "hash", () => {}) }));
      expect(screen.queryByRole("button", { name: "Debug notes.txt" })).toBeNull();
    } finally { screen.unmount(); vi.mocked(useFollowingDebugSession).mockReturnValue(null); }
  });
});
describe("simple Markdown formatting", () => {
  it("renders nested emphasis and headings while leaving code and HTML literal", () => {
    const screen = render(createElement(MarkdownMessage, { text: "# Title **bold**\n\n***both*** and a_b_c and \`**code**\`\n\n~~~\n# literal **code**\n~~~\n\n<script>alert(1)</script>" }));
    try {
      expect(screen.getByRole("heading", { level: 1 }).querySelector("strong")?.textContent).toBe("bold");
      expect(screen.container.querySelector("em strong")?.textContent).toBe("both");
      expect(screen.container.querySelector("code")?.textContent).toBe("**code**");
      expect(screen.container.querySelector("pre code")?.textContent).toBe("# literal **code**");
      expect(screen.container.querySelector("pre strong")).toBeNull();
      expect(screen.container.querySelector("script")).toBeNull();
      expect(screen.container.textContent).toContain("<script>alert(1)</script>");
      expect(screen.container.textContent).toContain("a_b_c");
    } finally { screen.unmount(); }
  });
  it("styles visible Markdown and preserves source, selection and undo when typing", async () => {
    const source = "# Title\n\n**bold** and *italic*\n\n~~~\n**literal**\n~~~";
    const buffer = new Buffer({ path: "notes.md", name: "notes.md", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
    try {
      await waitFor(() => expect(screen.container.querySelector(".cm-md-h1")?.textContent).toContain("Title"));
      expect(screen.container.querySelector(".cm-md-strong")?.textContent).toBe("bold");
      expect(screen.container.querySelector(".cm-md-em")?.textContent).toBe("italic");
      expect(screen.container.querySelectorAll(".cm-md-strong")).toHaveLength(1);
      expect(view.state.doc.toString()).toBe(source);
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: "\n**new**" }, selection: { anchor: 2 } }));
      expect(view.state.selection.main.anchor).toBe(2);
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      act(() => { undo(view); });
      expect(view.state.doc.toString()).toBe(source);
    } finally { screen.unmount(); }
  });
  it("leaves Markdown-looking text in plain files unformatted", () => {
    const buffer = new Buffer({ path: "notes.txt", name: "notes.txt", dir: false }, "# Title **bold**", "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try { expect(screen.container.querySelector(".cm-md-heading, .cm-md-strong")).toBeNull(); }
    finally { screen.unmount(); }
  });
});

describe("pane edge docking", () => {
  it.each(["left", "right", "top", "bottom", "center"] as const)("routes %s drops and retains document selection and undo", async edge => {
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [
      { name: "a.txt", path: "a.txt", dir: false }, { name: "b.txt", path: "b.txt", dir: false },
    ] });
    vi.mocked(api.file).mockImplementation(async path => ({ path, content: path, hash: "base", language: "text" }));
    const screen = render(createElement(App));
    try {
      await waitFor(() => expect(screen.getByRole("button", { name: "a.txt" })).toBeTruthy());
      fireEvent.click(screen.getByRole("button", { name: "a.txt" }));
      await waitFor(() => expect(screen.getByRole("tab", { name: "a.txt" })).toBeTruthy());
      fireEvent.click(screen.getByRole("button", { name: "b.txt" }));
      await waitFor(() => expect(screen.getByRole("tab", { name: "b.txt" })).toBeTruthy());
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: " unsaved" }, selection: { anchor: 2 } }));
      if (edge === "center") {
        fireEvent.click(screen.getByText("View", { selector: "summary" }));
        fireEvent.click(screen.getByRole("button", { name: "Split right" }));
        fireEvent.click(screen.getByRole("button", { name: "Move tab to next group" }));
      }
      const beforeCount = screen.getAllByRole("tabpanel").length;
      const pane = edge === "center" ? screen.getAllByRole("tabpanel")[1] : screen.getAllByRole("tabpanel").at(-1)!;
      vi.spyOn(pane, "getBoundingClientRect").mockReturnValue({ x: 0, y: 0, left: 0, top: 0, right: 500, bottom: 400, width: 500, height: 400, toJSON() {} });
      const x = edge === "left" ? 10 : edge === "right" ? 490 : 250;
      const y = edge === "top" ? 10 : edge === "bottom" ? 390 : 200;
      const dataTransfer = { types: ["application/x-hickory-tab"], getData: () => "b.txt", dropEffect: "" };
      for (const type of ["dragover", "drop"]) {
        const event = new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y });
        Object.defineProperty(event, "dataTransfer", { value: dataTransfer }); fireEvent(pane, event);
        if (type === "dragover") expect(pane.getAttribute("data-drop-edge")).toBe(edge);
      }
      const order = screen.getAllByRole("tablist").map(list => Array.from(list.querySelectorAll('[role="tab"]')).map(tab => tab.textContent));
      expect(order).toEqual(edge === "center" ? [['Files'], ['a.txt', 'b.txt •']] : [['Files'], ...(edge === "left" || edge === "top" ? [['b.txt •'], ['a.txt']] : [['a.txt'], ['b.txt •']])]);
      expect(screen.getAllByRole("tabpanel").length).toBe(beforeCount + (edge === "center" ? -1 : 1));
      const content = screen.container.querySelector<HTMLElement>('[aria-label="b.txt"].cm-content')!;
      expect(EditorView.findFromDOM(content)).toBe(view);
      expect(view.state.selection.main.head).toBe(2);
      expect(view.state.doc.toString()).toBe("b.txt unsaved");
      act(() => { undo(view); }); expect(view.state.doc.toString()).toBe("b.txt");
    } finally { screen.unmount(); }
  });
});

describe("tab drop placement", () => {
  it("inserts before the target without parent-strip appending or losing editor undo", async () => {
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [
      { name: "a.txt", path: "a.txt", dir: false }, { name: "b.txt", path: "b.txt", dir: false },
    ] });
    vi.mocked(api.file).mockImplementation(async path => ({ path, content: path, hash: "base", language: "text" }));
    const screen = render(createElement(App));
    try {
      await waitFor(() => expect(screen.getByRole("button", { name: "a.txt" })).toBeTruthy());
      fireEvent.click(screen.getByRole("button", { name: "a.txt" }));
      await waitFor(() => expect(screen.getByRole("tab", { name: "a.txt" })).toBeTruthy());
      fireEvent.click(screen.getByRole("button", { name: "b.txt" }));
      await waitFor(() => expect(screen.getByRole("tab", { name: "b.txt" })).toBeTruthy());
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: " unsaved" } }));
      const source = screen.getByRole("tab", { name: /b.txt/ }).parentElement!;
      const target = screen.getByRole("tab", { name: "a.txt" }).parentElement!;
      const dataTransfer = { types: ["application/x-hickory-tab"], getData: () => "b.txt", setData: vi.fn(), effectAllowed: "", dropEffect: "" };
      fireEvent.dragStart(source, { dataTransfer }); fireEvent.dragOver(target, { dataTransfer });
      expect(target.getAttribute("data-drop-before")).toBe("true");
      fireEvent.drop(target, { dataTransfer });
      expect(screen.getAllByRole("tab").map(tab => tab.textContent)).toEqual(["Files", "b.txt •", "a.txt"]);
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      expect(view.state.doc.toString()).toBe("b.txt unsaved");
      act(() => { undo(view); }); expect(view.state.doc.toString()).toBe("b.txt");
    } finally { screen.unmount(); }
  });
});

describe("simple editor capability attachment", () => {
  it("attaches server colors and completion after typing without replacing selection or undo", async () => {
    let colorRequests = 0; let holdColors = false;
    const channel = createLspChannel({ send(frame) {
      const message = decodeLspFrame(frame);
      if (!message || !("method" in message) || !("id" in message)) return;
      if (message.method === "textDocument/semanticTokens/full") { colorRequests++; if (holdColors) return; }
      const result = message.method === "textDocument/semanticTokens/full" ? { data: [0, 0, 3, 0, 0] }
        : message.method === "textDocument/completion" ? [{ label: "languageValue", insertText: "languageValue" }] : [];
      queueMicrotask(() => channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", id: message.id, result })));
    } });
    const client = new LspClient(channel);
    channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", method: "hick/serverCapabilities", params: { capabilities: {
      semanticTokensProvider: { legend: { tokenTypes: ["keyword"], tokenModifiers: [] } },
    } } }));
    const buffer = new Buffer({ path: "sample.ts", name: "sample.ts", dir: false }, "let a = 1", "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
    try {
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: "\nla" }, selection: { anchor: 12 } }));
      const selected = view.state.selection.main.head;
      vi.mocked(useWorkspaceLsp).mockReturnValue({ client, uri: "hick:///sample.ts", ready: true, diagnostics: [{ range: { start: { line: 0, character: 4 }, end: { line: 0, character: 5 } }, severity: 1, message: "Example type error" }] });
      screen.rerender(createElement(Document, { buffer }));
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      expect(view.state.selection.main.head).toBe(selected);
      await waitFor(() => expect(screen.container.querySelector(".cm-st-keyword")?.textContent).toBe("let"));
      await waitFor(() => expect(screen.container.querySelector(".cm-lsp-error")?.textContent).toBe("a"));
      fireEvent.keyDown(view.contentDOM, { key: " ", code: "Space", ctrlKey: true });
      await waitFor(() => expect(currentCompletions(view.state).some(option => option.label === "languageValue")).toBe(true));
      // A delayed server must not remove colors from unchanged text while typing.
      holdColors = true;
      const beforeRefresh = colorRequests;
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: " pending" }, annotations: isolateHistory.of("full") }));
      expect(screen.container.querySelector(".cm-st-keyword")?.textContent).toBe("let");
      await waitFor(() => expect(colorRequests).toBeGreaterThan(beforeRefresh));
      expect(screen.container.querySelector(".cm-st-keyword")?.textContent).toBe("let");
      act(() => { undo(view); });
      fireEvent.keyDown(screen.getByRole("navigation", { name: "Document minimap" }), { key: "End" });
      expect(view.state.selection.main.head).toBe(selected);
      act(() => { undo(view); });
      expect(buffer.text).toBe("let a = 1");
    } finally {
      screen.unmount(); client.dispose();
      vi.mocked(useWorkspaceLsp).mockReturnValue({ client: null, uri: "hick:///sample.ts", ready: false, diagnostics: [] });
    }
  });
});
describe("simple file commands", () => {
  it("exposes the complete File menu and saves a new tab with a guarded destination", async () => {
    // Contract: .hickory-source/frontends/contract.md
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [] });
    vi.mocked(api.fileOp).mockResolvedValue({ op: "create", path: "new.py" });
    vi.mocked(api.openFile).mockResolvedValue({ path: "new.py" });
    let saved = "";
    vi.mocked(api.file).mockImplementation(async () => ({ path: "new.py", content: saved, language: "python", hash: "empty-hash" }));
    vi.mocked(api.saveFile).mockImplementation(async (_path, content) => { saved = content; return { path: "new.py", hash: "saved-hash" }; });
    const prompt = vi.spyOn(window, "prompt").mockReturnValue("new.py");
    const screen = render(createElement(App));
    try {
      const menu = screen.container.querySelector("nav details .menu")!;
      expect([...menu.querySelectorAll("button")].map(item => item.textContent)).toEqual([
        "Open File", "Open Directory", "New Tab", "New Window", "Save", "Save As", "Save All", "Refresh", "Close tab",
      ]);
      fireEvent.click(within(menu as HTMLElement).getByRole("button", { name: "New Tab" }));
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ changes: { from: 0, insert: "print(1)" } }));
      fireEvent.click(within(menu as HTMLElement).getByRole("button", { name: "Save All" }));
      await waitFor(() => expect(saved).toBe("print(1)"));
      await waitFor(() => expect(screen.getByRole("tab", { name: "new.py" })).toBeTruthy());
      expect(api.saveFile).toHaveBeenLastCalledWith("new.py", "print(1)", "empty-hash");
      expect(screen.queryByRole("tab", { name: /Untitled/ })).toBeNull();
    } finally { screen.unmount(); prompt.mockRestore(); }
  });
  it("retains a draft when Save As refuses an existing destination", async () => {
    // Contract: .hickory-source/frontends/contract.md
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [] });
    vi.mocked(api.fileOp).mockRejectedValueOnce(new Error("already exists"));
    const prompt = vi.spyOn(window, "prompt").mockReturnValue("existing.py");
    const screen = render(createElement(App));
    try {
      const menu = screen.container.querySelector("nav details .menu")! as HTMLElement;
      fireEvent.click(within(menu).getByRole("button", { name: "New Tab" }));
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ changes: { from: 0, insert: "retained" } }));
      fireEvent.click(within(menu).getByRole("button", { name: "Save As" }));
      await waitFor(() => expect(screen.getByRole("status").textContent).toContain("already exists"));
      expect(view.state.doc.toString()).toBe("retained");
      expect(screen.getByRole("tab", { name: /Untitled/ })).toBeTruthy();
    } finally { screen.unmount(); prompt.mockRestore(); }
  });
});
describe("simple frontend text opening", () => {
  it.each(["notes.md", "cards.md"])("opens managed %s through its document source", async path => {
    vi.mocked(api.file).mockClear();
    vi.mocked(api.doc).mockResolvedValueOnce({ id: "managed", path, source: "# Readable source", updated_at: "" });
    const buffer = await new Buffers(() => {}).open({ name: path, path, dir: false, doc_id: "managed" });
    expect(api.doc).toHaveBeenLastCalledWith("managed");
    expect(api.file).not.toHaveBeenCalled();
    expect(buffer.text).toBe("# Readable source"); expect(buffer.base).toBe(buffer.text);
    buffer.edit("# Edited"); vi.mocked(api.saveDoc).mockResolvedValueOnce({ id: "managed", path, source: "# Edited", updated_at: "" });
    await buffer.save();
    expect(api.saveDoc).toHaveBeenLastCalledWith("managed", "# Edited", { source: "# Readable source", disk: "# Readable source" });
  });
  it("opens plain text with its file hash", async () => {
    vi.mocked(api.doc).mockClear();
    vi.mocked(api.file).mockResolvedValueOnce({ path: "notes.txt", content: "Plain text", hash: "plain-hash", language: "text" });
    const buffer = await new Buffers(() => {}).open({ name: "notes.txt", path: "notes.txt", dir: false });
    expect(api.file).toHaveBeenLastCalledWith("notes.txt"); expect(api.doc).not.toHaveBeenCalled();
    expect(buffer.text).toBe("Plain text"); expect(buffer.hash).toBe("plain-hash");
  });
});
describe("simple frontend save safety", () => {
  const plain = { name: "notes.txt", path: "notes.txt", dir: false };
  it("retains edits after a conflict and sends the original hash", async () => {
    vi.mocked(api.saveFile).mockRejectedValueOnce(new Error("changed on disk"));
    const buffer = new Buffer(plain, "before", "hash-before", () => {});
    buffer.edit("mine"); await buffer.save();
    expect(api.saveFile).toHaveBeenLastCalledWith("notes.txt", "mine", "hash-before");
    expect(buffer.text).toBe("mine"); expect(buffer.base).toBe("before"); expect(buffer.dirty).toBe(true);
    expect(buffer.error).toContain("edits are retained");
  });
  it("does not mark later keystrokes as saved", async () => {
    let finish!: (value: { path: string; hash: string }) => void;
    vi.mocked(api.saveFile).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    const buffer = new Buffer(plain, "before", "hash-before", () => {});
    buffer.edit("submitted"); const saving = buffer.save(); buffer.edit("typed later");
    finish({ path: plain.path, hash: "hash-submitted" }); await saving;
    expect(buffer.base).toBe("submitted"); expect(buffer.text).toBe("typed later"); expect(buffer.dirty).toBe(true);
  });
  it("uses source and disk expectations for literate source", async () => {
    vi.mocked(api.saveDoc).mockResolvedValueOnce({ id: "doc", path: "source.md", source: "next", updated_at: "" });
    const buffer = new Buffer({ ...plain, doc_id: "doc" }, "before", "hash", () => {});
    buffer.edit("next"); await buffer.save();
    expect(api.saveDoc).toHaveBeenLastCalledWith("doc", "next", { source: "before", disk: "before" });
  });
  it("does not edit or save generated outputs", async () => {
    const buffer = new Buffer({ ...plain, generated_by: "owner" }, "generated", "hash", () => {});
    const calls = vi.mocked(api.saveFile).mock.calls.length;
    buffer.edit("overwrite"); await buffer.save();
    expect(buffer.text).toBe("generated"); expect(vi.mocked(api.saveFile).mock.calls.length).toBe(calls);
  });
});
describe("terminal and agent tab identity", () => {
  it("opens the terminal once and retains unsent input when shown again", async () => {
    vi.mocked(api.openTerminal).mockResolvedValueOnce({ id: "pty" } as Awaited<ReturnType<typeof api.openTerminal>>);
    const session = new TerminalSession(); session.input = "pending input";
    await session.open(); await session.open();
    expect(api.openTerminal).toHaveBeenCalledTimes(1); expect(session.id).toBe("pty"); expect(session.input).toBe("pending input");
  });
  it("retains input after a refused terminal write without retry", async () => {
    vi.mocked(api.terminalInput).mockRejectedValueOnce(new Error("disconnected"));
    const session = new TerminalSession(); session.id = "pty"; session.input = "echo hello";
    await session.send();
    expect(api.terminalInput).toHaveBeenCalledTimes(1);
    expect(api.terminalInput).toHaveBeenLastCalledWith("pty", "echo hello\r");
    expect(session.input).toBe("echo hello"); expect(session.error).toContain("not retried");
  });

});
