<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/folderPane.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c66b482f3cc6"></a>
## `apps/web/src/views/folderPane.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The File Menu Preserves The Originating Window](../../../../../guarantees/authoring/file-menu-preserves-the-originating-window/contract.md).

<a id="f-c66b482f3cc6-useFolderPane"></a>
### `useFolderPane`

Wait for hydration before adding Files so restoration cannot overwrite it. Even a
restored layout that omitted Files must show the newly opened folder. Add it once
without taking editor focus; subsequent manual closing remains possible.

<storycode:copy id="f-c66b482f3cc6-useFolderPane">
import { useCallback, useEffect, useRef, type Dispatch, type SetStateAction } from "react";
import { api } from "../api/client";
import type { Doc, FilesResponse, PlainFile } from "../api/types";
import { findNodeByPath } from "../shell/workspacePaths";
import { activate, closeTab, panes, tab, treePane, withTree, type Layout } from "../shell/layout";

/** Folder visibility belongs to this window, independent of the engine's working directory. */
export function useFolderPane(
  folderOpen: boolean | null,
  hydrated: boolean,
  layout: Layout,
  setLayout: Dispatch<SetStateAction<Layout>>,
  requestFocus: Dispatch<SetStateAction<number>>,
  notice: (message: string) => void,
  restored = false,
) {
  const openedByDefault = useRef(false);
  useEffect(() => {
    if (folderOpen === false) openedByDefault.current = false;
    if (!hydrated || folderOpen !== true || openedByDefault.current) return;
    openedByDefault.current = true;
    // Wait for restoration, then add Files without taking editor focus.
    // Apply the default once so closing the pane remains a person's choice.
    setLayout((current) => treePane(current) ? current : withTree(current, tab("tree", "folder", "Files")));
  }, [folderOpen, hydrated, restored, setLayout]);
  useEffect(() => {
    if (folderOpen !== false) return;
    // Restored furniture may have come from a folder view of the same engine.
    setLayout((current) => panes(current.root).reduce((next, pane) =>
      pane.tabs.filter((entry) => entry.kind === "tree").reduce(
        (result, entry) => closeTab(result, pane.id, entry.id), next), current));
  }, [folderOpen, hydrated, layout, setLayout]);
  return useCallback(() => {
    if (folderOpen !== true) {
      notice("Open a folder to show Files.");
      return;
    }
    setLayout((current) => {
      const pane = treePane(current);
      if (!pane) return withTree(current, tab("tree", "folder", "Files"));
      const index = pane.tabs.findIndex((entry) => entry.kind === "tree");
      return activate(current, pane.id, Math.max(0, index));
    });
    requestFocus((request) => request + 1);
  }, [folderOpen, setLayout, requestFocus, notice]);
}

</storycode:copy>
<a id="f-c66b482f3cc6-openSelectedFile"></a>
### `openSelectedFile`

An explicit native file window can read its workspace-relative address while
the normal open, tree ownership, and guarded source requests settle. A preview
carries no write authority. Admit the original editor only when all observations
agree on the exact address, bytes, and workspace.

<storycode:copy id="f-c66b482f3cc6-openSelectedFile">
type StartupSource = Awaited<ReturnType<typeof api.startupSource>>;
export type StartupPlainFile = {
  source: StartupSource;
  current: () => boolean;
  authority: Promise<{file: PlainFile; namespace: string; workspace: string; generatedBy: string | null}>;
};

export type ManagedDocumentPreview = {
  source: Pick<StartupSource, "path" | "content">;
  docId: string;
  current: () => boolean;
  authority: Promise<Doc>;
  confirmRoom: () => Promise<void>;
};

export type StartupManagedDocument = ManagedDocumentPreview & {source: StartupSource};

// A verified navigation already has a normal document response. Show its
// source while the editor waits for the same saved authority and room bytes
// that a native startup preview has to confirm before it can edit.
export function admittedManagedPreview(doc: Doc, current: () => boolean): ManagedDocumentPreview {
  const authority = Promise.resolve(doc);
  return {source: {path: doc.path, content: doc.source}, docId: doc.id, current, authority,
    confirmRoom: () => confirmManagedRoom(authority, current)};
}

