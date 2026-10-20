import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError, api, installMockHandler } from "./client";

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

describe("a failed request says what the server said", () => {
  afterEach(() => {
    installMockHandler(null as never);
    vi.restoreAllMocks();
  });

  /** The real fetch, with a response the caller describes. */
  function answering(status: number, body: string, type: string) {
    installMockHandler(null as never);
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(body, {
          status,
          statusText: status === 422 ? "Unprocessable Entity" : "Bad Request",
          headers: { "Content-Type": type },
        }),
      ),
    );
  }

  it("reads our own JSON refusals", async () => {
    answering(422, JSON.stringify({ error: "`greeter/` already exists." }), "application/json");
    await expect(api.files()).rejects.toThrow("`greeter/` already exists.");
  });

  it("reads a plain-text body rather than showing the status line", async () => {
    // The case that cost a debugging session: axum answers a request whose
    // JSON body does not fit the handler's type BEFORE the handler runs, with
    // `text/plain` naming the exact field. That sentence is the whole
    // diagnosis, and it used to be replaced by "Unprocessable Entity".
    answering(
      422,
      "Failed to deserialize the JSON body into the target type: missing field `image` at line 1 column 47",
      "text/plain; charset=utf-8",
    );
    await expect(api.files()).rejects.toThrow(/missing field `image`/);
  });

  it("keeps the parsed body for the fields a screen is keyed off", async () => {
    answering(
      422,
      JSON.stringify({ error: "no dotnet here", missing: "dotnet" }),
      "application/json",
    );
    const error = await api.files().catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).body).toEqual({
      error: "no dotnet here",
      missing: "dotnet",
    });
  });

  it("falls back to the status line only when the body is empty", async () => {
    answering(422, "", "text/plain");
    await expect(api.files()).rejects.toThrow("Unprocessable Entity");
  });
});

it("uses the versioned attribution route for assisted scratchpad saves without fallback", async () => {
  const handler = vi.fn(async () => ({ path: "note.md" }));
  installMockHandler(handler);
  try {
    await api.saveScratchpad("Human text");
    expect(handler).toHaveBeenLastCalledWith("POST", "/api/scratchpad", { text: "Human text" });
    await api.saveScratchpad("Assisted text", true);
    expect(handler).toHaveBeenLastCalledWith("POST", "/api/v1/scratchpad", { text: "Assisted text", agent_assisted: true });
    handler.mockRejectedValueOnce(new Error("Versioned route unavailable"));
    await expect(api.saveScratchpad("Do not downgrade", true)).rejects.toThrow("Versioned route unavailable");
    expect(handler).toHaveBeenCalledTimes(3);
  } finally { installMockHandler(null as never); }
});

it("binds fleet mutation routes to observed keys without legacy fallback", async () => {
  const handler = vi.fn(async () => ({ machine: { name: "desktop" } }));
  installMockHandler(handler);
  try {
    await api.fleetGrant("desktop", "execute", true, "observed-key");
    expect(handler).toHaveBeenLastCalledWith("PUT", "/api/v1/fleet/grant", { machine: "desktop", grant: "execute", on: true, expected_key: "observed-key" });
    await api.fleetRemove("desktop", "observed-key");
    expect(handler).toHaveBeenLastCalledWith("POST", "/api/v1/fleet/remove", { machine: "desktop", expected_key: "observed-key" });
    handler.mockRejectedValueOnce(new Error("Versioned route missing"));
    await expect(api.fleetGrant("desktop", "execute", true, "observed-key")).rejects.toThrow("Versioned route missing");
    expect(handler).toHaveBeenCalledTimes(3);
    await api.fleetRemove("desktop", "");
    expect(handler).toHaveBeenLastCalledWith("POST", "/api/v1/fleet/remove", { machine: "desktop", expected_key: "" });
  } finally { installMockHandler(null as never); }
});
