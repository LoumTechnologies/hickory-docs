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
export interface S3Options {
  /** Isolated workspace prefix inside a bucket; no bucket listing is required. */
  prefix: string;
  request(input: S3Request): Promise<Response>;
}
interface Entry { path: string; hash: string; revision: string; }
interface Manifest { version: 1; files: Entry[]; }
interface Head { etag: string | null; manifest: Manifest; }
export class S3Failure extends Error {
  constructor(readonly status: number, readonly operation: string) {
    super(status === 401 || status === 403 ? "S3 credentials expired or access was denied; renew access before syncing" : `S3 ${operation} failed (${status})`);
  }
}
const hashPattern = /^[a-f0-9]{64}$/;
async function digest(bytes: Uint8Array): Promise<string> {
  const hash = await crypto.subtle.digest("SHA-256", bytes.slice().buffer);
  return [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}
const encode = (value: unknown) => new TextEncoder().encode(JSON.stringify(value));

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
