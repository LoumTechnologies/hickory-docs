import { mdLinks } from "./mdLinks";
import { codeFolding, foldEffect } from "@codemirror/language";
import { useEditorEditing } from "./useEditorEditing";
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads bounded Unicode lines and uses real editor selection without editing source", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const source = "# λ😀\nsecond line\nthird line";
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: source }) });
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorActions(view, "deadline.md"), { wrapper });
  const descriptor = (label: string) => registry.observe(registry.groups().groups[0]).actions.find(item => item.label === label)!;
  const actor = { id: "python:read", kind: "agent" as const };
  const read = descriptor("Read editor lines");
  expect(await registry.act(registry.scope, actor, { handle: read.handle, arguments: { first: 1, count: 1 }, request_id: "read" })).toMatchObject({ status: "result", outcome: { value: { text: "# λ😀", first_line: 1, last_line: 1, next_line: 2, edit_baseline: false } } });
  const select = descriptor("Select editor range");
  await act(async () => { await registry.act(registry.scope, actor, { handle: select.handle, arguments: { anchor: 0, head: 5 }, request_id: "select" }); });
  expect(view.state.selection.main.head).toBe(5);
  expect(view.state.doc.toString()).toBe(source);
  expect(await registry.act(registry.scope, actor, { handle: read.handle, arguments: { first: 1, count: 1 }, request_id: "old-state" })).toMatchObject({ status: "stale" });
  hook.unmount(); view.destroy(); host.remove();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads long Unicode source and large fold sets through explicit bounded pages", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const source = "λ😀".repeat(3000);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [codeFolding()] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorActions(view, "long.txt"), { wrapper });
  let id = 0;
  const action = (label: string) => registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === label)!;
  const run = async (label: string, args = {}) => registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action(label).handle, arguments: args, request_id: String(++id) });
  expect(await run("Read editor lines", { first: 1, count: 1 })).toMatchObject({ status: "error", reason: expect.stringContaining("8192 UTF-8 bytes") });
  let offset: number | null = 0; const bytes: number[] = [];
  while (offset !== null) {
    const result = await run("Read editor UTF-8 bytes", { offset, limit: 8192 }) as unknown as { outcome: { value: { bytes: number[]; next_offset: number | null } } };
    expect(result.outcome.value.bytes.length).toBeLessThanOrEqual(8192);
    bytes.push(...result.outcome.value.bytes); offset = result.outcome.value.next_offset;
  }
  expect(new TextDecoder().decode(new Uint8Array(bytes))).toBe(source);
  act(() => { view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "x\n".repeat(70) }, selection: EditorSelection.cursor(140) }); });
  act(() => { view.dispatch({ effects: Array.from({ length: 70 }, (_, i) => foldEffect.of({ from: i * 2, to: i * 2 + 1 })) }); });
  expect(await run("Read editor folds")).toMatchObject({ status: "error", reason: expect.stringContaining("More than 64 folds") });
  expect(await run("Read editor fold page", { first: 64, count: 64 })).toMatchObject({ status: "result", outcome: { value: { total: 70, next: null, folds: expect.any(Array) } } });
  const folds = await run("Read editor fold page", { first: 0, count: 64 }) as unknown as { outcome: { value: { folds: unknown[]; next: number } } };
  expect(folds.outcome.value.folds).toHaveLength(64); expect(folds.outcome.value.next).toBe(64);
  const beforeUnfold = view.state.doc.toString();
  await run("Expand or collapse all editor sections", { expanded: true });
  expect(await run("Read editor fold page", { first: 0, count: 64 })).toMatchObject({ status: "result", outcome: { value: { total: 0, folds: [] } } });
  expect(view.state.doc.toString()).toBe(beforeUnfold);
  hook.unmount(); view.destroy(); host.remove();
});

