import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler } from "../api/client";
import type { FolderTree } from "../shell/FolderTreePane";
import { treeMenuItems } from "../shell/treeMenu";
import { useFileTabMenu } from "./useFileTabMenu";
import type { Tab } from "../shell/layout";

const folder: FolderTree = { root: "notes", root_path: "/notes", file_manager: "Finder", tree: [
  { name: "note.md", path: "note.md", dir: false, doc_id: "doc1" },
  { name: "input.txt", path: "input.txt", dir: false },
  { name: "output.txt", path: "output.txt", dir: false, generated_by: "doc1" },
] };
const opened = vi.fn();
function Harness({ kind = "file", target = "input.txt" }: { kind?: Tab["kind"]; target?: string }) {
  const { onTabContextMenu, fileTabMenu } = useFileTabMenu([folder], "doc1", new Map(), opened);
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
