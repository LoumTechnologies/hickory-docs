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
type OpenFileSelection = Awaited<ReturnType<typeof api.openFile>>;
export type StartupPlainFile = {
  source: StartupSource;
  current: () => boolean;
  authority: Promise<{file: PlainFile; namespace: string; workspace: string; generatedBy: string | null}>;
  retry?: () => void;
};

export type ManagedDocumentPreview = {
  source: Pick<StartupSource, "path" | "content">;
  docId: string;
  current: () => boolean;
  authority: Promise<Doc>;
  confirmRoom: () => Promise<void>;
};

export type StartupManagedDocument = ManagedDocumentPreview & {
  source: StartupSource;
  retry?: () => void;
};

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

function startupPath(requested: string, hint: string | null) {
  const path = hint ?? requested;
  const normalizedRequest = requested.replaceAll("\\", "/");
  if (hint && requested !== hint && !normalizedRequest.endsWith("/" + hint)) return null;
  if (path.includes("\\") || path.includes(":")
    || /[\u0000-\u001f\u007f]/.test(path) || path.split("/").some(part => !part || part === "." || part === "..")) return null;
  return path;
}

function matchesRequested(requested: string, root: string, path: string) {
  const absolute = root.replaceAll("\\", "/").replace(/\/$/, "") + "/" + path;
  return requested === path || requested.replaceAll("\\", "/") === absolute;
}

async function readStartupSource(requested: string, path: string, current: () => boolean): Promise<StartupSource | null> {
  const source = await api.startupSource(path);
  if (!current() || source?.version !== 1 || source.path !== path || typeof source.content !== "string"
    || !source.root_path || !matchesRequested(requested, source.root_path, path)) return null;
  return source;
}

// The scoped file response and raw observation can arrive in either order.
// Present the first matching text. A failed read leaves its peer active.
function firstStartupSource(raw: Promise<StartupSource | null>, guarded: Promise<StartupSource | null>) {
  const rawText = raw.catch(() => null);
  const guardedText = guarded.catch(() => null);
  return Promise.race([rawText.then(text => text ?? guardedText), guardedText.then(text => text ?? rawText)]);
}

function scopedStartupText(requested: string, files: FilesResponse, path: string, content: string, doc_id?: string) {
  const root = files.root_path, workspace = files.workspace_id;
  const node = findNodeByPath([files], path);
  if (files.folder_open === false || !root || !workspace || !/^[a-f0-9]{32}$/.test(workspace)
    || !matchesRequested(requested, root, path) || !node || node.dir || (node.doc_id ?? undefined) !== doc_id
    || doc_id && node.generated_by) return null;
  return {version: 1 as const, path, content, root_path: root, workspace_id: workspace, doc_id};
}

// Open File can bind the selected path to its workspace before the tree
// arrives. Its scope is enough to show the exact text read-only; the separate
// Files response still has to confirm ownership before editing.
function readStartupSelection<T>(request: Promise<OpenFileSelection>, load: (opened: OpenFileSelection) => Promise<T>) {
  const text = request.then(async opened => ({opened, value: await load(opened)}));
  const files = request.then(() => api.files());
  void files.catch(() => {});
  return {text, files};
}

function selectedStartupText(requested: string, opened: OpenFileSelection, files: Promise<FilesResponse>, content: string) {
  const scope = opened.source_scope;
  if (!scope) {
    return files.then(tree => scopedStartupText(requested, tree, opened.path, content, opened.doc_id));
  }
  const {root_path, workspace_id} = scope;
  if (!root_path || !/^[a-f0-9]{32}$/.test(workspace_id)
    || !matchesRequested(requested, root_path, opened.path)) return null;
  return {version: 1 as const, path: opened.path, content, root_path, workspace_id, doc_id: opened.doc_id};
}

