<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Check shared layouts and refusal-safe saves

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Read the exact same layout scenarios as Swift. These are focused model witnesses, not evidence that native drag gestures or accessibility work on a device.

The Markdown witnesses check actual rendered emphasis, literal code and HTML safety, then edit and undo inside the retained document editor. A plain-file counterexample protects ordinary source text from accidental prose formatting. These component checks do not establish native rendering.

The folding witnesses exercise the dedicated left plus/minus gutter and CodeMirror fold state, preserving the document and undo history. Indentation discovery is local and uses UTF-16 positions, including tabs and Unicode. These are component checks, not native gesture evidence.

The inline DAP witnesses attach a controlled paused session after text is visible, exercise real editor decorations and action buttons, retain undo, and refuse an unsaved launch. They do not establish a live adapter connection.

The indentation witness checks four spaces, two spaces, literal tabs, whitespace-only fallback and two successive Tab presses after partial indentation. These assertions protect alignment without needing a language server.

The shared native diagram witness checks authored top-level graphs and wrapped scenes. The authored graph must draw both nodes and the edge using its saved layout, and a malformed collection must produce a visible status. Wrapped scenes still use text-only labels and route links through the native bridge.

<ui:copy id="simple-c6aff51ad5-0">
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
import { undo, redo, isolateHistory } from "@codemirror/commands";
import { currentCompletions } from "@codemirror/autocomplete";
import { Document, semanticPath, semanticLocations, querySemanticLocations, openSemanticQuery, simpleWordOffset, simpleWordMove, simpleFoldRanges, openFind, simpleIndent, simpleTab } from "./Document";
vi.mock("../debug/launchProfiles", () => ({ launchWithProfiles: vi.fn(async (_query, start, options) => { if (options.isCurrent()) start({ id: "test", fingerprint: "test" }); }) }));
import { launchWithProfiles } from "../debug/launchProfiles";
import { IDLE_SESSION } from "../debug/useDebugger";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
vi.mock("../debug/plainDebugHosts", async importOriginal => ({ ...await importOriginal<typeof import("../debug/plainDebugHosts")>(), useFollowingDebugSession: vi.fn(() => null), PlainDebugHosts: () => null }));
import { Agent, AgentHistory, AgentSession, MarkdownMessage, agentTranscript } from "./Agent";
import { App } from "./App";
import { applyWorkspacePlan } from "./WorkspaceChanges";
import { validateReferencedTargets, planWorkspaceEdit, invertChangePlan, validateChangePlan } from "./ChangePlan";
import changeCases from "../../../simple-contract/change-plan-cases.json";
import { WorkspaceSearch } from "./WorkspaceSearch";
import * as reviews from "../lib/reviews";
import { workspaceAnalysisScope, searchCharacter, NavigationTrail, reportedProblems, nextWorkspaceProblem } from "./Intelligence";
import { openLocation, revealLine as queueReveal, claimReveal, claimRevealPosition, resetReveals } from "../lib/revealLine";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { LspClient } from "../lsp/client";
import { createLspChannel, decodeLspFrame, encodeLspFrame } from "../lsp/channel";
vi.mock("../lsp/useLsp", () => ({ useWorkspaceLsp: vi.fn(() => ({ client: null, uri: "hick:///sample.ts", ready: false, diagnostics: [] })) }));
import { beforeAll,describe, expect, it, vi } from "vitest";
import fixture from "../../../simple-contract/layout-cases.json";
import { sourceTab,tabIdentity,activate, groups, initialLayout, move, remove, split, dock, dropEdge, isGroup, type Edge, type Layout } from "./layout";
import { TerminalSession } from "./Terminal";
import { Buffer, Buffers,createSourceCopy } from "./buffers";
import { api,request } from "../api/client";
import {historySourceHash} from "./SourceHistory";
vi.mock("../api/client", () => ({ request:vi.fn(),api: { find: vi.fn(), formulaLanguages: vi.fn().mockResolvedValue({ languages: ["python"] }), files: vi.fn(), startupSource: vi.fn(), doc: vi.fn(), file: vi.fn(), saveFile: vi.fn(), saveDoc: vi.fn(), fileOp: vi.fn(), openFile: vi.fn(), createDoc: vi.fn(), openTerminal: vi.fn(), terminalInput: vi.fn(), agentTurns: vi.fn().mockResolvedValue({ turns: [] }), agentStop: vi.fn(), agent: vi.fn(), gitChanges: vi.fn() } }));
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
async function saveReceipt(path:string,body:unknown,hash='confirmed-hash') {const value=body as {request_id:string;source?:string;content?:string};return {version:1,history_protocol:1,path,request_id:value.request_id,source_hash:await historySourceHash(value.source??value.content??''),save_guard:'confirmed-physical',hash,history:{available:true,outcome:'published'}};}
describe("shared frontend layout contract", () => {
  it("constructs patch fences and offers source-mapped patch debugging in Simple", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const source = "<hick:project/>\n<hick:file id=\"base\" path=\"x.py\">\nprint(1)\n</hick:file>\n```patch id=\"change\" after=\"base\"\n--- a/x.py\n+++ b/x.py\n@@ -1 +1 @@\n-print(1)\n+print(\"café 😀\")\n```\n```python path=\"other.py\"\nprint(3)\n```\n```diff\n--- a/fake.py\n+++ b/fake.py\n@@ -1 +1 @@\n-a\n+b\n```";
    expect(proseFences(parseHickDoc(source), source).map(fence => fence.info)).toEqual(["diff"]);
    const workbook = new TextDecoder().decode(readFileSync("../../examples/simple-ui/workbook.md"));
    const entry = new TextDecoder().decode(readFileSync("../../examples/simple-ui/project.md"));
    const demo = hickParser.materializeProject("project.md", { "project.md": entry, "workbook.md": workbook });
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
  it("resolves diagram file links without allowing external navigation or escaping the workspace", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const { diagramFilePath, onOpenLocation, openDiagramLocation } = await import("../lib/revealLine");
    expect(diagramFilePath("../src/main.rs", "docs/design.md")).toBe("src/main.rs");
    expect(diagramFilePath("space%20name.md", "docs/design.md")).toBe("docs/space name.md");
    for (const path of ["../../outside", "https://example.com", "javascript:alert(1)", "/tmp/file", "%2fetc/passwd", "..\\file"])
      expect(diagramFilePath(path, "docs/design.md")).toBeNull();
    const follow = vi.fn();
    const stop = onOpenLocation(follow);
    try {
      let current = true;
      const sourceCurrent = () => current;
      openDiagramLocation("../src/space%20name.md#L6", "docs/design.md", sourceCurrent);
      expect(follow).toHaveBeenCalledWith({ path: "src/space name.md", line: 6, originCurrent: sourceCurrent });
      current = false;
      expect(follow.mock.calls[0][0].originCurrent()).toBe(false);
      openDiagramLocation("../src/space%20name.md#L6", "docs/design.md", sourceCurrent);
      openDiagramLocation("../../outside", "docs/design.md", () => true);
      expect(follow).toHaveBeenCalledTimes(1);
    } finally { stop(); }
  });
  it("renders native graph previews safely and rejects obsolete Mermaid results", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const html = new TextDecoder().decode(readFileSync("../apple-simple/Sources/Resources/math.html"));
    const document = new DOMParser().parseFromString(html, "text/html");
    Object.defineProperty(document, "fonts", { value: { ready: Promise.resolve() } });
    const linked = vi.fn();
    const scope: any = { webkit: { messageHandlers: { diagramLink: { postMessage: linked } } } };
    const pending: ((value: { svg: string }) => void)[] = [];
    const mermaid = { initialize: vi.fn(), render: vi.fn(() => new Promise(resolve => pending.push(resolve as any))) };
    scope.mermaid = mermaid;
    const script = html.match(/<script nonce="hickory-math">([\s\S]*?)<\/script>/)![1];
    new Function("document", "globalThis", "mermaid", script)(document, scope, mermaid);
    await scope.renderDiagram(JSON.stringify({ topology: { nodes: [{ id: "a", href: "src/main.rs", label: "<img src=x onerror=alert(1)>" }, { id: "b" }], edges: [{ from: "a", to: "b" }] } }), "graph");
    expect(document.querySelectorAll("svg rect")).toHaveLength(2);
    document.querySelector("svg g")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(linked).toHaveBeenCalledWith({ href: "src/main.rs", revision: 0 });
    expect(document.querySelector("svg")?.getAttribute("role")).toBe("group");
    const link = document.querySelector('svg g[role="link"][tabindex="0"]');
    expect(link?.querySelector("text")?.getAttribute("text-decoration")).toBe("underline");
    link?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(linked).toHaveBeenCalledTimes(2);
    expect(document.querySelector("img")).toBeNull();
    expect(document.querySelector("svg text")?.textContent).toContain("<img");
    await scope.renderDiagram(JSON.stringify({
      nodes: [{ id: "disk", label: "Public vault" }, { id: "export", label: "HTML export" }],
      edges: [{ from: "disk", to: "export" }],
      layout: { disk: { x: 40, y: 30, w: 160, h: 64 } },
    }), "graph");
    expect([...document.querySelectorAll("svg text")].map(node => node.textContent)).toEqual(["Public vault", "HTML export"]);
    expect(document.querySelector("svg")?.getAttribute("role")).toBe("img");
    expect(document.querySelector("svg rect")?.getAttribute("x")).toBe("40");
    expect(document.querySelector("svg line")?.getAttribute("x1")).toBe("200");
    expect(document.querySelector("svg g title")?.textContent).toBe("No workspace destination");
    for (const [source, renderer, message] of [
      ['{"nodes": {}}', "graph", "Graph nodes and edges must be arrays."],
      ['{"nodes":[null]}', "graph", "Graph nodes need a text ID"],
      ['{"nodes":[]}', "graph", "Diagram has no nodes."],
      ["", "mermaid", "Diagram source is empty."],
      [" ".repeat(200_001), "graph", "Diagram is too large to preview."],
      ["{}", "unsupported", "No preview for renderer"],
    ]) {
      await scope.renderDiagram(source, renderer);
      expect(document.getElementById("status")?.textContent).toContain(message);
      expect(document.getElementById("status")?.textContent).toContain("Use the diagram gutter");
      expect(document.querySelector("#formula svg")).toBeNull();
    }
    const old = scope.renderDiagram("graph TD; A-->B", "mermaid");
    const fresh = scope.renderDiagram('graph TD; X-->Y;\nclick X "notes.md"', "mermaid");
    pending[1]({ svg: '<svg><g class="node" id="diagram-2-flowchart-X-7"><text>fresh</text></g></svg>' }); await fresh;
    pending[0]({ svg: "<svg><text>old</text></svg>" }); await old;
    expect(document.getElementById("formula")?.textContent).toBe("fresh");
    expect(document.getElementById("status")?.textContent).toBe("");
    document.querySelector("g.node")!.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    expect(linked).toHaveBeenLastCalledWith({ href: "notes.md", revision: 0 });
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
  it("folds all rows of a unified hunk and keeps its header", () => {
    const rows = ["```patch id=\"p\"", "--- a/main.py", "+++ b/main.py", "@@ -1,2 +1,3 @@",
      " def f():", "-    return 1", "+    x = 2", "+    return x", "```"];
    for (const newline of ["\n", "\r\n"]) {
      const source = rows.join(newline);
      const folds = simpleFoldRanges(source,true);
      expect(folds.map(fold=>fold.line)).toEqual([3]);
      expect(source.slice(folds[0].from,folds[0].to)).toBe(newline+rows.slice(4,8).join(newline));
    }
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
  it("opens readable CSV, then explicitly switches to the keyboard-navigable grid", async () => {
    const source = 'Name,Note\nCafé,"a,b"\n';
    const buffer = new Buffer({ path: "stock.csv", name: "stock.csv", dir: false }, source, "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    try {
      const csvRoot = () => screen.container.querySelector(".simple-csv-grid")?.shadowRoot;
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      expect(view.state.doc.toString()).toBe(source);
      const switchToGrid = screen.getByRole("button", { name: "Edit table as grid" }) as HTMLButtonElement;
      await waitFor(() => expect(switchToGrid.disabled).toBe(false));
      expect(csvRoot()).toBeUndefined();
      fireEvent.click(switchToGrid);
      await waitFor(() => expect(csvRoot()?.querySelector('[data-testid="table-panel"]')).toBeTruthy());
      const table = csvRoot()!.querySelector('[data-testid="table-panel"]')! as HTMLElement;
      const grid = within(table);
      fireEvent.click(grid.getByRole("gridcell", { name: "Café" }));
      fireEvent.keyDown(grid.getByRole("gridcell", { name: "Café" }), { key: "ArrowRight" });
      const input = grid.getByRole("textbox", { name: "Cell contents" });
      fireEvent.change(input, { target: { value: 'two,"quotes"' } });
      fireEvent.keyDown(input, { key: "Enter" });
      await waitFor(() => expect(buffer.text).toContain('"two,""quotes"""'));
      fireEvent.click(screen.getByRole("button", { name: "Edit source text" }));
      expect(csvRoot()?.querySelector('[data-testid="table-panel"]')).toBeFalsy();
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      act(() => { undo(view); });
      expect(buffer.text).toBe(source);
      fireEvent.click(screen.getByRole("button", { name: "Edit table as grid" }));
      await waitFor(() => expect(csvRoot()?.querySelector('[data-testid="table-panel"]')).toBeTruthy());
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
      fireEvent.click(screen.container.querySelector<HTMLButtonElement>('[aria-label="Edit source text"]')!);
      await waitFor(() => expect(root()).toBeUndefined()); expect(view.state.doc.toString()).toBe(source);
      fireEvent.click(screen.container.querySelector<HTMLButtonElement>('[aria-label="Render embedded content"]')!);
      await waitFor(() => expect(root()).toBeTruthy());
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      expect(view.state.doc.toString()).toBe(source);
    } finally { screen.unmount(); }
  }, 15000);
  it("renders an authored expectation with the same terminal before any run exists", async () => {
    const { execView } = await import("../elements/exec/view");
    const slot = { span: [0, 20], index: 0, at: 0, text: "echo hello", expectedOutput: "hello\n", outputOnly: false } as import("../editor/rendered").RenderedSlot;
    const context = { execBlocks: [], runningCells: new Set<string>(), replaying: [] } as unknown as import("../elements/types").SlotContext;
    const ui = render(execView.render(slot, context));
    try {
      expect(ui.getByLabelText("Expected output terminal")).toBeTruthy();
      await waitFor(() => expect([...ui.container.querySelectorAll(".xterm-rows > div")].map(row => row.textContent).join("\n")).toContain("hello"));
      expect(ui.container.textContent).not.toContain("Expected output matches");
      expect(ui.container.textContent).not.toContain("[stdout]");
    } finally { ui.unmount(); }
  });
  it("shows cached stdout when timed transcript events are absent", async () => {
    const { execView } = await import("../elements/exec/view");
    const slot = { span: [0, 20], index: 0, at: 0, text: "python3 orders.py", outputOnly: true } as import("../editor/rendered").RenderedSlot;
    const context = { execBlocks: [{ kind: "exec", id: "python:1", container: "python", command: slot.text, span: [0, 20], status: "ok", recorded_output: "Order: $18.80\nItems: 5\n" }], runningCells: new Set<string>(), replaying: [] } as unknown as import("../elements/types").SlotContext;
    const screen = render(execView.render(slot, context));
    try {
      await waitFor(() => expect([...screen.container.querySelectorAll(".xterm-rows > div")].map(row => row.textContent).join("\n")).toContain("Order: $18.80"));
      expect(screen.container.querySelector(".inline-output-terminal")).toBeTruthy();
      expect(screen.container.textContent).not.toContain("[stdout]");
    } finally { screen.unmount(); }
  });
  it("toggles equation source from the gutter even with the caret on its line", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const buffers = new Buffers(() => {}), source = "Before\n\n$$x^2$$\n\nAfter";
    const buffer = new Buffer({ path: "math.md", name: "math.md", dir: false }, source, "hash", () => {});
    buffers.values.set(buffer.file.path, buffer);
    const screen = render(createElement(Fragment, null, createElement(Document, { buffer }), createElement(RichOwners, { buffers })));
    try {
      const toggle = await screen.findByRole("button", { name: "Toggle equation source text", hidden: true });
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ selection: { anchor: source.indexOf("x^2") } }));
      await waitFor(() => expect(screen.container.querySelector(".cm-math")).toBeNull());
      fireEvent.click(toggle);
      await waitFor(() => expect(screen.container.querySelector(".cm-math")).toBeTruthy());
      fireEvent.click(toggle);
      await waitFor(() => expect(screen.container.querySelector(".cm-math")).toBeNull());
      expect(buffer.text).toBe(source); expect(buffer.dirty).toBe(false);
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
    } finally { screen.unmount(); }
  });
  it("typesets local math without trusting HTML or URL macros", () => {
    const html = new TextDecoder().decode(readFileSync("../apple-simple/Sources/Resources/math.html"));
    const script = /<script nonce="hickory-math">([\s\S]*?)<\/script>/.exec(html)![1];
    const host = document.createElement("div"); host.innerHTML = '<div id="formula"></div><div id="status"></div>';
    const page = { getElementById: (id: string) => host.querySelector(`#${id}`), fonts: { ready: Promise.resolve() }, body: host, querySelector: (selector: string) => host.querySelector(selector) };
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

// Inline runtime values require a captured frame whose source hash matches the
// retained draft. An unbound attach receipt must never acquire line authority.
describe("simple inline DAP debugging", () => {
  it("loads package launch profiles only on explicit request and retires a changed source", async () => {
    vi.mocked(launchWithProfiles).mockClear();
    const buffer = new Buffer({ path: "sample.ts", name: "sample.ts", dir: false }, "let value = 41", "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
    try {
      act(() => view.dispatch({ changes: { from: 0, insert: "// temporary\n" } }));
      expect(launchWithProfiles).not.toHaveBeenCalled();
      act(() => { undo(view); });
      expect(buffer.dirty).toBe(false);
      fireEvent.click(await screen.findByRole("button", { name: "Launch profiles for sample.ts" }));
      const options = vi.mocked(launchWithProfiles).mock.lastCall?.[2];
      expect(options?.choose).toBe(true);
      expect(options?.isCurrent?.()).toBe(true);
      act(() => view.dispatch({ changes: { from: 0, insert: "// temporary\n" } }));
      act(() => { undo(view); });
      expect(buffer.dirty).toBe(false);
      expect(options?.isCurrent?.()).toBe(false);
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
    } finally { screen.unmount(); }
  });

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
      const hash=await historySourceHash(buffer.text);
      const frame={id:1,name:"main",source:"sample.py",line:1,in_document:true,source_hash:hash};
      vi.mocked(useFollowingDebugSession).mockReturnValue({ ...IDLE_SESSION, status: "paused", pausedLine: 1,
        selectedFrame:1,frames:[frame],positionSourceBound:true,captureFrame:()=>({frame,current:()=>true}),
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

it('separates reserved source names from utilities without changing ordinary paths',()=>{for(const path of ['files','agent','terminal','\0source:literal']){const address=sourceTab(path);expect(address).not.toBe(path);expect(tabIdentity(address)).toEqual({kind:'source',key:path});}for(const name of ['files','agent','terminal'])expect(tabIdentity(name)).toEqual({kind:'utility',name});expect(sourceTab('Untitled-report.py')).toBe('Untitled-report.py');expect(tabIdentity('Untitled-report.py')).toEqual({kind:'source',key:'Untitled-report.py'});});
it('moves a reserved source address without removing the utility tab',()=>{let layout=initialLayout();layout=activate(layout,'document-group',sourceTab('files'));layout=move(layout,sourceTab('files'),'files-group');expect(groups(layout).flatMap(group=>group.tabs)).toContain('files');expect(groups(remove(layout,sourceTab('files'))).flatMap(group=>group.tabs)).toContain('files');});
it('classifies unnamed drafts explicitly rather than from a source filename',()=>{const source=new Buffer({path:'Untitled-real.py',name:'Untitled-real.py',dir:false},'readable','hash',()=>{});expect(source.isUntitledDraft).toBe(false);const draft=new Buffer({path:'opaque',name:'notes',dir:false},'notes','',()=>{});draft.isUntitledDraft=true;expect(draft.isUntitledDraft).toBe(true);});
</ui:copy>

## Exercise delayed language attachment in a real editor instance

The first editor witness drives tab drag/drop events through the workspace, checking the insert-before result, the fixed insertion marker, retained unsaved text and the same editor undo history. It also catches a drop bubbling to the parent strip and appending unexpectedly. These synthetic DOM events do not establish native gesture behavior.

The controlled JSON-RPC provider below supplies semantic colors and a completion result, without requiring an installed language server. Type before attaching it, then assert editor identity, selection, painted tokens, suggestions and undo. This witnesses the browser integration, not a native device or an external server. The Python acceptance driver separately exposes minimap navigation against the real backend.

<ui:copy id="simple-language-attachment-witness">
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
      expect(order).toEqual(edge === "center" ? [['Files', 'Agent History'], ['a.txt', 'b.txt •']] : [['Files', 'Agent History'], ...(edge === "left" || edge === "top" ? [['b.txt •'], ['a.txt']] : [['a.txt'], ['b.txt •']])]);
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
      expect(screen.getAllByRole("tab").map(tab => tab.textContent)).toEqual(["Files", "Agent History", "b.txt •", "a.txt"]);
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      expect(view.state.doc.toString()).toBe("b.txt unsaved");
      act(() => { undo(view); }); expect(view.state.doc.toString()).toBe("b.txt");
    } finally { screen.unmount(); }
  });
});

describe("simple background source loading", () => {
  it("keeps a tree-first guardless startup editor through late raw text and Refresh", async () => {
    const path = "nested/wanted.txt", source = "First row\nSecond readable row\n";
    const workspace = "a".repeat(32), namespace = "/owned";
    const previous = window.location.pathname + window.location.search + window.location.hash;
    window.history.replaceState(null, "", `?path=${encodeURIComponent(path)}`);
    let finishRaw!: (reply: Awaited<ReturnType<typeof api.startupSource>>) => void;
    vi.mocked(api.startupSource).mockImplementationOnce(() => new Promise(resolve => { finishRaw = resolve; }));
    vi.mocked(api.files).mockResolvedValue({ root: "Owned files", root_path: namespace, workspace_id: workspace, tree: [
      { name: "nested", path: "nested", dir: true, children: [
        { name: "wanted.txt", path, dir: false },
        { name: "peer.txt", path: "nested/peer.txt", dir: false },
      ] },
      { name: "outside.txt", path: "outside.txt", dir: false },
    ] });
    const currentSource = source + "Written while opening.\n";
    const reply = { path, content: source, language: "text", hash: "loaded" };
    vi.mocked(api.file).mockReset()
      .mockResolvedValueOnce({ ...reply, save_guard_reason: "Save permission is pending." })
      .mockResolvedValueOnce({ ...reply, content: currentSource, save_guard: "confirmed" });
    const screen = render(createElement(App));
    try {
      const content = await screen.findByLabelText(path);
      const view = EditorView.findFromDOM(content)!;
      expect(view.state.doc.toString()).toBe(source);
      expect(view.state.readOnly).toBe(true);
      expect(screen.getByRole("button", { name: "peer.txt" })).toBeTruthy();
      expect(screen.queryByRole("button", { name: "outside.txt" })).toBeNull();
      act(() => view.dispatch({ selection: { anchor: 10, head: 16 } }));
      view.scrollDOM.scrollTop = 125;
      await act(async () => finishRaw({ version: 1, path, content: source, root_path: namespace, workspace_id: workspace }));
      expect(EditorView.findFromDOM(screen.getByLabelText(path))).toBe(view);
      expect(view.state.readOnly).toBe(true);

      fireEvent.click(screen.getByText("File", { selector: "summary" }));
      fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
      await waitFor(() => expect(view.state.readOnly).toBe(false));
      expect(view.state.doc.toString()).toBe(currentSource);
      expect(api.file).toHaveBeenCalledTimes(2);
      expect(EditorView.findFromDOM(screen.getByLabelText(path))).toBe(view);
      expect(view.state.selection.main.anchor).toBe(10);
      expect(view.state.selection.main.head).toBe(16);
      expect(view.scrollDOM.scrollTop).toBe(125);
      act(() => view.dispatch({ changes: { from: 10, insert: "Draft " } }));
      const draft = currentSource.slice(0, 10) + "Draft " + currentSource.slice(10);
      expect(view.state.doc.toString()).toBe(draft);
      act(() => { expect(undo(view)).toBe(true); });
      expect(view.state.doc.toString()).toBe(currentSource);
      act(() => { expect(redo(view)).toBe(true); });
      expect(view.state.doc.toString()).toBe(draft);
    } finally {
      screen.unmount();
      window.history.replaceState(null, "", previous);
      vi.mocked(api.file).mockReset();
    }
  });
  it("refuses a stale tree-first startup reply from a former root", async () => {
    const buffers = new Buffers(() => {});
    const file = { name: "early.txt", path: "early.txt", dir: false };
    const workspace = "1".repeat(32);
    buffers.workspace = "/previous";
    buffers.workspaceId = workspace;
    vi.mocked(api.file).mockResolvedValueOnce({ path: file.path, language: "text", hash: "hash", save_guard: "old", content: "Old root." });
    const first = buffers.open(file, true);
    buffers.workspace = "/current";
    await expect(first).rejects.toThrow("different workspace or source");
    expect(buffers.values.has(file.path)).toBe(false);
  });
  it("updates a provisional editor with the current bytes from its guarded workspace", async () => {
    const buffers = new Buffers(() => {});
    const file = {name:"early.txt", path:"docs/early.txt", dir:false};
    const workspace = "1".repeat(32), root = "/workspace", content = "Readable 😀\r\n";
    const provisional = buffers.observeStartup({version:1,path:file.path,content,root_path:root,workspace_id:workspace})!;
    expect(provisional.readonly).toBe(true);
    expect(provisional.saveGuard).toBeUndefined();
    provisional.edit("Forbidden");
    expect(provisional.text).toBe(content);
    buffers.workspace = root; buffers.workspaceId = workspace;
    const guarded = {path:file.path,language:"text",hash:"hash",save_guard:"guard",content};
    vi.mocked(api.file).mockResolvedValueOnce({...guarded,path:"other.txt",content:"Wrong file"});
    await expect(buffers.open(file)).rejects.toThrow("startup observation differs");
    expect(provisional.text).toBe(content);
    expect(provisional.readonly).toBe(true);
    vi.mocked(api.file).mockResolvedValueOnce({...guarded,content:"Changed disk text"});
    expect(await buffers.open(file)).toBe(provisional);
    expect(provisional.text).toBe("Changed disk text");
    expect(provisional.dirty).toBe(false);
    expect(provisional.readonly).toBe(false);
    expect(provisional.saveGuard).toBe("guard");
  });
  it("refreshes an existing clean editor and keeps a divergent unsaved draft", async () => {
    const buffers = new Buffers(() => {});
    const file = {name:"notes.txt",path:"notes.txt",dir:false};
    buffers.workspace = "/owned"; buffers.workspaceId = "a".repeat(32);
    const disk = {path:file.path,language:"text",hash:"first",content:"First",save_guard:"original"};
    vi.mocked(api.file).mockResolvedValueOnce(disk);
    const opened = await buffers.open(file);
    vi.mocked(api.file).mockResolvedValueOnce({...disk,content:"Second",hash:"second",save_guard:"next"});
    expect(await buffers.refreshSavedSource(opened, () => true)).toContain("Reloaded");
    expect(buffers.values.get(file.path)).toBe(opened);
    expect(opened.text).toBe("Second");
    expect(opened.dirty).toBe(false);
    opened.edit("My draft");
    vi.mocked(api.file).mockResolvedValueOnce({...disk,content:"Third",hash:"third",save_guard:"third"});
    expect(await buffers.refreshSavedSource(opened, () => true)).toContain("changed on disk");
    expect(opened.text).toBe("My draft");
    expect(opened.base).toBe("Second");
    expect(opened.saveGuard).toBe("next");
  });
  it("retires a guarded read when its provisional startup editor closes", async () => {
    const buffers = new Buffers(() => {});
    const file = { name: "early.txt", path: "docs/early.txt", dir: false };
    const root = "/workspace", workspace = "1".repeat(32), content = "Readable early source";
    buffers.observeStartup({ version: 1, path: file.path, content, root_path: root, workspace_id: workspace });
    buffers.workspace = root;
    buffers.workspaceId = workspace;
    vi.mocked(api.file).mockResolvedValueOnce({ path: file.path, language: "text", hash: "hash", save_guard: "guard", content });
    const confirming = buffers.open(file);
    buffers.values.delete(file.path);
    await expect(confirming).rejects.toThrow("startup editor closed");
    expect(buffers.values.has(file.path)).toBe(false);
  });
  it("derives known authored candidates while counting generated and unclassified files explicitly", () => {
    // Contract: .hickory-source/frontends/contract.md
    const a = {name:"a.py",path:"src/a.py",dir:false,language:"python"};
    const scope=workspaceAnalysisScope([{name:"src",path:"src",dir:true,children:[a,{name:"out.py",path:"src/out.py",dir:false,language:"python",generated_by:"owner"}]},a,{name:"notes.md",path:"notes.md",dir:false,language:"markdown"},{name:"unknown",path:"unknown",dir:false}]);
    expect(scope.files.map(file=>file.path)).toEqual(["notes.md","src/a.py"]);expect(scope.generated).toBe(1);expect(scope.unclassified).toBe(1);
  });
  it("shares a raw read and never overwrites a draft that arrives while background acquisition is pending", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const changed=vi.fn(), buffers=new Buffers(changed), file={name:"background.py",path:"background.py",dir:false,language:"python"};
    let finish!: (value: Awaited<ReturnType<typeof api.file>>) => void;
    vi.mocked(api.file).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));
    const first=buffers.open(file), second=buffers.open(file);expect(first).toBe(second);
    const draft=new Buffer(file,"base","base-hash",()=>{});draft.edit("retained draft");buffers.values.set(file.path,draft);
    finish({path:file.path,language:"python",content:"late disk text",hash:"disk-hash"});
    expect(await first).toBe(draft);expect(await second).toBe(draft);expect(draft.text).toBe("retained draft");expect(draft.dirty).toBe(true);
    const other={name:"other.py",path:"other.py",dir:false,language:"python"};vi.mocked(api.file).mockResolvedValueOnce({path:other.path,language:"python",content:"new source",hash:"new-hash"});
    const loaded=await buffers.open(other);expect(loaded.text).toBe("new source");expect(changed).toHaveBeenCalledTimes(1);
  });
});
describe("simple workspace editor changes", () => {
  it("preflights all retained editors, applies both drafts, preserves local undo and guards coordinated restore", () => {
    // Contract: .hickory-source/frontends/contract.md
    const buffers = new Buffers(() => {});
    for (const [path, source] of Object.entries(changeCases.sources)) buffers.values.set(path, new Buffer({ path, name: path, dir: false }, source.text, "base", () => {}));
    const a = buffers.values.get("a.py")!, b = buffers.values.get("b.py")!;
    const screen = render(createElement(Fragment, null, createElement(Document, { buffer: a }), createElement(Document, { buffer: b })));
    try {
      const views = [...screen.container.querySelectorAll<HTMLElement>(".cm-editor")].map(element => EditorView.findFromDOM(element)!);
      const snapshots = new Map(Object.entries(changeCases.sources)), plan = planWorkspaceEdit("Rename", changeCases.edit, snapshots);
      b.saving = true; expect(() => applyWorkspacePlan(buffers, plan)).toThrow(/saving/); expect(a.text).toBe(changeCases.sources["a.py"].text); b.saving = false;
      act(() => applyWorkspacePlan(buffers, plan));
      expect(a.text).toBe(changeCases.after["a.py"]); expect(b.text).toBe(changeCases.after["b.py"]);
      expect(views.map(view => view.state.doc.toString())).toEqual([a.text, b.text]);
      const inverse = invertChangePlan(plan);
      act(() => { undo(views[0]); }); expect(a.text).toBe(changeCases.sources["a.py"].text);
      expect(() => applyWorkspacePlan(buffers, inverse)).toThrow(/Source changed/); expect(b.text).toBe(changeCases.after["b.py"]);
      act(() => { views[0].dispatch({ changes: { from: 0, to: views[0].state.doc.length, insert: plan.patches[0].after } }); applyWorkspacePlan(buffers, inverse); });
      expect(a.text).toBe(changeCases.sources["a.py"].text); expect(b.text).toBe(changeCases.sources["b.py"].text);
      expect([...screen.container.querySelectorAll<HTMLElement>(".cm-editor")].map(element => EditorView.findFromDOM(element))).toEqual(views);
    } finally { screen.unmount(); }
  });
});
describe("simple workspace change plans", () => {
  it("prepares the shared two-document Unicode change and its guarded inverse without editing source", () => {
    // Contract: .hickory-source/frontends/contract.md
    const sources = new Map(Object.entries(changeCases.sources));
    const plan = planWorkspaceEdit("Rename", changeCases.edit, sources);
    expect(Object.fromEntries(plan.patches.map(patch => [patch.path, patch.after]))).toEqual(changeCases.after);
    expect(validateChangePlan(plan, sources)).toBeNull();
    expect(() => validateReferencedTargets(plan, ["a.py", "b.py", "b.py"])).not.toThrow();
    expect(() => validateReferencedTargets(plan, ["a.py", "unopened.py"])).toThrow(/omitted reported reference/);
    const changed = new Map([...sources].map(([path, source]) => [path, { ...source, text: changeCases.after[path as keyof typeof changeCases.after] }]));
    expect(validateChangePlan(plan, changed)).toContain("Source changed");
    const inverse = invertChangePlan(plan); expect(validateChangePlan(inverse, changed)).toBeNull();
    for (const patch of inverse.patches) {
      let restored = patch.before;
      for (const edit of patch.edits.slice().reverse()) restored = restored.slice(0, edit.from) + edit.text + restored.slice(edit.to);
      expect(restored).toBe(changeCases.sources[patch.path as keyof typeof changeCases.sources].text);
    }
    expect(sources.get("a.py")?.text).toBe("é😀 one\nsecond");
    expect(validateChangePlan(inverse, new Map(changed).set("b.py", { text: "later edit", writable: true }))).toContain("b.py");
  });
  for (const item of changeCases.invalid) it(`refuses the entire shared proposal: ${item.name}`, () => {
    // Contract: .hickory-source/frontends/contract.md
    const sources = new Map(Object.entries(changeCases.sources).map(([path, source]) => [path, { ...source, writable: !("readonly" in item && item.readonly === path) }]));
    expect(() => planWorkspaceEdit(item.name, item.edit, sources)).toThrow();
    expect(sources.get("a.py")?.text).toBe("é😀 one\nsecond"); expect(sources.get("b.py")?.text).toBe("call(one)");
  });
});
describe("simple workspace search", () => {
  it("converts UTF-8 columns at scalar boundaries into UTF-16 positions", () => {
    // Contract: .hickory-source/frontends/contract.md
    expect(searchCharacter("é😀 target", 7)).toBe(4);
    expect(searchCharacter("é😀 target", 3)).toBeNull();
    expect(searchCharacter("é😀 target", -1)).toBeNull();
    expect(searchCharacter("é😀 target", 100)).toBeNull();
    expect(searchCharacter("", 0)).toBe(0);
  });
  it("rejects superseded query responses and guards source-bound navigation without saving", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const file = { name: "search.txt", path: "search.txt", dir: false };
    const source = "é😀 target", buffers = new Buffers(() => {}), buffer = new Buffer(file, source, "base", () => {});
    buffers.values.set(file.path, buffer);
    const hash = vi.spyOn(reviews, "sha256").mockResolvedValue("abc");
    let resolve!: (value: Awaited<ReturnType<typeof api.find>>) => void;
    vi.mocked(api.find).mockImplementationOnce(() => new Promise(done => { resolve = done; }));
    const close = vi.fn(), location = vi.fn();
    const navigate: Parameters<typeof WorkspaceSearch>[0]["navigate"] = async (file, line, character, current, verify) => {
      const owner = await buffers.open(file);
      if (!current()) return false;
      await verify(owner);
      if (!current()) return false;
      location({path: file.path, line, character, expectedSource: owner.text});
      return true;
    };
    const screen = render(createElement(WorkspaceSearch, { visible: true, close, buffers, files: [file], navigate, workspaceCurrent: () => true }));
    try {
      const input = screen.getByRole("textbox", { name: "Workspace search query" });
      fireEvent.change(input, { target: { value: "old" } }); fireEvent.click(screen.getByRole("button", { name: "Search" }));
      fireEvent.change(input, { target: { value: "target" } });
      await act(async () => resolve({ files: [{ path: "obsolete.txt", matches: [{ line: 1, text: "old", at: [{ column: 0, length: 3 }] }] }], truncated: false }));
      expect(screen.queryByText(/obsolete.txt/)).toBeNull();
      vi.mocked(api.find).mockResolvedValue({ files: [{ path: file.path, source_hash: "sha256:abc", matches: [{ line: 1, text: source, at: [{ column: 7, length: 6 }] }] }], truncated: true });
      fireEvent.click(screen.getByRole("checkbox", { name: "Whole word" })); fireEvent.click(screen.getByRole("button", { name: "Search" }));
      const result = await screen.findByRole("button", { name: "search.txt:1 · é😀 target" });
      expect(api.find).toHaveBeenLastCalledWith("target", { whole_word: true }); expect(screen.getByText(/Result cap reached/)).toBeTruthy();
      buffer.edit(source + " changed"); hash.mockResolvedValue("different");
      fireEvent.click(result); await screen.findByText(/Search source changed/); expect(close).not.toHaveBeenCalled();
      expect(buffer.text).toBe(source + " changed");
      buffer.edit(source); hash.mockResolvedValue("abc");
      vi.mocked(api.file).mockResolvedValueOnce({ path: file.path, content: "external\n" + source, hash: "disk", language: "text" });
      fireEvent.click(result); await screen.findByText(/changed on disk/);
      expect(close).not.toHaveBeenCalled(); expect(buffer.text).toBe(source);
      vi.mocked(api.file).mockResolvedValueOnce({ path: file.path, content: source, hash: "disk", language: "text" });
      fireEvent.click(result);
      await waitFor(() => expect(close).toHaveBeenCalledOnce());
      expect(location).toHaveBeenCalledWith(expect.objectContaining({ path: file.path, line: 1, character: 4, expectedSource: source }));
    } finally { screen.unmount(); hash.mockRestore(); }
  });

  // A same-name workspace can replace the file tree while search is visible.
  // Keep a same-workspace refresh in place and reject late replies from its predecessor.
  it("keeps same-workspace search and retires a prior checkout's results", async () => {
    let workspaceId = "first";
    const file = { name: "HtmlExporter.kt", path: "HtmlExporter.kt", dir: false };
    const result = { files: [{ path: file.path, matches: [{ line: 20, text: "old export branch", at: [{ column: 0, length: 3 }] }] }], truncated: false };
    const listing = () => ({ root: "Laura", root_path: "/owned/" + workspaceId + "/Laura", workspace_id: workspaceId, tree: [file] });
    vi.mocked(api.files).mockImplementation(async () => listing());
    vi.mocked(api.find).mockResolvedValueOnce(result);
    const screen = render(createElement(App));
    const refresh = () => {
      const menu = screen.getByText("File", { selector: "summary" });
      fireEvent.click(menu);
      fireEvent.click(within(menu.parentElement!).getByRole("button", { name: "Refresh" }));
    };
    try {
      await screen.findByRole("button", { name: file.name });
      fireEvent.click(screen.getByText("Edit", { selector: "summary" }));
      fireEvent.click(screen.getByRole("button", { name: "Find in workspace" }));
      const panel = () => within(screen.getByRole("dialog", { name: "Find in workspace" }));
      const query = () => panel().getByRole<HTMLInputElement>("textbox", { name: "Workspace search query" });
      const run = () => fireEvent.click(panel().getByRole("button", { name: "Search" }));
      fireEvent.change(query(), { target: { value: "old" } }); run();
      await panel().findByRole("button", { name: /old export branch/ });
      await act(async () => refresh());
      expect(query().value).toBe("old");
      expect(panel().getByRole("button", { name: /old export branch/ })).toBeTruthy();

      let reply!: (value: typeof result) => void;
      vi.mocked(api.find).mockImplementationOnce(() => new Promise(done => { reply = done; }));
      run();
      workspaceId = "second";
      await act(async () => refresh());
      expect(query().value).toBe("");
      await act(async () => reply(result));
      expect(panel().queryByRole("button", { name: /old export branch/ })).toBeNull();
    } finally { screen.unmount(); }
  });
});
describe("simple workspace overlays", () => {
  it("keeps Coverage and Git Review separate across same-name workspaces", async () => {
    let rootPath = "/owned/first/Laura", workspaceId = "first";
    const changedFiles = Array.from({ length: 21 }, (_, i) => ({ path: `case-${i}.kt`, index: " ", tree: "M" }));
    vi.mocked(api.files).mockImplementation(async () => ({
      root: "Laura", root_path: rootPath, workspace_id: workspaceId,
      tree: [{ name: `${workspaceId}.md`, path: `${workspaceId}.md`, dir: false }],
    }));
    vi.mocked(api.gitChanges).mockImplementation(async () => ({
      repository: true, branch: workspaceId, files: workspaceId === "first" ? changedFiles : [],
    }));
    vi.mocked(request).mockResolvedValue({ runs: [] });
    const consoleErrors = vi.spyOn(console, "error");
    const screen = render(createElement(App));
    const overlays = within(document.body);
    const refreshFiles = () => {
      fireEvent.click(screen.getByText("File", { selector: "summary" }));
      fireEvent.click(within(screen.getByText("File", { selector: "summary" }).parentElement!).getByRole("button", { name: "Refresh" }));
    };
    try {
      await screen.findByRole("button", { name: "first.md" });
      fireEvent.click(screen.getByText("Edit", { selector: "summary" }));
      fireEvent.click(screen.getByRole("button", { name: "Tests and coverage" }));
      fireEvent.click(screen.getByText("View", { selector: "summary" }));
      fireEvent.click(screen.getByRole("button", { name: "Repository changes" }));
      const coverage = await overlays.findByRole("dialog", { name: "Tests and code coverage" });
      const review = await overlays.findByRole("dialog", { name: "Repository changes" });
      const pathInput = within(coverage).getByLabelText("Project or source file") as HTMLInputElement;
      fireEvent.change(pathInput, { target: { value: "case-20.kt" } });
      await within(review).findByText("21 changed files · page 1");
      fireEvent.click(within(review).getByRole("button", { name: "Next" }));
      await within(review).findByText("21 changed files · page 2");

      const refreshCount = vi.mocked(api.files).mock.calls.length;
      refreshFiles();
      await waitFor(() => expect(api.files).toHaveBeenCalledTimes(refreshCount + 1));
      expect(overlays.getByRole("dialog", { name: "Tests and code coverage" })).toBe(coverage);
      expect(pathInput.value).toBe("case-20.kt");
      expect(within(review).getByText("21 changed files · page 2")).toBeTruthy();

      rootPath = "/owned/second/Laura";
      workspaceId = "second";
      refreshFiles();
      await screen.findByRole("button", { name: "second.md" });
      const newCoverage = overlays.getByRole("dialog", { name: "Tests and code coverage" });
      const newReview = overlays.getByRole("dialog", { name: "Repository changes" });
      expect(newCoverage).not.toBe(coverage);
      expect(newReview).not.toBe(review);
      expect((within(newCoverage).getByLabelText("Project or source file") as HTMLInputElement).value).toBe(".");
      await within(newReview).findByText("Repository changes second");
      await within(newReview).findByText("0 changed files · page 1");
      expect(overlays.getAllByRole("dialog")).toHaveLength(2);
      expect(consoleErrors.mock.calls.some(args => String(args[0]).includes("same key"))).toBe(false);
    } finally {
      screen.unmount();
      consoleErrors.mockRestore();
      vi.mocked(request).mockReset();
      vi.mocked(api.gitChanges).mockReset();
    }
  });
});

