import { StorageConflict, workspacePath } from "./storage";
import type { StoredFile, StorageMutation, WorkspaceStorage } from "./storage";

export interface SyncState { state: "synced" | "pending" | "syncing" | "conflict" | "disconnected"; reason?: string; }
export interface SyncedStorage extends WorkspaceStorage {
  sync(): Promise<void>;
  syncState(): SyncState;
  subscribe(listener: () => void): () => void;
  reviewConflicts(): Promise<SyncConflict[]>;
  resolveConflict(input: SyncResolution): Promise<void>;
}
export interface SyncConflict { path: string; base: Uint8Array | null; local: StoredFile | null; remote: StoredFile | null; }
export interface SyncResolution { path: string; localRevision: string | null; remoteRevision: string | null; bytes: Uint8Array | null; }
interface Base { path: string; revision: string; base64: string; }
interface Pending { path: string; }
interface Journal { version: 1; base: Base[]; pending: Pending[]; }
const toBase64 = (bytes: Uint8Array) => {
  let binary = ""; for (const byte of bytes) binary += String.fromCharCode(byte); return btoa(binary);
};
const journalBytes = (journal: Journal) => new TextEncoder().encode(JSON.stringify(journal));

/** Local commits and their outbox share one transaction. Sync is explicit, with no blind retries. */
export async function openSyncedStorage(local: WorkspaceStorage, remote: WorkspaceStorage, name: string): Promise<SyncedStorage> {
  if (local.durability === "remote") throw new Error("The offline outbox needs local or memory storage");
  const metadataPath = `.hick-sync/${workspacePath(name)}.json`;
  const listeners = new Set<() => void>();
  let state: SyncState = { state: "synced" }, closed = false, syncing: Promise<void> | null = null;
  const notify = (next: SyncState) => { state = next; for (const listener of listeners) listener(); };
  const check = (path?: string) => {
    if (closed) throw new Error("Synced workspace is closed");
    if (path !== undefined) { workspacePath(path); if (path.startsWith(".hick-sync/")) throw new Error("The sync journal is reserved workspace metadata"); }
  };
  async function journal() {
    check();
    const file = await local.read(metadataPath);
    if (!file) throw new Error("Sync journal is missing; do not overwrite remote files");
    const value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(file.bytes)) as Journal;
    if (value.version !== 1 || !Array.isArray(value.base) || !Array.isArray(value.pending)) throw new Error("Unsupported sync journal");
    const baseSeen = new Set<string>(), pendingSeen = new Set<string>();
    for (const base of value.base) {
      check(base.path);
      if (baseSeen.has(base.path) || typeof base.revision !== "string" || typeof base.base64 !== "string") throw new Error("Invalid sync base");
      baseSeen.add(base.path); atob(base.base64);
    }
    for (const pending of value.pending) {
      check(pending.path);
      if (pendingSeen.has(pending.path)) throw new Error("Invalid pending revision");
      pendingSeen.add(pending.path);
    }
    return { file, value };
  }
  // First connection hydrates an empty local workspace. Existing local drafts are
  // queued against the remote base and kept instead of being silently replaced.
  if (!await local.read(metadataPath)) {
    const published = await remote.snapshot();
    const existing = await local.snapshot();
    const existingMap = new Map(existing.map((file) => [file.path, file]));
    const value: Journal = { version: 1, base: published.map((file) => ({ path: file.path, revision: file.revision, base64: toBase64(file.bytes) })), pending: existing.map((file) => ({ path: file.path })) };
    for (const file of existing) check(file.path);
    const changes: StorageMutation[] = published.filter((file) => !existingMap.has(file.path)).map((file) => ({ path: file.path, bytes: file.bytes, expected: null }));
    for (const file of published) check(file.path);
    await local.batch([...changes, { path: metadataPath, bytes: journalBytes(value), expected: null }]);
  }
  if ((await journal()).value.pending.length) state = { state: "pending" };
  let sequence: Promise<unknown> = Promise.resolve();
  const serial = <T,>(action: () => Promise<T>): Promise<T> => {
    const operation = sequence.then(action); sequence = operation.catch(() => undefined); return operation;
  };
  function batch(changes: StorageMutation[]): Promise<StoredFile[]> {
    const frozen = changes.map((change) => "delete" in change ? { ...change } : { ...change, bytes: change.bytes.slice() });
    return serial(async () => {
      for (const change of frozen) check(change.path);
      const current = await journal();
      // Generate exact file revisions inside local.batch; the persisted outbox
      // tracks bytes by path, and sync freezes those revisions in one snapshot.
      const pending = new Map(current.value.pending.map((entry) => [entry.path, entry]));
      for (const change of frozen) pending.set(change.path, { path: change.path });
      const value = { ...current.value, pending: [...pending.values()] };
      const written = await local.batch([...frozen, { path: metadataPath, bytes: journalBytes(value), expected: current.file.revision }]);
      notify({ state: syncing ? "syncing" : "pending" });
      return written.filter((file) => file.path !== metadataPath);
    });
  }
  async function flush() {
    notify({ state: "syncing" });
    // The journal and file bytes MUST come from the same atomic snapshot.
    const frozen = await serial(async () => {
      check();
      const snapshot = await local.snapshot(), files = new Map(snapshot.map((file) => [file.path, file]));
      const metadata = files.get(metadataPath);
      if (!metadata) throw new Error("Sync journal is missing");
      const value = JSON.parse(new TextDecoder().decode(metadata.bytes)) as Journal;
      const base = new Map(value.base.map((entry) => [entry.path, entry]));
      const revisions = new Map<string, string | null>();
      const changes: StorageMutation[] = value.pending.map((entry) => {
        const file = files.get(entry.path), expected = base.get(entry.path)?.revision ?? null;
        revisions.set(entry.path, file?.revision ?? null);
        if (file) return { path: entry.path, bytes: file.bytes, expected };
        return { path: entry.path, delete: true as const, expected: expected ?? "absent" };
      }).filter((change) => !("delete" in change && change.expected === "absent"));
      return { changes, revisions };
    });
    const accepted = await remote.batch(frozen.changes);
    await serial(async () => {
      const current = await journal();
      const files = new Map((await local.snapshot()).map((file) => [file.path, file]));
      const base = new Map(current.value.base.map((entry) => [entry.path, entry]));
      for (const change of frozen.changes) {
        if ("delete" in change) base.delete(change.path);
        else {
          const file = accepted.find((entry) => entry.path === change.path);
          if (!file) throw new Error("Remote did not acknowledge an uploaded file");
          base.set(file.path, { path: file.path, revision: file.revision, base64: toBase64(file.bytes) });
        }
      }
      const pending = current.value.pending.filter((entry) => !frozen.revisions.has(entry.path) || (files.get(entry.path)?.revision ?? null) !== frozen.revisions.get(entry.path));
      await local.write(metadataPath, journalBytes({ version: 1, base: [...base.values()], pending }), current.file.revision);
      notify({ state: pending.length ? "pending" : "synced" });
    });
  }
  return {
    durability: local.durability,
    async read(path) { check(path); return local.read(path); },
    async list() { check(); return (await local.list()).filter((path) => !path.startsWith(".hick-sync/")); },
    async snapshot() { check(); return (await local.snapshot()).filter((file) => !file.path.startsWith(".hick-sync/")); },
    batch,
    async write(path, bytes, expected) { return (await batch([{ path, bytes, expected }]))[0]; },
    async delete(path, expected) { await batch([{ path, delete: true, expected }]); },
    sync() {
      check();
      if (!syncing) syncing = flush().catch((error) => {
        notify({ state: error instanceof StorageConflict ? "conflict" : "disconnected", reason: String(error) }); throw error;
      }).finally(() => { syncing = null; });
      return syncing;
    },
    syncState() { return state; },
    subscribe(listener) { listeners.add(listener); return () => listeners.delete(listener); },
    async reviewConflicts() {
      check();
      const snapshot = await local.snapshot(), files = new Map(snapshot.map((file) => [file.path, file]));
      const metadata = files.get(metadataPath);
      if (!metadata) throw new Error("Sync journal is missing");
      const value = JSON.parse(new TextDecoder().decode(metadata.bytes)) as Journal;
      const base = new Map(value.base.map((entry) => [entry.path, entry]));
      const published = new Map((await remote.snapshot()).map((file) => [file.path, file]));
      return value.pending.filter((entry) => (base.get(entry.path)?.revision ?? null) !== (published.get(entry.path)?.revision ?? null)).map((entry) => {
        const original = base.get(entry.path);
        return { path: entry.path, base: original ? Uint8Array.from(atob(original.base64), (char) => char.charCodeAt(0)) : null,
          local: files.get(entry.path) ?? null, remote: published.get(entry.path) ?? null };
      });
    },
    async resolveConflict(input) {
      check(input.path);
      const frozen = { ...input, bytes: input.bytes?.slice() ?? null };
      const published = await remote.read(frozen.path);
      if ((published?.revision ?? null) !== frozen.remoteRevision) throw new StorageConflict(frozen.path);
      await serial(async () => {
        const current = await journal(), file = await local.read(frozen.path);
        if ((file?.revision ?? null) !== frozen.localRevision) throw new StorageConflict(frozen.path);
        const base = current.value.base.filter((entry) => entry.path !== frozen.path);
        if (published) base.push({ path: published.path, revision: published.revision, base64: toBase64(published.bytes) });
        const pending = current.value.pending.filter((entry) => entry.path !== frozen.path); pending.push({ path: frozen.path });
        const changes: StorageMutation[] = frozen.bytes ? [{ path: frozen.path, bytes: frozen.bytes, expected: frozen.localRevision }]
          : file ? [{ path: file.path, delete: true, expected: file.revision }] : [];
        await local.batch([...changes, { path: metadataPath, bytes: journalBytes({ version: 1, base, pending }), expected: current.file.revision }]);
        notify({ state: "pending" });
      });
    },
    close() { closed = true; listeners.clear(); },
  };
}
