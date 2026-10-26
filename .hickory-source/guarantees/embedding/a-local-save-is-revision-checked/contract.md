<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A local save is revision checked

For: Host developers embedding the document editor without a native execution service.

Given a file read from browser-local storage with an opaque revision token,
when another tab changes it before a save or delete,
then the stale operation fails and preserves the newer stored bytes and the
editor's unsaved draft. Success is reported only after the IndexedDB transaction
commits. Quota/transaction failures do not mark the draft saved. Memory storage
identifies itself as memory only. Workspace imports either create every file or create none; conflicts never
partly replace a workspace. Workspace exports capture a consistent set
of source, binary assets and evidence without reclassifying their provenance.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/a-local-save-is-revision-checked.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-f43a127a1661"></a>
## `apps/web/src/embed/storage.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A local save is revision checked](contract.md).

<a id="f-f43a127a1661-StoredFile"></a>
### `StoredFile`

Storage is independent of execution and carries opaque revision tokens.

<storycode:copy id="f-f43a127a1661-StoredFile">
/** Storage is independent of execution and carries opaque revision tokens. */
export interface StoredFile { path: string; bytes: Uint8Array; revision: string; }
</storycode:copy>
<a id="f-f43a127a1661-StorageMutation"></a>
### `StorageMutation`

<storycode:copy id="f-f43a127a1661-StorageMutation">
export type StorageMutation =
  | { path: string; bytes: Uint8Array; expected: string | null }
  | { path: string; delete: true; expected: string };
</storycode:copy>
<a id="f-f43a127a1661-WorkspaceStorage"></a>
### `WorkspaceStorage`

<storycode:copy id="f-f43a127a1661-WorkspaceStorage">
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
</storycode:copy>
<a id="f-f43a127a1661-StorageConflict"></a>
### `StorageConflict`

<storycode:copy id="f-f43a127a1661-StorageConflict">
export class StorageConflict extends Error {
  constructor(readonly path: string) { super(`${path} changed since it was read; retain the draft and reload before saving`); }
}
</storycode:copy>
<a id="f-f43a127a1661-workspacePath"></a>
### `workspacePath`

<storycode:copy id="f-f43a127a1661-workspacePath">
export function workspacePath(path: string): string {
  if (!path || path.startsWith("/") || path.includes("\\") || path.split("/").some((part) => !part || part === "." || part === "..") || /[\u0000-\u001f]/.test(path)) throw new Error(`Invalid workspace path: ${path}`);
  return path;
}
</storycode:copy>
<a id="f-f43a127a1661-createMemoryStorage"></a>
### `createMemoryStorage`

<storycode:copy id="f-f43a127a1661-createMemoryStorage">
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

</storycode:copy>
<a id="f-f43a127a1661-openLocalStorage"></a>
### `openLocalStorage`

Transactional IndexedDB compare-and-write, including concurrent tabs.

<storycode:copy id="f-f43a127a1661-openLocalStorage">
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

</storycode:copy>
<a id="f-f43a127a1661-validateChanges"></a>
### `validateChanges`

<storycode:copy id="f-f43a127a1661-validateChanges">
function validateChanges(changes: StorageMutation[], files: Map<string, StoredFile>) {
  const seen = new Set<string>();
  for (const change of changes) {
    workspacePath(change.path);
    if (seen.has(change.path)) throw new Error(`Duplicate mutation: ${change.path}`);
    seen.add(change.path);
    if ((files.get(change.path)?.revision ?? null) !== change.expected) throw new StorageConflict(change.path);
  }
}

</storycode:copy>
<a id="f-f43a127a1661-WorkspaceExport"></a>
### `WorkspaceExport`

Lossless transfer format: source, assets and evidence are all ordinary files.

<storycode:copy id="f-f43a127a1661-WorkspaceExport">
/** Lossless transfer format: source, assets and evidence are all ordinary files. */
export interface WorkspaceExport { version: 1; files: { path: string; base64: string }[]; }
</storycode:copy>
<a id="f-f43a127a1661-exportWorkspace"></a>
### `exportWorkspace`

<storycode:copy id="f-f43a127a1661-exportWorkspace">
export async function exportWorkspace(storage: WorkspaceStorage): Promise<WorkspaceExport> {
  const files: WorkspaceExport["files"] = [];
  for (const file of await storage.snapshot()) {
    let binary = "";
    for (const byte of file.bytes) binary += String.fromCharCode(byte);
    files.push({ path: file.path, base64: btoa(binary) });
  }
  return { version: 1, files };
}
</storycode:copy>
<a id="f-f43a127a1661-decodeWorkspaceExport"></a>
### `decodeWorkspaceExport`

<storycode:copy id="f-f43a127a1661-decodeWorkspaceExport">
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

</storycode:copy>
<a id="f-f43a127a1661-importWorkspace"></a>
### `importWorkspace`

Imports only into absent paths. A conflicting path rejects the entire batch.

<storycode:copy id="f-f43a127a1661-importWorkspace">
/** Imports only into absent paths. A conflicting path rejects the entire batch. */
export async function importWorkspace(storage: WorkspaceStorage, value: unknown): Promise<StoredFile[]> {
  return storage.batch(decodeWorkspaceExport(value).map((file) => ({ ...file, expected: null })));
}

</storycode:copy>
<a id="f-f43a127a1661-SearchMatch"></a>
### `SearchMatch`

<storycode:copy id="f-f43a127a1661-SearchMatch">
export interface SearchMatch { path: string; line: number; text: string; }
</storycode:copy>
<a id="f-f43a127a1661-searchWorkspace"></a>
### `searchWorkspace`

Literal text search of saved .md bytes in one snapshot; no semantic service.

<storycode:copy id="f-f43a127a1661-searchWorkspace">
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
</storycode:copy>

### Assemble `apps/web/src/embed/storage.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/storage.ts"><storycode:paste select="#f-f43a127a1661-StoredFile"/><storycode:paste select="#f-f43a127a1661-StorageMutation"/><storycode:paste select="#f-f43a127a1661-WorkspaceStorage"/><storycode:paste select="#f-f43a127a1661-StorageConflict"/><storycode:paste select="#f-f43a127a1661-workspacePath"/><storycode:paste select="#f-f43a127a1661-createMemoryStorage"/><storycode:paste select="#f-f43a127a1661-openLocalStorage"/><storycode:paste select="#f-f43a127a1661-validateChanges"/><storycode:paste select="#f-f43a127a1661-WorkspaceExport"/><storycode:paste select="#f-f43a127a1661-exportWorkspace"/><storycode:paste select="#f-f43a127a1661-decodeWorkspaceExport"/><storycode:paste select="#f-f43a127a1661-importWorkspace"/><storycode:paste select="#f-f43a127a1661-SearchMatch"/><storycode:paste select="#f-f43a127a1661-searchWorkspace"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/embed/BrowserWorkspace.tsx](../../../mechanisms/apps/web/src/embed/BrowserWorkspace.tsx.md#output-14375bea23a0)
- [apps/web/src/embed/storage.ts](contract.md#output-f43a127a1661)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/embed/DocumentEmbed.test.tsx](../../../mechanisms/apps/web/src/embed/DocumentEmbed.test.tsx.md#output-bb54f1265d97)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