describe("simple workspace Problems", () => {
  it("keeps zero reports distinct from a source-bound empty report", async () => {
    const path = "Audit.kt", source = "val entry = 1";
    const file = { name: path, path, dir: false, language: "kotlin" };
    const client = new LspClient({ send: vi.fn(), onMessage: () => () => {}, handleFrame: () => false });
    let attached = false;
    let diagnosticSource: string | undefined;
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [file] });
    vi.mocked(api.file).mockResolvedValueOnce({ path, language: "kotlin", content: source, hash: "base" });
    vi.mocked(useWorkspaceLsp).mockImplementation(openPath => ({
      client: attached && openPath ? client : null, uri: "hick:///Audit.kt",
      ready: attached && !!openPath, diagnostics: [], diagnosticSource,
    }));
    const screen = render(createElement(App));
    try {
      fireEvent.click(await screen.findByRole("button", { name: path }));
      const editor = await waitFor(() => {
        const node = screen.container.querySelector<HTMLElement>('.cm-content[aria-label="Audit.kt"]');
        expect(node).toBeTruthy();
        return EditorView.findFromDOM(node!)!;
      });
      fireEvent.click(await screen.findByRole("button", { name: "Problems: 0 reported" }));
      const panel = within(await within(document.body).findByRole("dialog", { name: "Workspace Problems" }));
      await panel.findByText("Audit.kt · kotlin");
      await panel.findByText("No language session is attached to this file.");

      attached = true;
      screen.rerender(createElement(App));
      await panel.findByText("No source-bound diagnostic report for the current file text.");
      diagnosticSource = source;
      screen.rerender(createElement(App));
      await panel.findByText("The provider reported zero problems for the current file text.");

      act(() => editor.dispatch({ changes: { from: editor.state.doc.length, insert: " + 1" } }));
      await panel.findByText("No source-bound diagnostic report for the current file text.");
      expect(panel.queryByText(/provider reported zero/)).toBeNull();
    } finally {
      screen.unmount();
      client.dispose();
      vi.mocked(useWorkspaceLsp).mockReturnValue({ client: null, uri: "hick:///", ready: false, diagnostics: [] });
    }
  });
  it("keeps stale/unversioned messages but navigates only current source locations", () => {
    // Contract: .hickory-source/frontends/contract.md
    const range = { start: { line: 0, character: 1 }, end: { line: 0, character: 2 } };
    const records = reportedProblems([
      { uri: "hick:///a.py", currentSource: "value", source: "value", diagnostics: [{ message: "current", range }] },
      { uri: "hick:///b.py", currentSource: "changed", source: "old", diagnostics: [{ message: "stale", range }] },
      { uri: "hick:///c.py", currentSource: "value", diagnostics: [{ message: "unversioned", range }] },
      { uri: "hick:///d.py", currentSource: "value", source: "value", diagnostics: [{ message: "other", range }] },
    ]);
    const mixed = reportedProblems([{ uri: "hick:///mixed.py", currentSource: "value", diagnostics: [{ message: "bound", sourceSnapshot: "value", range }, { message: "unknown", range }] }]);
    expect(mixed.map(problem => problem.current)).toEqual([true, false]);
    expect(records.map(problem => problem.current)).toEqual([true, false, false, true]);
    expect(records).toHaveLength(4); expect(records[1].location).toBeNull(); expect(records[2].location).toBeNull();
    expect(nextWorkspaceProblem(records, records[0].location, 1)?.path).toBe("d.py");
    expect(nextWorkspaceProblem(records, records[3].location, 1)?.path).toBe("a.py");
    expect(nextWorkspaceProblem(records, records[0].location, -1)?.path).toBe("d.py");
    expect(nextWorkspaceProblem(records.filter(problem => !problem.current), null, 1)).toBeNull();
  });
  it("opens the status/menu Problems overlay without replacing the editor and retires locations while typing", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const source = "value = 1", range = { start: { line: 0, character: 1 }, end: { line: 0, character: 5 } };
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [{ name: "problem.py", path: "problem.py", dir: false }] });
    vi.mocked(api.file).mockResolvedValueOnce({ path: "problem.py", language: "python", content: source, hash: "base" });
    vi.mocked(useWorkspaceLsp).mockImplementation(path => ({ client: null, uri: path ? "hick:///problem.py" : "hick:///", ready: !!path, ...(path ? { diagnosticSource: source } : {}), diagnostics: path ? [{ message: "Example issue", range }] : [] }));
    const screen = render(createElement(App));
    try {
      fireEvent.click(await screen.findByRole("button", { name: "problem.py" }));
      const content = await waitFor(() => { const node = screen.container.querySelector<HTMLElement>('.cm-content[aria-label="problem.py"]'); expect(node).toBeTruthy(); return node!; });
      const view = EditorView.findFromDOM(content)!;
      fireEvent.click(await screen.findByRole("button", { name: "Problems: 1 reported" }));
      const panel = await within(document.body).findByRole("dialog", { name: "Workspace Problems" });
      const location = within(panel).getByRole("button", { name: "problem.py:1 · Example issue" });
      expect(location.hasAttribute("disabled")).toBe(false);
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: " changed" } }));
      await waitFor(() => expect(within(panel).getByRole("button", { name: "problem.py · Example issue" }).hasAttribute("disabled")).toBe(true));
      expect(panel.textContent).toContain("stale or unversioned");
      expect(EditorView.findFromDOM(content)).toBe(view); expect(api.saveFile).not.toHaveBeenCalledWith("problem.py", expect.anything(), expect.anything());
      fireEvent.keyDown(panel, { key: "Escape" });
      await waitFor(() => expect(within(document.body).queryByRole("dialog", { name: "Workspace Problems" })).toBeNull());
      expect(view.hasFocus).toBe(true);
    } finally { screen.unmount(); vi.mocked(useWorkspaceLsp).mockReturnValue({ client: null, uri: "hick:///sample.ts", ready: false, diagnostics: [] }); }
  });
});
describe("simple semantic navigation", () => {
  it("resolves workspace navigation outside the folder currently shown in Files", async () => {
    // Contract: .hickory-source/frontends/contract.md
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [{ name: "src", path: "src", dir: true, children: [{ name: "a.py", path: "src/a.py", dir: false }] }, { name: "other.py", path: "other.py", dir: false }] });
    vi.mocked(api.file).mockResolvedValueOnce({ path: "other.py", language: "python", content: "value = 1", hash: "base" });
    const prompt = vi.spyOn(window, "prompt").mockReturnValue("src");
    const screen = render(createElement(App));
    try {
      await screen.findByRole("button", { name: "other.py" });
      fireEvent.click(screen.getByText("File", { selector: "summary" }));
      fireEvent.click(screen.getByRole("button", { name: "Open Directory" }));
      await waitFor(() => expect(screen.queryByRole("button", { name: "other.py" })).toBeNull());
      act(() => openLocation("other.py", 1, 2));
      await waitFor(() => expect(screen.container.querySelector('.cm-content[aria-label="other.py"]')).toBeTruthy());
      expect(api.file).toHaveBeenCalledWith("other.py");
    } finally { screen.unmount(); prompt.mockRestore(); }
  });
  it("retains departure carets, prunes forward branches and bounds navigation history", () => {
    // Contract: .hickory-source/frontends/contract.md
    const point = (name: string, line = 0) => ({ uri: `hick:///${name}.py`, range: { start: { line, character: 0 }, end: { line, character: 0 } } });
    const trail = new NavigationTrail();
    trail.record(point("a"), point("b")); trail.record(point("b"), point("c"));
    expect(trail.step(-1, point("c"))?.uri).toBe("hick:///b.py");
    expect(trail.step(1, point("b", 7))?.uri).toBe("hick:///c.py");
    expect(trail.step(-1, point("c"))?.range.start.line).toBe(7);
    trail.record(point("b", 7), point("d")); expect(trail.canForward).toBe(false);
    const count = trail.entries.length; trail.record(point("d"), point("d")); expect(trail.entries).toHaveLength(count);
    for (let i = 0; i < 300; i++) trail.record(null, point(String(i)));
    expect(trail.entries).toHaveLength(256); expect(trail.step(1, null)).toBeNull();
  });
  it("keeps history stationary until the prepared editor reveal is accepted", () => {
    const point = (file: string, character: number, hash: string) => {
      const start = {line: 0, character};
      return {uri: `hick:///${file}`, range: {start, end: start}, hash};
    };
    const a = point("a.py", 2, "origin"), b = point("b.py", 3, "destination");
    const trail = new NavigationTrail<typeof a>();
    trail.record(a, b);
    const departure = point("b.py", 6, "latest");
    const abandoned = trail.prepareStep(-1, departure)!;
    expect(abandoned.location).toEqual(a);
    expect(trail.index).toBe(1);
    expect(trail.canForward).toBe(false);
    const retry = trail.prepareStep(-1, departure)!;
    expect(retry.accept()).toBe(true);
    expect(abandoned.accept()).toBe(false);
    expect(trail.step(1, null)).toEqual(departure);
    expect(abandoned.current()).toBe(false);
  });
  it("normalizes symbol trees, flat symbols and incoming/outgoing call items", () => {
    // Contract: .hickory-source/frontends/contract.md
    const range = { start: { line: 2, character: 4 }, end: { line: 2, character: 8 } };
    const values = semanticLocations([{ name: "Outer", range, selectionRange: range, children: [{ name: "Inner", range: { ...range, start: { line: 3, character: 6 } } }] }, { name: "Other", location: { uri: "hick:///other.py", range } }, { from: { name: "Caller", uri: "hick:///call.py", selectionRange: range } }, { to: { name: "Callee", uri: "hick:///callee.py", selectionRange: range } }], "hick:///source.py");
    expect(values.map(value => [value.name, value.uri, value.range.start.line])).toEqual([["Outer", "hick:///source.py", 2], ["Inner", "hick:///source.py", 3], ["Other", "hick:///other.py", 2], ["Caller", "hick:///call.py", 2], ["Callee", "hick:///callee.py", 2]]);
  });
  it("uses provider outline/symbol/call/type protocols and flushes the current source before position reads", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const methods: string[] = [];
    let syncs = 0;
    const range = { start: { line: 1, character: 2 }, end: { line: 1, character: 5 } };
    const item = { uri: "hick:///call.py", name: "caller", range, selectionRange: range };
    const channel = createLspChannel({ send(frame) {
      const message = decodeLspFrame(frame);
      if (!message || !("method" in message) || !("id" in message)) return;
      methods.push(message.method);
      const result = ["textDocument/prepareCallHierarchy","textDocument/prepareTypeHierarchy","typeHierarchy/supertypes","typeHierarchy/subtypes"].includes(message.method) ? [item]
        : message.method === "callHierarchy/incomingCalls" ? [{ from: item }]
        : message.method === "callHierarchy/outgoingCalls" ? [{ to: item }]
        : message.method === "textDocument/documentSymbol" ? [{ name: "Outline", range, selectionRange: range }]
        : [{ name: "Workspace", location: { uri: item.uri, range } }];
      queueMicrotask(() => channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", id: message.id, result })));
    } });
    const client = new LspClient(channel); const unregister = client.registerDocumentSync("hick:///source.py", () => { syncs++; });
    try {
      expect((await querySemanticLocations(client, "hick:///source.py", range.start, "outline"))[0].name).toBe("Outline");
      expect((await querySemanticLocations(client, "hick:///source.py", range.start, "symbols", "Work"))[0].name).toBe("Workspace");
      expect((await querySemanticLocations(client, "hick:///source.py", range.start, "callers"))[0].name).toBe("caller");
      expect((await querySemanticLocations(client, "hick:///source.py", range.start, "callees"))[0].name).toBe("caller");
      expect((await querySemanticLocations(client,"hick:///source.py",range.start,"supertypes"))[0].name).toBe("caller");
      expect((await querySemanticLocations(client,"hick:///source.py",range.start,"subtypes"))[0].name).toBe("caller");
      expect(methods).toContain("typeHierarchy/supertypes");expect(methods).toContain("typeHierarchy/subtypes");
      expect(syncs).toBe(5); expect(methods).toContain("workspace/symbol"); expect(methods).toContain("callHierarchy/incomingCalls"); expect(methods).toContain("callHierarchy/outgoingCalls");
    } finally { unregister(); client.dispose(); }
  });
  it("does not let a previous buffer consume the destination symbol reveal", async () => {
    // Contract: .hickory-source/frontends/contract.md
    const first = new Buffer({ path: "first.py", name: "first.py", dir: false }, "first = 1", "hash", () => {});
    const second = new Buffer({ path: "second.py", name: "second.py", dir: false }, "prefix = 0; value = 42", "hash", () => {});
    const screen = render(createElement(Document, { buffer: first }));
    const original = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
    try {
      queueReveal("second.py", 1, 12);
      screen.rerender(createElement(Document, { buffer: second }));
      await waitFor(() => {
        const target = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
        expect(target).not.toBe(original); expect(target.state.selection.main.head).toBe(12);
      });
      expect(original.state.selection.main.head).toBe(0); expect(second.dirty).toBe(false);
    } finally { screen.unmount(); }
  });
  it("retains a symbol column until the destination editor claims it once", () => {
    // Contract: .hickory-source/frontends/contract.md
    resetReveals(); queueReveal("other.py", 3, 7);
    expect(claimRevealPosition("other.py")).toEqual({ line: 3, character: 7 });
    expect(claimRevealPosition("other.py")).toBeNull();
    queueReveal("legacy.py", 4, 2); expect(claimReveal("legacy.py")).toBe(4);
    expect(claimRevealPosition("legacy.py")).toBeNull();
  });
  it("admits authored workspace paths and rejects unsafe provider identities", () => {
    // Contract: .hickory-source/frontends/contract.md
    expect(semanticPath("hick:///src/my%20file.py")).toBe("src/my file.py");
    expect(semanticPath("hick:///literal%2520.py")).toBe("literal%20.py");
    expect(semanticPath("hick:///name%3F%23.py")).toBe("name?#.py");
    for (const uri of ["hick:/main.py", "hick:///a?", "file:///etc/passwd", "hick-output:///main.py", "hick://host/main.py", "hick:///../secret", "hick:///%2e%2e/secret", "hick:///a%5Cb", "hick:///a%00b", "hick:///a?context=old", "hick:///a#old"]) expect(semanticPath(uri), uri).toBeNull();
    const location = { uri: "hick:///a.py", range: { start: { line: 2, character: 4 }, end: { line: 2, character: 4 } } };
    expect(semanticLocations([location, location, { ...location, range: { ...location.range, start: { line: -1, character: 0 } } }])).toEqual([location]);
  });
  it("queries the real bridge protocol and refuses obsolete locations without replacing the editor", async () => {
    // Contract: .hickory-source/frontends/contract.md
    let answer: (() => void) | undefined;
    let incomingRequests = 0;
    const channel = createLspChannel({ send(frame) {
      const message = decodeLspFrame(frame);
      if (!message || !("method" in message) || !("id" in message)) return;
      const reply = () => channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", id: message.id, result: [{ uri: "hick:///other.py", range: { start: { line: 1, character: 0 }, end: { line: 1, character: 4 } } }] }));
      if (message.method === "textDocument/references") { expect(message.params).toMatchObject({ textDocument: { uri: "hick:///sample.py" }, context: { includeDeclaration: true } }); answer = reply; }
      else if (message.method === "textDocument/prepareCallHierarchy") {
        const range = { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } };
        answer = () => channel.handleFrame(encodeLspFrame({
          jsonrpc: "2.0", id: message.id,
          result: [{ name: "value", kind: 12, uri: "hick:///sample.py", range, selectionRange: range }],
        }));
      } else {
        if (message.method === "callHierarchy/incomingCalls") incomingRequests++;
        queueMicrotask(() => channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", id: message.id, result: null })));
      }
    } });
    const client = new LspClient(channel);
    vi.mocked(useWorkspaceLsp).mockReturnValue({ client, uri: "hick:///sample.py", ready: true, diagnostics: [] });
    const buffer = new Buffer({ path: "sample.py", name: "sample.py", dir: false }, "value = 1", "hash", () => {});
    const screen = render(createElement(Document, { buffer }));
    const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
    try {
      act(() => openSemanticQuery(buffer, "references"));
      await waitFor(() => expect(answer).toBeDefined());
      act(() => { answer!(); });
      const result = await screen.findByRole("button", { name: "other.py:2:1" });
      expect(result.hasAttribute("disabled")).toBe(false);
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: "\n# unsaved" }, annotations: isolateHistory.of("full") }));
      await waitFor(() => expect(result.hasAttribute("disabled")).toBe(true));
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)).toBe(view);
      act(() => openSemanticQuery(buffer, "references"));
      act(() => view.dispatch({ selection: { anchor: 2 } }));
      act(() => { answer!(); });
      await waitFor(() => expect(screen.getByRole("dialog", { name: "Semantic locations" }).textContent).toContain("Source or cursor changed"));
      act(() => { undo(view); }); expect(buffer.text).toBe("value = 1");

      // An obsolete preparation must not start a second provider request.
      answer = undefined;
      act(() => openSemanticQuery(buffer, "callers"));
      await waitFor(() => expect(answer).toBeDefined());
      act(() => view.dispatch({ selection: { anchor: view.state.selection.main.head === 0 ? 1 : 0 } }));
      await act(async () => { answer!(); });
      expect(incomingRequests).toBe(0);
      expect(screen.getByRole("dialog", { name: "Semantic locations" }).textContent).toContain("Source or cursor changed");
    } finally { screen.unmount(); client.dispose(); vi.mocked(useWorkspaceLsp).mockReturnValue({ client: null, uri: "hick:///sample.ts", ready: false, diagnostics: [] }); }
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
        : message.method === "hick/completion" ? [{ label: "languageValue", insertText: "languageValue" }] : [];
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
      vi.mocked(useWorkspaceLsp).mockReturnValue({ client, uri: "hick:///sample.ts", ready: true, diagnosticSource: buffer.text, diagnostics: [{ range: { start: { line: 0, character: 4 }, end: { line: 0, character: 5 } }, severity: 1, message: "Example type error" }] });
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
</ui:copy>

