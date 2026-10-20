import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { api, installMockHandler } from "../api/client";
import type { MergedViewResponse } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useMergedViewControls, type MergedWriteReply } from "./useMergedViewControls";
const trees = [{ path: "/fixture/repo", name: "repo", current: true }, { path: "/fixture/side", name: "side", current: false }];
function view(path: string, targets?: string[]): MergedViewResponse {
  const sources = trees.filter(tree => !targets || targets.includes(tree.name));
  return { repository: true, path, sources, regions: sources.length ? [{ kind: "shared", text: `${path}\n` }] : [], missing: [], snapshot: Object.fromEntries(sources.map(source => [source.name, `${path}\n`])) };
}
beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(api, "worktrees").mockResolvedValue({ repository: true, worktrees: trees });
  vi.spyOn(api, "merged").mockImplementation(async (path, targets) => view(path, targets));
});
afterEach(() => { cleanup(); installMockHandler(null as never); });
function harness(path = "file.txt") {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const hook = renderHook(({ path }) => useMergedViewControls(path), { initialProps: { path },
    wrapper: ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>,
  });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  return { ...hook, registry, actions, run };
}
it("shares membership and write permissions without conflating an empty selection with all targets", async () => {
  installMockHandler(async (method, path, body) => {
    if (method === "POST" && path === "/api/revision-projects") {
      expect(body).toMatchObject({ kind: "directory", worktree: expect.stringMatching(/repo|side/) });
      return { files: [{ path: "file.txt", kind: "file", executable: false, bytes: [...new TextEncoder().encode("file.txt\n")] }] };
    }
    throw Error(`Unexpected fixture request: ${method} ${path}`);
  });
  const hook = harness();
  await waitFor(() => expect(hook.result.current.view?.path).toBe("file.txt"));
  await hook.run("Set comparison worktree writable: repo", { on: true });
  expect(hook.result.current.writable.has("repo")).toBe(true);
  await hook.run("Select comparison worktree: side", { on: false });
  await waitFor(() => expect(hook.result.current.shown).toEqual(["repo"]));
  expect(hook.result.current.writable.size).toBe(0);
  await hook.run("Select comparison worktree: repo", { on: false });
  await waitFor(() => expect(api.merged).toHaveBeenLastCalledWith("file.txt", []));
  expect(hook.result.current.shown).toEqual([]);
  await hook.run("Select comparison worktree: repo", { on: true });
  await hook.run("Select comparison worktree: side", { on: true });
  await waitFor(() => expect(hook.result.current.shown).toEqual(["repo", "side"]));
  expect(await hook.run("Compare complete worktree projects")).toMatchObject({ status: "result", outcome: { value: { response: true } } });
  expect(hook.result.current.comparison).toMatchObject({ policy: "neither", evidence: "unverified", old: "repo", new: "side" });
});
it("keeps an in-flight write bound to its original path and refuses late adoption or cached retargeting", async () => {
  let complete!: (reply: MergedWriteReply) => void;
  const posts: unknown[] = [];
  installMockHandler(async (method, path, body) => {
    expect([method, path]).toEqual(["POST", "/api/merged/write"]); posts.push(body);
    return new Promise(resolve => { complete = resolve; });
  });
  const hook = harness("old.txt");
  await waitFor(() => expect(hook.result.current.view?.path).toBe("old.txt"));
  await hook.run("Set comparison worktree writable: repo", { on: true });
  const cachedApply = hook.result.current.apply;
  let pending!: ReturnType<typeof cachedApply>;
  await act(async () => { pending = cachedApply(0, "edited", "repo", "shared"); });
  expect(posts).toEqual([expect.objectContaining({ path: "old.txt", expected: { repo: "old.txt\n", side: "old.txt\n" }, text: "edited" })]);
  hook.rerender({ path: "new.txt" });
  await waitFor(() => expect(hook.result.current.view?.path).toBe("new.txt"));
  const reply = { targets: [{ source: "repo", ok: true }], undo: [{ source: "repo", text: "old.txt\n", expected: "edited" }] };
  await act(async () => { complete(reply); expect(await pending).toEqual(reply); });
  expect(hook.result.current.result).toBeNull();
  expect(hook.result.current.writable.size).toBe(0);
  expect(hook.result.current.needsRefresh).toBe(false);
  await expect(cachedApply(0, "wrong", "repo", "shared")).rejects.toThrow(/closed, changed or busy/);
  expect(posts).toHaveLength(1);
  expect(hook.actions().some(action => action.label === "Undo worktree write: repo")).toBe(false);
});
it("retains actual partial write replies after refresh failure and binds explicit undo to their guarded bytes", async () => {
  const posts: { path: string; body: unknown }[] = [];
  const reply = { targets: [{ source: "repo", ok: true }, { source: "side", ok: false, error: "fixture target conflict" }], undo: [{ source: "repo", text: "PRIVATE undo bytes", expected: "edited" }] };
  installMockHandler(async (_method, path, body) => {
    posts.push({ path, body });
    if (path === "/api/merged/write") return reply;
    if (path === "/api/merged/undo") return { targets: [{ source: "repo", ok: true }] };
    throw Error("Unexpected fixture request");
  });
  const hook = harness();
  await waitFor(() => expect(hook.result.current.view?.path).toBe("file.txt"));
  await hook.run("Set comparison worktree writable: repo", { on: true });
  await hook.run("Set comparison worktree writable: side", { on: true });
  vi.mocked(api.merged).mockRejectedValueOnce(Error("fixture refresh refused"));
  await act(async () => { expect(await hook.result.current.apply(0, "edited", "repo", "shared")).toEqual(reply); });
  expect(hook.result.current.result).toEqual(reply);
  expect(hook.result.current.needsRefresh).toBe(true);
  expect(hook.result.current.error).toContain("Write reply received; refresh failed");
  const reading = await hook.run("Read worktree comparison bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  const text = new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes));
  expect(text).toContain("fixture target conflict"); expect(text).not.toContain("PRIVATE undo bytes");
  await act(async () => { await expect(hook.result.current.apply(0, "again", "repo", "shared")).rejects.toThrow(/changed/); });
  expect(posts).toHaveLength(1);
  expect(await hook.run("Undo worktree write: repo")).toMatchObject({ status: "result", outcome: { value: { response: { targets: [{ source: "repo", ok: true }] } } } });
  expect(posts[1]).toEqual({ path: "/api/merged/undo", body: { path: "file.txt", undo: reply.undo } });
  expect(hook.result.current.needsRefresh).toBe(false);
});
it("does not expose file bodies or hidden controls in a one-worktree status view", async () => {
  vi.mocked(api.worktrees).mockResolvedValue({ repository: true, worktrees: [trees[0]] });
  vi.mocked(api.merged).mockResolvedValue({ ...view("one.txt", ["repo"]), regions: [{ kind: "shared", text: "PRIVATE unrendered file" }] });
  const hook = harness("one.txt");
  await waitFor(() => expect(hook.result.current.view?.path).toBe("one.txt"));
  expect(hook.actions().some(action => action.label.startsWith("Select comparison worktree:"))).toBe(false);
  expect(hook.actions().some(action => action.label === "Compare complete worktree projects")).toBe(false);
  const reading = await hook.run("Read worktree comparison bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes))).not.toContain("PRIVATE unrendered file");
});
it("refuses an old initial capture and complete-project bytes that differ from the displayed snapshot", async () => {
  let finish!: (reply: MergedViewResponse) => void;
  vi.mocked(api.merged).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
  installMockHandler(async () => ({ files: [{ path: "new.txt", kind: "file", executable: false, bytes: [...new TextEncoder().encode("changed after display")] }] }));
  const hook = harness("old.txt");
  hook.rerender({ path: "new.txt" });
  await waitFor(() => expect(hook.result.current.view?.path).toBe("new.txt"));
  await act(async () => { finish(view("old.txt")); });
  expect(hook.result.current.view?.path).toBe("new.txt");
  expect(await hook.run("Compare complete worktree projects")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("changed since the displayed comparison") } });
  expect(hook.result.current.comparison).toBeNull();
});
