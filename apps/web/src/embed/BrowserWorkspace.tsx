import { useEffect, useId, useMemo, useRef, useState } from "react";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
import { DebuggableDocument } from "./DebuggableDocument";
import { exportWorkspace, importWorkspace, searchWorkspace, workspacePath } from "./storage";
import type { SearchMatch, WorkspaceStorage } from "./storage";
import type { SyncedStorage, SyncState, SyncConflict } from "./syncedStorage";

/** Optional browser workspace. The embeddable document does not require it. */
export function BrowserWorkspace({ storage }: { storage: WorkspaceStorage }) {
  const owner = useMemo(() => crypto.randomUUID(), [storage]);
  return <BrowserWorkspaceOwner key={owner} storage={storage} />;
}
function BrowserWorkspaceOwner({ storage }: { storage: WorkspaceStorage }) {
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
  const [exportCapture, setExportCapture] = useState<{ id: string; bytes: Uint8Array } | null>(null);
  const syncing = "sync" in storage && "syncState" in storage && "subscribe" in storage ? storage as SyncedStorage : null;
  const [syncState, setSyncState] = useState<SyncState | null>(null);
  const [conflicts, setConflicts] = useState<SyncConflict[]>([]);
  const generation = useRef(0), pending = useRef(false), alive = useRef(true), controller = useRef<AbortController | null>(null);
  const prompt = usePrompt();
  useEffect(() => { alive.current = true; return () => { alive.current = false; controller.current?.abort(); }; }, []);
  async function operation(work: (check: () => void, signal: AbortSignal) => Promise<UiValue>, signal?: AbortSignal, failurePrefix = "Operation failed; inspect saved files for any earlier effects. Your draft is still here."): Promise<UiOutcome> {
    if (pending.current || !alive.current || signal?.aborted) return { status: "unavailable", reason: "The originating browser workspace is busy, disposed or cancelled" };
    pending.current = true; ++generation.current; setBusy(true); setMessage(null);
    const own = new AbortController(), abort = () => own.abort(); controller.current = own;
    signal?.addEventListener("abort", abort, { once: true });
    const check = () => { if (!alive.current || own.signal.aborted) throw Error("The originating workspace operation was cancelled or disposed. Earlier storage effects may have occurred; inspect saved files before retrying."); };
    try { const value = await work(check, own.signal); check(); return { status: "completed", value }; }
    catch (error) {
      const reason = error instanceof DOMException && error.name === "QuotaExceededError" ? "Browser storage is full; export your note to keep it." : String(error);
      if (alive.current) setMessage(`${failurePrefix} ${reason}`);
      return { status: "unavailable", reason };
    } finally { signal?.removeEventListener("abort", abort); pending.current = false; controller.current = null; if (alive.current) setBusy(false); }
  }
  const latest = useRef({ source, path, base, revision, dirty }); latest.current = { source, path, base, revision, dirty };
  function adopt(text: string, nextPath: string, nextBase: string | null, isDirty: boolean) {
    const next = latest.current.revision + 1;
    latest.current = { source: text, path: nextPath, base: nextBase, revision: next, dirty: isDirty };
    setPath(nextPath); setSource(text); setBase(nextBase); setRevision(next); setDirty(isDirty);
  }
  const urls = useRef(new Map<string, { revision: string; url: string }>());
  useEffect(() => {
    if (!syncing) { setSyncState(null); return; }
    const update = () => setSyncState(syncing.syncState()); update();
    return syncing.subscribe(update);
  }, [syncing]);
  useEffect(() => {
    let disposed = false; const hydrate = generation.current;
    void storage.list().then(async (files) => {
      if (disposed) return;
      const notes = files.filter((name) => name.endsWith(".md")); setPaths(notes);
      if (notes.length) {
        const file = await storage.read(notes[0]);
        if (file && !disposed && hydrate === generation.current && !pending.current && latest.current.revision === 0) {
          adopt(new TextDecoder("utf-8", { fatal: true }).decode(file.bytes), file.path, file.revision, false);
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
  const draftIdentity = () => JSON.stringify(latest.current);
  async function confirmDiscard(title: string, signal: AbortSignal) {
    if (!latest.current.dirty) return true;
    return await prompt.askChoice(title, [{ label: "Discard and continue", value: true }], { agent: true, currentRevision: draftIdentity, signal }) === true;
  }
  async function open(next: string, signal?: AbortSignal) { return operation(async (check, signal): Promise<UiValue> => {
    workspacePath(next); const expected = draftIdentity();
    if (!await confirmDiscard("Discard unsaved edits to open another note?", signal)) return { cancelled: true, note_opened: false };
    check(); if (draftIdentity() !== expected) throw Error("The draft changed while its open decision was pending");
    const id = ++generation.current, file = await storage.read(next);
    check();
    if (id !== generation.current || draftIdentity() !== expected) throw Error("The note finished loading after you edited. Your draft is still here; open it again when ready.");
    adopt(file ? new TextDecoder("utf-8", { fatal: true }).decode(file.bytes) : "# A browser note\n", next, file?.revision ?? null, !file);
    return { note_opened: true, path: next, saved_revision: file?.revision ?? null };
  }, signal); }
  async function save(signal?: AbortSignal) { return operation(async check => {
    const frozen = latest.current;
    const file = await storage.write(frozen.path, new TextEncoder().encode(frozen.source), frozen.base);
    check();
    if (latest.current.path === frozen.path) { const newer = latest.current.revision !== frozen.revision; latest.current = { ...latest.current, base: file.revision, dirty: newer }; setBase(file.revision); setDirty(newer); }
    setPaths(previous => [...new Set([...previous, file.path])].sort());
    setMessage(storage.durability === "memory" ? "Saved in memory only; export to keep these bytes." : storage.durability === "remote" ? "Saved to remote storage." : "Saved in this browser.");
    return { saved: true, path: file.path, revision: file.revision, current_draft_matches: latest.current.revision === frozen.revision, durability: storage.durability };
  }, signal, "Save failed. Your draft is still here."); }
  async function importNote(name: string, bytes: () => Promise<ArrayBuffer>, signal?: AbortSignal) { return operation(async (check, signal): Promise<UiValue> => {
    if (!name.endsWith(".md")) throw Error("Documents use .md");
    const next = workspacePath(`notes/${name}`), expected = draftIdentity();
    if (!await confirmDiscard("Replace unsaved edits with the imported note?", signal)) return { cancelled: true, imported: false };
    check(); if (draftIdentity() !== expected) throw Error("The draft changed before import started");
    const captured = await bytes(); check();
    if (draftIdentity() !== expected) throw Error("Import finished after you edited. Your draft is still here.");
    const text = new TextDecoder("utf-8", { fatal: true }).decode(captured);
    adopt(text, next, null, true);
    return { imported: true, path: next, saved: false };
  }, signal); }
  async function importBundle(text: () => Promise<string>, signal?: AbortSignal) { return operation(async check => {
    const value = JSON.parse(await text()); check();
    let files;
    try { files = await importWorkspace(storage, value); } catch (error) { throw Error(`Import failed; no files were imported. ${String(error)}`); }
    check();
    setPaths(previous => [...new Set([...previous, ...files.map(entry => entry.path).filter(name => name.endsWith(".md"))])].sort());
    setMessage("Workspace imported. Existing files and your current draft were kept.");
    return { imported: files.map(file => ({ path: file.path, revision: file.revision })), draft_replaced: false };
  }, signal); }
  async function exportBundle(signal?: AbortSignal) { return operation(async check => {
    const value = await exportWorkspace(storage); check();
    const bytes = new TextEncoder().encode(JSON.stringify(value)), id = crypto.randomUUID(); setExportCapture({ id, bytes });
    const url = URL.createObjectURL(new Blob([bytes.buffer], { type: "application/json" }));
    const link = document.createElement("a"); link.href = url; link.download = "hickory-workspace.json"; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    return { download_requested: true, snapshot_files: value.files.length, export_capture: id, byte_length: bytes.length, file_persistence: "not observed", unsaved_draft_included: false };
  }, signal); }
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
  async function resolve(conflict: SyncConflict, choice: "local" | "remote", signal?: AbortSignal) { return operation(async check => {
    if (!syncing || !conflicts.includes(conflict)) throw Error("This retained sync conflict is no longer available");
    await syncing.resolveConflict({ path: conflict.path, localRevision: conflict.local?.revision ?? null,
      remoteRevision: conflict.remote?.revision ?? null, bytes: conflict[choice]?.bytes ?? null }); check();
    setConflicts(previous => previous.filter(entry => entry !== conflict));
    setMessage("Resolution saved locally. Sync saved files to publish it; reload the saved note to read it.");
    return { resolution_saved_locally: true, path: conflict.path, choice, remotely_published: false, editor_draft_replaced: false };
  }, signal); }
  async function sync(signal?: AbortSignal) { return operation(async check => {
    if (!syncing) throw Error("Remote synchronization is not configured");
    await syncing.sync(); check(); const state = syncing.syncState();
    if (state.state !== "synced") throw Error(`Synchronization is ${state.state}: ${state.reason ?? "inspect the current conflict/status"}`);
    return { synchronized: true, state: state.state };
  }, signal); }
  async function reviewConflicts(signal?: AbortSignal) { return operation(async check => {
    if (!syncing) throw Error("Remote synchronization is not configured");
    const captured = await syncing.reviewConflicts(); check(); setConflicts(captured);
    return { conflicts_disclosed: captured.length };
  }, signal); }
  const stateRevision = useUiRevision(() => [latest.current, paths, busy, message, query, matches, syncState, conflicts]);
  useUiReading("browser workspace status", { path, dirty, revision, base, durability: storage.durability, paths, busy, message, query, matches: query.trim() ? matches : [], sync: syncState, export_capture: exportCapture ? { id: exportCapture.id, byte_length: exportCapture.bytes.length, historical: true } : null });
  const command = (key: string, label: string, execute: (args: Record<string, UiValue>, signal: AbortSignal) => Promise<UiOutcome>, parameters: UiCommand["parameters"] = {}, enabled = true) => ({ key: `browser-workspace:${key}`, command: {
    label, parent: "browser-workspace", subject: "browser-workspace:operation", parameters, prerequisites: [], read: () => ({ revision: stateRevision(), enabled: alive.current && !pending.current && enabled }),
    execute: (args: Record<string, UiValue>, _actor: unknown, signal: AbortSignal) => execute(args, signal),
  } });
  useUiCommands([
    ...(exportCapture ? [{ key: "browser-workspace:export-read", command: { label: "Read captured browser workspace export bytes", parent: "browser-workspace:export", subject: "browser-workspace:export", parameters: { offset: { type: "number" as const, minimum: 0, maximum: 1e9, integer: true }, limit: { type: "number" as const, minimum: 1, maximum: 8192, integer: true } }, prerequisites: ["Historical saved-file snapshot; current unsaved draft is excluded."], read: () => ({ revision: exportCapture.id, enabled: true }), execute: async (args: Record<string, UiValue>) => {
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), exportCapture.bytes.length);
      if (offset > exportCapture.bytes.length) return { status: "unavailable" as const, reason: "Offset outside the retained workspace export" };
      return { status: "completed" as const, value: { capture: exportCapture.id, historical: true, bytes: Array.from(exportCapture.bytes.slice(offset, end)), offset, total_bytes: exportCapture.bytes.length, next_offset: end < exportCapture.bytes.length ? end : null } };
    } } }] : []),
    command("save", "Save browser note", (_args, signal) => save(signal)),
    command("reload", "Reload saved browser note", (_args, signal) => open(path, signal)),
    command("new", "Create browser note", (_args, signal) => open(`notes/note-${crypto.randomUUID()}.md`, signal)),
    command("open", "Open saved browser note", (args, signal) => paths.includes(args.path as string) ? open(args.path as string, signal) : Promise.resolve({ status: "unavailable", reason: "Select a currently listed saved note" }), { path: { type: "string", maxLength: 4096 } }),
    command("import-note", "Import browser note text", (args, signal) => importNote(args.name as string, async () => new TextEncoder().encode(args.text as string).buffer, signal), { name: { type: "string", maxLength: 1024 }, text: { type: "string", maxLength: 1048576 } }),
    command("import-workspace", "Import browser workspace JSON", (args, signal) => importBundle(async () => args.json as string, signal), { json: { type: "string", maxLength: 1048576 } }),
    command("export", "Export saved browser workspace", (_args, signal) => exportBundle(signal)),
    ...(syncing ? [command("sync", "Sync saved browser files", (_args, signal) => sync(signal), {}, syncState?.state !== "syncing"),
      ...(syncState?.state === "conflict" ? [command("review-conflicts", "Review browser sync conflicts", (_args, signal) => reviewConflicts(signal))] : []),
] : []),
    { key: "browser-workspace:search", command: { label: "Search saved browser notes", parent: "browser-workspace", subject: "browser-workspace:search", parameters: { query: { type: "string", maxLength: 8192 } }, prerequisites: [], read: () => ({ revision: stateRevision(), enabled: true }), execute: async args => { setQuery(args.query as string); return { status: "completed", value: { search_requested: true, results: "Read actual asynchronous saved-note matches" } }; } } },
    { key: "browser-workspace:cancel", command: { label: "Cancel browser workspace operation", parent: "browser-workspace", subject: "browser-workspace:cancel", parameters: {}, prerequisites: [], read: () => ({ revision: stateRevision(), enabled: pending.current }), execute: async () => { controller.current?.abort(); return { status: "completed", value: { cancellation_requested: true, rollback: false } }; } } },
  ]);
  return <div className="hickory-browser-workspace">
    <PromptPanel prompt={prompt.prompt} onSettle={prompt.settle} />
    <p>Editing {path} · {dirty ? "unsaved edits" : "saved"} · {storage.durability === "memory" ? "memory only" : storage.durability === "remote" ? "remote storage" : "local browser storage"}</p>
    {syncing && <div aria-label="Remote synchronization">
      <span>{syncState?.state}{syncState?.reason ? ` · ${syncState.reason}` : ""}</span>
      <button disabled={busy || syncState?.state === "syncing"} onClick={() => { void sync(); }}>Sync saved files</button>
      {syncState?.state === "conflict" && <button disabled={busy} onClick={() => { void reviewConflicts(); }}>Review sync conflicts</button>}
    </div>}
    {conflicts.map(conflict => <BrowserSyncConflict key={conflict.path} conflict={conflict} busy={busy} showBytes={showBytes} resolve={resolve} />)}
    <select disabled={busy} aria-label="Browser notes" value={paths.includes(path) ? path : ""} onChange={(event) => { void open(event.target.value); }}>
      <option value="" disabled>Select a saved note</option>{paths.map((name) => <option key={name}>{name}</option>)}
    </select>
    <button type="button" disabled={busy} onClick={() => { void save(); }}>Save note</button>
    <button type="button" disabled={busy} onClick={() => { void open(path); }}>Reload saved note</button>
    <button type="button" disabled={busy} onClick={() => { void open(`notes/note-${crypto.randomUUID()}.md`); }}>New note</button>
    <label>Import .md <input disabled={busy} type="file" accept=".md,text/markdown" aria-label="Import note" onChange={(event) => {
      const file = event.target.files?.[0]; if (!file) return;
      void importNote(file.name, () => file.arrayBuffer());
    }} /></label>
    <button type="button" disabled={busy} onClick={() => { void exportBundle(); }}>Export saved workspace</button>
    <label>Import workspace <input disabled={busy} type="file" accept="application/json,.json" aria-label="Import workspace" onChange={event => {
      const file = event.target.files?.[0]; event.target.value = ""; if (file) void importBundle(() => file.text());
    }} /></label>
    <label>Search saved notes <input type="search" aria-label="Search saved notes" value={query} onChange={(event) => setQuery(event.target.value)} /></label>
    {query.trim() && <div aria-label="Search results">{matches.length ? <ul>{matches.map((match) =>
      <li key={`${match.path}:${match.line}`}><button disabled={busy} onClick={() => { void open(match.path); }}>{match.path}:{match.line} · {match.text}</button></li>)}</ul>
      : <p>No matching saved notes.</p>}</div>}
    {message && <p role="status">{message}</p>}
    <DebuggableDocument path={path} source={source} revision={String(revision)} host={host}
      onChange={change => { const next = latest.current.revision + 1; latest.current = { ...latest.current, source: change.source, revision: next, dirty: true }; setSource(change.source); setDirty(true); setRevision(next); }} />
  </div>;
}

function BrowserSyncConflict({ conflict, busy, showBytes, resolve }: { conflict: SyncConflict; busy: boolean; showBytes: (bytes: Uint8Array | null) => string; resolve: (conflict: SyncConflict, choice: "local" | "remote", signal?: AbortSignal) => Promise<UiOutcome> }) {
  const [open, setOpen] = useState(true), parent = `browser-sync-conflict:${useId()}`;
  const revision = useUiRevision(() => [conflict, open, busy]);
  useUiReading("browser sync conflict summary", { path: conflict.path, open });
  useUiCommands([
    { key: `${parent}:visibility`, command: { label: `Set browser conflict visibility: ${conflict.path}`, parent, subject: `${parent}:visibility`, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: { visibility_requested: args.open } }; } } },
    ...(open ? (["local", "remote"] as const).map(choice => ({ key: `${parent}:${choice}`, command: { label: `Resolve browser conflict ${conflict.path}: ${choice}`, parent, subject: "browser-workspace:operation", parameters: {}, prerequisites: ["Changes the saved local version and queues publication; current unsaved editor text remains separate."], read: () => ({ revision: revision(), enabled: !busy }), execute: async (_args: Record<string, UiValue>, _actor: unknown, signal: AbortSignal) => resolve(conflict, choice, signal) } })) : []),
  ]);
  const reading = { path: conflict.path, original_base: showBytes(conflict.base), local: showBytes(conflict.local?.bytes ?? null), remote: showBytes(conflict.remote?.bytes ?? null), local_revision: conflict.local?.revision ?? null, remote_revision: conflict.remote?.revision ?? null };
  return <details open={open} onToggle={event => setOpen(event.currentTarget.open)}>
    <summary>Resolve {conflict.path}</summary>
    {open && <BrowserConflictReading value={reading} />}
    <p>Choose a saved version to queue for publication. Unsaved editor text stays in its draft.</p>
    <div aria-label={`Sync conflict ${conflict.path}`}>
      <h3>Original base</h3><pre>{reading.original_base}</pre><h3>Local saved version</h3><pre>{reading.local}</pre><h3>Remote version</h3><pre>{reading.remote}</pre>
      <button disabled={busy} onClick={() => { void resolve(conflict, "local"); }}>Keep local saved version</button>
      <button disabled={busy} onClick={() => { void resolve(conflict, "remote"); }}>Use remote version</button>
    </div>
  </details>;
}
function BrowserConflictReading({ value }: { value: unknown }) { useUiReading("disclosed browser sync conflict", value); return null; }
