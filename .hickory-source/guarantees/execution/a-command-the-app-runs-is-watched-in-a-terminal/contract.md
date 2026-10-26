<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Command The App Runs On Your Behalf Is Watched In A Terminal

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a button, menu item or gutter mark in the app whose job is to run an
**external command line program** on this machine — `dotnet new`, `cargo
build`, `pytest`, `dotnet test`, `go test` — when a person triggers it, then
that program runs in a terminal session they can read, scroll and keep, and
its stdout and stderr are the program's own, byte for byte. The app reports
*around* the output; it never reports *instead of* it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-7aab2bc50e5c"></a>
## `apps/web/src/api/client.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Command The App Runs On Your Behalf Is Watched In A Terminal](contract.md).

<a id="f-7aab2bc50e5c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7aab2bc50e5c-expression_statement">
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

</storycode:copy>
<a id="f-7aab2bc50e5c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7aab2bc50e5c-expression_statement-2">
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

</storycode:copy>
<a id="f-7aab2bc50e5c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7aab2bc50e5c-expression_statement-3">
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

</storycode:copy>
<a id="f-7aab2bc50e5c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-7aab2bc50e5c-expression_statement-4">
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

</storycode:copy>
<a id="f-7aab2bc50e5c-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-7aab2bc50e5c-expression_statement-5">
it("keeps pairing jobs on the versioned originating-window routes without fallback", async () => {
  const handler = vi.fn(async () => ({}));
  installMockHandler(handler);
  try {
    await api.startFleetPairing("window", "retry-id", "host", "phrase");
    expect(handler).toHaveBeenLastCalledWith("POST", "/api/v1/fleet/pairings", { window: "window", request_id: "retry-id", mode: "host", phrase: "phrase" });
    await api.fleetPairings("window");
    expect(handler).toHaveBeenLastCalledWith("GET", "/api/v1/fleet/pairings?window=window", undefined);
    await api.fleetPairing("window", "job");
    expect(handler).toHaveBeenLastCalledWith("GET", "/api/v1/fleet/pairings/job?window=window", undefined);
    await api.cancelFleetPairing("window", "job");
    expect(handler).toHaveBeenLastCalledWith("POST", "/api/v1/fleet/pairings/job/cancel?window=window", {});
    await api.forgetFleetPairing("window", "job");
    expect(handler).toHaveBeenLastCalledWith("DELETE", "/api/v1/fleet/pairings/job?window=window", undefined);
    handler.mockRejectedValueOnce(Error("Protocol unavailable"));
    await expect(api.startFleetPairing("window", "retry-id", "host", "phrase")).rejects.toThrow("Protocol unavailable");
    expect(handler).toHaveBeenCalledTimes(6);
  } finally { installMockHandler(null as never); }
});

</storycode:copy>
<a id="f-7aab2bc50e5c-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-7aab2bc50e5c-expression_statement-6">
it("preserves versioned merged-view all, empty and literal-name selections without downgrade", async () => {
  const handler = vi.fn(async () => ({})); installMockHandler(handler);
  try {
    await api.merged("file.txt");
    expect(handler).toHaveBeenLastCalledWith("GET", "/api/v1/merged?path=file.txt", undefined);
    await api.merged("file.txt", []);
    expect(handler).toHaveBeenLastCalledWith("GET", "/api/v1/merged?path=file.txt&targets=%5B%5D", undefined);
    await api.merged("file.txt", ["branch,with,commas", "λ"]);
    expect(handler).toHaveBeenLastCalledWith("GET", `/api/v1/merged?path=file.txt&targets=${encodeURIComponent(JSON.stringify(["branch,with,commas", "λ"]))}`, undefined);
    handler.mockRejectedValueOnce(Error("Versioned selection unavailable"));
    await expect(api.merged("file.txt", [])).rejects.toThrow("Versioned selection unavailable");
    expect(handler).toHaveBeenCalledTimes(4);
    handler.mockRejectedValueOnce(new ApiError(404, "Not found"));
    await expect(api.merged("file.txt", [])).rejects.toThrow("Update the app and engine together");
    expect(handler).toHaveBeenCalledTimes(5);
  } finally { installMockHandler(null as never); }
});
</storycode:copy>

### Assemble `apps/web/src/api/client.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/client.test.ts"><storycode:paste select="#f-7aab2bc50e5c-expression_statement"/><storycode:paste select="#f-7aab2bc50e5c-expression_statement-2"/><storycode:paste select="#f-7aab2bc50e5c-expression_statement-3"/><storycode:paste select="#f-7aab2bc50e5c-expression_statement-4"/><storycode:paste select="#f-7aab2bc50e5c-expression_statement-5"/><storycode:paste select="#f-7aab2bc50e5c-expression_statement-6"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/App.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-25cc6f821b46)
- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [crates/hick-term/src/session.rs](../../../mechanisms/crates/hick-term/src/session.rs.md#output-5705d2b8acc0)
- [crates/hickory-cli/src/serve/scaffold.rs](../../../mechanisms/crates/hickory-cli/src/serve/scaffold.rs.md#output-3fd865a33211)

### Tests and fixtures

- [apps/web/src/api/client.test.ts](contract.md#output-7aab2bc50e5c)
- [apps/web/src/components/NewProjectDialog.test.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-065ce188ea68)
- [crates/hickory-cli/src/serve/test_run.rs](../../../mechanisms/crates/hickory-cli/src/serve/test_run.rs.md#output-c99ee9641f1f)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/serve_scaffold.rs.md#output-5a76df8fd4d1)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
