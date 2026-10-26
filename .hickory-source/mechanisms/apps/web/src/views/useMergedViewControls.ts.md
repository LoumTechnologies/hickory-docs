<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useMergedViewControls.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5c0c1024bb7f"></a>
## `apps/web/src/views/useMergedViewControls.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5c0c1024bb7f-MergedWriteReply"></a>
### `MergedWriteReply`

<storycode:copy id="f-5c0c1024bb7f-MergedWriteReply">
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { api, request } from "../api/client";
import type { MergedViewResponse, WorktreeInfo } from "../api/types";
import type { ProjectEntry } from "../editor/hickLang";
import type { ComparisonModel } from "../lib/projectComparison";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
export type MergedWriteReply = { targets: { source: string; ok: boolean; error?: string }[]; undo?: { source: string; text: string; expected: string }[] };
</storycode:copy>
<a id="f-5c0c1024bb7f-Owner"></a>
### `Owner`

<storycode:copy id="f-5c0c1024bb7f-Owner">
type Owner = { token: string; live: boolean; pending: boolean; reads: number; requiresRefresh: boolean };
</storycode:copy>
<a id="f-5c0c1024bb7f-json"></a>
### `json`

<storycode:copy id="f-5c0c1024bb7f-json">
const json = (value: unknown): UiValue => JSON.parse(JSON.stringify(value));

</storycode:copy>
<a id="f-5c0c1024bb7f-useMergedViewControls"></a>
### `useMergedViewControls`

