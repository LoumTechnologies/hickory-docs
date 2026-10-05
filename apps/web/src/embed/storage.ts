/** Storage is independent of execution and carries opaque revision tokens. */
export interface StoredFile { path: string; bytes: Uint8Array; revision: string; }
export type StorageMutation =
  | { path: string; bytes: Uint8Array; expected: string | null }
  | { path: string; delete: true; expected: string };
export interface WorkspaceStorage {
  readonly durability: "memory" | "local" | "remote";
  read(path: string): Promise<StoredFile | null>;
  list(): Promise<string[]>;
  /** One consistent revision of source, assets and evidence for export/render. */
  snapshot(): Promise<StoredFile[]>;
  /** All revision checks and mutations commit together, or none commit. */
  batch(changes: StorageMutation[]): Promise<StoredFile[]>;
  /** null means create-if-absent. Never blind overwrite. */
  write(path: string, bytes: Uint8Array, expected: string | null): Promise<StoredFile>;
  delete(path: string, expected: string): Promise<void>;
  close(): void;
}
export class StorageConflict extends Error {
  constructor(readonly path: string) { super(`${path} changed since it was read; retain the draft and reload before saving`); }
}
export function workspacePath(path: string): string {
  if (!path || path.startsWith("/") || path.includes("\\") || path.split("/").some((part) => !part || part === "." || part === "..") || /[\u0000-\u001f]/.test(path)) throw new Error(`Invalid workspace path: ${path}`);
  return path;
}
export function createMemoryStorage(): WorkspaceStorage {
  const files = new Map<string, StoredFile>();
  let closed = false;
  function check(path?: string) { if (closed) throw new Error("Workspace storage is closed"); if (path !== undefined) workspacePath(path); }
  const copy = (file: StoredFile): StoredFile => ({ ...file, bytes: file.bytes.slice() });
  return {
    durability: "memory",
    async read(path) { check(path); const file = files.get(path); return file ? copy(file) : null; },
    async list() { check(); return [...files.keys()].sort(); },
    async snapshot() { check(); return [...files.values()].map(copy); },
    async batch(changes) {
      check(); validateChanges(changes, files);
      const written = changes.flatMap((change) => "delete" in change ? [] : [{ path: change.path, bytes: change.bytes.slice(), revision: crypto.randomUUID() }]);
      for (const change of changes) {
        if ("delete" in change) files.delete(change.path);
      }
      for (const file of written) files.set(file.path, file);
      return written.map(copy);
    },
    async write(path, bytes, expected) {
      check(path); if ((files.get(path)?.revision ?? null) !== expected) throw new StorageConflict(path);
      const file = { path, bytes: bytes.slice(), revision: crypto.randomUUID() }; files.set(path, file); return copy(file);
    },
    async delete(path, expected) { check(path); if (files.get(path)?.revision !== expected) throw new StorageConflict(path); files.delete(path); },
    close() { closed = true; },
  };
}

/** Transactional IndexedDB compare-and-write, including concurrent tabs. */
export async function openLocalStorage(name: string): Promise<WorkspaceStorage> {
  const db = await new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open(`hickory-embed:${name}`, 1);
    request.onupgradeneeded = () => request.result.createObjectStore("files", { keyPath: "path" });
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
    request.onblocked = () => reject(new Error("Close other workspace tabs to upgrade local storage"));
  });
  db.onversionchange = () => db.close();
  function transaction<T>(mode: IDBTransactionMode, action: (store: IDBObjectStore, done: (result: T) => void, fail: (error: unknown) => void) => void): Promise<T> {
    return new Promise((resolve, reject) => {
      const tx = db.transaction("files", mode);
      let result: T;
      let failure: unknown;
      tx.oncomplete = () => resolve(result);
      tx.onabort = () => reject(failure ?? tx.error ?? new Error("Local save aborted"));
      tx.onerror = () => { failure ??= tx.error; };
      action(tx.objectStore("files"), (value) => { result = value; }, (error) => { failure = error; tx.abort(); });
    });
  }
  return {
    durability: "local",
    read(path) { workspacePath(path); return transaction("readonly", (store, done) => { const req = store.get(path); req.onsuccess = () => done(req.result ?? null); }); },
    list() { return transaction("readonly", (store, done) => { const req = store.getAllKeys(); req.onsuccess = () => done(req.result.map(String).sort()); }); },
    snapshot() { return transaction("readonly", (store, done) => { const req = store.getAll(); req.onsuccess = () => done(req.result); }); },
    batch(changes) {
      const frozen = changes.map((change) => "delete" in change ? { ...change } : { ...change, bytes: change.bytes.slice() });
      return transaction("readwrite", (store, done, fail) => {
        const req = store.getAll();
        req.onsuccess = () => {
          try {
            validateChanges(frozen, new Map((req.result as StoredFile[]).map((file) => [file.path, file])));
            const written: StoredFile[] = [];
            for (const change of frozen) {
              if ("delete" in change) store.delete(change.path);
              else { const file = { path: change.path, bytes: change.bytes.slice(), revision: crypto.randomUUID() }; store.put(file); written.push(file); }
            }
            done(written);
          } catch (error) { fail(error); }
        };
      });
    },
    write(path, bytes, expected) {
      workspacePath(path);
      const frozen = bytes.slice();
      return transaction("readwrite", (store, done, fail) => {
        const req = store.get(path);
        req.onsuccess = () => {
          if ((req.result?.revision ?? null) !== expected) { fail(new StorageConflict(path)); return; }
          try {
            const file = { path, bytes: frozen, revision: crypto.randomUUID() };
            store.put(file); done(file);
          } catch (error) { fail(error); }
        };
      });
    },
    delete(path, expected) {
      workspacePath(path);
      return transaction("readwrite", (store, done, fail) => {
        const req = store.get(path);
        req.onsuccess = () => { if (req.result?.revision !== expected) { fail(new StorageConflict(path)); return; } store.delete(path); done(undefined); };
      });
    },
    close() { db.close(); },
  };
}

