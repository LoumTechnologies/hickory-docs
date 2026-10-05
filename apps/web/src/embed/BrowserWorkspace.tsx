import { useEffect, useMemo, useRef, useState } from "react";
import { DebuggableDocument } from "./DebuggableDocument";
import { exportWorkspace, importWorkspace, searchWorkspace, workspacePath } from "./storage";
import type { SearchMatch, WorkspaceStorage } from "./storage";
import type { SyncedStorage, SyncState, SyncConflict } from "./syncedStorage";

/** Optional browser workspace. The embeddable document does not require it. */
export function BrowserWorkspace({ storage }: { storage: WorkspaceStorage }) {
  const [paths, setPaths] = useState<string[]>([]);
  const [path, setPath] = useState("notes/note.md");
  const [source, setSource] = useState("# A browser note\n");
  const [base, setBase] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [dirty, setDirty] = useState(true);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [query, setQuery] = useState("");
  const [matches, setMatches] = useState<SearchMatch[]>([]);
  const syncing = "sync" in storage && "syncState" in storage && "subscribe" in storage ? storage as SyncedStorage : null;
  const [syncState, setSyncState] = useState<SyncState | null>(null);
  const [conflicts, setConflicts] = useState<SyncConflict[]>([]);
  const generation = useRef(0);
  const latest = useRef({ source, path, base, revision }); latest.current = { source, path, base, revision };
  const urls = useRef(new Map<string, { revision: string; url: string }>());
  useEffect(() => {
    if (!syncing) { setSyncState(null); return; }
    const update = () => setSyncState(syncing.syncState()); update();
    return syncing.subscribe(update);
  }, [syncing]);
  useEffect(() => {
    let disposed = false;
    void storage.list().then(async (files) => {
      if (disposed) return;
      const notes = files.filter((name) => name.endsWith(".md")); setPaths(notes);
      if (notes.length) {
        const file = await storage.read(notes[0]);
        if (file && !disposed && latest.current.revision === 0) {
          setPath(file.path); setSource(new TextDecoder("utf-8", { fatal: true }).decode(file.bytes));
          setBase(file.revision); setDirty(false); setRevision((r) => r + 1);
        }
      }
    }).catch((error) => { if (!disposed) setMessage(String(error)); });
    return () => { disposed = true; generation.current++; for (const entry of urls.current.values()) URL.revokeObjectURL(entry.url); urls.current.clear(); };
  }, [storage]);
  useEffect(() => {
    let disposed = false;
    void searchWorkspace(storage, query).then((results) => { if (!disposed) setMatches(results); })
      .catch((error) => { if (!disposed) setMessage(String(error)); });
    return () => { disposed = true; };
  }, [storage, query, paths]);
  async function open(next: string) {
    if (dirty && !window.confirm("Discard unsaved edits to open another note?")) return;
    const id = ++generation.current;
    const expectedRevision = latest.current.revision;
    try {
      const file = await storage.read(next);
      if (id !== generation.current) return;
      if (latest.current.revision !== expectedRevision) { setMessage("The note finished loading after you edited. Your draft is still here; open it again when ready."); return; }
      setPath(next); setSource(file ? new TextDecoder("utf-8", { fatal: true }).decode(file.bytes) : "# A browser note\n");
      setBase(file?.revision ?? null); setRevision((r) => r + 1); setDirty(!file); setMessage(null);
    } catch (error) { if (id === generation.current) setMessage(String(error)); }
  }
  async function save() {
    const frozen = latest.current; setBusy(true); setMessage(null);
    try {
      const file = await storage.write(frozen.path, new TextEncoder().encode(frozen.source), frozen.base);
      if (latest.current.path === frozen.path) { setBase(file.revision); setDirty(latest.current.revision !== frozen.revision); }
      setPaths((previous) => [...new Set([...previous, file.path])].sort());
      setMessage(storage.durability === "memory" ? "Saved in memory only; export to keep these bytes." : storage.durability === "remote" ? "Saved to remote storage." : "Saved in this browser.");
    } catch (error) {
      const reason = error instanceof DOMException && error.name === "QuotaExceededError" ? "Browser storage is full; export your note to keep it." : String(error);
      setMessage(`Save failed. Your draft is still here. ${reason}`);
    }
    finally { setBusy(false); }
  }
  const host = useMemo(() => ({ resolveAsset: async (asset: string, signal: AbortSignal) => {
    const folder = latest.current.path.split("/").slice(0, -1).join("/");
    const file = await storage.read(workspacePath(folder ? `${folder}/${asset}` : asset));
    if (signal.aborted) throw new Error("Asset lookup cancelled");
    if (!file) throw new Error("Asset is missing");
    const cached = urls.current.get(file.path);
    if (cached?.revision === file.revision) return cached.url;
    if (cached) URL.revokeObjectURL(cached.url);
    const ext = asset.split(".").at(-1);
    const type = ext === "svg" ? "image/svg+xml" : ext === "png" ? "image/png" : ext === "jpg" || ext === "jpeg" ? "image/jpeg" : "application/octet-stream";
    const url = URL.createObjectURL(new Blob([file.bytes.slice().buffer], { type })); urls.current.set(file.path, { revision: file.revision, url }); return url;
  } }), [storage]);
  const showBytes = (bytes: Uint8Array | null) => {
    if (bytes === null) return "(deleted or absent)";
    try { return new TextDecoder("utf-8", { fatal: true }).decode(bytes); } catch { return `(binary file: ${bytes.length} bytes)`; }
  };
  async function resolve(conflict: SyncConflict, choice: "local" | "remote") {
    if (!syncing) return;
    try {
      await syncing.resolveConflict({ path: conflict.path, localRevision: conflict.local?.revision ?? null,
        remoteRevision: conflict.remote?.revision ?? null, bytes: conflict[choice]?.bytes ?? null });
      setConflicts((previous) => previous.filter((entry) => entry.path !== conflict.path));
      setMessage("Resolution saved locally. Sync saved files to publish it; reload the saved note to read it.");
    } catch (error) { setMessage(`Resolution failed; both versions were kept. ${String(error)}`); }
  }
  return <div className="hickory-browser-workspace">
    <p>Editing {path} · {dirty ? "unsaved edits" : "saved"} · {storage.durability === "memory" ? "memory only" : storage.durability === "remote" ? "remote storage" : "local browser storage"}</p>
    {syncing && <div aria-label="Remote synchronization">
      <span>{syncState?.state}{syncState?.reason ? ` · ${syncState.reason}` : ""}</span>
      <button disabled={syncState?.state === "syncing"} onClick={() => { void syncing.sync().catch(() => undefined); }}>Sync saved files</button>
      {syncState?.state === "conflict" && <button onClick={() => { void syncing.reviewConflicts().then(setConflicts).catch((error) => setMessage(String(error))); }}>Review sync conflicts</button>}
    </div>}
    {conflicts.map((conflict) => <details key={conflict.path} open>
      <summary>Resolve {conflict.path}</summary>
      <p>Choose a saved version to queue for publication. Unsaved editor text stays in its draft.</p>
      <div aria-label={`Sync conflict ${conflict.path}`}>
        <h3>Original base</h3><pre>{showBytes(conflict.base)}</pre>
        <h3>Local saved version</h3><pre>{showBytes(conflict.local?.bytes ?? null)}</pre>
        <h3>Remote version</h3><pre>{showBytes(conflict.remote?.bytes ?? null)}</pre>
        <button onClick={() => { void resolve(conflict, "local"); }}>Keep local saved version</button>
        <button onClick={() => { void resolve(conflict, "remote"); }}>Use remote version</button>
      </div>
    </details>)}
    <select disabled={busy} aria-label="Browser notes" value={paths.includes(path) ? path : ""} onChange={(event) => { void open(event.target.value); }}>
      <option value="" disabled>Select a saved note</option>{paths.map((name) => <option key={name}>{name}</option>)}
    </select>
    <button type="button" disabled={busy} onClick={() => { void save(); }}>Save note</button>
    <button type="button" disabled={busy} onClick={() => { void open(path); }}>Reload saved note</button>
    <button type="button" disabled={busy} onClick={() => { void open(`notes/note-${crypto.randomUUID()}.md`); }}>New note</button>
    <label>Import .md <input disabled={busy} type="file" accept=".md,text/markdown" aria-label="Import note" onChange={(event) => {
      const file = event.target.files?.[0]; if (!file) return;
      if (!file.name.endsWith(".md")) { setMessage("Documents use .md"); return; }
      if (dirty && !window.confirm("Replace unsaved edits with the imported note?")) return;
      const id = ++generation.current, expectedRevision = latest.current.revision;
      void file.arrayBuffer().then((bytes) => {
        if (id !== generation.current || latest.current.revision !== expectedRevision) { setMessage("Import finished after you edited. Your draft is still here."); return; }
        setSource(new TextDecoder("utf-8", { fatal: true }).decode(bytes)); setPath(workspacePath(`notes/${file.name}`));
        setBase(null); setDirty(true); setRevision((r) => r + 1);
      }).catch((error) => setMessage(String(error)));
    }} /></label>
    <button type="button" disabled={busy} onClick={() => {
      void exportWorkspace(storage).then((value) => {
        const url = URL.createObjectURL(new Blob([JSON.stringify(value)], { type: "application/json" }));
        const link = document.createElement("a"); link.href = url; link.download = "hickory-workspace.json"; link.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
      }).catch((error) => setMessage(String(error)));
    }}>Export saved workspace</button>
    <label>Import workspace <input disabled={busy} type="file" accept="application/json,.json" aria-label="Import workspace" onChange={(event) => {
      const file = event.target.files?.[0]; event.target.value = ""; if (!file) return;
      setBusy(true); setMessage(null);
      void file.text().then((text) => importWorkspace(storage, JSON.parse(text))).then((files) => {
        setPaths((previous) => [...new Set([...previous, ...files.map((entry) => entry.path).filter((name) => name.endsWith(".md"))])].sort());
        setMessage("Workspace imported. Existing files and your current draft were kept.");
      }).catch((error) => setMessage(`Import failed; no files were imported. ${String(error)}`)).finally(() => setBusy(false));
    }} /></label>
    <label>Search saved notes <input type="search" aria-label="Search saved notes" value={query} onChange={(event) => setQuery(event.target.value)} /></label>
    {query.trim() && <div aria-label="Search results">{matches.length ? <ul>{matches.map((match) =>
      <li key={`${match.path}:${match.line}`}><button disabled={busy} onClick={() => { void open(match.path); }}>{match.path}:{match.line} · {match.text}</button></li>)}</ul>
      : <p>No matching saved notes.</p>}</div>}
    {message && <p role="status">{message}</p>}
    <DebuggableDocument path={path} source={source} revision={String(revision)} host={host}
      onChange={(change) => { setSource(change.source); setDirty(true); setRevision((r) => r + 1); }} />
  </div>;
}
