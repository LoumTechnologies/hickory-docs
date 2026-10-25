<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/uiActions.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)

Captured SHA-256: `5a2f41e07a276ab7bf8a69fe11b834cd19f75e6cd94ef3743226dce0d6a6a285`.

<book:entry path="apps/web/src/lib/uiActions.test.ts">
import { describe, expect, it, vi } from "vitest";
import { UiActions, type UiCommand, type UiReceipt, type UiState, type UiValue } from "./uiActions";

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
const scope = { workspace: "practice", window: "one" };
const actor = { id: "python:turn1", kind: "agent" as const };
function fixture() {
  let state: UiState = { revision: "source:1/stop:1", enabled: true, value: "λ😀" };
  const receipts: UiReceipt[] = [];
  const execute = vi.fn<UiCommand["execute"]>().mockResolvedValue({ status: "completed", value: "executed" });
  const policy = vi.fn(() => true);
  const registry = new UiActions(scope, policy, async receipt => { receipts.push(receipt); });
  const command: UiCommand = { label: "Run", subject: "deadline.md#one", parent: "editor", parameters: {}, prerequisites: ["saved source"], read: () => state, execute };
  const unregister = registry.register("run-one", command);
  const handle = registry.observe("editor").actions[0].handle;
  const act = (request_id = "request-1", arguments_ = {}) => registry.act(scope, actor, { handle, arguments: arguments_, request_id });
  return { registry, execute, policy, receipts, command, unregister, handle, act, state: (next: Partial<UiState>) => { state = { ...state, ...next }; } };
}

