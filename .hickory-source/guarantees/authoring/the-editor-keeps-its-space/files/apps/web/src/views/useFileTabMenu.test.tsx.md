<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useFileTabMenu.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The editor keeps its space](../../../../../contract.md)

Captured SHA-256: `8d4a2017fa1f2551dcf075ee770201e6679c5f3bb75ed487a7aa35269a1d36cd`.

<book:entry path="apps/web/src/views/useFileTabMenu.test.tsx">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler } from "../api/client";
import type { FolderTree } from "../shell/FolderTreePane";
import { treeMenuItems } from "../shell/treeMenu";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useFileTabMenu } from "./useFileTabMenu";
import type { Tab } from "../shell/layout";

const folder: FolderTree = { root: "notes", root_path: "/notes", file_manager: "Finder", tree: [
  { name: "note.md", path: "note.md", dir: false, doc_id: "doc1" },
  { name: "input.txt", path: "input.txt", dir: false },
  { name: "output.txt", path: "output.txt", dir: false, generated_by: "doc1" },
] };
const opened = vi.fn();
function Harness({ kind = "file", target = "input.txt" }: { kind?: Tab["kind"]; target?: string }) {
  const { onTabContextMenu, fileTabMenu } = useFileTabMenu([folder], "doc1", new Map(), opened, [{ id: "tab1", kind, target }]);
  return <><button onContextMenu={event => onTabContextMenu(event, { id: "tab1", kind, target })}>File tab</button>{fileTabMenu}</>;
}
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it("offers every Files menu action and acts on the right-clicked tab", async () => {
  const calls: unknown[] = [];
  installMockHandler(async (method, path, body) => { calls.push([method, path, body]); return {}; });
  render(<Harness />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  const expected = treeMenuItems({ rootPath: "/notes", fileManager: "Finder" }, "input.txt", false, { plainText: true, activeDoc: { path: "note.md", name: "note.md" } });
  expect(screen.getAllByRole("menuitem").map(item => item.textContent)).toEqual(expected.map(item => item.label));
  fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
  expect(calls).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "Delete" }));
  await waitFor(() => expect(calls).toEqual([["POST", "/api/files/op", { op: "delete", path: "input.txt" }]]));
});
it("uses the shared adoption action and opens its resulting document", async () => {
  installMockHandler(async (method, path) => {
    if (method === "POST" && path === "/api/adopt") return { doc_id: "adopted" };
    throw new Error(`unexpected ${method} ${path}`);
  });
  render(<Harness />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  fireEvent.click(screen.getByRole("menuitem", { name: /Make literate/ }));
  await waitFor(() => expect(opened).toHaveBeenCalledWith("adopted"));
});
it.each([["document", "note.md"], ["generated", "output.txt"], ["file", "output.txt"]] as const)("does not offer adoption for %s %s", (kind, target) => {
  render(<Harness kind={kind} target={target} />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  expect(screen.queryByRole("menuitem", { name: /Make literate/ })).toBeNull();
  expect(screen.getByRole("menuitem", { name: "Rename…" })).toBeTruthy();
});
it("does not give file actions to unsaved or tool tabs", () => {
  render(<Harness kind="untitled" target="draft" />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  expect(screen.queryByRole("menu")).toBeNull();
});

// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("discloses current file-tab targets, opens the real menu and retires the former tab owner", async () => {
  const registry = new UiActions({ workspace: "p", window: "file-menu" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  expect(actions().some(action => action.label === "Delete")).toBe(false);
  expect(await run("Read file tab menu targets", { first: 0, count: 64 })).toMatchObject({ outcome: { value: { tabs: [{ id: "tab1", path: "input.txt" }] } } });
  expect(await run("Open file tab context menu", { tab: "missing" })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(await run("Open file tab context menu", { tab: "tab1" })).toMatchObject({ outcome: { status: "completed" } });
  expect(screen.getByRole("menuitem", { name: "Rename…" })).toBeTruthy();
  const old = actions().find(action => action.label === "Delete")!;
  mounted.rerender(<UiActionContext.Provider value={registry}><Harness target="output.txt" /></UiActionContext.Provider>);
  expect(screen.queryByRole("menu")).toBeNull();
  expect(await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).not.toMatchObject({ outcome: { status: "completed" } });
  await run("Open file tab context menu", { tab: "tab1" });
  expect(screen.queryByRole("menuitem", { name: /Make literate/ })).toBeNull();
});
</book:entry>

</book:doc>
