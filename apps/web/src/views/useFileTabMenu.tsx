import { useEffect, useId, useMemo, useRef, useState, type MouseEvent } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { FileNode } from "../api/types";
import { docPathOf, isLikelyBinaryPath, type FolderTree } from "../shell/FolderTreePane";
import { FileActionPrompts, runFileMenuItem } from "../shell/fileMenuActions";
import { TreeContextMenu } from "../shell/TreeContextMenu";
import { ContextMenu } from "../components/ContextMenu";
import { api, request } from "../api/client";
import { createPortal } from "react-dom";
import { showRepresentation } from "../lib/openRepresentation";
import { representations } from "../api/representations";
import { openCommit } from "../lib/readingViews";
import type { GitCommit } from "../api/gitHistoryTypes";
import { treeMenuItems } from "../shell/treeMenu";
import { useDired } from "../shell/useDired";
import type { Tab } from "../shell/layout";

function findFile(nodes: readonly FileNode[], path: string): FileNode | undefined {
  for (const node of nodes) {
    if (node.path === path) return node;
    const found = node.children && findFile(node.children, path);
    if (found) return found;
  }
}

export function useFileTabMenu(roots: readonly FolderTree[], activeDocId: string | null | undefined, openable: ReadonlyMap<string, string>, openDocument: (id: string) => void, tabs: readonly Tab[] = []) {
  const ownerSignature = JSON.stringify([roots, activeDocId, [...openable], tabs]);
  const owner = useMemo(() => crypto.randomUUID(), [ownerSignature]);
  const [menu, setMenu] = useState<{ owner: string; x: number; y: number; path: string; folder: FolderTree; plainText: boolean } | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [webMenu, setWebMenu] = useState<{ target: string; x: number; y: number } | null>(null);
  const dired = useDired(setNotice);
  const [history, setHistory] = useState<string | null>(null);
  useEffect(() => {
    const show = (event: Event) => { const path = (event as CustomEvent<{ path: string }>).detail?.path; if (path && (roots.some(folder => findFile(folder.tree, path)) || targets.some(tab => tab.target === path))) setHistory(path); };
    window.addEventListener("hickory-file-history", show);
    return () => window.removeEventListener("hickory-file-history", show);
  }, [owner]);
  useEffect(() => { setWebMenu(null); setHistory(null); }, [owner]);
  useEffect(() => { if (menu && menu.owner !== owner) setMenu(null); }, [menu, owner]);
  const open = (tab: Tab, position: { x: number; y: number }) => {
    if (!["document", "generated", "file"].includes(tab.kind) || (tabs.length > 0 && !tabs.some(current => current.id === tab.id && current.kind === tab.kind && current.target === tab.target && current.docId === tab.docId))) return false;
    for (const folder of roots) {
      const path = tab.kind === "document" && tab.docId ? docPathOf(folder.tree, tab.docId) ?? tab.target : tab.target;
      const node = findFile(folder.tree, path);
      // Tabs can outlive a truncated listing. Their kind still identifies
      // document and generated files, which must never offer adoption.
      if (!node && roots.length !== 1) continue;
      setNotice(null);
      setMenu({ owner, ...position, path, folder,
        plainText: tab.kind === "file" && !node?.doc_id && !node?.generated_by && !openable.has(path) && !isLikelyBinaryPath(path) });
      return true;
    }
    return false;
  };
  const onTabContextMenu = (event: MouseEvent, tab: Tab) => {
    if (open(tab, { x: event.clientX, y: event.clientY })) { event.preventDefault(); event.stopPropagation(); }
  };
  // Links use the same current tree metadata and verbs as Files and tabs.
  useEffect(() => {
    const onLink = (event: Event) => {
      const detail = (event as CustomEvent<{ path: string | null; target: string; x: number; y: number }>).detail;
      if (!detail) return;
      setMenu(null); setWebMenu(null);
      if (!detail.path) {
        if (/^(https?|mailto|tel):/i.test(detail.target)) setWebMenu(detail);
        return;
      }
      const matches = roots.filter(folder => findFile(folder.tree, detail.path!));
      if (matches.length !== 1) return;
      const folder = matches[0], node = findFile(folder.tree, detail.path)!;
      setNotice(null);
      setMenu({ owner, x: detail.x, y: detail.y, path: detail.path, folder,
        plainText: !node.doc_id && !node.generated_by && !openable.has(detail.path) && !isLikelyBinaryPath(detail.path) });
    };
    const onError = (event: Event) => setNotice((event as CustomEvent<string>).detail);
    window.addEventListener("hickory-link-menu", onLink);
    window.addEventListener("hickory-link-error", onError);
    return () => {
      window.removeEventListener("hickory-link-menu", onLink);
      window.removeEventListener("hickory-link-error", onError);
    };
  }, [owner, roots, openable]);
  const parent = `file-tab-menus:${useId()}`;
  const targets = tabs.filter(tab => ["document", "generated", "file"].includes(tab.kind));
  const revision = useUiRevision(() => [roots, activeDocId, [...openable], targets, menu]);
  useUiCommands([{ key: `${parent}:read`, command: { parent, subject: `${parent}:reading`, label: "Read file tab menu targets", parameters: { first: { type: "number", integer: true, minimum: 0, maximum: 100000000 }, count: { type: "number", integer: true, minimum: 1, maximum: 64 } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      const first = Number(args.first), count = Number(args.count);
      if (first > targets.length) return { status: "unavailable", reason: "Index exceeds current file tabs" };
      const value = { tabs: targets.slice(first, first + count).map(tab => ({ id: tab.id, kind: tab.kind, path: tab.target })), total: targets.length, next: first + count < targets.length ? first + count : null };
      if (new TextEncoder().encode(JSON.stringify(value)).length > 8192) return { status: "unavailable", reason: "Tab page exceeds 8192 UTF-8 bytes; request fewer tabs" };
      return { status: "completed", value };
    } } }, { key: `${parent}:open`, command: { parent, subject: parent, label: "Open file tab context menu", parameters: { tab: { type: "string", maxLength: 4096 } }, prerequisites: ["Current document/generated/plain-file tab in its owning folder"],
      read: () => ({ revision: revision(), enabled: targets.length > 0 }), execute: async args => {
        const matches = targets.filter(tab => tab.id === args.tab);
        if (matches.length !== 1 || !open(matches[0], { x: 16, y: 16 })) return { status: "unavailable", reason: "Tab closed, ambiguous or its owning folder is unavailable" };
        return { status: "completed", value: { menu_open_requested: true, source_changed: false } };
      } } }]);
  const activePath = menu && activeDocId ? docPathOf(menu.folder.tree, activeDocId) : null;
  return {
    onTabContextMenu,
    fileTabMenu: <>
      {history && <FileRevisionHistory key={history} path={history} close={() => setHistory(null)} />}
      {webMenu && <ContextMenu x={webMenu.x} y={webMenu.y} subject={webMenu.target}
        items={[["default", "default browser"], ["chrome", "Chrome"], ["firefox", "Firefox"], ["edge", "Edge"], ["brave", "Brave"],
          ...(/mac/i.test(navigator.platform) ? [["safari", "Safari"]] : [])].map(([id, name]) => ({ id, label: `Open in ${name}` }))}
        onClose={() => setWebMenu(null)} onPick={item => {
          const target = webMenu.target;
          setWebMenu(null);
          void request("POST", "/api/open-external", { url: target, browser: item }).catch(error => setNotice(String(error)));
        }} /> }
      {menu && menu.owner === owner && <TreeContextMenu x={menu.x} y={menu.y} subject={menu.path}
        items={treeMenuItems({ rootPath: menu.folder.root_path, separator: menu.folder.separator, fileManager: menu.folder.file_manager }, menu.path, false, {
          plainText: menu.plainText,
          activeDoc: activePath ? { path: activePath, name: activePath.split("/").pop() ?? activePath } : undefined,
        })}
        onClose={() => setMenu(null)} onPick={item => {
          setMenu(null);
          runFileMenuItem(item, dired, setNotice, openDocument);
        }} />}
      {notice && <p role="alert">{notice}</p>}
      <FileActionPrompts dired={dired} />
    </>,
  };
}
/** Read revision choices independently; comparison loads Git bytes on demand. */
function FileRevisionHistory({ path, close }: { path: string; close: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [commits, setCommits] = useState<GitCommit[] | null>(null);
  const [base, setBase] = useState(""), [target, setTarget] = useState("");
  const [error, setError] = useState(""), [busy, setBusy] = useState(false);
  const live = useRef(true), pending = useRef(false);
  useEffect(() => {
    live.current = true; dialog.current?.showModal();
    api.gitLog(1000, path).then(log => { if (live.current) { setCommits(log.commits); setBase(log.commits[0]?.sha ?? ""); } }, cause => { if (live.current) setError(String(cause)); });
    return () => { live.current = false; };
  }, [path]);
  const parent = `file-history:${useId()}`;
  const revision = useUiRevision(() => [path, commits, base, target, busy, error]);
  const invoke = useUiCommands([
    { key: `${parent}:select`, command: { parent, subject: parent, label: "Select file comparison revisions", parameters: { base: { type: "string", maxLength: 256 }, target: { type: "string", maxLength: 256 } }, prerequisites: [], read: () => ({ revision: revision(), enabled: !!commits && !busy }), execute: async args => {
      if (!commits?.some(commit => commit.sha === args.base) || args.target !== "" && !commits?.some(commit => commit.sha === args.target)) return { status: "unavailable", reason: "Select revisions from this file’s loaded history" };
      setBase(String(args.base)); setTarget(String(args.target)); return { status: "completed", value: { selected: true } };
    } } },
    { key: `${parent}:compare`, command: { parent, subject: parent, label: "Compare selected file revisions", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!base && base !== target && !busy }), execute: async () => {
      if (!base || base === target || pending.current) return { status: "unavailable", reason: "Choose two different revisions and wait for the current comparison" };
      pending.current = true; setBusy(true); setError("");
      try {
        const reading = await representations.create({ kind: "files", paths: [path] });
        if (!live.current) return { status: "unavailable", reason: "The file history was closed while its comparison loaded" };
        showRepresentation(reading.id, base, target || undefined); close();
        return { status: "completed", value: { comparison_requested: true, path } };
      }
      catch (cause) { if (live.current) setError(String(cause)); return { status: "unavailable", reason: String(cause) }; }
      finally { pending.current = false; if (live.current) setBusy(false); }
    } } },
  ]);
  const choices = (commits ?? []).map(commit => <option key={commit.sha} value={commit.sha}>{commit.short} · {commit.subject}</option>);
  return createPortal(<dialog ref={dialog} aria-label={`Revision history for ${path}`} onCancel={close} style={{ width: "min(720px,85vw)", maxHeight: "80vh", overflow: "auto", background: "var(--bg-raised)", color: "var(--fg)", border: "1px solid var(--border)", borderRadius: 8 }}>
    <header style={{ display: "flex", justifyContent: "space-between" }}><strong>{path}</strong><button type="button" onClick={close} aria-label="Close revision history">×</button></header>
    {error && <p role="alert">{error}</p>}
    {!commits && !error && <p role="status">Loading revisions…</p>}
    {commits && !commits.length && <p>No recorded revisions.</p>}
    {!!commits?.length && <><form onSubmit={event => { event.preventDefault(); void invoke(`${parent}:compare`); }} style={{ display: "flex", gap: 8, margin: "12px 0", flexWrap: "wrap" }}>
      <label>From <select aria-label="Earlier file revision" value={base} disabled={busy} onChange={event => setBase(event.target.value)}>{choices}</select></label>
      <label>To <select aria-label="Later file revision" value={target} disabled={busy} onChange={event => setTarget(event.target.value)}><option value="">Working tree</option>{choices}</select></label>
      <button disabled={busy || !base || base === target}>{busy ? "Opening…" : "Compare"}</button>
    </form><ul style={{ paddingLeft: 20 }}>{commits.map(commit => <li key={commit.sha}><button type="button" className="btn-link" onClick={() => openCommit(commit.sha)}>{commit.short} · {commit.subject}</button> <time>{new Date(commit.time * 1000).toLocaleDateString()}</time></li>)}</ul></>}
  </dialog>, document.body);
}
