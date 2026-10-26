<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# S3 publishes one manifest

For: Host developers embedding the document editor without a native execution service.

Given an S3-compatible host callback that passes the adapter's conditional-write
and visible-ETag probe, a workspace batch uploads immutable content-addressed
objects and a manifest before conditionally updating its one head object.
Readers follow a single published manifest. A failed upload does not publish
part of a batch; a stale file revision or head cannot silently overwrite another
session's publication. Object reads verify content hashes; ETags remain opaque
concurrency tokens. No object revision claims git history or verified execution.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/s3-publishes-one-manifest.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-81a03dd838e9"></a>
## `apps/web/src/embed/s3Storage.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [S3 publishes one manifest](contract.md).

<a id="f-81a03dd838e9-bucket"></a>
### `bucket`

<storycode:copy id="f-81a03dd838e9-bucket">
import { describe, expect, it } from "vitest";
import { openS3Storage, S3Failure } from "./s3Storage";
import type { S3Request } from "./s3Storage";
import { exportWorkspace, StorageConflict } from "./storage";

// Guarantee: docs/guarantees/embedding/s3-publishes-one-manifest.md
function bucket() {
  const objects = new Map<string, { bytes: Uint8Array; etag: string }>();
  let counter = 0, expired = false, failedObject = false, ignoreConditions = false;
  const request = async (input: S3Request): Promise<Response> => {
    if (input.signal.aborted) throw new DOMException("Closed", "AbortError");
    if (expired) return new Response(null, { status: 403 });
    const stored = objects.get(input.key);
    if (input.method === "GET") return stored ? new Response(stored.bytes.slice().buffer, { headers: { ETag: stored.etag } }) : new Response(null, { status: 404 });
    if (failedObject && input.key.includes("/objects/")) return new Response(null, { status: 503 });
    if (!ignoreConditions && (input.headers["If-None-Match"] === "*" && stored || input.headers["If-Match"] && input.headers["If-Match"] !== stored?.etag)) return new Response(null, { status: 412 });
    const etag = `"opaque-${++counter}"`; objects.set(input.key, { bytes: input.body!.slice(), etag });
    return new Response(null, { headers: { ETag: etag } });
  };
  return { objects, request, expire(value: boolean) { expired = value; }, fail(value: boolean) { failedObject = value; }, ignore() { ignoreConditions = true; } };
}
</storycode:copy>
<a id="f-81a03dd838e9-text"></a>
### `text`

