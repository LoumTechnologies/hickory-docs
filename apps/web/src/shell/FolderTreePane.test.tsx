import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { installMockHandler } from "../api/client";
import type { FileNode, FilesResponse } from "../api/types";
import {
  FolderTreePane,
  fileAction,
  isLikelyBinaryPath,
  loadExpanded,
  saveExpanded,
  toggleExpanded,
  useFolderTrees,
  type FileAction,
} from "./FolderTreePane";

// The /api/files contract, as the backend serves it: dirs first, each level
// already alphabetical — the pane renders the order it is given.
const TREE: FileNode[] = [
  {
    name: "src",
    path: "src",
    dir: true,
    children: [
      {
        name: "gen",
        path: "src/gen",
        dir: true,
        children: [{ name: "orders.py", path: "src/gen/orders.py", dir: false }],
      },
      { name: "main.rs", path: "src/main.rs", dir: false },
    ],
  },
  { name: "paper.hick", path: "paper.hick", dir: false, doc_id: "d1" },
  { name: "readme.txt", path: "readme.txt", dir: false },
  { name: "logo.png", path: "logo.png", dir: false },
];

const RESPONSE: FilesResponse = { root: "/home/me/notebook", tree: TREE };

function mockFiles(response: FilesResponse = RESPONSE) {
  installMockHandler(async (method, path) => {
    if (method === "GET" && path === "/api/files") return response;
    throw new Error(`unexpected ${method} ${path}`);
  });
}

function Harness({
  onOpen = () => {},
  openable = new Set<string>(),
  onNew = () => {},
}: {
  onOpen?: (action: Exclude<FileAction, { kind: "inert" }>) => void;
  openable?: Set<string>;
  onNew?: () => void;
}) {
  const { roots, error } = useFolderTrees();
  return (
    <FolderTreePane
      roots={roots}
      error={error}
      openable={openable}
      onOpen={onOpen}
      onNewDocument={onNew}
    />
  );
}

beforeEach(() => {
  localStorage.clear();
  mockFiles();
});
afterEach(cleanup);

describe("rendering the folder", () => {
  it("names the folder after its last path segment and offers a new document", async () => {
    const onNew = vi.fn();
    render(<Harness onNew={onNew} />);
    expect(await screen.findByText("notebook")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "New document" }));
    expect(onNew).toHaveBeenCalledOnce();
  });

  it("renders the server's order untouched: dirs first, then files", async () => {
    render(<Harness />);
    await screen.findByText("paper.hick");
    const labels = [...document.querySelectorAll(".folder-tree__dir, .folder-tree__file")].map(
      (el) => el.textContent?.replace(/^[▾▸]/, ""),
    );
    // src is collapsed, so its children are not in the page yet.
    expect(labels).toEqual(["src", "paper.hick", "readme.txt", "logo.png"]);
  });

  it("expands a directory to its children, nested dirs collapsible in turn", async () => {
    render(<Harness />);
    fireEvent.click(await screen.findByText(/src/));
    expect(screen.getByText("main.rs")).toBeTruthy();
    expect(screen.queryByText("orders.py")).toBeNull(); // gen still folded
    fireEvent.click(screen.getByText(/gen/));
    expect(screen.getByText("orders.py")).toBeTruthy();
  });

  it("tags every visible file row for the ribbon overlay, by path", async () => {
    // The contract with shell/Ribbons.tsx: a connection whose target file is
    // visible as a tree row (and not open as a tab) terminates ON that row,
    // found by data-tree-path. Collapsed directories render no row at all,
    // which is what lets the overlay fall through to a divider port.
    render(<Harness openable={new Set(["src/main.rs"])} />);
    await screen.findByText("paper.hick");
    expect(document.querySelector('[data-tree-path="paper.hick"]')?.getAttribute("data-tree-kind")).toBe("doc");
    expect(document.querySelector('[data-tree-path="readme.txt"]')?.getAttribute("data-tree-kind")).toBe("file");
    expect(document.querySelector('[data-tree-path="logo.png"]')?.getAttribute("data-tree-kind")).toBe("inert");
    // src is collapsed: its children have no rows to terminate on.
    expect(document.querySelector('[data-tree-path="src/main.rs"]')).toBeNull();
    fireEvent.click(screen.getByText(/src/));
    expect(document.querySelector('[data-tree-path="src/main.rs"]')?.getAttribute("data-tree-kind")).toBe("generated");
  });

  it("says when the walk was truncated", async () => {
    mockFiles({ ...RESPONSE, truncated: true });
    render(<Harness />);
    expect(await screen.findByText(/not everything is listed/i)).toBeTruthy();
  });
});