async function confirmManagedRoom(authority: Promise<Doc>, current: () => boolean): Promise<void> {
  const original = await authority;
  if (!current()) throw Error("The original document request changed.");
  if (!original.save_guard) throw Error(original.save_guard_reason || "The document's saved source authority is unavailable. Reopen it to check again.");
  const latest = await api.doc(original.id);
  if (!current() || latest.path !== original.path || latest.id !== original.id || latest.source !== original.source
      || latest.save_guard !== original.save_guard) {
    throw Error("The document's saved authority changed while its room loaded. Reopen the read-only preview.");
  }
}

async function readStartupSource(requested: string, hint: string | null, current: () => boolean): Promise<StartupSource | null> {
  const path = hint ?? requested;
  const normalizedRequest = requested.replaceAll("\\", "/");
  if (hint && requested !== hint && !normalizedRequest.endsWith("/" + hint)) return null;
  if (path.includes("\\") || path.includes(":")
    || /[\u0000-\u001f\u007f]/.test(path) || path.split("/").some(part => !part || part === "." || part === "..")) return null;
  const source = await api.startupSource(path);
  const absolute = source?.root_path?.replaceAll("\\", "/").replace(/\/$/, "") + "/" + path;
  if (!current() || source?.version !== 1 || source.path !== path || typeof source.content !== "string"
    || !source.root_path || (requested !== path && normalizedRequest !== absolute)) return null;
  return source;
}

function ownsStartupSource(files: FilesResponse, source: StartupSource) {
  const workspace = files.workspace_id, namespace = files.root_path;
  return !!namespace && namespace === source.root_path && !!workspace && /^[a-f0-9]{32}$/.test(workspace)
    && source.workspace_id === workspace && files.folder_open !== false;
}

export async function observePlainStartup(
  requested: string, hint: string | null, selected: ReturnType<typeof api.openFile>,
  current: () => boolean,
): Promise<StartupPlainFile | null> {
  if ((hint ?? requested).endsWith(".md")) return null;
  const source = await readStartupSource(requested, hint, current);
  if (!source || source.doc_id) return null;
  const {path} = source;
  const authority = Promise.all([selected, api.files(), api.file(path)]).then(([opened, files, file]) => {
    const workspace = files.workspace_id, namespace = files.root_path;
    const node = findNodeByPath([files], path);
    if (!current() || opened.doc_id || opened.path !== path || !namespace || !workspace
      || !ownsStartupSource(files, source) || !node || node.dir || node.doc_id
      || file.path !== path || file.content !== source.content) {
      throw Error("The requested file no longer matches the startup observation. Reopen it to check the current source.");
    }
    if (!node.generated_by && !file.save_guard) throw Error(file.save_guard_reason || "The file's saved source authority is unavailable. Reopen it to check again.");
    return {file, namespace, workspace, generatedBy: node.generated_by ?? null};
  });
  // The preview may close or be superseded before its pane mounts.
  void authority.catch(() => {});
  return {source, current, authority};
}

export async function observeManagedStartup(
  requested: string, hint: string | null, selected: ReturnType<typeof api.openFile>,
  current: () => boolean,
): Promise<StartupManagedDocument | null> {
  if (!(hint ?? requested).endsWith(".md")) return null;
  const source = await readStartupSource(requested, hint, current);
  const docId = source?.doc_id;
  if (!source || !docId || !/^[a-f0-9]{16}$/.test(docId)) return null;
  const {path} = source;
  const authority = selected.then(async opened => {
    const refuse = () => Error("The requested document no longer matches its startup observation. Reopen it to check the current source.");
    if (!current() || opened.path !== path || opened.doc_id !== docId) throw refuse();
    const [doc, files] = await Promise.all([api.doc(docId), api.files()]);
    const node = findNodeByPath([files], path);
    if (!current() || !ownsStartupSource(files, source) || !node || node.dir
      || node.generated_by || node.doc_id !== docId || doc.id !== docId || doc.path !== path || doc.source !== source.content) throw refuse();
    if (!doc.save_guard) throw Error(doc.save_guard_reason || "The document's saved source authority is unavailable. Reopen it to check again.");
    return doc;
  });
  const confirmRoom = () => confirmManagedRoom(authority, current);
  void authority.catch(() => {});
  return {source, docId, current, authority, confirmRoom};
}

