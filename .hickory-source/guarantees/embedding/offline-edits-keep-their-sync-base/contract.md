<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Offline edits keep their sync base

For: Host developers embedding the document editor without a native execution service.

Given an initialized local-first workspace, saving edits commits the local
files and their pending publication together. Disconnection leaves those files
editable and saved locally. The original remote revisions and base bytes stay
in a durable sync journal until an explicit sync succeeds. A conflict retains
both the local files and the remote files; it never retries as a blind overwrite.
An edit saved during a slow sync remains pending after that older upload finishes.
Local-save and remote-sync status are separate. Sync is explicit and has no
unbounded polling or background retry loop.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/offline-edits-keep-their-sync-base.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-9767e774e44c"></a>
## `apps/web/src/embed/syncedStorage.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Offline edits keep their sync base](contract.md).

<a id="f-9767e774e44c-bytes"></a>
### `bytes`

<storycode:copy id="f-9767e774e44c-bytes">
import { describe, expect, it } from "vitest";
import { createMemoryStorage, exportWorkspace, StorageConflict } from "./storage";
import { openSyncedStorage } from "./syncedStorage";
import type { WorkspaceStorage } from "./storage";
const bytes = (source: string) => new TextEncoder().encode(source);
</storycode:copy>
<a id="f-9767e774e44c-read"></a>
### `read`

