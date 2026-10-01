import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
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
  { name: "paper.md", path: "paper.md", dir: false, doc_id: "d1" },
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
  onCloseTerminal,
  dirtyPaths = new Set<string>(),
  focusRequest = 0,
}: {
  onOpen?: (action: Exclude<FileAction, { kind: "inert" }>) => void;
  openable?: Set<string>;
  onNew?: () => void;
  sessions?: readonly TreeSession[];
  onOpenTerminal?: (id: string) => void;
  onCloseTerminal?: (id: string) => void;
  dirtyPaths?: ReadonlySet<string>;
  focusRequest?: number;
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
      onCloseTerminal={onCloseTerminal}
      dirtyPaths={dirtyPaths}
      focusRequest={focusRequest}
    />
  );
}

function filesEditor(): EditorView {
  const element = document.querySelector<HTMLElement>(".filesystem-editor .cm-editor");
  const editor = element && EditorView.findFromDOM(element);
  if (!editor) throw new Error("Files editor did not mount");
  return editor;
}

beforeEach(() => {
  localStorage.clear();
  mockFiles();
});
afterEach(cleanup);

describe("rendering the folder", () => {
  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("marks a file dirty from the buffer state supplied by the workspace", async () => {
    render(<Harness dirtyPaths={new Set(["paper.md"])} />);
    await screen.findByText("paper.md");
    expect(document.querySelector('[data-tree-path="paper.md"]')?.className).toContain("filesystem-editor__line--dirty");
    expect(document.querySelector('[data-tree-path="readme.txt"]')?.className).not.toContain("filesystem-editor__line--dirty");
  });
  it("names the folder after its last path segment and offers a new document", async () => {
    const onNew = vi.fn();
    render(<Harness onNew={onNew} />);
    expect(await screen.findByText("notebook")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "New document" }));
    expect(onNew).toHaveBeenCalledOnce();
  });

  it("renders the server's order untouched: dirs first, then files", async () => {
    render(<Harness />);
    await screen.findByText("paper.md");
    expect(filesEditor().state.doc.toString()).toBe(
      "src/\n  gen/\n    orders.py\n  main.rs\npaper.md\nreadme.txt\nlogo.png",
    );
  });

  it("represents hierarchy as editable, significant whitespace", async () => {
    render(<Harness />);
    await screen.findByText("paper.md");
    expect(filesEditor().state.doc.line(1).text).toBe("src/");
    expect(filesEditor().state.doc.line(2).text).toBe("  gen/");
    expect(filesEditor().state.doc.line(3).text).toBe("    orders.py");
  });

  it("tags every visible file row for the ribbon overlay, by path", async () => {
    // The contract with shell/Ribbons.tsx: a connection whose target file is
    // visible as a buffer line (and not open as a tab) terminates ON that line,
    // found by data-tree-path.
    render(<Harness openable={new Set(["src/main.rs"])} />);
    await screen.findByText("paper.md");
    expect(document.querySelector('[data-tree-path="paper.md"]')?.getAttribute("data-tree-kind")).toBe("doc");
    expect(document.querySelector('[data-tree-path="readme.txt"]')?.getAttribute("data-tree-kind")).toBe("file");
    expect(document.querySelector('[data-tree-path="logo.png"]')?.getAttribute("data-tree-kind")).toBe("inert");
    expect(document.querySelector('[data-tree-path="src/main.rs"]')?.getAttribute("data-tree-kind")).toBe("generated");
  });

  it("says when the walk was truncated", async () => {
    mockFiles({ ...RESPONSE, truncated: true });
    render(<Harness />);
    expect(await screen.findByText(/not everything is listed/i)).toBeTruthy();
  });
});

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("editor-grade tree navigation", () => {
  it("moves point into the tree when the shell asks to focus Files", async () => {
    const view = render(<Harness />);
    await screen.findByText("paper.md");
    view.rerender(<Harness focusRequest={1} />);
    await vi.waitFor(() => expect(document.activeElement).toBe(filesEditor().contentDOM));
  });

  it("uses ordinary editor arrows, Home, and Shift-selection", async () => {
    render(<Harness />);
    await screen.findByText("paper.md");
    const editor = filesEditor();
    editor.focus();
    editor.dispatch({ selection: { anchor: 0 } });
    fireEvent.keyDown(editor.contentDOM, { key: "ArrowRight" });
    expect(editor.state.selection.main.head).toBe(1);
    fireEvent.keyDown(editor.contentDOM, { key: "Home" });
    expect(editor.state.selection.main.head).toBe(0);
    fireEvent.keyDown(editor.contentDOM, { key: "ArrowRight", shiftKey: true });
    expect(editor.state.selection.main.empty).toBe(false);
  });
});