<storycode:copy id="f-5c0c1024bb7f-useMergedViewControls">
export function useMergedViewControls(path: string) {
  const parent = `merged-worktrees:${useId()}`;
  const [chosen, setChosen] = useState<ReadonlySet<string> | null>(null);
  const targets = useMemo(() => chosen ? [...chosen].sort() : undefined, [chosen]);
  const selection = JSON.stringify([path, targets ?? null]);
  const token = useMemo(() => crypto.randomUUID(), [selection]);
  const currentToken = useRef(token); currentToken.current = token;
  const owner = useRef<Owner>({ token, live: false, pending: false, reads: 0, requiresRefresh: false });
  const [loaded, setLoaded] = useState<{ token: string; view: MergedViewResponse; worktrees: WorktreeInfo[] } | null>(null);
  const current = loaded?.token === token ? loaded : null;
  const view = current?.view ?? null, worktrees = current?.worktrees ?? null;
  const [permission, setPermission] = useState<{ token: string; names: ReadonlySet<string> } | null>(null);
  const empty = useMemo<ReadonlySet<string>>(() => new Set(), []);
  const writable = useMemo<ReadonlySet<string>>(() => permission?.token === token && current ? new Set([...permission.names].filter(name =>
    !!current.view.snapshot && Object.hasOwn(current.view.snapshot, name) && current.worktrees.filter(tree => tree.name === name).length === 1)) : empty, [permission, token, current, empty]);
  const [editState, setEditState] = useState<{ token: string; index: number } | null>(null);
  const editing = editState?.token === token ? editState.index : null;
  const [comparisonState, setComparisonState] = useState<{ token: string; model: ComparisonModel } | null>(null);
  const comparison = comparisonState?.token === token ? comparisonState.model : null;
  const [replyState, setReplyState] = useState<{ token: string; reply: MergedWriteReply; id: string; kind: "write" | "undo" } | null>(null);
  const result = replyState?.token === token ? replyState.reply : null;
  const [requestState, setRequestState] = useState<{ token: string; id: string; kind: "write" | "undo"; state: "pending" | "replied" | "failed_or_uncertain" } | null>(null);
  const latestRequest = requestState?.token === token ? requestState : null;
  const lastReplyKind = replyState?.token === token ? replyState.kind : null;
  const resultIsLatest = !!latestRequest && replyState?.token === token && latestRequest.id === replyState.id;
  const [errorState, setErrorState] = useState<{ token: string; message: string } | null>(null);
  const error = errorState?.token === token ? errorState.message : null;
  const [refreshRequiredToken, setRefreshRequiredToken] = useState<string | null>(null);
  const needsRefresh = refreshRequiredToken === token;
  const [busyToken, setBusyToken] = useState<string | null>(null);
  const busy = busyToken === token;
  const isCurrent = (captured: Owner) => captured.live && captured.token === currentToken.current;
  const setError = (message: string) => setErrorState({ token, message });
  async function load(captured: Owner) {
    const ticket = ++captured.reads;
    const [roster, capturedView] = await Promise.all([api.worktrees(), api.merged(path, targets)]);
    if (!isCurrent(captured) || ticket !== captured.reads) return false;
    captured.requiresRefresh = false; setRefreshRequiredToken(null);
    setLoaded({ token, view: capturedView, worktrees: roster.worktrees }); setErrorState(null);
    return true;
  }
  useEffect(() => {
    const captured: Owner = { token, live: true, pending: false, reads: 0, requiresRefresh: false }; owner.current = captured;
    void load(captured).catch(cause => { if (isCurrent(captured)) setError(String(cause)); });
    return () => { captured.live = false; captured.reads++; };
    // A token names one exact path/membership lifetime, including A→B→A changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);
  async function perform<T>(what: (captured: Owner) => Promise<T>): Promise<T> {
    const captured = owner.current;
    if (!isCurrent(captured) || captured.token !== token || captured.pending) throw Error("Worktree comparison is closed, changed or busy");
    captured.pending = true; setBusyToken(token); setErrorState(null);
    try { return await what(captured); }
    catch (cause) {
      if (isCurrent(captured)) setError(`${String(cause)}. Inspect per-target status before retrying; effects are not rolled back.`);
      throw cause;
    } finally { captured.pending = false; if (isCurrent(captured)) setBusyToken(null); }
  }
  const shown = view?.sources.map(source => source.name) ?? [];
  const rendered = !!view?.repository && !!worktrees && worktrees.length >= 2;
  const unambiguous = (name: string) => worktrees?.filter(tree => tree.name === name).length === 1;
  const ready = () => rendered && !!view && isCurrent(owner.current) && owner.current.token === token && !owner.current.pending;
  function choose(name: string, on: boolean) {
    if (!ready() || !unambiguous(name)) throw Error("Refresh the current unambiguous worktree roster before selecting a target");
    const next = new Set(chosen ?? worktrees!.map(tree => tree.name));
    if (on) next.add(name); else next.delete(name);
    setChosen(next);
  }
  function allowWrite(name: string, on: boolean) {
    if (!ready() || owner.current.requiresRefresh || !unambiguous(name) || !shown.includes(name) || !view?.snapshot || !Object.hasOwn(view.snapshot, name)) throw Error("A current captured file is required before opening this target for writing");
    const names = new Set(writable); if (on) names.add(name); else names.delete(name);
    setPermission({ token, names }); setEditState(null);
  }
  function edit(index: number) {
    if (!ready() || owner.current.requiresRefresh || !view?.snapshot || !view.regions[index]) throw Error("Refresh a current captured region before editing");
    setEditState(editing === index ? null : { token, index });
  }
  async function refreshedAfterWrite(captured: Owner) {
    if (!isCurrent(captured)) return;
    try { await load(captured); }
    catch (cause) { if (isCurrent(captured)) setError(`Write reply received; refresh failed: ${String(cause)}. Inspect its per-target reply before retrying.`); }
  }
  async function hostWrite(captured: Owner, kind: "write" | "undo", body: unknown) {
    const id = crypto.randomUUID();
    captured.requiresRefresh = true; setRefreshRequiredToken(token);
    setRequestState({ token, id, kind, state: "pending" });
    try {
      const answer = await request<MergedWriteReply>("POST", `/api/merged/${kind}`, body);
      if (isCurrent(captured)) {
        setReplyState({ token, reply: answer, id, kind }); setRequestState({ token, id, kind, state: "replied" });
      }
      return answer;
    } catch (cause) {
      if (isCurrent(captured)) setRequestState({ token, id, kind, state: "failed_or_uncertain" });
      throw cause;
    }
  }
  const apply = (region: number, text: string, target: string, route: "shared" | "just-here") => perform(async captured => {
    if (captured.requiresRefresh || !view?.snapshot || !view.regions[region] || !writable.has(target) || !shown.includes(target) || (route === "shared" && view.regions[region].kind !== "shared")) throw Error("Region, snapshot, selected target or write route changed");
    const answer = await hostWrite(captured, "write", { path, targets: shown, writable: [...writable], expected: view.snapshot, region, text, route, source: target });
    if (isCurrent(captured)) { setEditState(null); setComparisonState(null); }
    await refreshedAfterWrite(captured); return answer;
  });
  const undo = (source: string) => perform(async captured => {
    const entries = result?.undo?.filter(item => item.source === source) ?? [];
    if (entries.length !== 1) throw Error("No unambiguous current undo entry for this target");
    const answer = await hostWrite(captured, "undo", { path, undo: entries });
    await refreshedAfterWrite(captured); return answer;
  });
  const inspect = () => perform(async captured => {
    if (captured.requiresRefresh || !view?.snapshot || shown.length < 2) throw Error("Select two captured worktrees for a comparison");
    const sources = await Promise.all(shown.map(async name => {
      const revision = await request<{ files: ProjectEntry[] }>("POST", "/api/revision-projects", { kind: "directory", worktree: name });
      const file = revision.files.find(entry => entry.path === path);
      if (!file || file.kind !== "file") throw Error(`${name} has no regular file ${path}`);
      const text = new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(file.bytes));
      if (text !== view.snapshot![name]) throw Error(`${name} changed since the displayed comparison; refresh before inspecting a complete project`);
      return { id: name, label: name, kind: "directory" as const, text, project: revision.files };
    }));
    if (!isCurrent(captured)) return false;
    setComparisonState({ token, model: { sources, old: sources[0].id, new: sources[1].id, policy: "neither", evidence: "unverified" } }); return true;
  });
  const refresh = () => perform(captured => load(captured));
  const revision = useUiRevision(() => [token, view, worktrees, chosen ? [...chosen] : null, [...writable], editing, comparison, result, latestRequest, busy, needsRefresh, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  const outcome = async (run: () => Promise<unknown>): Promise<UiOutcome> => {
    try { return { status: "completed", value: { response: json(await run()), scope: "Actual comparison controller reply; inspect per-target effects. No atomic multi-target write or rollback is asserted." } }; }
    catch (cause) { return { status: "unavailable", reason: String(cause) }; }
  };
  const add = (id: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${id}`, command: {
    parent, subject: id === "read" ? `${parent}:reading` : parent, label, parameters, prerequisites: ["Current file/membership lifetime and captured host snapshots; targets are independent"],
    read: () => ({ revision: revision(), enabled }), execute,
  } });
  add("read", "Read worktree comparison bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ path, loading: !current, busy, needs_refresh_after_write: needsRefresh, error,
      worktree_count: worktrees?.length ?? null,
      worktrees: (rendered ? worktrees : null)?.map(tree => ({ name: tree.name, branch: tree.branch ?? null, current: tree.current, selected: chosen ? chosen.has(tree.name) : true, writable: writable.has(tree.name), ambiguous: !unambiguous(tree.name) })) ?? null,
      view: view ? { repository: view.repository, sources: rendered ? shown : [], regions: rendered ? view.regions : [], missing: rendered ? view.missing : [], shared_lines: rendered ? view.shared_lines ?? 0 : 0, variants: rendered ? view.variants ?? 0 : 0, snapshot_available: rendered && !!view.snapshot } : null,
      latest_request: latestRequest ? { kind: latestRequest.kind, state: latestRequest.state } : null, last_reply_kind: lastReplyKind, result_is_latest: resultIsLatest,
      editing_region: editing, complete_project_comparison_open: !!comparison,
      result: result ? { targets: result.targets, undo_available: result.undo?.map(item => item.source) ?? [] } : null,
      scope: "Displayed captured file regions and per-target replies. Full callback reply has its own disclosure. A lens, not an atomic disk snapshot or independent verification." }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside this comparison reading" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("refresh", "Refresh worktree comparison", {}, !busy, () => outcome(refresh));
  if (rendered) add("inspect", "Compare complete worktree projects", {}, !busy && !needsRefresh && !!view?.snapshot && shown.length >= 2, () => outcome(inspect));
  for (const name of new Set(rendered ? worktrees!.map(tree => tree.name) : [])) {
    const tree = worktrees!.find(tree => tree.name === name)!;
    add(`select:${tree.name}`, `Select comparison worktree: ${tree.name}`, { on: { type: "boolean" } }, !busy && unambiguous(tree.name), async args => {
      try { choose(tree.name, args.on as boolean); return { status: "completed", value: { selection_requested: true, name: tree.name, selected: args.on } }; }
      catch (cause) { return { status: "unavailable", reason: String(cause) }; }
    });
    add(`write:${tree.name}`, `Set comparison worktree writable: ${tree.name}`, { on: { type: "boolean" } }, !busy && !needsRefresh && unambiguous(tree.name) && !!view?.snapshot && Object.hasOwn(view.snapshot, tree.name), async args => {
      try { allowWrite(tree.name, args.on as boolean); return { status: "completed", value: { permission_updated: true, name: tree.name, writable: args.on } }; }
      catch (cause) { return { status: "unavailable", reason: String(cause) }; }
    });
  }
  for (const [index, region] of (rendered ? view!.regions : []).entries()) add(`region:${index}`, `Toggle ${region.kind} worktree region editor ${index + 1}`, {}, !busy && !needsRefresh && !!view?.snapshot, async () => {
    try { edit(index); return { status: "completed", value: { region: index, editing_requested: editing !== index } }; }
    catch (cause) { return { status: "unavailable", reason: String(cause) }; }
  });
  for (const item of (rendered ? result?.undo ?? [] : [])) add(`undo:${item.source}`, `Undo worktree write: ${item.source}`, {}, !busy, () => outcome(() => undo(item.source)));
  const invoke = useUiCommands(commands);
  const human = (id: string, args = {}) => void invoke(`${parent}:${id}`, args).catch(cause => { if (currentToken.current === token && owner.current.live) setError(String(cause)); });
  return { view, worktrees, chosen, writable, editing, comparison, result, resultIsLatest, lastReplyKind, latestRequest, error, busy, needsRefresh, shown, human, apply,
    canSelect: (name: string) => rendered && unambiguous(name),
    canWrite: (name: string) => !needsRefresh && !!view?.snapshot && Object.hasOwn(view.snapshot, name) && unambiguous(name),
    setComparison: (model: ComparisonModel) => { if (ready()) setComparisonState({ token, model }); } };
}
</storycode:copy>

### Assemble `apps/web/src/views/useMergedViewControls.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useMergedViewControls.ts"><storycode:paste select="#f-5c0c1024bb7f-MergedWriteReply"/><storycode:paste select="#f-5c0c1024bb7f-Owner"/><storycode:paste select="#f-5c0c1024bb7f-json"/><storycode:paste select="#f-5c0c1024bb7f-useMergedViewControls"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-724d3488d157"></a>
## `apps/web/src/views/useMergedViewControls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-724d3488d157-trees"></a>
### `trees`

<storycode:copy id="f-724d3488d157-trees">
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { api, installMockHandler } from "../api/client";
import type { MergedViewResponse } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useMergedViewControls, type MergedWriteReply } from "./useMergedViewControls";
const trees = [{ path: "/fixture/repo", name: "repo", current: true }, { path: "/fixture/side", name: "side", current: false }];
</storycode:copy>
<a id="f-724d3488d157-view"></a>
### `view`

<storycode:copy id="f-724d3488d157-view">
function view(path: string, targets?: string[]): MergedViewResponse {
  const sources = trees.filter(tree => !targets || targets.includes(tree.name));
  return { repository: true, path, sources, regions: sources.length ? [{ kind: "shared", text: `${path}\n` }] : [], missing: [], snapshot: Object.fromEntries(sources.map(source => [source.name, `${path}\n`])) };
}
</storycode:copy>
<a id="f-724d3488d157-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement">
beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(api, "worktrees").mockResolvedValue({ repository: true, worktrees: trees });
  vi.spyOn(api, "merged").mockImplementation(async (path, targets) => view(path, targets));
});
</storycode:copy>
<a id="f-724d3488d157-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement-2">
afterEach(() => { cleanup(); installMockHandler(null as never); });
</storycode:copy>
<a id="f-724d3488d157-harness"></a>
### `harness`

<storycode:copy id="f-724d3488d157-harness">
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
</storycode:copy>
<a id="f-724d3488d157-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement-3">
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
</storycode:copy>
<a id="f-724d3488d157-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement-4">
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
</storycode:copy>
<a id="f-724d3488d157-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement-5">
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
</storycode:copy>
<a id="f-724d3488d157-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement-6">
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
</storycode:copy>
<a id="f-724d3488d157-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-724d3488d157-expression_statement-7">
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
</storycode:copy>

### Assemble `apps/web/src/views/useMergedViewControls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useMergedViewControls.test.tsx"><storycode:paste select="#f-724d3488d157-trees"/><storycode:paste select="#f-724d3488d157-view"/><storycode:paste select="#f-724d3488d157-expression_statement"/><storycode:paste select="#f-724d3488d157-expression_statement-2"/><storycode:paste select="#f-724d3488d157-harness"/><storycode:paste select="#f-724d3488d157-expression_statement-3"/><storycode:paste select="#f-724d3488d157-expression_statement-4"/><storycode:paste select="#f-724d3488d157-expression_statement-5"/><storycode:paste select="#f-724d3488d157-expression_statement-6"/><storycode:paste select="#f-724d3488d157-expression_statement-7"/></storycode:file>


</storycode:doc>