<storycode:copy id="f-81a03dd838e9-text">
const text = (value: string) => new TextEncoder().encode(value);
</storycode:copy>
<a id="f-81a03dd838e9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-81a03dd838e9-expression_statement">
describe("S3 manifest publication", () => {
  it("refuses a provider that ignores conditional writes", async () => {
    const backend = bucket(); backend.ignore();
    await expect(openS3Storage({ prefix: "workspace", request: backend.request })).rejects.toThrow("does not enforce If-None-Match");
    expect(backend.objects.has("workspace/head.json")).toBe(false);
  });
  it("conflicting sessions retain the published bytes and support atomic rename/delete", async () => {
    const backend = bucket();
    const a = await openS3Storage({ prefix: "workspace", request: backend.request });
    const b = await openS3Storage({ prefix: "workspace", request: backend.request });
    const first = await a.write("note.md", text("base"), null);
    const read = await b.read("note.md");
    await a.write("note.md", text("newer"), first.revision);
    await expect(b.write("note.md", text("draft"), read!.revision)).rejects.toBeInstanceOf(StorageConflict);
    expect(new TextDecoder().decode((await b.read("note.md"))!.bytes)).toBe("newer");
    const current = (await a.read("note.md"))!;
    await a.batch([{ path: current.path, delete: true, expected: current.revision }, { path: "renamed.md", bytes: current.bytes, expected: null }]);
    expect(await b.list()).toEqual(["renamed.md"]);
    await a.write("image.bin", new Uint8Array([0, 255]), null);
    expect((await exportWorkspace(b)).files).toEqual(expect.arrayContaining([{ path: "image.bin", base64: "AP8=" }]));
  });
  it("interrupted uploads and expired credentials never publish a partial workspace", async () => {
    const backend = bucket(), storage = await openS3Storage({ prefix: "workspace", request: backend.request });
    await storage.write("base.md", text("base"), null);
    const before = backend.objects.get("workspace/head.json")!.etag;
    backend.fail(true);
    await expect(storage.batch([{ path: "one.md", bytes: text("one"), expected: null }, { path: "two.md", bytes: text("two"), expected: null }])).rejects.toBeInstanceOf(S3Failure);
    expect(backend.objects.get("workspace/head.json")!.etag).toBe(before);
    backend.fail(false); backend.expire(true);
    await expect(storage.write("one.md", text("draft"), null)).rejects.toMatchObject({ status: 403 });
    backend.expire(false);
    await storage.write("one.md", text("draft"), null);
    expect(await storage.list()).toEqual(["base.md", "one.md"]);
  });
  it("checks immutable bytes and refuses a racing head update", async () => {
    const backend = bucket();
    const a = await openS3Storage({ prefix: "workspace", request: backend.request });
    const b = await openS3Storage({ prefix: "workspace", request: backend.request });
    const results = await Promise.allSettled([a.write("a.md", text("a"), null), b.write("b.md", text("b"), null)]);
    expect(results.filter((result) => result.status === "fulfilled")).toHaveLength(1);
    expect(await a.list()).toHaveLength(1);
    const published = new TextDecoder().decode((await a.snapshot())[0].bytes);
    const object = [...backend.objects.values()].find((entry) => new TextDecoder().decode(entry.bytes) === published)!;
    object.bytes = text("corrupt");
    await expect(a.snapshot()).rejects.toThrow("content hash check");
  });
});
</storycode:copy>

### Assemble `apps/web/src/embed/s3Storage.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/s3Storage.test.ts"><storycode:paste select="#f-81a03dd838e9-bucket"/><storycode:paste select="#f-81a03dd838e9-text"/><storycode:paste select="#f-81a03dd838e9-expression_statement"/></storycode:file>

<a id="output-d9413be077e5"></a>
## `apps/web/src/embed/s3Storage.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [S3 publishes one manifest](contract.md).

<a id="f-d9413be077e5-S3Request"></a>
### `S3Request`

<storycode:copy id="f-d9413be077e5-S3Request">
import { StorageConflict, workspacePath } from "./storage";
import type { StoredFile, StorageMutation, WorkspaceStorage } from "./storage";

/** The host signs every request, including conditional headers, with current credentials. */
export interface S3Request {
  method: "GET" | "PUT";
  key: string;
  headers: Record<string, string>;
  body?: Uint8Array;
  signal: AbortSignal;
}
</storycode:copy>
<a id="f-d9413be077e5-S3Options"></a>
### `S3Options`

<storycode:copy id="f-d9413be077e5-S3Options">
export interface S3Options {
  /** Isolated workspace prefix inside a bucket; no bucket listing is required. */
  prefix: string;
  request(input: S3Request): Promise<Response>;
}
</storycode:copy>
<a id="f-d9413be077e5-Entry"></a>
### `Entry`

<storycode:copy id="f-d9413be077e5-Entry">
interface Entry { path: string; hash: string; revision: string; }
</storycode:copy>
<a id="f-d9413be077e5-Manifest"></a>
### `Manifest`

<storycode:copy id="f-d9413be077e5-Manifest">
interface Manifest { version: 1; files: Entry[]; }
</storycode:copy>
<a id="f-d9413be077e5-Head"></a>
### `Head`

<storycode:copy id="f-d9413be077e5-Head">
interface Head { etag: string | null; manifest: Manifest; }
</storycode:copy>
<a id="f-d9413be077e5-S3Failure"></a>
### `S3Failure`

<storycode:copy id="f-d9413be077e5-S3Failure">
export class S3Failure extends Error {
  constructor(readonly status: number, readonly operation: string) {
    super(status === 401 || status === 403 ? "S3 credentials expired or access was denied; renew access before syncing" : `S3 ${operation} failed (${status})`);
  }
}
</storycode:copy>
<a id="f-d9413be077e5-hashPattern"></a>
### `hashPattern`