export async function openSelectedFile(
  path: string,
  openDoc: (id: string, path: string) => void,
  openPlain: (path: string) => void,
  notice: (message: string) => void,
  current: () => boolean = () => true,
) {
  try {
    const selected = await api.openFile(path);
    if (!current()) return;
    if (selected.doc_id) openDoc(selected.doc_id, selected.path);
    else openPlain(selected.path);
  } catch (error) {
    if (!current()) return;
    notice(error instanceof Error ? error.message : String(error));
  }
}
</storycode:copy>

### Assemble `apps/web/src/views/folderPane.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/folderPane.ts"><storycode:paste select="#f-c66b482f3cc6-useFolderPane"/><storycode:paste select="#f-c66b482f3cc6-openSelectedFile"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c29616a4a7b1"></a>
## `apps/web/src/views/folderPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md).

<a id="f-c29616a4a7b1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement">
import { act, renderHook, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, tab, withTree } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { observeManagedStartup, observePlainStartup, openSelectedFile, useFolderPane } from "./folderPane";

afterEach(() => vi.restoreAllMocks());

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-2">
it("keeps the raw startup bytes readable without granting missing or stale authority", async () => {
  const path = "nested/code #é.py", namespace = "/workspace", workspace = "a".repeat(32);
  const content = "value = 1\n";
  vi.spyOn(api, "startupSource").mockResolvedValue({version: 1, path, content, root_path: namespace, workspace_id: workspace});
  vi.spyOn(api, "files").mockResolvedValue({root: "workspace", root_path: namespace, workspace_id: workspace,
    tree: [{name: "code #é.py", path, dir: false}]});
  const read = vi.spyOn(api, "file").mockResolvedValue({path, content, language: "python", hash: "base"});
  const open = Promise.resolve({path});
  const preview = await observePlainStartup(namespace + "/" + path, path, open, () => true);
  expect(preview?.source.content).toBe(content);
  await expect(preview?.authority).rejects.toThrow(/saved source authority/);

  read.mockResolvedValue({path, content, language: "python", hash: "base", save_guard: "current"});
  let current = true;
  let complete!: (opened: {path: string}) => void;
  const later = new Promise<{path: string}>(resolve => { complete = resolve; });
  const stale = await observePlainStartup(namespace + "/" + path, path, later, () => current);
  current = false;
  complete({path});
  await expect(stale?.authority).rejects.toThrow(/no longer matches/);
  const foreign = await observePlainStartup("/elsewhere/" + path, path, open, () => true);
  expect(foreign).toBeNull();
  vi.mocked(api.startupSource).mockResolvedValueOnce({version: 1, path, content, root_path: namespace, workspace_id: null});
  const unbound = await observePlainStartup(namespace + "/" + path, path, open, () => true);
  expect(unbound?.source.content).toBe(content);
  await expect(unbound?.authority).rejects.toThrow(/no longer matches/);
});

it("rechecks the original managed guard after an equal-byte room wait", async () => {
  const path = "nested/new #é.md", docId = "b".repeat(16), root = "/workspace", workspace = "a".repeat(32);
  const content = "# Readable\n";
  vi.spyOn(api, "startupSource").mockResolvedValue({version: 1, path, doc_id: docId, content, root_path: root, workspace_id: workspace});
  vi.spyOn(api, "files").mockResolvedValue({root, root_path: root, workspace_id: workspace,
    tree: [{name: "new #é.md", path, dir: false, doc_id: docId}]});
  const file = {id: docId, path, source: content, updated_at: "", save_guard: "first physical source"};
  const read = vi.spyOn(api, "doc").mockResolvedValue(file);
  let open!: (selected: {path: string; doc_id: string}) => void;
  const selected = new Promise<{path: string; doc_id: string}>(resolve => { open = resolve; });
  const preview = await observeManagedStartup(root + "/" + path, path, selected, () => true);
  expect(preview?.source.content).toBe(content);
  expect(read).not.toHaveBeenCalled();
  open({path, doc_id: docId});
  await expect(preview?.authority).resolves.toEqual(file);
  read.mockResolvedValue({...file, save_guard: "equal-byte replacement"});
  await expect(preview?.confirmRoom()).rejects.toThrow(/saved authority changed/);
});

