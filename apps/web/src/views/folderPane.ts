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

type StartupSource = Awaited<ReturnType<typeof api.startupSource>>;
export type StartupPlainFile = {
  source: StartupSource;
  current: () => boolean;
  authority: Promise<{file: PlainFile; namespace: string; workspace: string; generatedBy: string | null}>;
};

export type StartupManagedDocument = {
  source: StartupSource;
  docId: string;
  current: () => boolean;
  authority: Promise<Doc>;
  confirmRoom: () => Promise<void>;
};

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
  const confirmRoom = async () => {
    const original = await authority;
    if (!current()) throw Error("The original document request changed.");
    const latest = await api.doc(docId);
    if (!current() || latest.path !== path || latest.id !== docId || latest.source !== original.source
      || latest.save_guard !== original.save_guard) {
      throw Error("The document's saved authority changed while its room loaded. Reopen the read-only preview.");
    }
  };
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
