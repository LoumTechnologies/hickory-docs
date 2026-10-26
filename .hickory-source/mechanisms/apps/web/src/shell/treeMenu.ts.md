<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/treeMenu.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-19605cc8cff0"></a>
## `apps/web/src/shell/treeMenu.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A File Is Ingested From The Tree](../../../../../guarantees/authoring/a-file-is-ingested-from-the-tree/contract.md), [A Tree Row Hands Its File To The Rest Of The Machine](../../../../../guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md), [Filesystem Acts Keep Dired's Safety](../../../../../guarantees/authoring/the-tree-is-a-dired/contract.md), [A GitHub Issue Belongs To Its Associated Folder](../../../../../guarantees/integrations/a-github-issue-belongs-to-its-associated-folder/contract.md).

<a id="f-19605cc8cff0-TreeFolder"></a>
### `TreeFolder`

What a right-click on a tree row offers, as data.
The menu is a list of items with labels and actions, computed here and only
rendered by the pane. Two reasons for the split: the interesting decisions
are textual (what an absolute path looks like on Windows, what the file
manager is called on this desktop, which items a directory has that a file
does not), and a right-click is one of the more annoying things to reach
through a UI test.
Nothing here talks to the OS. Copying is the clipboard; revealing and
opening are two server routes (crates/hickory-cli/src/serve/reveal.rs) —
the machine whose Finder opens is the machine the engine runs on, which is
the user's own.
The folder a menu is being built inside, as the server described it.

<storycode:copy id="f-19605cc8cff0-TreeFolder">
// What a right-click on a tree row offers, as data.
//
// The menu is a list of items with labels and actions, computed here and only
// rendered by the pane. Two reasons for the split: the interesting decisions
// are textual (what an absolute path looks like on Windows, what the file
// manager is called on this desktop, which items a directory has that a file
// does not), and a right-click is one of the more annoying things to reach
// through a UI test.
//
// Nothing here talks to the OS. Copying is the clipboard; revealing and
// opening are two server routes (crates/hickory-cli/src/serve/reveal.rs) —
// the machine whose Finder opens is the machine the engine runs on, which is
// the user's own.

/** The folder a menu is being built inside, as the server described it. */
export interface TreeFolder {
  /** The open folder's absolute path, in the platform's spelling. */
  rootPath?: string;
  /** What joins `rootPath` to a node path. Defaults to `/`. */
  separator?: string;
  /** "Finder" / "File Explorer" / "file manager". */
  fileManager?: string;
}

</storycode:copy>
<a id="f-19605cc8cff0-TreeMenuAction"></a>
### `TreeMenuAction`

<storycode:copy id="f-19605cc8cff0-TreeMenuAction">
export type TreeMenuAction =
  /** Put `text` on the clipboard. */
  | { kind: "copy"; text: string }
  /** Show this root-relative path in the file manager. */
  | { kind: "reveal"; path: string }
  /** Open this root-relative path in its default program. */
  | { kind: "open"; path: string }
  /** Start a terminal working in this root-relative directory. Terminals live
   * in the folder tree now — they have a working directory, and the tree
   * already draws directories, so "open one here" is a verb of the row. */
  | { kind: "terminal"; path: string }
  /** Start a terminal in a fresh git worktree, on its own branch. */
  | { kind: "worktree"; path: string }
  /** Persist a credentialless GitHub issue → folder association. */
  | { kind: "associate-github-issue"; path: string }
  /** Stop this session. The only place a terminal can be closed now that
   * they have no list of their own. */
  | { kind: "close-terminal"; id: string; title: string }
  // The dired verbs (docs/guarantees/authoring/the-tree-is-a-dired.md).
  // Each acts on `paths`: the marked rows when the row is marked, else the
  // row alone — the way `D` in dired acts on the marks if there are any.
  | { kind: "rename"; paths: string[] }
  | { kind: "move"; paths: string[] }
  | { kind: "copy-to"; paths: string[] }
  | { kind: "delete"; paths: string[] }
  /** New file / new folder inside this directory (`""` for the root). */
  | { kind: "create"; dir: string }
  | { kind: "mkdir"; dir: string }
  // Ingest (docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md).
  /** A new document beside the file, owning its bytes — "Make literate". */
  | { kind: "literate"; path: string }
  /** The file's bytes appended to an existing document as a block. */
  | { kind: "ingest"; path: string; into: string };