it("removes restored Files furniture without removing the editor", async () => {
  const notice = vi.fn();
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState(() => withTree(openUntitledTab(initialWorkspace()), tab("tree", "folder", "Files")));
    const [, requestFocus] = useState(0);
    const focusFiles = useFolderPane(false, true, layout, setLayout, requestFocus, notice);
    return { layout, focusFiles };
  });
  await waitFor(() => expect(panes(result.current.layout.root).flatMap((pane) => pane.tabs).map((entry) => entry.kind)).toEqual(["untitled"]));
  act(() => result.current.focusFiles());
  expect(notice).toHaveBeenCalledWith("Open a folder to show Files.");
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-3">
it("opens the selected document independently of folder visibility", async () => {
  vi.spyOn(api, "openFile").mockResolvedValue({ path: "/notes/note.md", doc_id: "note" });
  const openDoc = vi.fn();
  const openPlain = vi.fn();
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", openDoc, openPlain, notice);
  expect(openDoc).toHaveBeenCalledWith("note", "/notes/note.md");
  expect(openPlain).not.toHaveBeenCalled();
  expect(notice).not.toHaveBeenCalled();
});

it("ignores a selected-file identity that arrives after the route changes", async () => {
  let deliver!: (value: { path: string; doc_id: string }) => void;
  vi.spyOn(api, "openFile").mockReturnValueOnce(new Promise(resolve => { deliver = resolve; }));
  const openDoc = vi.fn(), openPlain = vi.fn(), notice = vi.fn();
  let current = true;
  const opening = openSelectedFile("/notes/old.md", openDoc, openPlain, notice, () => current);
  current = false;
  deliver({ path: "/notes/old.md", doc_id: "retired" });
  await opening;
  expect(openDoc).not.toHaveBeenCalled();
  expect(openPlain).not.toHaveBeenCalled();
  expect(notice).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-4">
it("opens an ordinary text file without requiring a document session", async () => {
  vi.spyOn(api, "openFile").mockResolvedValue({ path: "/outside/script.py" });
  const openPlain = vi.fn();
  await openSelectedFile("/notes/script.py", vi.fn(), openPlain, vi.fn());
  expect(openPlain).toHaveBeenCalledWith("/outside/script.py");
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-5">
it("explains a failed file lookup", async () => {
  vi.spyOn(api, "openFile").mockRejectedValue(new Error("File unavailable"));
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", vi.fn(), vi.fn(), notice);
  expect(notice).toHaveBeenCalledWith("File unavailable");
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-6">
it("adds the first folder's Files pane while preserving existing editor tabs", async () => {
  const { result, rerender } = renderHook(({ folderOpen }) => {
    const [layout, setLayout] = useState(() => openUntitledTab(openUntitledTab(initialWorkspace()), true));
    const [, requestFocus] = useState(0);
    useFolderPane(folderOpen, true, layout, setLayout, requestFocus, vi.fn());
    return layout;
  }, { initialProps: { folderOpen: false } });
  const before = panes(result.current.root).flatMap(pane => pane.tabs).map(tab => tab.id);
  rerender({ folderOpen: true });
  await waitFor(() => expect(panes(result.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "tree")).toHaveLength(1));
  expect(panes(result.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind !== "tree").map(tab => tab.id)).toEqual(before);
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-7">
it("adds Files even when the stored layout left it closed", async () => {
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState(initialWorkspace);
    useFolderPane(true, true, layout, setLayout, vi.fn(), vi.fn(), true);
    return layout;
  });
  await waitFor(() => expect(panes(result.current.root).flatMap(pane => pane.tabs).map(entry => entry.kind)).toEqual(["tree"]));
});
</storycode:copy>

### Assemble `apps/web/src/views/folderPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/folderPane.test.tsx"><storycode:paste select="#f-c29616a4a7b1-expression_statement"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-2"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-3"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-4"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-5"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-6"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-7"/></storycode:file>


</storycode:doc>