// docs/guarantees/integrations/a-github-issue-belongs-to-its-associated-folder.md
describe("GitHub issues in folders", () => {
  it("indents an associated issue beneath its folder and opens its details", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "GET" && path === "/api/workspace/github") return {
        status: "available", repository: "acme/widget", branch: "main", reviews: [],
        issues: [{ repository: "acme/widget", number: 7, folder: "src", title: "Fix retry", state: "OPEN", freshness: "live" }],
      };
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const issue = await screen.findByText(/issue #7: Fix retry/);
    expect(filesEditor().state.doc.toString()).toContain("src/\n  issue #7: Fix retry · open\n  gen/");
    fireEvent.doubleClick(issue);
    expect(await screen.findByRole("button", { name: /Issue #7: Fix retry/ })).toBeTruthy();
  });

  it("associates an issue from a folder's context menu", async () => {
    const associated: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "GET" && path === "/api/workspace/github") return {
        status: "available", repository: "acme/widget", branch: "main", reviews: [], issues: [],
      };
      if (method === "POST" && path === "/api/workspace/github/issues") {
        associated.push(body); return body;
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const src = await screen.findByText("src/");
    fireEvent.contextMenu(src);
    fireEvent.click(document.querySelector<HTMLElement>('[data-menu-item="associate-github-issue"]')!);
    const input = await screen.findByRole("textbox", { name: /GitHub issue for src/ });
    fireEvent.change(input, { target: { value: "acme/widget#19" } });
    fireEvent.click(screen.getByRole("button", { name: "Associate" }));
    await vi.waitFor(() => expect(associated).toEqual([{ repository: "acme/widget", number: 19, folder: "src" }]));
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
      if (method === "GET" && path === "/api/workspace/github") {
        return { status: "not_repository", reviews: [], issues: [] };
      }
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

    fireEvent.contextMenu(screen.getByText("src/"));
    // A directory has no default program of its own worth naming.
    expect(document.querySelector('[data-menu-item="open-external"]')).toBeNull();
    expect(menuItem("copy-name").textContent).toBe("Copy folder name");
    // ...and right-clicking it did not change the editable buffer.
    expect(filesEditor().state.doc.line(1).text).toBe("src/");
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

  it("opens text entries on double-click while a single click only places the caret", async () => {
    const onOpen = vi.fn();
    render(<Harness onOpen={onOpen} openable={new Set(["src/main.rs"])} />);
    fireEvent.click(await screen.findByText("paper.md"));
    expect(onOpen).not.toHaveBeenCalled();
    fireEvent.doubleClick(screen.getByText("paper.md"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "doc", id: "d1" });
    fireEvent.doubleClick(screen.getByText("main.rs"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "generated", path: "src/main.rs" });
    fireEvent.doubleClick(screen.getByText("readme.txt"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "file", path: "readme.txt" });
    fireEvent.doubleClick(screen.getByText("logo.png"));
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
    expect([...paths].every((p) => !p.endsWith(".md") && !p.startsWith("/"))).toBe(true);
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

// Guarantees:
// - docs/guarantees/terminal/a-session-appears-where-it-is-working.md
// - docs/guarantees/authoring/the-workspace-tree-is-a-lens.md
describe("showing a terminal from the tree", () => {
  it("lists a session at the folder it is working in, and opening it names it", async () => {
    const onOpenTerminal = vi.fn();
    render(
      <Harness
        sessions={[session("t1", "/home/me/notebook", { title: "cargo test", state: "working" })]}
        onOpenTerminal={onOpenTerminal}
      />,
    );
    const row = await screen.findByRole("button", { name: /cargo test/ });
    expect(row.getAttribute("data-workspace-node-key")).toBe("terminal:t1");
    expect(row.className).toContain("folder-tree__terminal");
    expect(row.textContent).toContain("working");
    fireEvent.click(row);
    expect(onOpenTerminal).toHaveBeenCalledWith("t1");
  });

  it("indents a nested terminal beneath its cwd and activates it from the buffer", async () => {
    const onOpenTerminal = vi.fn();
    render(<Harness sessions={[session("t1", "/home/me/notebook/src", { title: "cargo watch" })]} onOpenTerminal={onOpenTerminal} />);
    const terminal = await screen.findByText("terminal: cargo watch · running");
    expect(filesEditor().state.doc.toString()).toContain("src/\n  terminal: cargo watch · running\n  gen/");
    fireEvent.doubleClick(terminal);
    expect(onOpenTerminal).toHaveBeenCalledWith("t1");
  });

  it("offers close on the visible terminal node's own menu", async () => {
    const onCloseTerminal = vi.fn();
    render(
      <Harness
        sessions={[session("t1", "/home/me/notebook", { title: "cargo test" })]}
        onCloseTerminal={onCloseTerminal}
      />,
    );
    fireEvent.contextMenu(await screen.findByRole("button", { name: /cargo test/ }));
    fireEvent.click(screen.getByText("Close cargo test"));
    expect(onCloseTerminal).toHaveBeenCalledWith("t1");
  });

  it("keeps a nested session visible as text even before a folder is folded", async () => {
    render(<Harness sessions={[session("t1", "/home/me/notebook/src")]} />);
    await screen.findByText("terminal: t1 · running");
    expect(filesEditor().state.doc.toString()).toContain("  terminal: t1 · running");
  });

  it("shows nothing for a session working outside the folder", async () => {
    render(<Harness sessions={[session("t1", "/somewhere/else", { title: "elsewhere" })]} />);
    await screen.findByText("notebook");
    expect(screen.queryByRole("button", { name: /elsewhere/ })).toBeNull();
  });
});

// docs/guarantees/authoring/the-tree-is-a-dired.md
describe("the tree as dired", () => {
  it("keeps destructive deletion behind the existing confirmation", async () => {
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
    fireEvent.contextMenu(readme);
    fireEvent.click(await screen.findByText("Delete"));
    const prompt = await screen.findByRole("dialog");
    expect(prompt.textContent).toContain("Delete readme.txt?");
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await vi.waitFor(() => expect(ops).toEqual([{ op: "delete", path: "readme.txt" }]));
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("renames by editing the buffer text and saving it", async () => {
    const ops: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/files/op") {
        ops.push(body);
        return { op: "rename" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    await screen.findByText("readme.txt");
    const editor = filesEditor();
    const line = editor.state.doc.line(6);
    editor.dispatch({ changes: { from: line.from, to: line.to, insert: "notes.txt" } });
    fireEvent.keyDown(editor.contentDOM, { key: "s", ctrlKey: true });
    await vi.waitFor(() => expect(ops).toEqual([{ op: "rename", path: "readme.txt", to: "notes.txt" }]));
    expect(await screen.findByText("Applied 1 filesystem edit.")).toBeTruthy();
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
        return { doc_id: "d9", doc_path: "readme.md", file_path: "readme.txt" };
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