describe("explicit UI command dispatch", () => {
  it("validates integer and bounded array arguments before terminal dispatch", async () => {
    const f = fixture();
    f.registry.register("argv", { ...f.command, parent: "terminal", parameters: { argv: { type: "array", maxItems: 2, items: { type: "string", maxLength: 2 } }, offset: { type: "number", integer: true, minimum: 0, maximum: 100 } } });
    const handle = f.registry.observe("terminal").actions[0].handle;
    const invoke = (arguments_: Record<string, UiValue>, request_id: string) => f.registry.act(scope, actor, { handle, arguments: arguments_, request_id });
    for (const [i, argv] of [["abc"], ["a", "b", "c"], [1], "a"].entries()) {
      expect(await invoke({ argv, offset: 0 }, String(i))).toMatchObject({ status: "invalid" });
    }
    expect(await invoke({ argv: ["λ😀"], offset: 0.5 }, "fraction")).toMatchObject({ status: "invalid" });
    expect(f.execute).not.toHaveBeenCalled();
    expect(await invoke({ argv: ["λ😀"], offset: 0 }, "valid-array")).toMatchObject({ status: "result" });
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("discloses only the requested parent and invokes the registered semantic handler", async () => {
    const f = fixture();
    expect(f.registry.observe("closed menu").actions).toEqual([]);
    const view = f.registry.observe("editor");
    expect(view.actions[0]).toMatchObject({ label: "Run", subject: "deadline.md#one", value: "λ😀", enabled: true });
    expect(view.edit_baseline).toBe(false);
    expect(await f.act()).toEqual({ status: "result", outcome: { status: "completed", value: "executed" } });
    expect(f.execute).toHaveBeenCalledOnce();
    expect(f.execute.mock.calls[0][1]).toEqual(actor);
    expect(f.receipts.map(r => r.stage)).toEqual(["requested", "result"]);
    expect(f.receipts[0].result).toBeUndefined();
    expect(f.registry.observe("editor").observation).not.toBe(view.observation);
  });
  it("distinguishes duplicate labels by subject and bounds disclosure", () => {
    const f = fixture();
    f.registry.register("run-two", { ...f.command, subject: "deadline.md#two" });
    const first = f.registry.observe("editor", 1);
    const second = f.registry.observe("editor", 1, first.next!);
    expect(first.coverage).toEqual({ disclosed: 1, total: 2 });
    expect(first.actions[0].handle).not.toBe(second.actions[0].handle);
    expect(second.actions[0].subject).toBe("deadline.md#two");
    expect(() => f.registry.observe("editor", 10000)).toThrow();
  });
  it("rejects stale source/stop and returns a refreshed bounded descriptor", async () => {
    const f = fixture(); f.state({ revision: "source:2/stop:2" });
    expect(await f.act()).toMatchObject({ status: "stale", observation: { revision: "source:2/stop:2" } });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("checks enabled state and policy at invocation", async () => {
    const f = fixture(); f.policy.mockReturnValue(false);
    expect(await f.act()).toMatchObject({ status: "forbidden" });
    f.policy.mockReturnValue(true); f.state({ enabled: false, reason: "Save first" });
    const handle = f.registry.observe("editor").actions[0].handle;
    expect(await f.registry.act(scope, actor, { handle, arguments: {}, request_id: "disabled" })).toMatchObject({ status: "disabled", reason: "Save first" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("never resolves invented, quoted, cross-window or unregistered references", async () => {
    const f = fixture();
    for (const handle of ["action:forged", `Run [${f.handle}]`, JSON.stringify(f.handle)]) {
      expect(await f.registry.act(scope, actor, { handle, arguments: {}, request_id: handle })).toMatchObject({ status: "unavailable" });
    }
    expect(await f.registry.act({ ...scope, window: "two" }, actor, { handle: f.handle, arguments: {}, request_id: "other-window" })).toMatchObject({ status: "forbidden" });
    const other = new UiActions(scope, () => true, async () => {});
    expect(await other.act(scope, actor, { handle: f.handle, arguments: {}, request_id: "other-registry" })).toMatchObject({ status: "unavailable" });
    f.unregister(); expect(await f.act()).toMatchObject({ status: "unavailable" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("validates typed inputs, required keys and unknown parameters", async () => {
    const f = fixture();
    f.registry.register("input", { ...f.command, parent: "dialog", parameters: { text: { type: "string", maxLength: 4 }, count: { type: "number", minimum: 1, maximum: 3 }, enabled: { type: "boolean" } } });
    const handle = f.registry.observe("dialog").actions[0].handle;
    const invalid: Record<string, UiValue>[] = [{}, { text: "abcde", count: 1, enabled: true }, { text: "ok", count: Infinity, enabled: true }, { text: "ok", count: 1, enabled: "true" }, { text: "ok", count: 1, enabled: true, extra: 1 }];
    for (const [i, arguments_] of invalid.entries()) {
      expect(await f.registry.act(scope, actor, { handle, arguments: arguments_, request_id: String(i) })).toMatchObject({ status: "invalid" });
    }
    expect(await f.registry.act(scope, actor, { handle, arguments: { text: "λ😀", count: 2, enabled: false }, request_id: "valid" })).toMatchObject({ status: "result" });
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("deduplicates retries while allowing intentional repeat runs and rejects key reuse", async () => {
    const f = fixture();
    await Promise.all([f.act(), f.act()]); expect(f.execute).toHaveBeenCalledOnce();
    expect(await f.act("request-1", { unexpected: 1 })).toMatchObject({ status: "conflict" });
    await f.act("intentional-repeat"); expect(f.execute).toHaveBeenCalledTimes(2);
  });
  it("cancels only owned requests and passes the signal to the real handler", async () => {
    const f = fixture();
    f.execute.mockImplementation(async (_args, _actor, signal) => {
      await new Promise<void>(resolve => signal.addEventListener("abort", () => resolve(), { once: true }));
      return { status: "unavailable", reason: "Cancelled by executor" };
    });
    const pending = f.act();
    await vi.waitFor(() => expect(f.execute).toHaveBeenCalledOnce());
    expect(f.registry.cancel(scope, { id: "other", kind: "agent" }, "request-1")).toBe(false);
    expect(f.registry.cancel(scope, actor, "request-1")).toBe(true);
    expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  });
  it("serializes actions on the same subject and captures parameters before asynchronous execution", async () => {
    const f = fixture(); let release!: () => void;
    f.execute.mockImplementation(async () => { await new Promise<void>(resolve => { release = resolve; }); return { status: "completed", value: null }; });
    const arguments_ = {}; const pending = f.act("one", arguments_);
    Object.assign(arguments_, { injected: true });
    await vi.waitFor(() => expect(f.execute).toHaveBeenCalledOnce());
    expect(await f.act("two")).toMatchObject({ status: "conflict" });
    expect(f.execute.mock.calls[0][0]).toEqual({});
    release(); await pending;
  });
  it("rechecks state and permission after retaining intent", async () => {
    let revision = "one";
    const execute = vi.fn<UiCommand["execute"]>();
    const registry = new UiActions(scope, () => true, async receipt => { if (receipt.stage === "requested") revision = "two"; });
    registry.register("run", { ...fixture().command, read: () => ({ revision, enabled: true }), execute });
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "stale" });
    expect(execute).not.toHaveBeenCalled();
  });
  it("fails closed when intent cannot be retained and does not invent a completed result", async () => {
    const f = fixture();
    const registry = new UiActions(scope, () => true, async () => { throw Error("disk full"); });
    registry.register("run", f.command);
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "error", reason: "Could not retain the action intent; no action was submitted: Error: disk full" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("cannot revive a command unregistered while intent is being retained", async () => {
    const f = fixture(); let remove!: () => void;
    const registry = new UiActions(scope, () => true, async receipt => { if (receipt.stage === "requested") remove(); });
    remove = registry.register("run", f.command);
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "unavailable" });
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r2" })).toMatchObject({ status: "unavailable" });
    expect(registry.observe("editor").actions).toEqual([]);
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("honors revoked policy after asynchronous intent persistence", async () => {
    const f = fixture(); let permitted = true;
    const registry = new UiActions(scope, () => permitted, async receipt => { if (receipt.stage === "requested") permitted = false; });
    registry.register("run", f.command);
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "forbidden" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("does not execute a request cancelled before its handler starts", async () => {
    const f = fixture(); const pending = f.act();
    expect(f.registry.cancel(scope, actor, "request-1")).toBe(true);
    expect(await pending).toMatchObject({ status: "unavailable" });
    expect(f.execute).not.toHaveBeenCalled();
    expect(f.registry.cancel(scope, actor, "request-1")).toBe(false);
  });
  it("retains uncertain-result errors and never repeats completed side effects", async () => {
    const f = fixture();
    const registry = new UiActions(scope, () => true, async receipt => { if (receipt.stage === "result") throw Error("disk full"); });
    registry.register("run", f.command);
    const invocation = { handle: registry.observe("editor").actions[0].handle, arguments: {}, request_id: "r" };
    expect(await registry.act(scope, actor, invocation)).toMatchObject({ status: "error", reason: expect.stringContaining("Error: disk full") });
    await registry.act(scope, actor, invocation);
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("refuses new requests at the journal limit without evicting retry protection", async () => {
    const f = fixture(); const registry = new UiActions(scope, () => true, async () => {}, 1);
    registry.register("run", f.command);
    const invocation = { handle: registry.observe("editor").actions[0].handle, arguments: {}, request_id: "r" };
    await registry.act(scope, actor, invocation);
    expect(await registry.act(scope, actor, { ...invocation, request_id: "r2" })).toMatchObject({ status: "unavailable" });
    expect(await registry.act(scope, actor, invocation)).toMatchObject({ status: "result" });
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("disposes a closed window without dispatching queued handlers", async () => {
    const f = fixture(); const pending = f.act();
    f.registry.dispose();
    expect(await pending).toMatchObject({ status: "unavailable" });
    expect(await f.act("after-close")).toMatchObject({ status: "unavailable" });
    expect(f.registry.groups().groups).toEqual([]);
    expect(() => f.registry.register("new", f.command)).toThrow("closed");
    expect(f.execute).not.toHaveBeenCalled();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("snapshots mutable outcome data before retention and prevents caller tampering with retries", async () => {
  const f = fixture(), live = { items: [{ state: "before" }] };
  f.execute.mockResolvedValue({ status: "completed", value: live });
  const pending = f.act();
  const result = await pending;
  live.items[0].state = "after";
  expect(result).toMatchObject({ status: "result", outcome: { value: { items: [{ state: "before" }] } } });
  if (result.status !== "result" || result.outcome.status !== "completed") throw Error("Expected completed snapshot");
  const delivered = result.outcome.value as typeof live;
  expect(() => { delivered.items[0].state = "spoofed"; }).toThrow(TypeError);
  expect(f.act()).toBe(pending);
  expect(await f.act()).toEqual(result);
  expect(f.receipts[1].result).toEqual(result);
  expect(f.execute).toHaveBeenCalledOnce();
});

it("refuses a known handle when mutable disclosed metadata changed without a manual revision bump", async () => {
  const f = fixture(), value = { count: 1 };
  f.state({ value });
  const handle = f.registry.observe("editor").actions[0].handle;
  value.count = 2;
  expect(await f.registry.act(scope, actor, { handle, arguments: {}, request_id: "changed-metadata" })).toMatchObject({ status: "stale", observation: { value: { count: 2 } } });
  expect(f.execute).not.toHaveBeenCalled();
});

it("retains an uncertain result and refuses to repeat side effects when an executor returns non-serializable data", async () => {
  const f = fixture(), value: Record<string, UiValue> = {}; value.self = value;
  f.execute.mockResolvedValue({ status: "completed", value });
  expect(await f.act()).toMatchObject({ status: "error", reason: expect.stringContaining("side effects may have occurred") });
  expect(await f.act()).toMatchObject({ status: "error" });
  expect(f.execute).toHaveBeenCalledOnce();
  expect(f.receipts.map(r => r.stage)).toEqual(["requested", "result"]);
});

it("monotonically confines disclosure, undispatched actions and cached ordinary results", async () => {
  const f = fixture();
  await f.act("before-confinement");
  f.registry.register("safe", { ...f.command, parent: "private", subject: "private", label: "Private metadata", execute: async () => ({ status: "completed", value: "withheld" }) });
  f.registry.confine((_key, command) => command.parent === "private");
  expect(f.registry.groups().groups).toEqual(["private"]);
  expect(f.registry.observe("editor").actions).toEqual([]);
  expect(f.registry.descriptor("run-one")).toBeNull();
  expect(await f.act("before-confinement")).toMatchObject({ status: "forbidden" });
  expect(await f.act("after-confinement")).toMatchObject({ status: "forbidden" });
  expect(f.execute).toHaveBeenCalledOnce();
  f.registry.confine(() => true); // A later permissive guard cannot reopen it.
  expect(f.registry.observe("editor").actions).toEqual([]);
});

it("withholds an ordinary in-flight result and its retained result receipt when confinement begins", async () => {
  const f = fixture(); let release!: () => void;
  f.execute.mockImplementation(async () => { await new Promise<void>(resolve => { release = resolve; }); return { status: "completed", value: "held-out-secret" }; });
  const pending = f.act("in-flight");
  await vi.waitFor(() => expect(release).toBeTypeOf("function"));
  f.registry.confine(() => false); release();
  const result = await pending;
  expect(result).toMatchObject({ status: "unavailable" });
  expect(JSON.stringify(result)).not.toContain("held-out-secret");
  expect(JSON.stringify(f.receipts.find(receipt => receipt.stage === "result"))).not.toContain("held-out-secret");
  expect(await f.act("in-flight")).toMatchObject({ status: "forbidden" });
});

it("refuses an ordinary handler if confinement starts during intent persistence", async () => {
  const f = fixture(); let release!: () => void;
  const registry = new UiActions(scope, () => true, receipt => receipt.stage === "requested" ? new Promise<void>(resolve => { release = resolve; }) : Promise.resolve());
  registry.register("run", f.command);
  const handle = registry.observe("editor").actions[0].handle;
  const pending = registry.act(scope, actor, { handle, arguments: {}, request_id: "pending-intent" });
  await vi.waitFor(() => expect(release).toBeTypeOf("function"));
  registry.confine(() => false); release();
  expect(await pending).toMatchObject({ status: "unavailable" });
  expect(f.execute).not.toHaveBeenCalled();
});
</book:entry>

</book:doc>
