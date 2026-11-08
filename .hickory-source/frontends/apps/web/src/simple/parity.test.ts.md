<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Check shared layouts and refusal-safe saves

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Read the exact same layout scenarios as Swift. These are focused model witnesses, not evidence that native drag gestures or accessibility work on a device.

The Markdown witnesses check actual rendered emphasis, literal code and HTML safety, then edit and undo inside the retained document editor. A plain-file counterexample protects ordinary source text from accidental prose formatting. These component checks do not establish native rendering.

The inline DAP witnesses attach a controlled paused session after text is visible, exercise real editor decorations and action buttons, retain undo, and refuse an unsaved launch. They do not establish a live adapter connection.

<ui:copy id="simple-c6aff51ad5-0">
// Contract: .hickory-source/frontends/contract.md
import { createElement } from "react";
import { render, waitFor, act, fireEvent } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { undo } from "@codemirror/commands";
import { currentCompletions } from "@codemirror/autocomplete";
import { Document } from "./Document";
vi.mock("../debug/launchProfiles", () => ({ launchWithProfiles: vi.fn(async (_query, start, options) => { if (options.isCurrent()) start({ id: "test", fingerprint: "test" }); }) }));
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
vi.mock("../api/client", () => ({ api: { files: vi.fn(), doc: vi.fn(), file: vi.fn(), saveFile: vi.fn(), saveDoc: vi.fn(), openTerminal: vi.fn(), terminalInput: vi.fn(), agent: vi.fn() } }));
describe("shared frontend layout contract", () => {
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


describe("simple inline DAP debugging", () => {
  it("launches the generated target from a literate file block", async () => {
    const start = vi.fn();
    vi.mocked(useFollowingDebugSession).mockReturnValue({ ...IDLE_SESSION, start });
    const text = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="hello.py">\nprint(1)\n</h:file>\n</h:doc>\n';
    const screen = render(createElement(Document, { buffer: new Buffer({ path: "source.md", name: "source.md", dir: false }, text, "hash", () => {}) }));
    try {
      fireEvent.click(await screen.findByRole("button", { name: "Debug hello.py" }));
      await waitFor(() => expect(start).toHaveBeenCalledWith("hello.py", {}, undefined, { id: "test", fingerprint: "test" }));
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
    const channel = createLspChannel({ send(frame) {
      const message = decodeLspFrame(frame);
      if (!message || !("method" in message) || !("id" in message)) return;
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

A stale save retains text and baseline. A successful save acknowledges only the submitted snapshot, including when more text arrived while the request was pending. Generated files do not dispatch whole-file writes.

<ui:copy id="simple-c6aff51ad5-1">
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/parity.test.ts"><ui:paste select="#simple-c6aff51ad5-0"/><ui:paste select="#simple-language-attachment-witness"/><ui:paste select="#simple-c6aff51ad5-1"/></ui:file>

</ui:doc>
