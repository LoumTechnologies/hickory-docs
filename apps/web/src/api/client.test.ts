import { afterEach, describe, expect, it, vi } from "vitest";
import { api, installMockHandler } from "./client";

describe("api.render deduplication", () => {
  afterEach(() => {
    installMockHandler(null as never);
    vi.restoreAllMocks();
  });

  it("collapses concurrent renders of the same doc onto one request", async () => {
    let resolve!: (v: unknown) => void;
    const pending = new Promise((r) => {
      resolve = r;
    });
    const handler = vi.fn(async (_m: string, path: string) => {
      await pending;
      return { blocks: [{ kind: "prose", id: path }] };
    });
    installMockHandler(handler);

    // Three call sites ask at once (initial load, run event, save).
    const calls = [api.render("doc-1"), api.render("doc-1"), api.render("doc-1")];
    // A different doc is a different request.
    const other = api.render("doc-2");
    resolve(undefined);

    const results = await Promise.all(calls);
    await other;
    expect(handler).toHaveBeenCalledTimes(2);
    expect(results[0]).toBe(results[1]);
    expect(results[1]).toBe(results[2]);
  });

  it("fetches again once the in-flight request settles", async () => {
    const handler = vi.fn(async () => ({ blocks: [] }));
    installMockHandler(handler);

    await api.render("doc-1");
    await api.render("doc-1");
    expect(handler).toHaveBeenCalledTimes(2);
  });

  it("does not wedge the doc after a failed render", async () => {
    let fail = true;
    const handler = vi.fn(async () => {
      if (fail) throw new Error("render failed");
      return { blocks: [] };
    });
    installMockHandler(handler);

    await expect(api.render("doc-1")).rejects.toThrow("render failed");
    fail = false;
    await expect(api.render("doc-1")).resolves.toEqual({ blocks: [] });
  });
});