</storycode:copy>
<a id="f-19605cc8cff0-TreeMenuItem"></a>
### `TreeMenuItem`

<storycode:copy id="f-19605cc8cff0-TreeMenuItem">
export interface TreeMenuItem {
  /** Stable across renders and platforms; what a test clicks by. */
  id: string;
  label: string;
  action: TreeMenuAction;
  /** A separator line is drawn above this item. */
  group?: boolean;
}

</storycode:copy>
<a id="f-19605cc8cff0-fileManagerName"></a>
### `fileManagerName`

The file manager's name, with the generic fallback the server also uses.

<storycode:copy id="f-19605cc8cff0-fileManagerName">
/** The file manager's name, with the generic fallback the server also uses. */
export function fileManagerName(folder: TreeFolder): string {
  return folder.fileManager?.trim() || "file manager";
}

</storycode:copy>
<a id="f-19605cc8cff0-absolutePath"></a>
### `absolutePath`

A node's absolute path, or `null` when the server did not say where the
folder is.
The node path is always forward-slashed (it is a tree key, not an OS path),
so on Windows the separators have to be rewritten as well as joined —
copying `C:\notes/src/main.rs` into a terminal is the kind of almost-right
that wastes a minute.

<storycode:copy id="f-19605cc8cff0-absolutePath">
/**
 * A node's absolute path, or `null` when the server did not say where the
 * folder is.
 *
 * The node path is always forward-slashed (it is a tree key, not an OS path),
 * so on Windows the separators have to be rewritten as well as joined —
 * copying `C:\notes/src/main.rs` into a terminal is the kind of almost-right
 * that wastes a minute.
 */