describe("what clicking a file does", () => {
  it("maps documents, generated files, plain text files, and binaries apart", () => {
    const openable = new Set(["src/gen/orders.py"]);
    expect(fileAction(TREE[1], openable)).toEqual({ kind: "doc", id: "d1" });
    expect(fileAction({ name: "orders.py", path: "src/gen/orders.py", dir: false }, openable)).toEqual({
      kind: "generated",
      path: "src/gen/orders.py",
    });
    // Not a document, not woven by an open document: still a text file in
    // the folder, so it opens as a plain file rather than sitting inert.
    expect(fileAction(TREE[2], openable)).toEqual({ kind: "file", path: "readme.txt" });
    expect(fileAction(TREE[3], openable)).toEqual({ kind: "inert" });
  });

  it("judges binary-ness by extension, with no-extension and dotfiles assumed text", () => {
    expect(isLikelyBinaryPath("assets/logo.png")).toBe(true);
    expect(isLikelyBinaryPath("dist/app.WASM")).toBe(true);
    expect(isLikelyBinaryPath("justfile")).toBe(false);
    expect(isLikelyBinaryPath(".gitignore")).toBe(false);
    expect(isLikelyBinaryPath("Cargo.lock")).toBe(false);
  });

  it("reports document, generated and plain-file clicks; binaries render without a button", async () => {
    const onOpen = vi.fn();
    render(<Harness onOpen={onOpen} openable={new Set(["src/main.rs"])} />);
    fireEvent.click(await screen.findByText("paper.hick"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "doc", id: "d1" });
    fireEvent.click(screen.getByText(/src/));
    fireEvent.click(screen.getByText("main.rs"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "generated", path: "src/main.rs" });
    fireEvent.click(screen.getByText("readme.txt"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "file", path: "readme.txt" });
    // logo.png is bytes this app cannot show: named, but not clickable.
    expect(screen.getByText("logo.png").tagName).not.toBe("BUTTON");
    expect(onOpen).toHaveBeenCalledTimes(3);
  });
});

describe("expand state, across sessions", () => {
  it("round-trips through localStorage, keyed by root", () => {
    saveExpanded("/a", new Set(["src", "src/gen"]));
    saveExpanded("/b", new Set(["docs"]));
    expect(loadExpanded("/a")).toEqual(new Set(["src", "src/gen"]));
    expect(loadExpanded("/b")).toEqual(new Set(["docs"]));
    expect(loadExpanded("/c")).toEqual(new Set());
  });

  it("toggles without mutating what it was given", () => {
    const before = new Set(["src"]);
    expect(toggleExpanded(before, "docs")).toEqual(new Set(["src", "docs"]));
    expect(toggleExpanded(before, "src")).toEqual(new Set());
    expect(before).toEqual(new Set(["src"]));
  });

  it("shrugs at a corrupt entry instead of breaking the pane", () => {
    localStorage.setItem("hickory.tree.expanded:/a", "{not json");
    expect(loadExpanded("/a")).toEqual(new Set());
  });

  it("keeps a directory open across unmount and remount", async () => {
    const first = render(<Harness />);
    fireEvent.click(await screen.findByText(/src/));
    expect(screen.getByText("main.rs")).toBeTruthy();
    first.unmount();

    render(<Harness />);
    // Re-fetched, re-mounted — and src is still expanded, from localStorage.
    expect(await screen.findByText("main.rs")).toBeTruthy();
  });
});
