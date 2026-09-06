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
  directoryPaths,
  placeSessions,
  relativeCwd,
  sessionsUnder,
  type FileAction,
  type TreeSession,
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
  sessions = [],
  onOpenTerminal,
}: {
  onOpen?: (action: Exclude<FileAction, { kind: "inert" }>) => void;
  openable?: Set<string>;
  onNew?: () => void;
  sessions?: readonly TreeSession[];
  onOpenTerminal?: (id: string) => void;
}) {
  const { roots, error } = useFolderTrees();
  return (
    <FolderTreePane
      roots={roots}
      error={error}
      openable={openable}
      onOpen={onOpen}
      onNewDocument={onNew}
      sessions={sessions}
      onOpenTerminal={onOpenTerminal}
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

// The right-click menu: the tree's door to the rest of the machine.
// Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.
describe("the right-click menu", () => {
  const PLATFORM: FilesResponse = {
    ...RESPONSE,
    root_path: "/home/me/notebook",
    separator: "/",
    file_manager: "Finder",
  };

  function menuItem(id: string): HTMLElement {
    const el = document.querySelector<HTMLElement>(`[data-menu-item="${id}"]`);
    if (!el) throw new Error(`no menu item ${id} in ${document.body.innerHTML}`);
    return el;
  }

  it("copies the absolute path, the relative path, and the name", async () => {
    const written: string[] = [];
    Object.assign(navigator, {
      clipboard: { writeText: (t: string) => (written.push(t), Promise.resolve()) },
    });
    mockFiles(PLATFORM);
    render(<Harness />);
    const row = await screen.findByText("readme.txt");

    for (const id of ["copy-absolute", "copy-relative", "copy-name"]) {
      fireEvent.contextMenu(row);
      fireEvent.click(menuItem(id));
    }
    expect(written).toEqual(["/home/me/notebook/readme.txt", "readme.txt", "readme.txt"]);
  });

  it("hands a file to the file manager and to its default program", async () => {
    const calls: [string, unknown][] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return PLATFORM;
      calls.push([`${method} ${path}`, body]);
      return { ok: true };
    });
    render(<Harness />);
    const row = await screen.findByText("readme.txt");

    fireEvent.contextMenu(row);
    expect(menuItem("reveal").textContent).toBe("Reveal in Finder");
    fireEvent.click(menuItem("reveal"));
    fireEvent.contextMenu(row);
    fireEvent.click(menuItem("open-external"));

    expect(calls).toEqual([
      ["POST /api/reveal", { path: "readme.txt" }],
      ["POST /api/open-external", { path: "readme.txt" }],
    ]);
  });

  it("offers the menu on a directory and on a binary the app cannot open", async () => {
    mockFiles(PLATFORM);
    render(<Harness />);
    // A binary row is inert to a click and still has a machine that can open
    // it — that is the whole reason the menu is on every row.
    fireEvent.contextMenu(await screen.findByText("logo.png"));
    expect(menuItem("open-external")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });

    fireEvent.contextMenu(screen.getByText(/src/));
    // A directory has no default program of its own worth naming.
    expect(document.querySelector('[data-menu-item="open-external"]')).toBeNull();
    expect(menuItem("copy-name").textContent).toBe("Copy folder name");
    // ...and right-clicking it did not also expand it.
    expect(screen.queryByText("main.rs")).toBeNull();
  });

  it("says so when the platform refuses, instead of appearing to do nothing", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path === "/api/files") return PLATFORM;
      throw new Error("could not start the file manager (`xdg-open`)");
    });
    render(<Harness />);
    fireEvent.contextMenu(await screen.findByText("readme.txt"));
    fireEvent.click(menuItem("reveal"));
    expect(await screen.findByText(/could not start the file manager/)).toBeTruthy();
  });

  it("closes on Escape and on a click elsewhere", async () => {
    mockFiles(PLATFORM);
    render(<Harness />);
    const row = await screen.findByText("readme.txt");

    fireEvent.contextMenu(row);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(document.querySelector(".tree-menu")).toBeNull();

    fireEvent.contextMenu(row);
    fireEvent.mouseDown(document.body);
    expect(document.querySelector(".tree-menu")).toBeNull();
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

// ---------------------------------------------------------------------------
// Terminals, shown where they are running
// ---------------------------------------------------------------------------

function session(id: string, cwd: string, over: Partial<TreeSession> = {}): TreeSession {
  return {
    id,
    title: id,
    cwd,
    state: "running",
    monitor: false,
    cwdIsLive: true,
    ...over,
  };
}

describe("placing sessions in the tree", () => {
  // Node paths in a listing are RELATIVE to the root; session directories are
  // absolute. Keeping the fixture honest about that is the whole point — the
  // first version of this compared the two directly and matched nothing.
  const root = "/w";
  const dirs = new Set(["src", "src/deep", "docs"]);

  it("puts a session at its own directory", () => {
    const placed = placeSessions([session("a", "/w/src")], root, dirs);
    expect(placed.get("src")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("puts a session working in the root at the root", () => {
    const placed = placeSessions([session("a", "/w")], root, dirs);
    expect(placed.get("")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("climbs to the nearest directory the tree actually lists", () => {
    // A large folder is truncated, and a session may be working somewhere the
    // listing never mentioned. Showing it at the nearest ancestor beats not
    // showing it at all.
    const placed = placeSessions([session("a", "/w/src/deep/unlisted/x")], root, dirs);
    expect(placed.get("src/deep")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("leaves out a session working outside the folder", () => {
    // The question is "what is running IN HERE".
    const placed = placeSessions([session("a", "/elsewhere")], root, dirs);
    expect(placed.size).toBe(0);
  });

  it("is not fooled by a sibling with a shared prefix", () => {
    // `/w-other` starts with `/w` as a string and is not inside it.
    const placed = placeSessions([session("a", "/w-other/src")], root, dirs);
    expect(placed.size).toBe(0);
  });

  it("treats a trailing slash as the same directory", () => {
    const placed = placeSessions([session("a", "/w/src/")], "/w/", dirs);
    expect(placed.get("src")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("keeps several sessions in one directory", () => {
    const placed = placeSessions([session("a", "/w/src"), session("b", "/w/src")], root, dirs);
    expect(placed.get("src")).toHaveLength(2);
  });
});

describe("counting what a collapsed directory hides", () => {
  it("counts the whole subtree, not just the directory itself", () => {
    const sessions = [session("a", "/w/src"), session("b", "/w/src/deep"), session("c", "/w")];
    expect(sessionsUnder(sessions, "/w", "src")).toBe(2);
    expect(sessionsUnder(sessions, "/w", "src/deep")).toBe(1);
  });

  it("does not count a sibling with a shared prefix", () => {
    expect(sessionsUnder([session("a", "/w/srcother")], "/w", "src")).toBe(0);
  });

  it("does not count a session outside the folder", () => {
    expect(sessionsUnder([session("a", "/elsewhere/src")], "/w", "src")).toBe(0);
  });
});

describe("directoryPaths", () => {
  it("collects directories, root-relative, and no files", () => {
    const paths = directoryPaths(TREE);
    expect(paths.has("src")).toBe(true);
    expect([...paths].every((p) => !p.endsWith(".hick") && !p.startsWith("/"))).toBe(true);
  });
});

describe("relativeCwd", () => {
  it("answers empty for the root itself and null for anything outside", () => {
    expect(relativeCwd("/w", "/w")).toBe("");
    expect(relativeCwd("/w/src/deep", "/w")).toBe("src/deep");
    expect(relativeCwd("/w-other", "/w")).toBeNull();
    expect(relativeCwd("/", "/w")).toBeNull();
  });
});

describe("showing a terminal from the tree", () => {
  it("lists a session at the folder it is working in, and opening it names it", async () => {
    const onOpenTerminal = vi.fn();
    render(
      <Harness
        sessions={[session("t1", "/home/me/notebook", { title: "cargo test" })]}
        onOpenTerminal={onOpenTerminal}
      />,
    );
    const row = await screen.findByRole("button", { name: /cargo test/ });
    fireEvent.click(row);
    expect(onOpenTerminal).toHaveBeenCalledWith("t1");
  });

  it("says how many are hidden inside a collapsed directory", async () => {
    // Otherwise the reason to show processes at all — seeing the one you
    // forgot about — is defeated by the directory being shut.
    render(<Harness sessions={[session("t1", "/home/me/notebook/src")]} />);
    const badge = await screen.findByText("1");
    expect(badge.getAttribute("data-tip")).toContain("1 terminal");
  });

  it("shows nothing for a session working outside the folder", async () => {
    render(<Harness sessions={[session("t1", "/somewhere/else", { title: "elsewhere" })]} />);
    await screen.findByText("notebook");
    expect(screen.queryByRole("button", { name: /elsewhere/ })).toBeNull();
  });
});

// docs/guarantees/authoring/the-tree-is-a-dired.md
describe("the tree as dired", () => {
  it("marks rows with Ctrl+click, deletes the marks through one prompt, and refetches", async () => {
    const ops: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/files/op") {
        ops.push(body);
        return { op: "delete" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const readme = await screen.findByText("readme.txt");
    fireEvent.click(readme, { ctrlKey: true });
    fireEvent.click(screen.getByText("src"));
    fireEvent.click(await screen.findByText("main.rs"), { ctrlKey: true });
    expect(readme.closest("button")?.className).toContain("marked");

    fireEvent.contextMenu(readme);
    fireEvent.click(await screen.findByText("Delete (2 marked)"));
    const prompt = await screen.findByRole("dialog");
    expect(prompt.textContent).toContain("Delete 2 items?");
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await vi.waitFor(() => expect(ops).toHaveLength(2));
    expect(ops).toEqual([
      { op: "delete", path: "readme.txt" },
      { op: "delete", path: "src/main.rs" },
    ]);
    // The prompt is gone and the marks are cleared.
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(readme.closest("button")?.className).not.toContain("marked");
  });

  it("renames through the prompt with the new name, and says why when refused", async () => {
    const ops: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/files/op") {
        ops.push(body);
        if ((body as { to: string }).to === "taken.txt") throw new Error("taken.txt already exists");
        return { op: "rename" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const readme = await screen.findByText("readme.txt");
    fireEvent.keyDown(readme, { key: "R", shiftKey: true });
    const input = await screen.findByRole("textbox", { name: "Rename readme.txt" });
    fireEvent.change(input, { target: { value: "taken.txt" } });
    fireEvent.click(screen.getByRole("button", { name: "Rename" }));
    await screen.findByText("taken.txt already exists");
    fireEvent.change(input, { target: { value: "notes.txt" } });
    fireEvent.click(screen.getByRole("button", { name: "Rename" }));
    await vi.waitFor(() => expect(ops).toHaveLength(2));
    expect(ops[1]).toEqual({ op: "rename", path: "readme.txt", to: "notes.txt" });
  });
});

// docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md
describe("ingest from the tree", () => {
  it("makes a plain file literate and opens the document it became", async () => {
    const adopted: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/adopt") {
        adopted.push(body);
        return { doc_id: "d9", doc_path: "readme.hick", file_path: "readme.txt" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    const onOpen = vi.fn();
    render(<Harness onOpen={onOpen} />);
    fireEvent.contextMenu(await screen.findByText("readme.txt"));
    fireEvent.click(await screen.findByText(/Make literate/));
    await vi.waitFor(() => expect(onOpen).toHaveBeenCalledWith({ kind: "doc", id: "d9" }));
    expect(adopted).toEqual([{ path: "readme.txt" }]);
    // A binary and a document get no such verb.
    fireEvent.contextMenu(screen.getByText("logo.png"));
    expect(screen.queryByText(/Make literate/)).toBeNull();
  });
});