function selectionMatchesSource(opened: OpenFileSelection, source: StartupSource) {
  const scope = opened.source_scope;
  return !scope || (scope.root_path === source.root_path && scope.workspace_id === source.workspace_id);
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
  const path = startupPath(requested, hint);
  if (!path || path.endsWith(".md")) return null;
  const refuse = () => Error("The requested file no longer matches the startup observation. Reopen it to check the current source.");
  const read = (request: typeof selected) => readStartupSelection(request, async opened => {
    if (!current() || opened.doc_id || opened.path !== path) throw refuse();
    return api.file(path);
  });
  const initial = read(selected);
  const source = await firstStartupSource(
    readStartupSource(requested, path, current).then(raw => raw?.doc_id ? null : raw),
    initial.text.then(({opened, value: file}) => file.path === path
      ? selectedStartupText(requested, opened, initial.files, file.content) : null),
  );
  if (!source || !current()) return null;
  const authorize = (request: typeof initial) => {
    const authority = Promise.all([request.text, request.files]).then(([{opened, value: file}, files]) => {
      const workspace = files.workspace_id, namespace = files.root_path;
      const node = findNodeByPath([files], path);
      if (!current() || !namespace || !workspace
        || !selectionMatchesSource(opened, source) || !ownsStartupSource(files, source) || !node || node.dir || node.doc_id
        || file.path !== path || file.content !== source.content) {
        throw refuse();
      }
      if (!node.generated_by && !file.save_guard) throw Error(file.save_guard_reason || "The file's saved source authority is unavailable. Reopen it to check again.");
      return {file, namespace, workspace, generatedBy: node.generated_by ?? null};
    });
    // The preview may close or be superseded before its pane mounts.
    void authority.catch(() => {});
    return authority;
  };
  let authority = authorize(initial);
  // Refresh permission against the same text and ownership lease. Replacing
  // the preview would replace the managed editor and its native view state.
  return {source, current, get authority() { return authority; },
    retry: () => { if (current()) authority = authorize(read(api.openFile(requested))); }};
}

export async function observeManagedStartup(
  requested: string, hint: string | null, selected: ReturnType<typeof api.openFile>,
  current: () => boolean,
): Promise<StartupManagedDocument | null> {
  const path = startupPath(requested, hint);
  if (!path || !path.endsWith(".md")) return null;
  const validDocId = (id?: string) => !!id && /^[a-f0-9]{16}$/.test(id);
  const read = (request: typeof selected) => readStartupSelection(request, async opened => {
    const docId = opened.doc_id;
    if (!current() || opened.path !== path || !docId || !validDocId(docId)) throw Error("The selected document changed.");
    return api.doc(docId);
  });
  const initial = read(selected);
  const source = await firstStartupSource(
    readStartupSource(requested, path, current).then(raw => validDocId(raw?.doc_id) ? raw : null),
    initial.text.then(({opened, value: doc}) => doc.path === path && doc.id === opened.doc_id
      ? selectedStartupText(requested, opened, initial.files, doc.source) : null),
  );
  const docId = source?.doc_id;
  if (!source || !docId || !current()) return null;
  const authorize = (request: typeof initial) => {
    const authority = Promise.all([request.text, request.files]).then(([{opened, value: doc}, files]) => {
      const refuse = () => Error("The requested document no longer matches its startup observation. Reopen it to check the current source.");
      const node = findNodeByPath([files], path);
      if (!current() || opened.doc_id !== docId || !selectionMatchesSource(opened, source) || !ownsStartupSource(files, source) || !node || node.dir
        || node.generated_by || node.doc_id !== docId || doc.id !== docId || doc.path !== path || doc.source !== source.content) throw refuse();
      if (!doc.save_guard) throw Error(doc.save_guard_reason || "The document's saved source authority is unavailable. Reopen it to check again.");
      return doc;
    });
    void authority.catch(() => {});
    return authority;
  };
  let authority = authorize(initial);
  return {source, docId, current, get authority() { return authority; },
    confirmRoom: () => confirmManagedRoom(authority, current),
    retry: () => { if (current()) authority = authorize(read(api.openFile(requested))); }};
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

</storycode:doc>