function validateChanges(changes: StorageMutation[], files: Map<string, StoredFile>) {
  const seen = new Set<string>();
  for (const change of changes) {
    workspacePath(change.path);
    if (seen.has(change.path)) throw new Error(`Duplicate mutation: ${change.path}`);
    seen.add(change.path);
    if ((files.get(change.path)?.revision ?? null) !== change.expected) throw new StorageConflict(change.path);
  }
}

/** Lossless transfer format: source, assets and evidence are all ordinary files. */
export interface WorkspaceExport { version: 1; files: { path: string; base64: string }[]; }
export async function exportWorkspace(storage: WorkspaceStorage): Promise<WorkspaceExport> {
  const files: WorkspaceExport["files"] = [];
  for (const file of await storage.snapshot()) {
    let binary = "";
    for (const byte of file.bytes) binary += String.fromCharCode(byte);
    files.push({ path: file.path, base64: btoa(binary) });
  }
  return { version: 1, files };
}
export function decodeWorkspaceExport(value: unknown): { path: string; bytes: Uint8Array }[] {
  if (!value || typeof value !== "object" || !("version" in value) || value.version !== 1 || !("files" in value) || !Array.isArray(value.files)) throw new Error("Unsupported workspace export");
  const seen = new Set<string>();
  return value.files.map((file: unknown) => {
    if (!file || typeof file !== "object" || !("path" in file) || typeof file.path !== "string" || !("base64" in file) || typeof file.base64 !== "string") throw new Error("Invalid exported file");
    workspacePath(file.path);
    if (seen.has(file.path)) throw new Error(`Duplicate exported path: ${file.path}`); seen.add(file.path);
    return { path: file.path, bytes: Uint8Array.from(atob(file.base64), (c) => c.charCodeAt(0)) };
  });
}

/** Imports only into absent paths. A conflicting path rejects the entire batch. */
export async function importWorkspace(storage: WorkspaceStorage, value: unknown): Promise<StoredFile[]> {
  return storage.batch(decodeWorkspaceExport(value).map((file) => ({ ...file, expected: null })));
}

export interface SearchMatch { path: string; line: number; text: string; }
/** Literal text search of saved .md bytes in one snapshot; no semantic service. */
export async function searchWorkspace(storage: WorkspaceStorage, query: string, limit = 100): Promise<SearchMatch[]> {
  if (!query.trim()) return [];
  const needle = query.toLocaleLowerCase();
  const matches: SearchMatch[] = [];
  for (const file of (await storage.snapshot()).sort((a, b) => a.path.localeCompare(b.path))) {
    if (!file.path.endsWith(".md")) continue;
    let text: string;
    try { text = new TextDecoder("utf-8", { fatal: true }).decode(file.bytes); } catch { continue; }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].toLocaleLowerCase().includes(needle)) matches.push({ path: file.path, line: i + 1, text: lines[i] });
      if (matches.length >= limit) return matches;
    }
  }
  return matches;
}