it("selects and reads normalized multiple ranges and replaces them through the actual input command", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "aa bb cc", extensions: [EditorState.allowMultipleSelections.of(true)] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => { useEditorActions(view, "multi.txt"); useEditorEditing(view, { source: "multi.txt", label: "owned editor", identity: () => "multi.txt", current: () => true }); }, { wrapper });
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { const action = registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === label)!; result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Select editor ranges", { anchors: [0, 6], heads: [2, 8], main: 1 });
  expect(await run("Read editor selection page", { first: 0, count: 1 })).toMatchObject({ status: "result", outcome: { value: { total: 2, main_index: 1, next: 1, ranges: [{ index: 0, anchor: 0, head: 2 }] } } });
  await run("Make editor range primary", { index: 0 });
  expect(view.state.selection.mainIndex).toBe(0);
  expect(await run("Replace owned editor selections", { text: "λ".repeat(300000) })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("1 MiB insertion budget") } });
  expect(view.state.doc.toString()).toBe("aa bb cc");
  expect(await run("Replace owned editor selections", { text: "X" })).toMatchObject({ status: "result", outcome: { value: { requested_applied: true, source_changed: true, approved: false } } });
  expect(view.state.doc.toString()).toBe("X bb X");
  expect(view.state.selection.ranges).toHaveLength(2);
  hook.unmount(); view.destroy(); host.remove();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("follows only current prose links through the native navigation handler and retires changed path handles", async () => {
  const host = document.createElement("div"); document.body.append(host);
  let path = "notes/a.md";
  const text = "[plan](../plan.md) [web](https://example.com) [code](secret.md)";
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: text, extensions: [mdLinks({ docPath: () => path, skip: () => [[text.indexOf("[code]"), text.length]] })] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorActions(view, "a.md"), { wrapper });
  const action = (label: string) => registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === label)!;
  let id = 0;
  const run = (label: string, args: Record<string, number>) => registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action(label).handle, arguments: args, request_id: String(++id) });
  const seen: string[] = []; const listener = (event: Event) => seen.push((event as CustomEvent<string>).detail);
  window.addEventListener("hickory-open-path", listener);
  const open = vi.spyOn(window, "open").mockReturnValue(null);
  try {
    expect(await run("Read markdown link page", { first: 0, count: 1 })).toMatchObject({ outcome: { value: { total: 2, next: 1, links: [{ target: "../plan.md" }] } } });
    expect(await run("Follow markdown link", { position: 2 })).toMatchObject({ outcome: { value: { navigation_requested: true, path: "plan.md" } } });
    expect(seen).toEqual(["/plan.md"]);
    expect(await run("Follow markdown link", { position: text.indexOf("[code]") + 2 })).toMatchObject({ outcome: { status: "unavailable" } });
    expect(await run("Follow markdown link", { position: text.indexOf("[web]") + 2 })).toMatchObject({ outcome: { value: { navigation_requested: true, tab_handle_returned: false } } });
    expect(open).toHaveBeenCalledWith("https://example.com", "_blank", "noopener,noreferrer");
    const old = action("Follow markdown link"); path = "other/a.md";
    expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: { position: 2 }, request_id: "old-path" })).toMatchObject({ status: "stale" });
    act(() => { view.dispatch({ changes: { from: 0, to: text.length, insert: "no links" } }); });
    expect(await run("Follow markdown link", { position: 2 })).toMatchObject({ outcome: { status: "unavailable" } });
  } finally { open.mockRestore(); window.removeEventListener("hickory-open-path", listener); hook.unmount(); view.destroy(); host.remove(); }
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("opens only generated prose image sources through the native image callback", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const open = vi.fn((target: string) => target === "notes/generated.png");
  const text = "![generated](generated.png) ![external](external.png) [text](generated.png)";
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: text, extensions: [mdLinks({ docPath: () => "notes/a.md", images: true, onOpenImageSource: open })] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorActions(view, "a.md"), { wrapper });
  let id = 0;
  const run = (position: number) => {
    const action = registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === "Open generated markdown image source")!;
    return registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action.handle, arguments: { position }, request_id: String(++id) });
  };
  try {
    expect(await run(3)).toMatchObject({ outcome: { value: { source_selection_requested: true, source_written: false } } });
    expect(open).toHaveBeenLastCalledWith("notes/generated.png");
    expect(await run(text.indexOf("![external]") + 3)).toMatchObject({ outcome: { status: "unavailable" } });
    open.mockClear();
    expect(await run(text.indexOf("[text]") + 2)).toMatchObject({ outcome: { status: "unavailable" } });
    expect(open).not.toHaveBeenCalled();
    view.contentDOM.querySelector("img")?.click();
    expect(open).toHaveBeenLastCalledWith("notes/generated.png");
    expect(view.state.doc.toString()).toBe(text);
  } finally { hook.unmount(); view.destroy(); host.remove(); }
});
