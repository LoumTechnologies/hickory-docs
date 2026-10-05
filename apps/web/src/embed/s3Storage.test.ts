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
const text = (value: string) => new TextEncoder().encode(value);
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
    const object = [...backend.objects.values()].find((entry) => new TextDecoder().decode(entry.bytes) === "a" || new TextDecoder().decode(entry.bytes) === "b")!;
    object.bytes = text("corrupt");
    await expect(a.snapshot()).rejects.toThrow("content hash check");
  });
});