<storycode:copy id="f-d9413be077e5-hashPattern">
const hashPattern = /^[a-f0-9]{64}$/;
</storycode:copy>
<a id="f-d9413be077e5-digest"></a>
### `digest`

<storycode:copy id="f-d9413be077e5-digest">
async function digest(bytes: Uint8Array): Promise<string> {
  const hash = await crypto.subtle.digest("SHA-256", bytes.slice().buffer);
  return [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}
</storycode:copy>
<a id="f-d9413be077e5-encode"></a>
### `encode`

<storycode:copy id="f-d9413be077e5-encode">
const encode = (value: unknown) => new TextEncoder().encode(JSON.stringify(value));

</storycode:copy>
<a id="f-d9413be077e5-openS3Storage"></a>
### `openS3Storage`

A real conditional-write probe is mandatory; no provider is assumed compatible.

<storycode:copy id="f-d9413be077e5-openS3Storage">
/** A real conditional-write probe is mandatory; no provider is assumed compatible. */
export async function openS3Storage(options: S3Options): Promise<WorkspaceStorage> {
  workspacePath(options.prefix);
  const controller = new AbortController();
  const request = (method: S3Request["method"], key: string, body?: Uint8Array, headers: Record<string, string> = {}) => {
    if (controller.signal.aborted) throw new Error("S3 workspace is closed");
    return options.request({ method, key: `${options.prefix}/${key}`, body, headers, signal: controller.signal });
  };
  const checked = async (response: Response, operation: string) => {
    if (!response.ok) throw new S3Failure(response.status, operation);
    return response;
  };
  function etag(response: Response) {
    const value = response.headers.get("ETag");
    if (!value) throw new Error("S3 must return ETag and expose it through CORS");
    return value;
  }
  try {
    const probe = `probes/${crypto.randomUUID()}`;
    const first = await checked(await request("PUT", probe, encode("first"), { "If-None-Match": "*" }), "conditional probe");
    const token = etag(first);
    const occupied = await request("PUT", probe, encode("must not publish"), { "If-None-Match": "*" });
    if (occupied.status !== 412) throw new Error("S3 does not enforce If-None-Match; publication is unavailable");
    const updated = await checked(await request("PUT", probe, encode("second"), { "If-Match": token }), "conditional probe");
    if (etag(updated) === token) throw new Error("S3 ETag must change when the object changes");
    const stale = await request("PUT", probe, encode("must not publish"), { "If-Match": token });
    if (stale.status !== 412) throw new Error("S3 does not enforce If-Match; publication is unavailable");
    const content = await checked(await request("GET", probe), "probe read");
    if (await content.text() !== JSON.stringify("second")) throw new Error("S3 probe read did not match its conditional publication");
  } catch (error) { controller.abort(); throw error; }

  async function object(hash: string): Promise<Uint8Array> {
    if (!hashPattern.test(hash)) throw new Error("Invalid S3 object hash");
    const response = await checked(await request("GET", `objects/${hash}`), "read object");
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (await digest(bytes) !== hash) throw new Error("S3 immutable object failed its content hash check");
    return bytes;
  }
  async function head(): Promise<Head> {
    const response = await request("GET", "head.json");
    if (response.status === 404) return { etag: null, manifest: { version: 1, files: [] } };
    await checked(response, "read head");
    const token = etag(response);
    const value: unknown = await response.json();
    if (!value || typeof value !== "object" || !("version" in value) || value.version !== 1 || !("manifest" in value) || typeof value.manifest !== "string") throw new Error("Unsupported S3 head");
    const manifest: unknown = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(await object(value.manifest)));
    if (!manifest || typeof manifest !== "object" || !("version" in manifest) || manifest.version !== 1 || !("files" in manifest) || !Array.isArray(manifest.files)) throw new Error("Unsupported S3 manifest");
    const seen = new Set<string>();
    for (const entry of manifest.files) {
      if (!entry || typeof entry.path !== "string" || typeof entry.hash !== "string" || !hashPattern.test(entry.hash) || typeof entry.revision !== "string" || !entry.revision) throw new Error("Invalid S3 manifest entry");
      workspacePath(entry.path);
      if (seen.has(entry.path)) throw new Error("Duplicate S3 manifest path");
      seen.add(entry.path);
    }
    return { etag: token, manifest: manifest as Manifest };
  }
  async function putObject(bytes: Uint8Array): Promise<string> {
    const hash = await digest(bytes);
    const response = await request("PUT", `objects/${hash}`, bytes, { "If-None-Match": "*" });
    if (response.status === 412) {
      // A preexisting object still has to contain the bytes its name claims.
      await object(hash);
    } else await checked(response, "upload object");
    return hash;
  }
  let sequence: Promise<unknown> = Promise.resolve();
  function batch(changes: StorageMutation[]): Promise<StoredFile[]> {
    // Copy at submission, before any await; host edits cannot mutate an upload.
    const frozen = changes.map((change) => "delete" in change ? { ...change } : { ...change, bytes: change.bytes.slice() });
    const operation = sequence.then(async () => {
      const base = await head();
      const entries = new Map(base.manifest.files.map((entry) => [entry.path, entry]));
      const seen = new Set<string>();
      for (const change of frozen) {
        workspacePath(change.path);
        if (seen.has(change.path)) throw new Error(`Duplicate mutation: ${change.path}`);
        seen.add(change.path);
        if ((entries.get(change.path)?.revision ?? null) !== change.expected) throw new StorageConflict(change.path);
      }
      const written: StoredFile[] = [];
      for (const change of frozen) {
        if ("delete" in change) entries.delete(change.path);
        else {
          const hash = await putObject(change.bytes), revision = crypto.randomUUID();
          entries.set(change.path, { path: change.path, hash, revision });
          written.push({ path: change.path, bytes: change.bytes.slice(), revision });
        }
      }
      if (!frozen.length) return written;
      const manifest = await putObject(encode({ version: 1, files: [...entries.values()].sort((a, b) => a.path.localeCompare(b.path)) }));
      const response = await request("PUT", "head.json", encode({ version: 1, manifest }), base.etag === null ? { "If-None-Match": "*" } : { "If-Match": base.etag });
      if ([404, 409, 412].includes(response.status)) throw new StorageConflict("workspace head");
      await checked(response, "publish head"); etag(response);
      return written;
    });
    sequence = operation.catch(() => undefined);
    return operation;
  }
  return {
    durability: "remote",
    async read(path) { workspacePath(path); const entry = (await head()).manifest.files.find((file) => file.path === path); return entry ? { path, revision: entry.revision, bytes: await object(entry.hash) } : null; },
    async list() { return (await head()).manifest.files.map((file) => file.path).sort(); },
    async snapshot() { const published = await head(); return Promise.all(published.manifest.files.map(async (file) => ({ path: file.path, revision: file.revision, bytes: await object(file.hash) }))); },
    batch,
    async write(path, bytes, expected) { return (await batch([{ path, bytes, expected }]))[0]; },
    async delete(path, expected) { await batch([{ path, delete: true, expected }]); },
    close() { controller.abort(); },
  };
}
</storycode:copy>

### Assemble `apps/web/src/embed/s3Storage.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/s3Storage.ts"><storycode:paste select="#f-d9413be077e5-S3Request"/><storycode:paste select="#f-d9413be077e5-S3Options"/><storycode:paste select="#f-d9413be077e5-Entry"/><storycode:paste select="#f-d9413be077e5-Manifest"/><storycode:paste select="#f-d9413be077e5-Head"/><storycode:paste select="#f-d9413be077e5-S3Failure"/><storycode:paste select="#f-d9413be077e5-hashPattern"/><storycode:paste select="#f-d9413be077e5-digest"/><storycode:paste select="#f-d9413be077e5-encode"/><storycode:paste select="#f-d9413be077e5-openS3Storage"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/embed/s3Storage.ts](contract.md#output-d9413be077e5)

### Tests and fixtures

- [apps/web/src/embed/s3Storage.test.ts](contract.md#output-81a03dd838e9)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