export function absolutePath(folder: TreeFolder, path: string): string | null {
  const root = folder.rootPath;
  if (!root) return null;
  const sep = folder.separator || "/";
  const trimmedRoot = root.length > 1 ? root.replace(/[/\\]+$/, "") : root;
  if (path === "") return trimmedRoot;
  const native = sep === "/" ? path : path.replace(/\//g, sep);
  return `${trimmedRoot}${trimmedRoot.endsWith(sep) ? "" : sep}${native}`;
}

</storycode:copy>
<a id="f-19605cc8cff0-baseName"></a>
### `baseName`

The last segment of a root-relative path: the file or folder's own name.

<storycode:copy id="f-19605cc8cff0-baseName">
/** The last segment of a root-relative path: the file or folder's own name. */
export function baseName(path: string): string {
  return path.split("/").filter(Boolean).pop() ?? path;
}

</storycode:copy>
<a id="f-19605cc8cff0-TreeMenuContext"></a>
### `TreeMenuContext`

The menu for one row.
`path` is root-relative (`""` for the folder itself). `dir` says which of
the two "open" verbs makes sense: a directory has no default program of its
own worth naming, so it only offers the file manager.
"Copy absolute path" is left out entirely when the server did not say where
the folder is, rather than offered and then quietly copying a relative path.
What else the menu knows about the row: the marks, and the document a
file could be ingested into.

<storycode:copy id="f-19605cc8cff0-TreeMenuContext">
/**
 * The menu for one row.
 *
 * `path` is root-relative (`""` for the folder itself). `dir` says which of
 * the two "open" verbs makes sense: a directory has no default program of its
 * own worth naming, so it only offers the file manager.
 *
 * "Copy absolute path" is left out entirely when the server did not say where
 * the folder is, rather than offered and then quietly copying a relative path.
 */
/** What else the menu knows about the row: the marks, and the document a
 * file could be ingested into. */
export interface TreeMenuContext {
  /** Every marked row, root-relative. */
  marked?: ReadonlySet<string>;
  /** The focused document, if there is one, for "Ingest into …". */
  activeDoc?: { path: string; name: string };
  /** Whether this row is a plain text file some document does not already
   * write — the only kind that can be made literate or ingested. */
  plainText?: boolean;
}

</storycode:copy>
<a id="f-19605cc8cff0-subjects"></a>
### `subjects`

The rows a verb acts on: the marks when this row is one of them, else
the row alone.

<storycode:copy id="f-19605cc8cff0-subjects">
/** The rows a verb acts on: the marks when this row is one of them, else
 * the row alone. */
export function subjects(path: string, marked?: ReadonlySet<string>): string[] {
  return marked && marked.has(path) ? [...marked] : [path];
}

</storycode:copy>
<a id="f-19605cc8cff0-treeMenuItems"></a>
### `treeMenuItems`

<storycode:copy id="f-19605cc8cff0-treeMenuItems">
export function treeMenuItems(
  folder: TreeFolder,
  path: string,
  dir: boolean,
  context: TreeMenuContext = {},
): TreeMenuItem[] {
  const items: TreeMenuItem[] = [];
  const many = subjects(path, context.marked);
  const count = many.length > 1 ? ` (${many.length} marked)` : "";
  if (path !== "" && !dir && context.plainText) {
    items.push({
      id: "literate",
      label: "Make literate — a new document owning this file",
      action: { kind: "literate", path },
    });
    if (context.activeDoc && context.activeDoc.path !== path) {
      items.push({
        id: "ingest",
        label: `Ingest into ${context.activeDoc.name}`,
        action: { kind: "ingest", path, into: context.activeDoc.path },
      });
    }
  }
  if (dir) {
    items.push({ id: "new-file", label: "New file…", action: { kind: "create", dir: path }, group: items.length > 0 });
    items.push({ id: "new-folder", label: "New folder…", action: { kind: "mkdir", dir: path } });
  }
  if (path !== "") {
    items.push({ id: "rename", label: `Rename…${count}`, action: { kind: "rename", paths: many }, group: true });
    items.push({ id: "move", label: `Move to…${count}`, action: { kind: "move", paths: many } });
    items.push({ id: "copy-to", label: `Copy to…${count}`, action: { kind: "copy-to", paths: many } });
    items.push({ id: "delete", label: `Delete${count}`, action: { kind: "delete", paths: many } });
  }
  const absolute = absolutePath(folder, path);
  if (absolute) {
    items.push({
      id: "copy-absolute",
      label: "Copy absolute path",
      action: { kind: "copy", text: absolute },
      group: items.length > 0,
    });
  }
  if (path !== "") {
    items.push({
      id: "copy-relative",
      label: "Copy relative path",
      action: { kind: "copy", text: path },
    });
    items.push({
      id: "copy-name",
      label: dir ? "Copy folder name" : "Copy file name",
      action: { kind: "copy", text: baseName(path) },
    });
  }
  items.push({
    id: "reveal",
    label: `Reveal in ${fileManagerName(folder)}`,
    action: { kind: "reveal", path },
    group: true,
  });
  if (!dir) {
    items.push({
      id: "open-external",
      label: "Open in default program",
      action: { kind: "open", path },
    });
  }
  if (dir) {
    items.push({
      id: "new-terminal",
      label: "Open terminal here",
      action: { kind: "terminal", path },
      group: true,
    });
    items.push({
      id: "new-worktree",
      label: "New worktree here…",
      action: { kind: "worktree", path },
    });
    items.push({
      id: "associate-github-issue",
      label: "Associate GitHub issue…",
      action: { kind: "associate-github-issue", path },
    });
  }
  return items;
}

</storycode:copy>
<a id="f-19605cc8cff0-terminalMenuItems"></a>
### `terminalMenuItems`

The menu for one terminal icon.
Short on purpose. An icon is a small target and a long menu on one is a
menu nobody reads; the two verbs a running session has are "show me" and
"stop".

<storycode:copy id="f-19605cc8cff0-terminalMenuItems">
/**
 * The menu for one terminal icon.
 *
 * Short on purpose. An icon is a small target and a long menu on one is a
 * menu nobody reads; the two verbs a running session has are "show me" and
 * "stop".
 */
export function terminalMenuItems(session: { id: string; title: string }): TreeMenuItem[] {
  return [
    {
      id: "close-terminal",
      label: `Close ${session.title}`,
      action: { kind: "close-terminal", id: session.id, title: session.title },
    },
  ];
}

</storycode:copy>
<a id="f-19605cc8cff0-copyText"></a>
### `copyText`

Put text on the clipboard.
The async Clipboard API is what a modern browser and the desktop shell both
have; the textarea dance is for the case where it is absent or refused
(an insecure origin, a denied permission), because "copy path" silently
doing nothing is worse than a slightly ugly fallback.

<storycode:copy id="f-19605cc8cff0-copyText">
/**
 * Put text on the clipboard.
 *
 * The async Clipboard API is what a modern browser and the desktop shell both
 * have; the textarea dance is for the case where it is absent or refused
 * (an insecure origin, a denied permission), because "copy path" silently
 * doing nothing is worse than a slightly ugly fallback.
 */
export async function copyText(text: string): Promise<void> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return;
    }
  } catch {
    // Fall through to the textarea below.
  }
  const field = document.createElement("textarea");
  field.value = text;
  field.setAttribute("readonly", "");
  field.style.position = "fixed";
  field.style.opacity = "0";
  document.body.appendChild(field);
  field.select();
  try {
    if (!document.execCommand("copy")) throw new Error("Clipboard permission is unavailable; copy the visible path manually");
  } finally {
    field.remove();
  }
}
</storycode:copy>