The new-tab shortcut witness checks both platform modifiers, default prevention, focus and draft retention; shifted and Alt combinations leave the tab count unchanged.

A stale save retains text and baseline. A successful save acknowledges only the submitted snapshot, including when more text arrived while the request was pending. Generated files do not dispatch whole-file writes.

<ui:copy id="simple-c6aff51ad5-1">
describe("simple file commands", () => {
  it.each(["ctrlKey", "metaKey"])("creates and focuses a new tab with %s+N while retaining edits", async modifier => {
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", tree: [] });
    const screen = render(createElement(App));
    try {
      const press = (extra = {}) => fireEvent.keyDown(window, { key: "n", [modifier]: true, ...extra });
      expect(press()).toBe(false);
      const view = EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!;
      act(() => view.dispatch({ changes: { from: 0, insert: "retained draft" } }));
      expect(press({ shiftKey: true })).toBe(true);
      expect(press({ altKey: true })).toBe(true);
      expect(screen.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(1);
      expect(press()).toBe(false);
      expect(screen.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2);
      expect(screen.getByRole("tab", { name: "Untitled 2" }).getAttribute("aria-selected")).toBe("true");
      fireEvent.click(screen.getByRole("tab", { name: /Untitled 1/ }));
      expect(EditorView.findFromDOM(screen.container.querySelector(".cm-editor")!)!.state.doc.toString()).toBe("retained draft");
    } finally { screen.unmount(); }
  });
  it("exposes the complete File menu and saves a new tab with a guarded destination", async () => {
    // Contract: .hickory-source/frontends/contract.md
    vi.mocked(api.files).mockResolvedValue({ root: "workspace",root_path:"/owned",workspace_id:"a".repeat(32), tree: [] });
    vi.mocked(api.fileOp).mockClear();
    vi.mocked(api.openFile).mockResolvedValue({ path: "new.py" });
    let saved = "";
    vi.mocked(api.file).mockImplementation(async () => ({ path: "new.py", content: saved, language: "python", hash: "empty-hash" }));
    vi.mocked(request).mockImplementation(async(_method,_path,body)=>{const b=body as {source:string;workspace:string;request_id:string};saved=b.source;let hash=0xcbf29ce484222325n;for(const byte of new TextEncoder().encode(saved))hash=BigInt.asUintN(64,(hash^BigInt(byte))*0x100000001b3n);return {version:1,history_protocol:1,workspace:b.workspace,namespace:"/owned",path:"new.py",source:saved,source_hash:await historySourceHash(saved),hash:hash.toString(16).padStart(16,"0"),request_id:b.request_id,save_guard:"created",history:{available:true,outcome:"published",id:b.request_id}};});
    const prompt = vi.spyOn(window, "prompt").mockReturnValue("new.py");
    const screen = render(createElement(App));
    try {
      await waitFor(()=>expect(screen.container.querySelector("footer")?.textContent).toContain("workspace"));
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
      expect(request).toHaveBeenLastCalledWith("POST","/api/source-history/create",expect.objectContaining({source:"print(1)",path:"new.py"}));expect(api.fileOp).not.toHaveBeenCalled();
      expect(screen.queryByRole("tab", { name: /Untitled/ })).toBeNull();
    } finally { screen.unmount(); prompt.mockRestore(); }
  });
  it("retains a draft when Save As refuses an existing destination", async () => {
    // Contract: .hickory-source/frontends/contract.md
    vi.mocked(api.files).mockResolvedValue({ root: "workspace",root_path:"/owned",workspace_id:"a".repeat(32), tree: [] });
    vi.mocked(request).mockRejectedValueOnce(new Error("already exists"));
    const prompt = vi.spyOn(window, "prompt").mockReturnValue("existing.py");
    const screen = render(createElement(App));
    try {
      await waitFor(()=>expect(screen.container.querySelector("footer")?.textContent).toContain("workspace"));
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
    vi.mocked(api.doc).mockResolvedValueOnce({ id: "managed", path, source: "# Readable source", updated_at: "",save_guard:"managed-physical" });
    const buffer = await new Buffers(() => {}).open({ name: path, path, dir: false, doc_id: "managed" });
    expect(api.doc).toHaveBeenLastCalledWith("managed");
    expect(api.file).not.toHaveBeenCalled();
    expect(buffer.text).toBe("# Readable source"); expect(buffer.base).toBe(buffer.text);
    buffer.edit("# Edited");vi.mocked(request).mockImplementationOnce(async(_method,_route,body)=>saveReceipt(path,body));
    await buffer.save();
    expect(request).toHaveBeenLastCalledWith("PUT","/api/source-history/docs/managed",expect.objectContaining({version:1,source:"# Edited",save_guard:"managed-physical",expected:{source:"# Readable source",disk:"# Readable source"}}));
    expect(buffer.base).toBe("# Edited");
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
    vi.mocked(request).mockRejectedValueOnce(new Error("changed on disk"));
    const buffer = new Buffer(plain, "before", "hash-before", () => {});buffer.saveGuard="original-physical";
    buffer.edit("mine"); await buffer.save();
    expect(request).toHaveBeenLastCalledWith("PUT","/api/source-history/file",expect.objectContaining({path:"notes.txt",content:"mine",base_hash:"hash-before",save_guard:"original-physical"}));
    expect(buffer.text).toBe("mine"); expect(buffer.base).toBe("before"); expect(buffer.dirty).toBe(true);
    expect(buffer.error).toContain("edits are retained");
  });
  it("does not mark later keystrokes as saved", async () => {
    let finish!: (value: unknown) => void,body:unknown;
    vi.mocked(request).mockImplementationOnce(async(_method,_route,value)=>{body=value;return new Promise(resolve => { finish = resolve; });});
    const buffer = new Buffer(plain, "before", "hash-before", () => {});buffer.saveGuard="original-physical";
    buffer.edit("submitted"); const saving = buffer.save();buffer.edit("typed later");
    await waitFor(()=>expect(finish).toBeTypeOf("function"));finish(await saveReceipt(plain.path,body,"hash-submitted"));await saving;
    expect(buffer.base).toBe("submitted"); expect(buffer.text).toBe("typed later"); expect(buffer.dirty).toBe(true);
  });
  it("uses source and disk expectations for literate source", async () => {
    vi.mocked(request).mockImplementationOnce(async(_method,_route,body)=>saveReceipt(plain.path,body));
    const buffer = new Buffer({ ...plain, doc_id: "doc" }, "before", "hash", () => {});buffer.saveGuard="original-physical";
    buffer.edit("next"); await buffer.save();
    expect(request).toHaveBeenLastCalledWith("PUT","/api/source-history/docs/doc",expect.objectContaining({source:"next",save_guard:"original-physical",expected:{source:"before",disk:"before"}}));expect(buffer.base).toBe("next");
  });
  it("does not edit or save generated outputs", async () => {
    const buffer = new Buffer({ ...plain, generated_by: "owner" }, "generated", "hash", () => {});
    const calls = vi.mocked(api.saveFile).mock.calls.length;
    buffer.edit("overwrite"); await buffer.save();
    expect(buffer.text).toBe("generated"); expect(vi.mocked(api.saveFile).mock.calls.length).toBe(calls);
  });
});
describe("terminal and agent tab identity", () => {
  // Contract: .hickory-source/frontends/contract.md — attaching is not spawning.
  it("opens the terminal once when shown again", async () => {
    vi.mocked(api.openTerminal).mockResolvedValueOnce({ id: "pty" } as Awaited<ReturnType<typeof api.openTerminal>>);
    const session = new TerminalSession();
    await session.open(); await session.open();
    expect(api.openTerminal).toHaveBeenCalledTimes(1); expect(session.id).toBe("pty");
  });

});

describe("simple literate subscription agent", () => {
  it("discards a history reply from a retired workspace", async () => {
    const session = new AgentSession();
    let current = true;
    session.workspaceCurrent = () => current;
    const turn = { id: "foreign", parent_id: null, prompt: "Not from this workspace", answer: null, error: null, status: "ok",
      created_at: "", provider: "", model: "", usage: null };
    let deliver!: (reply: Awaited<ReturnType<typeof api.agentTurns>>) => void;
    vi.mocked(api.agentTurns).mockImplementationOnce(() => new Promise(resolve => { deliver = resolve; }));
    const screen = render(createElement(AgentHistory, { session, visible: true }));
    try {
      expect(deliver).toBeTypeOf("function");
      current = false;
      await act(async () => deliver({ turns: [turn], message_forks: true, provider: "", model: "",
        totals: { usd: null, input: 0, output: 0, cache_read: 0, cache_write: 0 } }));
      expect(session.turns).toEqual([]);
      expect(session.historyLoaded).toBe(false);
    } finally { screen.unmount(); }
  });
  it("reopens a nested branch from Agent History, cuts at the chosen message, then opens an empty conversation without sending", async () => {
    const row = (id: string, parent_id: string | null, prompt: string, session: string, parent_is_user = false) => ({
      id, parent_id, parent_is_user, prompt, session, answer: `Answer ${id}`, error: null, status: "ok", backend: "python",
      created_at: "", provider: "codex-subscription", model: "", usage: null,
    });
    const turns = [row("root", null, "Root conversation", "original.md"),
      row("later", "root", "Original continuation", "original.md"),
      row("fork", "root", "Retained follow-up", "fork.md", true)];
    expect(agentTranscript(turns, "fork").map(({ turn, userOnly }) => [turn.id, userOnly])).toEqual([["root", true], ["fork", false]]);
    vi.mocked(api.files).mockResolvedValueOnce({ root: "workspace", tree: [] });
    vi.mocked(api.agentTurns).mockResolvedValue({ turns, message_forks: true, provider: "", model: "", totals: { usd: null, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    const before = vi.mocked(api.agent).mock.calls.length;
    const screen = render(createElement(App));
    try {
      fireEvent.click(await screen.findByRole("tab", { name: "Agent History" }));
      const branch = await screen.findByRole("button", { name: /Branch from user · Retained follow-up/ });
      expect(branch.closest("li")?.parentElement?.closest("li")?.textContent).toContain("Root conversation");
      fireEvent.click(branch);
      const log = within(await screen.findByRole("log", { name: "Conversation" }));
      expect(log.getByText("Answer fork")).toBeTruthy();
      expect(log.queryByText("Answer root")).toBeNull();
      expect(log.queryByText("Answer later")).toBeNull();
      fireEvent.click(log.getByRole("button", { name: "Branch from assistant message" }));
      expect(screen.getByText(/New branch from assistant message/)).toBeTruthy();
      fireEvent.click(log.getAllByRole("button", { name: "Branch from user message" }).at(-1)!);
      expect(log.queryByText("Answer fork")).toBeNull();
      fireEvent.click(within(screen.getByRole("region", { name: "Agent History" })).getByRole("button", { name: "New conversation" }));
      expect(log.getByText("What shall we work on?")).toBeTruthy();
      expect(api.agent).toHaveBeenCalledTimes(before);
    } finally {
      screen.unmount();
      vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "", model: "", totals: { usd: null, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    }
  });
  it("sends a message fork exactly once and retains the branch point after a refused send", async () => {
    const session = new AgentSession();
    const turn = { id: "origin", parent_id: null, prompt: "Earlier", answer: "Earlier reply", error: null, status: "ok",
      created_at: "", provider: "", model: "", usage: null };
    session.setTurns([turn]);
    session.messageForks = true;
    session.select(turn.id, "user");
    session.prompt = "Try another way";
    vi.mocked(api.agent).mockRejectedValueOnce(new Error("Unavailable"));
    await session.send();
    expect(session.selection).toEqual({ id: "origin", fork: "user" });
    expect(session.prompt).toBe("Try another way");
    expect(session.select(null)).toBe(false);
    session.messageForks = false;
    const before = vi.mocked(api.agent).mock.calls.length;
    await session.send();
    expect(api.agent).toHaveBeenCalledTimes(before);
    session.messageForks = true;
    vi.mocked(api.agent).mockResolvedValueOnce({ session_id: "fork" });
    await session.send();
    expect(api.agent).toHaveBeenLastCalledWith("workspace", "Try another way", "origin", "codex-subscription", "", "python", undefined, undefined, undefined, "user");
    expect(session.transcript.map(({ turn, userOnly }) => [turn.id, userOnly])).toEqual([["origin", true], ["fork", false]]);
  });
  // Contract: .hickory-source/frontends/contract.md — only Python executes code.
  it("sends the exact draft with subscription, model and effort; retains edits made during send", async () => {
    let resolve!: (value: { session_id: string }) => void;
    vi.mocked(api.agent).mockReturnValueOnce(new Promise(done => { resolve = done; }));
    const session = new AgentSession();
    session.provider = "claude-subscription"; session.model = "sonnet"; session.effort = "high";
    session.prompt = "# Task\n\n```python\nprint('café')\n```";
    const submitted = session.prompt;
    const sending = session.send();
    await session.send(); // A second click cannot create another turn.
    expect(api.agent).toHaveBeenLastCalledWith("workspace", submitted, undefined, "claude-subscription", "sonnet", "python", undefined, undefined, "high");
    session.prompt = "# Next task";
    resolve({ session_id: "turn-1" }); await sending;
    expect(session.prompt).toBe("# Next task");
    expect(session.turns.at(-1)?.prompt).toBe(submitted);
    const history = session.document("turn-1-user", submitted);
    history.edit("changed"); await history.save(); await session.draft.save();
    expect(history.text).toBe(submitted); expect(history.readonly).toBe(true);
    expect(session.draft.transient).toBe(true);
  });
  it("retains a failed draft without retry or API-provider fallback", async () => {
    vi.mocked(api.agent).mockRejectedValueOnce(new Error("Sign in to the subscription"));
    const session = new AgentSession(); session.prompt = "Keep my draft";
    const before = vi.mocked(api.agent).mock.calls.length;
    await session.send();
    expect(session.prompt).toBe("Keep my draft"); expect(session.turns).toEqual([]);
    expect(session.error).toContain("not retried"); expect(api.agent).toHaveBeenCalledTimes(before + 1);
  });
  it("shows live agent activity while keeping protocol output and tool details out of the answer", async () => {
    const session = new AgentSession();
    session.prompt = "First draft";
    let accept!: (response: {session_id: string}) => void;
    vi.mocked(api.agent).mockReturnValueOnce(new Promise(resolve => { accept = resolve; }));
    const screen = render(createElement(Agent, { session }));
    const pending = session.send();
    const receive = (event: import("../api/types").AgentWsEvent, id = "live") =>
      session.receive({run_id: id, exec_id: "agent", event});
    receive({kind: "thinking"});
    await act(async () => { accept({session_id: "live"}); await pending; });
    expect(screen.getByText("Waiting for the model…")).toBeTruthy();
    expect(screen.getByRole("button", {name: "Stop"})).toBeTruthy();
    act(() => receive({kind: "tool_started", name: "read_file", data: JSON.stringify({
      arguments: {path: "project.md", from: 1, to: 20},
    })}));
    expect(screen.getByText("Read file · project.md")).toBeTruthy();
    act(() => {
      receive({kind: "token", data: "worker protocol"});
      receive({kind: "tool_finished", name: "read_file", ok: true, text: "tool receipt"});
      receive({kind: "tool_started", name: "edit_doc", data: "private invocation"});
      receive({kind: "token", data: "other turn"}, "other");
      session.prompt = "Next draft";
    });
    expect(screen.queryByText("worker protocol")).toBeNull();
    expect(screen.queryByText("private invocation")).toBeNull();
    expect(screen.getByText("Tool calls (2)").closest("details")?.open).toBe(false);
    expect(screen.getByText("Read file · project.md · Completed").closest("details")?.open).toBe(false);
    expect(screen.getByText("Propose source edit · Running")).toBeTruthy();
    act(() => receive({kind: "response_complete", text: "## Result\n\n**Ready.**"}));
    expect(screen.getByRole("heading", {name: "Result"})).toBeTruthy();
    expect(session.prompt).toBe("Next draft");
    act(() => { session.setTurns(session.turns.map(turn => ({...turn, status: "ok", answer: "## Result"}))); session.publish(); });
    expect(screen.queryByRole("button", {name: "Stop"})).toBeNull();
    expect(screen.getByRole("button", {name: "Send"})).toBeTruthy();
  });
  it("opens Debug only on an explicit recorded fence click and retires a late capture", async () => {
    const prompt = "Question 😀\r\n```python\r\nvalue = 41\r\nprint(value + 1)\r\n```";
    const answer = "Answer\n\n~~~python\nprint(42)\n~~~";
    const session = new AgentSession();
    session.setTurns([{ id: "fence-turn", prompt, answer, status: "ok", parent_id: null, error: null, created_at: "", provider: "codex-subscription", model: "test", usage: null }]);
    vi.mocked(api.agentTurns).mockReturnValueOnce(new Promise(() => {}));
    const before = vi.mocked(request).mock.calls.length;
    const screen = render(createElement(Agent, { session }));
    const user = screen.container.querySelector<HTMLElement>(".user-message")!;
    const assistant = screen.container.querySelectorAll<HTMLElement>(".agent-message")[1];
    fireEvent.click(await within(user).findByLabelText("Fence actions on line 2"));
    const userLaunch = await within(user).findByLabelText("Debug python fence on line 2");
    expect(await within(assistant).findByLabelText("Fence actions on line 3")).toBeTruthy();
    expect(vi.mocked(request).mock.calls.slice(before).some(([, path]) => /fence-(debug|run)$/.test(path))).toBe(false);
    const content = user.querySelector<HTMLElement>(".cm-content")!;
    const view = EditorView.findFromDOM(content)!;
    act(() => view.dispatch({ changes: { from: 0, insert: "changed" } }));
    expect(view.state.doc.toString()).toBe(prompt.replaceAll("\r\n", "\n"));
    let deliver!: (value: unknown) => void;
    vi.mocked(request).mockImplementationOnce(() => new Promise(resolve => { deliver = resolve; }));
    fireEvent.click(userLaunch);
    const source_hash = await historySourceHash(prompt);
    await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/docs/workspace/agent/turns/fence-turn/fence-debug", { side: "user", source_hash, opening_line: 1 }));
    screen.unmount();
    vi.mocked(request).mockResolvedValueOnce({ removed: true });
    const id = "0123456789abcdef";
    await act(async () => { deliver({ id, version: 1, uri: `hick-project:///${id}/message.md`, source_hash, workspace: "workspace", opening_line: 1, started: false }); });
    expect(request).toHaveBeenCalledWith("DELETE", `/api/project-contexts/${id}`);
    expect(session.turns[0].prompt).toBe(prompt);
  });
  it("reads a declared patch on its exact recorded line and displays compiler output without a write action", async () => {
    const prompt = 'Inspect this construction.\n\n<hick:patch id="change" after="make">\n--- a/out.txt\n+++ b/out.txt\n</hick:patch>';
    const session = new AgentSession();
    session.setTurns([{ id: "declared-turn", prompt, answer: null, status: "ok", parent_id: null, error: null,
      created_at: "", provider: "codex-subscription", model: "test", usage: null }]);
    vi.mocked(api.agentTurns).mockReturnValueOnce(new Promise(() => {}));
    const screen = render(createElement(Agent, { session }));
    const user = within(screen.container.querySelector<HTMLElement>(".user-message")!);
    const review = await user.findByLabelText("Review construction patch on line 3");
    const source_hash = await historySourceHash(prompt);
    const address = { side: "user", source_hash, opening_line: 2 };
    const endpoint = "/api/docs/workspace/agent/turns/declared-turn/fence-construction";
    expect(request).not.toHaveBeenCalledWith("POST", endpoint, address);
    vi.mocked(request).mockResolvedValueOnce({ version: 1, read_only: true, turn: "declared-turn", ...address, workspace: "workspace",
      entry: "main.md", document: "recorded.md", operation_id: "change", opening_byte: 28,
      changes: [{ path: "out.txt", before: { kind: "file", bytes: 4, sha256: "a".repeat(64), executable: false, text: "old\n", text_status: "exact" },
        after: { kind: "file", bytes: 4, sha256: "b".repeat(64), executable: false, text: "new\n", text_status: "exact" } }] });
    fireEvent.click(review);
    const reading = within(await user.findByLabelText("Recorded construction patch"));
    await reading.findByText("out.txt");
    expect(request).toHaveBeenCalledWith("POST", endpoint, address);
    expect(reading.queryByRole("button", { hidden: true })).toBeNull();
    expect(user.queryByLabelText("Edit source text")).toBeNull();
    expect(session.turns[0].prompt).toBe(prompt);
  });
  it("uses literate editors for history and draft and keeps the editor while changing provider", async () => {
    const session = new AgentSession(); session.prompt = "# Draft\n\n**Hello**";
    session.turns = [{ id: "old", prompt: "# Question", answer: "# Answer", status: "ok", parent_id: null, error: null, created_at: "", provider: "codex-subscription", model: "test", usage: null }];
    // Do not let asynchronous history replace the deliberately supplied turn.
    vi.mocked(api.agentTurns).mockReturnValueOnce(new Promise(() => {}));
    const screen = render(createElement(Agent, { session }));
    try {
      await waitFor(() => expect(screen.container.querySelectorAll(".cm-editor").length).toBe(3));
      const content = screen.container.querySelector<HTMLElement>('[aria-label="conversation/draft.md"]')!;
      const view = EditorView.findFromDOM(content)!;
      act(() => view.dispatch({ selection: { anchor: 3 } }));
      const composer = within(screen.container.querySelector<HTMLElement>(".agent-composer")!);
      fireEvent.change(composer.getByLabelText("Provider"), { target: { value: "grok-subscription" } });
      expect(composer.getByLabelText("Effort").hasAttribute("disabled")).toBe(true);
      expect(screen.container.querySelector('[aria-label="conversation/draft.md"]')).toBe(content);
      expect(view.state.selection.main.head).toBe(3); expect(session.prompt).toContain("**Hello**");
      const previous = screen.container.querySelector<HTMLElement>('[aria-label="conversation/old-user.md"]')!;
      expect(EditorView.findFromDOM(previous)!.state.readOnly).toBe(true);
      act(() => { session.setTurns(session.turns.map(turn => ({ ...turn, answer: "# Updated answer" }))); session.publish(); });
      expect(screen.container.querySelector('[aria-label="conversation/old-user.md"]')).toBe(previous);
      expect(screen.container.querySelector('[aria-label="conversation/old-agent.md"]')?.textContent).toContain("Updated answer");
      expect(screen.container.querySelector('[aria-label="conversation/draft.md"]')).toBe(content);
      act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: "!" } }));
      expect(session.prompt.endsWith("!")).toBe(true);
      act(() => { undo(view); }); expect(session.prompt.endsWith("!")).toBe(false);
      expect(vi.mocked(useWorkspaceLsp).mock.calls.some(([path]) => path === "conversation/draft.md")).toBe(false);
    } finally { screen.unmount(); }
  });
});
describe('strict Save As copies',()=>{
 function setup(){const buffers=new Buffers(()=>{});buffers.workspace='/owned';buffers.workspaceId='a'.repeat(32);const source=new Buffer({path:'original.py',name:'original.py',dir:false},'before','base-hash',()=>{},false,buffers.workspace);source.saveGuard='loaded-guard';source.edit('draft λ');buffers.values.set(source.file.path,source);return {buffers,source};}
 async function receipt(body:unknown){const b=body as {source:string;path:string;request_id:string;workspace:string};let hash=0xcbf29ce484222325n;for(const byte of new TextEncoder().encode(b.source))hash=BigInt.asUintN(64,(hash^BigInt(byte))*0x100000001b3n);return {version:1,history_protocol:1,workspace:b.workspace,namespace:'/owned',path:b.path,source:b.source,source_hash:await historySourceHash(b.source),hash:hash.toString(16).padStart(16,'0'),request_id:b.request_id,save_guard:'new-guard',doc_id:null,history:{available:true,outcome:'published',id:b.request_id}};}
 it('captures every guarded authored draft and adopts direct confirmed bytes without replacing the original',async()=>{const {buffers,source}=setup();const other=new Buffer({path:'construction.md',name:'construction.md',dir:false},'<h:doc/>','other',()=>{},false,buffers.workspace);other.saveGuard='other-guard';other.edit('retained construction');buffers.values.set(other.file.path,other);const loose=new Buffer({path:'Untitled-other',name:'other',dir:false},'unnamed note','',()=>{});loose.isUntitledDraft=true;buffers.values.set(loose.file.path,loose);const send=vi.fn(async(_method,_path,body)=>receipt(body));const copy=await createSourceCopy(buffers,source,'copy.py',send as typeof request);expect(send).toHaveBeenCalledOnce();expect(send.mock.calls[0][1]).toBe('/api/source-history/create');const body=send.mock.calls[0][2] as any;expect(body.ownership.ownership_sources).toEqual({'original.py':'draft λ','construction.md':'retained construction'});expect(body.saved_guards).toEqual({'original.py':'loaded-guard','construction.md':'other-guard'});expect(Object.values(body.unaddressed_sources)).toEqual(['unnamed note']);expect(copy.text).toBe('draft λ');expect(copy.base).toBe(copy.text);expect(copy.saveGuard).toBe('new-guard');expect(buffers.values.get('original.py')).toBe(source);expect(source.base).toBe('before');expect(source.dirty).toBe(true);});
 it('replays the exact consumed request after a lost reply and refuses changed rosters',async()=>{const {buffers,source}=setup();let first:unknown;const send=vi.fn(async(_m,_p,body)=>{first??=body;if(send.mock.calls.length===1)throw Error('lost reply');expect(body).toBe(first);return receipt(body);});await expect(createSourceCopy(buffers,source,'copy.py',send as typeof request)).rejects.toThrow('lost reply');await createSourceCopy(buffers,source,'copy.py',send as typeof request);const next=setup();const fail=vi.fn(async()=>{throw Error('lost reply');});await expect(createSourceCopy(next.buffers,next.source,'copy.py',fail as typeof request)).rejects.toThrow();next.buffers.values.set('new.py',new Buffer({path:'new.py',name:'new.py',dir:false},'new','',()=>{}));await expect(createSourceCopy(next.buffers,next.source,'copy.py',fail as typeof request)).rejects.toThrow('context changed');expect(fail).toHaveBeenCalledOnce();});
 it.each(['path','source_hash','hash','workspace','namespace','request_id','save_guard'])('refuses an unconfirmed %s without adopting or retiring any editor',async field=>{const {buffers,source}=setup();const send=vi.fn(async(_m,_p,body)=>({...await receipt(body),[field]:field==='save_guard'?'':'foreign'}));await expect(createSourceCopy(buffers,source,'copy.py',send as typeof request)).rejects.toThrow('not confirmed');expect([...buffers.values.values()]).toEqual([source]);expect(source.text).toBe('draft λ');expect(source.base).toBe('before');});
 it('keeps a retired caller lease from adopting or fresh-retrying a confirmed copy',async()=>{const {buffers,source}=setup();let current=true;const send=vi.fn(async(_m,_p,body)=>{current=false;return receipt(body);});await expect(createSourceCopy(buffers,source,'copy.py',send as typeof request,()=>current)).rejects.toThrow('not confirmed');await expect(createSourceCopy(buffers,source,'copy.py',send as typeof request,()=>true)).rejects.toThrow('context changed');expect(send).toHaveBeenCalledOnce();expect(buffers.values.has('copy.py')).toBe(false);});
 it('rejects same-object address mutation and a newly retained draft while a reply is held',async()=>{for(const change of [(_buffers:Buffers,source:Buffer)=>{source.file.path='moved.py';},(buffers:Buffers)=>{buffers.retainedResources.set('retained',new Buffer({path:'gone.py',name:'gone.py',dir:false},'draft','',()=>{}));}]){const {buffers,source}=setup();let resolve!:(value:unknown)=>void;const pending=new Promise(resolveValue=>{resolve=resolveValue;});let body:unknown;const send=vi.fn(async(_m,_p,value)=>{body=value;return pending;});const result=createSourceCopy(buffers,source,'copy.py',send as typeof request);await waitFor(()=>expect(send).toHaveBeenCalledOnce());change(buffers,source);resolve(await receipt(body));await expect(result).rejects.toThrow('not confirmed');expect(buffers.values.has('copy.py')).toBe(false);}});
});
</ui:copy>

## Keep closed plain editor history without holding saved language authority

Use the actual Files tree, tab controls and CodeMirror history to separate
switching tabs from explicitly closing a clean file. Reopen identical saved
bytes, then check that the pane still uses the same editor and Redo works.
A different saved revision gets a new clean buffer. A held read may finish
after the cached source gained a draft or changed its address. Those replies
must leave the retained source in place.

<ui:copy id="simple-closed-saved-source-witness">
describe("simple closed saved source", () => {
  const leaf = { path: "leaf.ts", name: "leaf.ts", dir: false };
  const scope = { workspace: "/owned", id: "a".repeat(32) };
  const saved = { path: leaf.path, content: "export const leaf = 1;\n", hash: "first", save_guard: "first-guard", language: "typescript" };
  async function closedBuffer() {
    const buffers = new Buffers(() => {});
    buffers.workspace = scope.workspace;
    buffers.workspaceId = scope.id;
    vi.mocked(api.file).mockReset().mockResolvedValueOnce(saved);
    const original = await buffers.open(leaf);
    buffers.closeSavedPlainSource(original);
    expect(original.languageSuspended).toBe(true);
    return { buffers, original };
  }

  it("suspends only an explicitly closed source and reuses its editor and history for unchanged bytes", async () => {
    const other = { path: "other.txt", name: "other.txt", dir: false };
    vi.mocked(api.files).mockResolvedValue({ root: "workspace", root_path: scope.workspace, workspace_id: scope.id, tree: [leaf, other] });
    vi.mocked(api.file).mockReset().mockImplementation(async path => path === leaf.path ? saved
      : { path, content: "other", hash: "other-hash", save_guard: "other-guard", language: "text" });
    const screen = render(createElement(App));
    try {
      fireEvent.click(await screen.findByRole("button", { name: leaf.path }));
      const view = EditorView.findFromDOM(await screen.findByLabelText(leaf.path))!;
      act(() => {
        view.dispatch({ changes: { from: 0, insert: "// unsaved\n" }, userEvent: "input.type" });
        expect(undo(view)).toBe(true);
        view.dispatch({ selection: { anchor: 5 } });
      });
      fireEvent.click(screen.getByRole("button", { name: other.path }));
      await screen.findByLabelText(other.path);
      fireEvent.click(screen.getByRole("tab", { name: leaf.path }));
      expect(EditorView.findFromDOM(await screen.findByLabelText(leaf.path))).toBe(view);
      expect(vi.mocked(api.file).mock.calls.filter(([path]) => path === leaf.path)).toHaveLength(1);

      vi.mocked(useWorkspaceLsp).mockClear();
      fireEvent.click(screen.getByRole("button", { name: "Close leaf.ts" }));
      expect(screen.queryByRole("tab", { name: leaf.path })).toBeNull();
      expect(vi.mocked(useWorkspaceLsp).mock.calls).toContainEqual(["", saved.content]);
      expect(vi.mocked(useWorkspaceLsp).mock.calls.some(([path]) => path === leaf.path)).toBe(false);

      fireEvent.click(screen.getByRole("button", { name: leaf.path }));
      expect(EditorView.findFromDOM(await screen.findByLabelText(leaf.path))).toBe(view);
      expect(vi.mocked(api.file).mock.calls.filter(([path]) => path === leaf.path)).toHaveLength(2);
      expect(view.state.selection.main.head).toBe(5);
      act(() => { expect(redo(view)).toBe(true); });
      expect(view.state.doc.toString()).toBe("// unsaved\n" + saved.content);
    } finally { screen.unmount(); vi.mocked(api.file).mockReset(); }
  });

  it("admits a different saved revision as a new clean buffer", async () => {
    const { buffers, original } = await closedBuffer();
    const updated = { ...saved, content: "// shifted\n" + saved.content, hash: "again", save_guard: "new-guard" };
    vi.mocked(api.file).mockResolvedValueOnce(updated);
    const reopened = await buffers.open(leaf);
    expect(reopened).not.toBe(original);
    expect(buffers.values.get(leaf.path)).toBe(reopened);
    expect(reopened.text).toBe(updated.content);
    expect(reopened.dirty).toBe(false);
    expect(reopened.saveGuard).toBe(updated.save_guard);
    expect(original.text).toBe(saved.content);
    expect(original.languageSuspended).toBe(true);
  });

  it.each(["draft", "workspace", "path"])("refuses a held reopen after a %s change", async reason => {
    const { buffers, original } = await closedBuffer();
    let finish!: (value: typeof saved) => void;
    vi.mocked(api.file).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    const pending = buffers.open(leaf);
    if (reason === "draft") original.edit("retained draft");
    if (reason === "workspace") buffers.workspaceId = "b".repeat(32);
    finish({ ...saved, path: reason === "path" ? "wrong.ts" : leaf.path, content: "another saved file" });
    await expect(pending).rejects.toThrow("cached source changed");
    expect(buffers.values.get(leaf.path)).toBe(original);
    expect(original.text).toBe(reason === "draft" ? "retained draft" : saved.content);
    expect(original.saveGuard).toBe(saved.save_guard);
    if (reason === "draft") {
      expect(original.languageSuspended).toBe(false);
      expect(await buffers.open(leaf)).toBe(original);
      expect(original.text).toBe("retained draft");
    }
  });
});
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/parity.test.ts"><ui:paste select="#simple-c6aff51ad5-0"/><ui:paste select="#simple-language-attachment-witness"/><ui:paste select="#simple-c6aff51ad5-1"/><ui:paste select="#simple-closed-saved-source-witness"/></ui:file>

</ui:doc>