<storycode:copy id="f-9767e774e44c-read">
const read = async (storage: WorkspaceStorage) => new TextDecoder().decode((await storage.read("note.md"))!.bytes);
</storycode:copy>
<a id="f-9767e774e44c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9767e774e44c-expression_statement">
// Guarantee: docs/guarantees/embedding/offline-edits-keep-their-sync-base.md
describe("local-first workspace sync", () => {
  it("persists an offline outbox and its original base across reopening", async () => {
    const local = createMemoryStorage(), remote = createMemoryStorage();
    const initial = await remote.write("note.md", bytes("base"), null);
    let offline = false;
    const connection: WorkspaceStorage = { ...remote, async batch(changes) { if (offline) throw new Error("offline"); return remote.batch(changes); } };
    const storage = await openSyncedStorage(local, connection, "one");
    const file = (await storage.read("note.md"))!;
    offline = true;
    await storage.write(file.path, bytes("draft"), file.revision);
    expect(storage.syncState().state).toBe("pending");
    await expect(storage.sync()).rejects.toThrow("offline");
    expect(await read(storage)).toBe("draft"); expect(await read(remote)).toBe("base");
    storage.close();
    const reopened = await openSyncedStorage(local, connection, "one");
    expect(reopened.syncState().state).toBe("pending");
    const metadata = JSON.parse(new TextDecoder().decode((await local.read(".hick-sync/one.json"))!.bytes));
    expect(metadata.base[0]).toEqual({ path: initial.path, revision: initial.revision, base64: btoa("base") });
    offline = false; await reopened.sync();
    expect(await read(remote)).toBe("draft"); expect(reopened.syncState().state).toBe("synced");
    expect((await exportWorkspace(reopened)).files).toEqual([{ path: "note.md", base64: btoa("draft") }]);
  });
  it("keeps conflicting local/remote versions without retrying a blind overwrite", async () => {
    const local = createMemoryStorage(), remote = createMemoryStorage();
    const base = await remote.write("note.md", bytes("base"), null);
    const storage = await openSyncedStorage(local, remote, "one");
    const file = (await storage.read("note.md"))!;
    await storage.write(file.path, bytes("ours"), file.revision);
    await remote.write(base.path, bytes("theirs"), base.revision);
    await expect(storage.sync()).rejects.toBeInstanceOf(StorageConflict);
    expect(storage.syncState().state).toBe("conflict");
    expect(await read(storage)).toBe("ours"); expect(await read(remote)).toBe("theirs");
    const conflict = (await storage.reviewConflicts())[0];
    expect(new TextDecoder().decode(conflict.base!)).toBe("base");
    const resolution = { path: conflict.path, localRevision: conflict.local!.revision, remoteRevision: conflict.remote!.revision, bytes: conflict.local!.bytes };
    const newer = await remote.write(conflict.path, bytes("newer theirs"), conflict.remote!.revision);
    await expect(storage.resolveConflict(resolution)).rejects.toBeInstanceOf(StorageConflict);
    expect(await read(storage)).toBe("ours");
    await storage.resolveConflict({ ...resolution, remoteRevision: newer.revision });
    await storage.sync(); expect(await read(remote)).toBe("ours");
  });
  it("saves during a slow sync and leaves the later revision queued", async () => {
    const local = createMemoryStorage(), remote = createMemoryStorage();
    let release!: () => void, submitted!: () => void;
    const held = new Promise<void>((resolve) => { release = resolve; });
    const ready = new Promise<void>((resolve) => { submitted = resolve; });
    let slow = true;
    const connection: WorkspaceStorage = { ...remote, async batch(changes) { if (slow) { submitted(); await held; } return remote.batch(changes); } };
    const storage = await openSyncedStorage(local, connection, "one");
    const first = await storage.write("note.md", bytes("first"), null);
    const sync = storage.sync(); await ready;
    await storage.write(first.path, bytes("second"), first.revision);
    release(); await sync;
    expect(await read(remote)).toBe("first"); expect(await read(storage)).toBe("second");
    expect(storage.syncState().state).toBe("pending");
    slow = false; await storage.sync(); expect(await read(remote)).toBe("second");
  });
  it("deletion is queued and a local transaction failure publishes no outbox", async () => {
    const local = createMemoryStorage(), remote = createMemoryStorage();
    await remote.write("note.md", bytes("base"), null);
    const storage = await openSyncedStorage(local, remote, "one");
    const file = (await storage.read("note.md"))!;
    await expect(storage.write(file.path, bytes("bad"), "stale")).rejects.toBeInstanceOf(StorageConflict);
    expect(storage.syncState().state).toBe("synced");
    await storage.delete(file.path, file.revision); await storage.sync();
    expect(await remote.list()).toEqual([]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/embed/syncedStorage.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/syncedStorage.test.ts"><storycode:paste select="#f-9767e774e44c-bytes"/><storycode:paste select="#f-9767e774e44c-read"/><storycode:paste select="#f-9767e774e44c-expression_statement"/></storycode:file>

<a id="output-a6bbe299624f"></a>
## `apps/web/src/embed/syncedStorage.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Offline edits keep their sync base](contract.md).

<a id="f-a6bbe299624f-SyncState"></a>
### `SyncState`

<storycode:copy id="f-a6bbe299624f-SyncState">
import { StorageConflict, workspacePath } from "./storage";
import type { StoredFile, StorageMutation, WorkspaceStorage } from "./storage";

export interface SyncState { state: "synced" | "pending" | "syncing" | "conflict" | "disconnected"; reason?: string; }
</storycode:copy>
<a id="f-a6bbe299624f-SyncedStorage"></a>
### `SyncedStorage`

<storycode:copy id="f-a6bbe299624f-SyncedStorage">
export interface SyncedStorage extends WorkspaceStorage {
  sync(): Promise<void>;
  syncState(): SyncState;
  subscribe(listener: () => void): () => void;
  reviewConflicts(): Promise<SyncConflict[]>;
  resolveConflict(input: SyncResolution): Promise<void>;
}
</storycode:copy>
<a id="f-a6bbe299624f-SyncConflict"></a>
### `SyncConflict`

<storycode:copy id="f-a6bbe299624f-SyncConflict">
export interface SyncConflict { path: string; base: Uint8Array | null; local: StoredFile | null; remote: StoredFile | null; }
</storycode:copy>
<a id="f-a6bbe299624f-SyncResolution"></a>
### `SyncResolution`

<storycode:copy id="f-a6bbe299624f-SyncResolution">
export interface SyncResolution { path: string; localRevision: string | null; remoteRevision: string | null; bytes: Uint8Array | null; }
</storycode:copy>
<a id="f-a6bbe299624f-Base"></a>
### `Base`

<storycode:copy id="f-a6bbe299624f-Base">
interface Base { path: string; revision: string; base64: string; }
</storycode:copy>
<a id="f-a6bbe299624f-Pending"></a>
### `Pending`

<storycode:copy id="f-a6bbe299624f-Pending">
interface Pending { path: string; }
</storycode:copy>
<a id="f-a6bbe299624f-Journal"></a>
### `Journal`

<storycode:copy id="f-a6bbe299624f-Journal">
interface Journal { version: 1; base: Base[]; pending: Pending[]; }
</storycode:copy>
<a id="f-a6bbe299624f-toBase64"></a>
### `toBase64`

<storycode:copy id="f-a6bbe299624f-toBase64">
const toBase64 = (bytes: Uint8Array) => {
  let binary = ""; for (const byte of bytes) binary += String.fromCharCode(byte); return btoa(binary);
};
</storycode:copy>
<a id="f-a6bbe299624f-journalBytes"></a>
### `journalBytes`

<storycode:copy id="f-a6bbe299624f-journalBytes">
const journalBytes = (journal: Journal) => new TextEncoder().encode(JSON.stringify(journal));

</storycode:copy>
<a id="f-a6bbe299624f-openSyncedStorage"></a>
### `openSyncedStorage`

Local commits and their outbox share one transaction. Sync is explicit, with no blind retries.

<storycode:copy id="f-a6bbe299624f-openSyncedStorage">
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
</storycode:copy>

### Assemble `apps/web/src/embed/syncedStorage.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/syncedStorage.ts"><storycode:paste select="#f-a6bbe299624f-SyncState"/><storycode:paste select="#f-a6bbe299624f-SyncedStorage"/><storycode:paste select="#f-a6bbe299624f-SyncConflict"/><storycode:paste select="#f-a6bbe299624f-SyncResolution"/><storycode:paste select="#f-a6bbe299624f-Base"/><storycode:paste select="#f-a6bbe299624f-Pending"/><storycode:paste select="#f-a6bbe299624f-Journal"/><storycode:paste select="#f-a6bbe299624f-toBase64"/><storycode:paste select="#f-a6bbe299624f-journalBytes"/><storycode:paste select="#f-a6bbe299624f-openSyncedStorage"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/embed/syncedStorage.ts](contract.md#output-a6bbe299624f)

### Tests and fixtures

- [apps/web/src/embed/syncedStorage.test.ts](contract.md#output-9767e774e44c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