### Assemble `apps/web/src/shell/treeMenu.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/treeMenu.ts"><storycode:paste select="#f-19605cc8cff0-TreeFolder"/><storycode:paste select="#f-19605cc8cff0-TreeMenuAction"/><storycode:paste select="#f-19605cc8cff0-TreeMenuItem"/><storycode:paste select="#f-19605cc8cff0-fileManagerName"/><storycode:paste select="#f-19605cc8cff0-absolutePath"/><storycode:paste select="#f-19605cc8cff0-baseName"/><storycode:paste select="#f-19605cc8cff0-TreeMenuContext"/><storycode:paste select="#f-19605cc8cff0-subjects"/><storycode:paste select="#f-19605cc8cff0-treeMenuItems"/><storycode:paste select="#f-19605cc8cff0-terminalMenuItems"/><storycode:paste select="#f-19605cc8cff0-copyText"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-f48c486ac3c1"></a>
## `apps/web/src/shell/treeMenu.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A File Is Ingested From The Tree](../../../../../guarantees/authoring/a-file-is-ingested-from-the-tree/contract.md), [A Tree Row Hands Its File To The Rest Of The Machine](../../../../../guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md), [Filesystem Acts Keep Dired's Safety](../../../../../guarantees/authoring/the-tree-is-a-dired/contract.md).

<a id="f-f48c486ac3c1-POSIX"></a>
### `POSIX`

<storycode:copy id="f-f48c486ac3c1-POSIX">
import { describe, expect, it } from "vitest";

import { absolutePath, baseName, fileManagerName, treeMenuItems,
  terminalMenuItems,
} from "./treeMenu";

const POSIX = { rootPath: "/home/me/notebook", separator: "/", fileManager: "Finder" };
</storycode:copy>
<a id="f-f48c486ac3c1-WINDOWS"></a>
### `WINDOWS`

<storycode:copy id="f-f48c486ac3c1-WINDOWS">
const WINDOWS = { rootPath: "C:\\Users\\me\\notebook", separator: "\\", fileManager: "File Explorer" };

