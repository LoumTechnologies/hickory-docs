<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/responseRecovery.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Unsent responses survive restart](../../../../../contract.md)

Captured SHA-256: `dd4b8366da484d79187bfd66a1c09e40d4521d5bd7c4ca9cad57217e3a2d6f53`.

<book:entry path="apps/web/src/lib/responseRecovery.test.ts">
// Guarantee: docs/guarantees/agent/unsent-responses-survive-restart.md
import { beforeEach, expect, it, vi } from "vitest";
import { ResponseRecovery, responseThread, useResponseRecovery, flushResponseRecovery } from "./responseRecovery";
import { act, renderHook, waitFor } from "@testing-library/react";
import { api } from "../api/client";
import type { AgentTurn } from "../api/types";
vi.mock("../api/client", () => ({ api: { responseDraft: vi.fn(), saveResponseDraft: vi.fn() } }));
const turns = [{ id: "a", parent_id: null }, { id: "b", parent_id: "a" }, { id: "other", parent_id: null }] as AgentTurn[];
let disk: unknown;
beforeEach(() => {
  disk = null;
  vi.mocked(api.responseDraft).mockImplementation(async () => ({ scope: "folder/window/subject", state: structuredClone(disk) }));
  vi.mocked(api.saveResponseDraft).mockImplementation(async (_doc, _scope, state) => { disk = structuredClone(state); return { ok: true }; });
});
const make = (key = "workspace") => new ResponseRecovery("workspace", key, "");
function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(r => { resolve = r; }); return { promise, resolve }; }
it("recovers after a fresh controller, shares rewinds and isolates threads", async () => {
  const first = make(); await first.load(turns);
  first.select("b"); first.edit("branch response"); await first.flush();
  first.select("a"); expect(first.text).toBe("branch response");
  first.select("other"); expect(first.text).toBe(""); first.edit("other response"); await first.flush();
  const restarted = make(); await restarted.load(turns);
  expect(restarted.state.tip).toBe("other"); expect(restarted.text).toBe("other response");
  restarted.select("b"); expect(restarted.text).toBe("branch response"); await restarted.flush();
  expect(responseThread(turns, "b")).toBe("a");
});
it("keeps early typing while retaining the other recovered threads", async () => {
  const read = deferred<{ scope: string; state: unknown }>();
  vi.mocked(api.responseDraft).mockReturnValueOnce(read.promise);
  const recovery = make(); const loading = recovery.load(turns);
  recovery.edit("already typing");
  read.resolve({ scope: "scope", state: { version: 1, tip: "a", drafts: { other: "old response", a: "another response" } } });
  await loading; await recovery.flush();
  expect(recovery.text).toBe("already typing"); recovery.select("a"); expect(recovery.text).toBe("another response"); await recovery.flush();
});
it("orders delayed writes before send clearing and explicit discard across remounts", async () => {
  const write = deferred<{ ok: true }>();
  const recovery = make(); await recovery.load([]);
  vi.mocked(api.saveResponseDraft).mockImplementationOnce(async (_doc, _scope, state) => { await write.promise; disk = structuredClone(state); return { ok: true }; });
  recovery.edit("send this");
  await new Promise(resolve => setTimeout(resolve, 0)); // the old write is actually in flight
  const submitted = recovery.snapshot();
  recovery.accepted(submitted, "sent");
  const restarted = make(); const loading = restarted.load([{ id: "sent", parent_id: null }] as AgentTurn[]);
  write.resolve({ ok: true }); await loading;
  expect(restarted.text).toBe("");
  restarted.edit("discard this"); restarted.discard(); await restarted.flush();
  const again = make(); await again.load([]); expect(again.text).toBe("");
});
it("leaves failed send text alone and preserves newer typing after acceptance", async () => {
  const recovery = make(); await recovery.load([]); recovery.edit("submit");
  const submitted = recovery.snapshot();
  // A rejected request never invokes accepted.
  await recovery.flush(); const restart = make(); await restart.load([]); expect(restart.text).toBe("submit");
  recovery.edit("new response"); recovery.accepted(submitted, "sent"); await recovery.flush();
  expect(recovery.text).toBe("new response");
  const again = make(); await again.load([{ id: "sent", parent_id: null }] as AgentTurn[]); expect(again.text).toBe("new response");
});
it("reports unavailable and malformed storage without blocking typing", async () => {
  disk = { version: 55 };
  const recovery = make(); await recovery.load([]); expect(recovery.error).toContain("could not be saved");
  recovery.edit("usable"); expect(recovery.text).toBe("usable");
  vi.mocked(api.responseDraft).mockRejectedValueOnce(new Error("offline"));
  const offline = make("offline"); await offline.load([]); offline.edit("still usable"); expect(offline.error).toBeTruthy();
});
it("reports write failure and retries the current revision", async () => {
  const recovery = make(); await recovery.load([]);
  vi.mocked(api.saveResponseDraft).mockRejectedValueOnce(new Error("full"));
  recovery.edit("retain me"); await new Promise(resolve => setTimeout(resolve, 0)); expect(recovery.error).toBeTruthy();
  await recovery.flush(); expect(recovery.error).toBeNull();
  const again = make(); await again.load([]); expect(again.text).toBe("retain me");
});