</storycode:copy>
<a id="f-f48c486ac3c1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f48c486ac3c1-expression_statement">
describe("absolutePath", () => {
  it("joins the root to a tree path", () => {
    expect(absolutePath(POSIX, "src/main.rs")).toBe("/home/me/notebook/src/main.rs");
    expect(absolutePath(POSIX, "")).toBe("/home/me/notebook");
  });

  it("rewrites separators on Windows, not just the join", () => {
    // `C:\Users\me\notebook/src/main.rs` is the almost-right answer: it
    // resolves, and it is wrong to paste anywhere a person will read it.
    expect(absolutePath(WINDOWS, "src/main.rs")).toBe("C:\\Users\\me\\notebook\\src\\main.rs");
  });

  it("does not double the separator when the root already ends in one", () => {
    expect(absolutePath({ ...POSIX, rootPath: "/home/me/notebook/" }, "a.txt")).toBe(
      "/home/me/notebook/a.txt",
    );
    expect(absolutePath({ ...WINDOWS, rootPath: "C:\\" }, "a.txt")).toBe("C:\\a.txt");
  });

  it("is null when the server did not say where the folder is", () => {
    expect(absolutePath({}, "a.txt")).toBeNull();
  });
});

</storycode:copy>
<a id="f-f48c486ac3c1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f48c486ac3c1-expression_statement-2">
describe("baseName", () => {
  it("is the last segment", () => {
    expect(baseName("src/deep/inner.md")).toBe("inner.md");
    expect(baseName("readme.txt")).toBe("readme.txt");
    expect(baseName("src/")).toBe("src");
  });
});

</storycode:copy>
<a id="f-f48c486ac3c1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-f48c486ac3c1-expression_statement-3">
describe("terminalMenuItems", () => {
  it("offers the one verb a running session has that its own tab does not", () => {
    // Short on purpose: an icon is a small target, and a long menu on one is
    // a menu nobody reads.
    const items = terminalMenuItems({ id: "t1", title: "build" });
    expect(items.map((i) => i.id)).toEqual(["close-terminal"]);
    expect(items[0].label).toBe("Close build");
    expect(items[0].action).toEqual({ kind: "close-terminal", id: "t1", title: "build" });
  });
});

</storycode:copy>
<a id="f-f48c486ac3c1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-f48c486ac3c1-expression_statement-4">
describe("treeMenuItems", () => {
  it("offers the three copies, the file manager, and the default program for a file", () => {
    const items = treeMenuItems(POSIX, "src/main.rs", false);
    expect(items.map((i) => i.id)).toEqual([
      // The dired verbs first: they are what a right-click on a row is for.
      "rename",
      "move",
      "copy-to",
      "delete",
      "copy-absolute",
      "copy-relative",
      "copy-name",
      "reveal",
      "open-external",
    ]);
    expect(items.map((i) => i.label)).toContain("Reveal in Finder");
    expect(items.find((i) => i.id === "copy-absolute")?.action).toEqual({
      kind: "copy",
      text: "/home/me/notebook/src/main.rs",
    });
    expect(items.find((i) => i.id === "copy-name")?.action).toEqual({
      kind: "copy",
      text: "main.rs",
    });
  });

  it("names the platform's file manager", () => {
    expect(treeMenuItems(WINDOWS, "a.txt", false).find((i) => i.id === "reveal")?.label).toBe(
      "Reveal in File Explorer",
    );
    expect(fileManagerName({})).toBe("file manager");
    expect(treeMenuItems({}, "a.txt", false).find((i) => i.id === "reveal")?.label).toBe(
      "Reveal in file manager",
    );
  });

  it("gives a directory no default-program item, and calls the name a folder", () => {
    const items = treeMenuItems(POSIX, "src", true);
    expect(items.map((i) => i.id)).toEqual([
      "new-file",
      "new-folder",
      "rename",
      "move",
      "copy-to",
      "delete",
      "copy-absolute",
      "copy-relative",
      "copy-name",
      "reveal",
      // Terminals live in the tree now: a directory is where you open one.
      "new-terminal",
      "new-worktree",
      "associate-github-issue",
    ]);
    expect(items.find((i) => i.id === "copy-name")?.label).toBe("Copy folder name");
  });

  it("gives the folder itself only what applies to it", () => {
    // No relative path and no name: the row IS the root, and "." is not
    // something anyone wants on their clipboard.
    const items = treeMenuItems(POSIX, "", true);
    expect(items.map((i) => i.id)).toEqual([
      // A new entry can go in the root; the root itself cannot be renamed,
      // moved or deleted from inside the app that is open on it.
      "new-file",
      "new-folder",
      "copy-absolute",
      "reveal",
      "new-terminal",
      "new-worktree",
      "associate-github-issue",
    ]);
  });

  it("leaves out the absolute path rather than copying a relative one", () => {
    // An older server, or the mock: offering "Copy absolute path" and then
    // quietly copying `src/main.rs` is the failure worth avoiding.
    expect(treeMenuItems({}, "src/main.rs", false).map((i) => i.id)).toEqual([
      "rename",
      "move",
      "copy-to",
      "delete",
      "copy-relative",
      "copy-name",
      "reveal",
      "open-external",
    ]);
  });
});

</storycode:copy>
<a id="f-f48c486ac3c1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-f48c486ac3c1-expression_statement-5">
// docs/guarantees/authoring/the-tree-is-a-dired.md and
// docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md
describe("the dired and ingest verbs", () => {
  it("offers Make literate and Ingest into the focused document for a plain text file only", () => {
    const active = { path: "notes/today.md", name: "today.md" };
    const plain = treeMenuItems(POSIX, "src/main.rs", false, { plainText: true, activeDoc: active });
    expect(plain.slice(0, 2).map((i) => i.id)).toEqual(["literate", "ingest"]);
    expect(plain[1].label).toBe("Ingest into today.md");
    expect(plain[1].action).toEqual({ kind: "ingest", path: "src/main.rs", into: "notes/today.md" });
    // No focused document: nothing to ingest into, but a new document is
    // always possible.
    expect(treeMenuItems(POSIX, "src/main.rs", false, { plainText: true }).map((i) => i.id)[0]).toBe("literate");
    // A generated file, a binary, a document: neither verb.
    expect(treeMenuItems(POSIX, "a.png", false, { activeDoc: active }).map((i) => i.id)).not.toContain("ingest");
    // A directory: neither.
    expect(treeMenuItems(POSIX, "src", true, { plainText: true, activeDoc: active }).map((i) => i.id)).not.toContain("literate");
  });

  it("acts on the marks when the row is marked, and says how many", () => {
    const marked = new Set(["a.md", "b.md", "c.md"]);
    const onMarked = treeMenuItems(POSIX, "a.md", false, { marked });
    expect(onMarked.find((i) => i.id === "delete")?.label).toBe("Delete (3 marked)");
    expect(onMarked.find((i) => i.id === "move")?.action).toEqual({ kind: "move", paths: ["a.md", "b.md", "c.md"] });
    expect(onMarked.find((i) => i.id === "rename")?.label).toBe("Rename… (3 marked)");
    expect(onMarked.find((i) => i.id === "rename")?.action).toEqual({ kind: "rename", paths: ["a.md", "b.md", "c.md"] });
    const offMarks = treeMenuItems(POSIX, "z.md", false, { marked });
    expect(offMarks.find((i) => i.id === "delete")?.label).toBe("Delete");
    expect(offMarks.find((i) => i.id === "delete")?.action).toEqual({ kind: "delete", paths: ["z.md"] });
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/treeMenu.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/treeMenu.test.ts"><storycode:paste select="#f-f48c486ac3c1-POSIX"/><storycode:paste select="#f-f48c486ac3c1-WINDOWS"/><storycode:paste select="#f-f48c486ac3c1-expression_statement"/><storycode:paste select="#f-f48c486ac3c1-expression_statement-2"/><storycode:paste select="#f-f48c486ac3c1-expression_statement-3"/><storycode:paste select="#f-f48c486ac3c1-expression_statement-4"/><storycode:paste select="#f-f48c486ac3c1-expression_statement-5"/></storycode:file>


</storycode:doc>