it("saves pre-restoration typing before a remounted reader and keeps close flushing ordered", async () => {
  const read = deferred<{ scope: string; state: unknown }>();
  vi.mocked(api.responseDraft).mockReturnValueOnce(read.promise);
  const first = make("early-close"); const loading = first.load([]);
  first.edit("early text");
  const second = make("early-close"); const reopening = second.load([]);
  read.resolve({ scope: "scope", state: null });
  await loading; await reopening;
  expect(second.text).toBe("early text");
});

it("does not move another thread's response when a pending send completes", async () => {
  const recovery = make(); await recovery.load(turns);
  recovery.select("a"); recovery.edit("submitted"); const submitted = recovery.snapshot();
  recovery.select("other"); recovery.edit("belongs to other"); recovery.accepted(submitted, "accepted");
  await recovery.flush();
  expect(recovery.state.tip).toBe("other"); expect(recovery.text).toBe("belongs to other");
  recovery.select("a"); expect(recovery.text).toBe(""); await recovery.flush();
});

it("restores the root draft when response storage arrives before conversation history", async () => {
  disk = { version: 1, tip: "b", drafts: { a: "reply to this thread" } };
  const recovery = make(); await recovery.load([]);
  recovery.history(turns); expect(recovery.text).toBe("reply to this thread");
  const early = make("early-history"); await early.load([]);
  early.edit("typing before history"); early.history(turns); await early.flush();
  const restarted = make("early-history"); await restarted.load(turns);
  expect(restarted.text).toBe("typing before history");
});

it("flushes pending writes before close and preserves reopened-pane typing when an old send accepts", async () => {
  const history: AgentTurn[] = [];
  const first = renderHook(() => useResponseRecovery("workspace", "remount-send", "", history, true));
  await waitFor(() => expect(first.result.current.scope).toBeTruthy());
  act(() => first.result.current.edit("submit from old pane"));
  const submitted = first.result.current.snapshot(); const old = first.result.current;
  await flushResponseRecovery(); first.unmount();
  const second = renderHook(() => useResponseRecovery("workspace", "remount-send", "", history, true));
  await waitFor(() => expect(second.result.current.text).toBe("submit from old pane"));
  act(() => second.result.current.edit("new text in reopened pane"));
  act(() => old.accepted(submitted, "accepted"));
  await flushResponseRecovery();
  expect(second.result.current.text).toBe("new text in reopened pane");
  expect((disk as { drafts: Record<string, string> }).drafts.accepted).toBe("new text in reopened pane");
  second.unmount();
});
</book:entry>

</book:doc>
