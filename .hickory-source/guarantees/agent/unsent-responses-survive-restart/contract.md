<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Unsent responses survive restart

For: Authors using an agent to change documents while retaining review control.

Given text in **Your response**, editing saves recovery to personal workspace
storage. Closing the Agent pane retains it. Reopening the pane or restarting the
app restores the response to its workspace, window slot, editor subject and
conversation thread. Recovery never creates a repository file, names an untitled
editor, or writes response text into an immutable recorded session.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/unsent-responses-survive-restart.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4d7fe59844bb"></a>
## `apps/web/e2e/responseRecovery.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Unsent responses survive restart](contract.md).

<a id="f-4d7fe59844bb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4d7fe59844bb-expression_statement">
// Guarantee: docs/guarantees/agent/unsent-responses-survive-restart.md
import { expect, test } from "@playwright/test";

test('unsent response survives a fresh browser page and explicit discard removes it', async ({ page, context }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, 'Use just test-response-recovery-ui.');
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole('tablist', { name: 'Center views' })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  const agent = page.getByRole('region', { name: 'Agent chat' });
  const response = agent.getByRole('textbox', { name: 'Agent conversation and response' });
  await response.click(); await response.press('ControlOrMeta+End');
  await response.pressSequentially('Durable unsent response');
  // Observe the real server's persisted bytes, independently of the editor.
  await expect.poll(async () => (await (await page.request.get('/api/workspace/responses?doc=workspace')).json()).state?.drafts.new).toBe('Durable unsent response');
  const old = await (await page.request.get('/api/workspace/responses?doc=workspace')).json();
  expect((await page.request.put('/api/workspace/responses?doc=workspace', { data: { scope: 'another workspace', state: old.state } })).status()).toBe(409);
  const url = page.url();
  await page.close();
  const fresh = await context.newPage(); await fresh.goto(url);
  await fresh.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  const reopened = fresh.getByRole('region', { name: 'Agent chat' });
  await expect(reopened.getByRole('textbox', { name: 'Agent conversation and response' })).toContainText('Durable unsent response');
  await reopened.getByRole('button', { name: 'Discard response', exact: true }).click();
  await expect.poll(async () => (await (await fresh.request.get('/api/workspace/responses?doc=workspace')).json()).state?.drafts.new).toBe('');
  await fresh.reload();
  await fresh.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  await expect(fresh.getByRole('textbox', { name: 'Agent conversation and response' })).not.toContainText('Durable unsent response');
  // Recovery never writes its text into document drafts or session evidence.
  const drafts = await (await fresh.request.get('/api/workspace/drafts')).json();
  expect(JSON.stringify(drafts)).not.toContain('Durable unsent response');
  const turns = await (await fresh.request.get('/api/docs/workspace/agent/turns')).json();
  expect(turns.turns).toHaveLength(0);
});

</storycode:copy>
<a id="f-4d7fe59844bb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4d7fe59844bb-expression_statement-2">
test('existing thread restores when durable storage arrives before turn history', async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, 'Use just test-response-recovery-ui.');
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  const stored = await (await page.request.get('/api/workspace/responses?doc=workspace')).json();
  expect((await page.request.put('/api/workspace/responses?doc=workspace', { data: { scope: stored.scope,
    state: { version: 1, tip: 'child', drafts: { root: 'Reply to existing thread' } } } })).ok()).toBeTruthy();
  await page.route('**/api/docs/workspace/agent/turns', async route => {
    await new Promise(resolve => setTimeout(resolve, 400));
    const turn = (id: string, parent_id: string | null) => ({ id, parent_id, prompt: id, answer: 'answer', status: 'ok', error: null,
      created_at: '2026-10-17T00:00:00Z', provider: 'anthropic', model: 'test', usage: null });
    await route.fulfill({ json: { turns: [turn('root', null), turn('child', 'root')], provider: 'anthropic', model: 'test',
      totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } } });
  });
  await page.reload();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent('hickory-menu', { detail: 'show-agent' })));
  await expect(page.getByRole('textbox', { name: 'Agent conversation and response' })).toContainText('Reply to existing thread');
});
</storycode:copy>

### Assemble `apps/web/e2e/responseRecovery.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/responseRecovery.spec.ts"><storycode:paste select="#f-4d7fe59844bb-expression_statement"/><storycode:paste select="#f-4d7fe59844bb-expression_statement-2"/></storycode:file>

<a id="output-8203da095c26"></a>
## `apps/web/src/lib/responseRecovery.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Unsent responses survive restart](contract.md).

<a id="f-8203da095c26-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement">
// Guarantee: docs/guarantees/agent/unsent-responses-survive-restart.md
import { beforeEach, expect, it, vi } from "vitest";
import { ResponseRecovery, responseThread, useResponseRecovery, flushResponseRecovery } from "./responseRecovery";
import { act, renderHook, waitFor } from "@testing-library/react";
import { api } from "../api/client";
import type { AgentTurn } from "../api/types";
vi.mock("../api/client", () => ({ api: { responseDraft: vi.fn(), saveResponseDraft: vi.fn() } }));
</storycode:copy>
<a id="f-8203da095c26-turns"></a>
### `turns`

<storycode:copy id="f-8203da095c26-turns">
const turns = [{ id: "a", parent_id: null }, { id: "b", parent_id: "a" }, { id: "other", parent_id: null }] as AgentTurn[];
</storycode:copy>
<a id="f-8203da095c26-disk"></a>
### `disk`

<storycode:copy id="f-8203da095c26-disk">
let disk: unknown;
</storycode:copy>
<a id="f-8203da095c26-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-2">
beforeEach(() => {
  disk = null;
  vi.mocked(api.responseDraft).mockImplementation(async () => ({ scope: "folder/window/subject", state: structuredClone(disk) }));
  vi.mocked(api.saveResponseDraft).mockImplementation(async (_doc, _scope, state) => { disk = structuredClone(state); return { ok: true }; });
});
</storycode:copy>
<a id="f-8203da095c26-make"></a>
### `make`

<storycode:copy id="f-8203da095c26-make">
const make = (key = "workspace") => new ResponseRecovery("workspace", key, "");
</storycode:copy>
<a id="f-8203da095c26-deferred"></a>
### `deferred`

<storycode:copy id="f-8203da095c26-deferred">
function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(r => { resolve = r; }); return { promise, resolve }; }
</storycode:copy>
<a id="f-8203da095c26-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-3">
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
</storycode:copy>
<a id="f-8203da095c26-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-4">
it("keeps early typing while retaining the other recovered threads", async () => {
  const read = deferred<{ scope: string; state: unknown }>();
  vi.mocked(api.responseDraft).mockReturnValueOnce(read.promise);
  const recovery = make(); const loading = recovery.load(turns);
  recovery.edit("already typing");
  read.resolve({ scope: "scope", state: { version: 1, tip: "a", drafts: { other: "old response", a: "another response" } } });
  await loading; await recovery.flush();
  expect(recovery.text).toBe("already typing"); recovery.select("a"); expect(recovery.text).toBe("another response"); await recovery.flush();
});
</storycode:copy>
<a id="f-8203da095c26-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-5">
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
</storycode:copy>
<a id="f-8203da095c26-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-6">
it("leaves failed send text alone and preserves newer typing after acceptance", async () => {
  const recovery = make(); await recovery.load([]); recovery.edit("submit");
  const submitted = recovery.snapshot();
  // A rejected request never invokes accepted.
  await recovery.flush(); const restart = make(); await restart.load([]); expect(restart.text).toBe("submit");
  recovery.edit("new response"); recovery.accepted(submitted, "sent"); await recovery.flush();
  expect(recovery.text).toBe("new response");
  const again = make(); await again.load([{ id: "sent", parent_id: null }] as AgentTurn[]); expect(again.text).toBe("new response");
});
</storycode:copy>
<a id="f-8203da095c26-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-7">
it("reports unavailable and malformed storage without blocking typing", async () => {
  disk = { version: 55 };
  const recovery = make(); await recovery.load([]); expect(recovery.error).toContain("could not be saved");
  recovery.edit("usable"); expect(recovery.text).toBe("usable");
  vi.mocked(api.responseDraft).mockRejectedValueOnce(new Error("offline"));
  const offline = make("offline"); await offline.load([]); offline.edit("still usable"); expect(offline.error).toBeTruthy();
});
</storycode:copy>
<a id="f-8203da095c26-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-8">
it("reports write failure and retries the current revision", async () => {
  const recovery = make(); await recovery.load([]);
  vi.mocked(api.saveResponseDraft).mockRejectedValueOnce(new Error("full"));
  recovery.edit("retain me"); await new Promise(resolve => setTimeout(resolve, 0)); expect(recovery.error).toBeTruthy();
  await recovery.flush(); expect(recovery.error).toBeNull();
  const again = make(); await again.load([]); expect(again.text).toBe("retain me");
});

</storycode:copy>
<a id="f-8203da095c26-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-9">
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

</storycode:copy>
<a id="f-8203da095c26-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-10">
it("does not move another thread's response when a pending send completes", async () => {
  const recovery = make(); await recovery.load(turns);
  recovery.select("a"); recovery.edit("submitted"); const submitted = recovery.snapshot();
  recovery.select("other"); recovery.edit("belongs to other"); recovery.accepted(submitted, "accepted");
  await recovery.flush();
  expect(recovery.state.tip).toBe("other"); expect(recovery.text).toBe("belongs to other");
  recovery.select("a"); expect(recovery.text).toBe(""); await recovery.flush();
});

</storycode:copy>
<a id="f-8203da095c26-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-11">
it("restores the root draft when response storage arrives before conversation history", async () => {
  disk = { version: 1, tip: "b", drafts: { a: "reply to this thread" } };
  const recovery = make(); await recovery.load([]);
  recovery.history(turns); expect(recovery.text).toBe("reply to this thread");
  const early = make("early-history"); await early.load([]);
  early.edit("typing before history"); early.history(turns); await early.flush();
  const restarted = make("early-history"); await restarted.load(turns);
  expect(restarted.text).toBe("typing before history");
});

</storycode:copy>
<a id="f-8203da095c26-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-8203da095c26-expression_statement-12">
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
</storycode:copy>

### Assemble `apps/web/src/lib/responseRecovery.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/responseRecovery.test.ts"><storycode:paste select="#f-8203da095c26-expression_statement"/><storycode:paste select="#f-8203da095c26-turns"/><storycode:paste select="#f-8203da095c26-disk"/><storycode:paste select="#f-8203da095c26-expression_statement-2"/><storycode:paste select="#f-8203da095c26-make"/><storycode:paste select="#f-8203da095c26-deferred"/><storycode:paste select="#f-8203da095c26-expression_statement-3"/><storycode:paste select="#f-8203da095c26-expression_statement-4"/><storycode:paste select="#f-8203da095c26-expression_statement-5"/><storycode:paste select="#f-8203da095c26-expression_statement-6"/><storycode:paste select="#f-8203da095c26-expression_statement-7"/><storycode:paste select="#f-8203da095c26-expression_statement-8"/><storycode:paste select="#f-8203da095c26-expression_statement-9"/><storycode:paste select="#f-8203da095c26-expression_statement-10"/><storycode:paste select="#f-8203da095c26-expression_statement-11"/><storycode:paste select="#f-8203da095c26-expression_statement-12"/></storycode:file>

<a id="output-9fc35081631a"></a>
## `apps/web/src/lib/responseRecovery.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Unsent responses survive restart](contract.md).

<a id="f-9fc35081631a-SavedResponses"></a>
### `SavedResponses`

Personal response recovery, separate from immutable conversation evidence.

<storycode:copy id="f-9fc35081631a-SavedResponses">
// Personal response recovery, separate from immutable conversation evidence.
import { useEffect, useMemo, useState } from "react";
import { api } from "../api/client";
import type { AgentTurn } from "../api/types";

interface SavedResponses { version: 1; tip: string | null; drafts: Record<string, string> }
</storycode:copy>
<a id="f-9fc35081631a-queues"></a>
### `queues`

<storycode:copy id="f-9fc35081631a-queues">
const queues = new Map<string, Promise<unknown>>();
</storycode:copy>
<a id="f-9fc35081631a-active"></a>
### `active`

<storycode:copy id="f-9fc35081631a-active">
const active = new Set<ResponseRecovery>();

</storycode:copy>
<a id="f-9fc35081631a-flushResponseRecovery"></a>
### `flushResponseRecovery`

The native close gate waits for response writes, including closed panes.

<storycode:copy id="f-9fc35081631a-flushResponseRecovery">
/** The native close gate waits for response writes, including closed panes. */
export async function flushResponseRecovery() {
  await Promise.all([...active].map(recovery => recovery.flush()));
  while (queues.size) await Promise.allSettled([...queues.values()]);
}
</storycode:copy>
<a id="f-9fc35081631a-ordered"></a>
### `ordered`

<storycode:copy id="f-9fc35081631a-ordered">
function ordered<T>(key: string, operation: () => Promise<T>): Promise<T> {
  const next = (queues.get(key) ?? Promise.resolve()).catch(() => {}).then(operation);
  queues.set(key, next);
  void next.finally(() => { if (queues.get(key) === next) queues.delete(key); }).catch(() => {});
  return next;
}
</storycode:copy>
<a id="f-9fc35081631a-responseThread"></a>
### `responseThread`

<storycode:copy id="f-9fc35081631a-responseThread">
export function responseThread(turns: readonly AgentTurn[], tip: string | null): string {
  const seen = new Set<string>();
  let root = tip;
  while (root && !seen.has(root)) {
    seen.add(root);
    const parent = turns.find(turn => turn.id === root)?.parent_id;
    if (!parent) break;
    root = parent;
  }
  return root ?? "new";
}
</storycode:copy>
<a id="f-9fc35081631a-readSaved"></a>
### `readSaved`

<storycode:copy id="f-9fc35081631a-readSaved">
function readSaved(raw: unknown): SavedResponses {
  if (raw === null) return { version: 1, tip: null, drafts: {} };
  const value = raw as Partial<SavedResponses>;
  if (!value || value.version !== 1 || !(value.tip === null || typeof value.tip === "string") ||
    !value.drafts || typeof value.drafts !== "object" || Array.isArray(value.drafts) ||
    Object.values(value.drafts).some(text => typeof text !== "string")) throw new Error("Damaged response recovery");
  return { version: 1, tip: value.tip, drafts: { ...value.drafts } };
}

</storycode:copy>
<a id="f-9fc35081631a-ResponseRecovery"></a>
### `ResponseRecovery`

A controller outlives a pane while queued writes or a send finish.

<storycode:copy id="f-9fc35081631a-ResponseRecovery">
/** A controller outlives a pane while queued writes or a send finish. */
export class ResponseRecovery {
  state: SavedResponses = { version: 1, tip: null, drafts: {} };
  text: string;
  error: string | null = null;
  scope: string | null = null;
  turns: readonly AgentTurn[] = [];
  private changed = false;
  private followLatest = true;
  private loaded = false;
  private revision = 0;
  private textRevision = 0;
  private savedRevision = -1;
  private listeners = new Set<() => void>();
  constructor(private doc: string, private key: string, private initial: string) { this.text = initial; }
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  private notify() { this.listeners.forEach(listener => listener()); }
  private failed() { this.error = "Response recovery could not be saved. Keep this pane open or copy your response before closing."; this.notify(); }
  async load(turns: readonly AgentTurn[]) {
    this.turns = turns;
    if (!this.changed) this.state.tip = turns.at(-1)?.id ?? null;
    const early = this.text;
    try {
      await ordered(this.key, async () => {
        const result = await api.responseDraft(this.doc);
        this.scope = result.scope;
        const stored = readSaved(result.state);
        if (!this.changed) {
          this.state = stored;
          // Preserve an unknown/deleted thread identity rather than binding its
          // draft to whichever conversation happens to be newest.
          this.followLatest = result.state === null;
          if (this.followLatest) this.state.tip = this.turns.at(-1)?.id ?? null;
          this.text = this.state.drafts[responseThread(this.turns, this.state.tip)] ?? early;
        } else {
          // Typing/selection beats late restoration, but retain other threads.
          const thread = responseThread(this.turns, this.state.tip);
          this.state.drafts = { ...stored.drafts, ...this.state.drafts, [thread]: this.text };
        }
        // Complete pre-hydration edits before a remounted pane can read.
        if (this.changed) {
          const revision = this.revision;
          await api.saveResponseDraft(this.doc, this.scope, structuredClone(this.state));
          this.savedRevision = revision;
        }
        this.error = null;
      });
    } catch { this.failed(); }
    this.loaded = true;
    this.notify();
    if (this.changed && this.scope) void this.flush();
  }
  history(turns: readonly AgentTurn[]) {
    const previous = responseThread(this.turns, this.state.tip);
    this.turns = turns;
    if (this.followLatest && !this.changed) this.state.tip = turns.at(-1)?.id ?? null;
    const thread = responseThread(turns, this.state.tip);
    if (!this.changed) this.text = this.state.drafts[thread] ?? this.initial;
    else if (previous !== thread) {
      this.state.drafts[thread] = this.text;
      delete this.state.drafts[previous];
      this.revision++; void this.flush();
    }
    this.notify();
  }
  edit = (text: string) => {
    this.text = text; this.changed = true; this.revision++; this.textRevision++;
    this.state.drafts[responseThread(this.turns, this.state.tip)] = text;
    this.notify();
    // Save during editing, even if shutdown never delivers a lifecycle event.
    void this.flush();
  };
  select = (tip: string | null) => {
    this.state.tip = tip;
    this.changed = true; this.revision++;
    this.text = this.state.drafts[responseThread(this.turns, tip)] ?? "";
    this.notify(); void this.flush();
  };
  snapshot() { return { thread: responseThread(this.turns, this.state.tip), revision: this.textRevision, text: this.text }; }
  accepted(snapshot: ReturnType<ResponseRecovery["snapshot"]>, id: string) {
    // A send can finish after its pane closed and reopened. Let the live
    // owner preserve any edits/discard made since that restoration.
    const owner = [...active].reverse().find(recovery => recovery.key === this.key);
    if (owner && owner !== this) {
      owner.accepted({ ...snapshot, revision: owner.changed ? -1 : owner.textRevision }, id);
      return;
    }
    // Clear only the submitted revision. New typing (even identical text) wins.
    const currentThread = responseThread(this.turns, this.state.tip);
    if (currentThread !== snapshot.thread) {
      if (this.state.drafts[snapshot.thread] === snapshot.text) this.state.drafts[snapshot.thread] = "";
      this.changed = true; this.revision++; this.notify(); void this.flush();
      return;
    }
    if (this.textRevision === snapshot.revision) this.text = "";
    this.state.drafts[snapshot.thread] = this.text;
    if (snapshot.thread === "new" && currentThread === "new") {
      this.state.drafts[id] = this.text;
      this.state.drafts.new = "";
    }
    this.state.tip = id;
    this.changed = true; this.revision++;
    this.notify(); void this.flush();
  }
  discard = () => { this.edit(""); };
  async flush(keepalive = false) {
    if (!this.loaded) return;
    if (!this.scope) { this.loaded = false; await this.load(this.turns); return; }
    if (!this.changed) return;
    if (this.savedRevision === this.revision) { await queues.get(this.key)?.catch(() => {}); return; }
    const revision = this.revision;
    const state = structuredClone(this.state);
    const scope = this.scope;
    // Capture bytes now, and serialize across remounts as well as this pane.
    this.savedRevision = revision;
    try {
      await ordered(this.key, () => revision < this.revision ? Promise.resolve({ ok: true }) : api.saveResponseDraft(this.doc, scope, state, keepalive));
      this.error = null; this.notify();
    } catch { this.savedRevision = -1; this.failed(); }
  }
}

</storycode:copy>
<a id="f-9fc35081631a-useResponseRecovery"></a>
### `useResponseRecovery`

<storycode:copy id="f-9fc35081631a-useResponseRecovery">
export function useResponseRecovery(doc: string, workspace: string, initial: string, turns: AgentTurn[], ready: boolean) {
  const recovery = useMemo(() => new ResponseRecovery(doc, JSON.stringify([workspace, doc]), initial), [doc, workspace]);
  const [, render] = useState(0);
  useEffect(() => recovery.subscribe(() => render(value => value + 1)), [recovery]);
  useEffect(() => { void recovery.load(turns); }, [recovery]);
  useEffect(() => { if (ready) recovery.history(turns); }, [recovery, ready, turns]);
  useEffect(() => {
    active.add(recovery);
    const flush = () => { void recovery.flush(true); };
    const hide = () => { if (document.visibilityState === "hidden") flush(); };
    const timer = window.setInterval(() => void recovery.flush(), 2000);
    window.addEventListener("pagehide", flush);
    document.addEventListener("visibilitychange", hide);
    return () => { active.delete(recovery); clearInterval(timer); window.removeEventListener("pagehide", flush); document.removeEventListener("visibilitychange", hide); flush(); };
  }, [recovery]);
  return recovery;
}
</storycode:copy>

### Assemble `apps/web/src/lib/responseRecovery.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/responseRecovery.ts"><storycode:paste select="#f-9fc35081631a-SavedResponses"/><storycode:paste select="#f-9fc35081631a-queues"/><storycode:paste select="#f-9fc35081631a-active"/><storycode:paste select="#f-9fc35081631a-flushResponseRecovery"/><storycode:paste select="#f-9fc35081631a-ordered"/><storycode:paste select="#f-9fc35081631a-responseThread"/><storycode:paste select="#f-9fc35081631a-readSaved"/><storycode:paste select="#f-9fc35081631a-ResponseRecovery"/><storycode:paste select="#f-9fc35081631a-useResponseRecovery"/></storycode:file>

<a id="output-c910953d9e7f"></a>
## `apps/web/src/views/WorkspaceChat.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Unsent responses survive restart](contract.md), [New Document Creates Another Unnamed Buffer](../../authoring/new-document-is-an-act/contract.md), [Observations read beside their captured source](../../debugging/source-anchored-observation-reading/contract.md).

<a id="f-c910953d9e7f-editorTabs"></a>
### `editorTabs`

<storycode:copy id="f-c910953d9e7f-editorTabs">
import { providerObservationContext } from "../debug/providerObservations";
import { useContextProjectionActions } from "./uiContextProjectionActions";
import { captureUiWindow } from "../lib/uiActionBridge";
import "./contextProjection.css";
import { projectDocument, projectionEstimate, retainedReadingState, type ReadingProjection } from "../lib/contextProjection";
import { observationContext } from "../debug/observations";
import { bufferTarget } from "../lib/editorBufferTargets";
import { useMemo, useRef, useState, type MutableRefObject } from "react";
import { ChatDock } from "../components/ChatDock";
import { api } from "../api/client";
import { getWorkspaceRealtime, LocalRealtime } from "../api/realtime";
import type { AgentEditorContext } from "../api/agentTypes";
import { panes, type Layout, type Tab } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { bufferEditor } from "../lib/editorBufferTargets";
import { applyAgentEdit } from "../lib/agentEdit";
import type { AgentChange } from "../api/acp";
import { openPlainSearchFiles, plainSearchEditor } from "../lib/workspaceSearch";

export function editorTabs(layout: Layout): Tab[] {
  return panes(layout.root).flatMap(pane => pane.tabs).filter(tab =>
    ["document", "generated", "file", "untitled"].includes(tab.kind),
  );
}

</storycode:copy>
<a id="f-c910953d9e7f-editorContext"></a>
### `editorContext`

<storycode:copy id="f-c910953d9e7f-editorContext">
export async function editorContext(
  tabs: Tab[], focused: string | null, registry: SessionRegistry,
  untitled: Record<string, string>, plain: Map<string, string>,
  selection?: Record<string, "expanded" | "pinned">,
): Promise<AgentEditorContext> {
  const originatingWindow = await captureUiWindow();
  const visiblePlain = new Map(openPlainSearchFiles().map(file => [file.path, file.content]));
  const buffers = await Promise.all(tabs.map(async tab => {
    const session = registry.get(tab.docId);
    let content: string;
    if (tab.kind === "untitled") content = untitled[tab.id] ?? "";
    else if (tab.kind === "document") {
      content = session?.docEditor?.state.doc.toString() ?? session?.liveSource
        ?? (await api.doc(tab.docId!)).source;
    } else {
      content = plain.get(tab.id) ?? session?.openOutputs.get(tab.target)?.state.doc.toString()
        ?? visiblePlain.get(tab.target)
        ?? session?.outputs.get(tab.target)?.content ?? (await api.file(tab.target)).content;
    }
    return { id: tab.id, kind: tab.kind, ...(tab.kind === "untitled" ? { provenance_uri: bufferTarget(tab.id) } : {}), ...(tab.kind === "generated" ? { document: session?.doc?.path ?? tabs.find(t => t.kind === "document" && t.docId === tab.docId)?.target } : {}), name: tab.title ?? tab.target, path: tab.kind === "untitled" ? null : tab.target,
      observations: [...observationContext(tab.target, content), ...providerObservationContext(tab.target, content)], content, focused: tab.id === focused };
  }));
  const documents: ReadingProjection["documents"] = [];
  for (const buffer of buffers) {
    if (!selection?.[buffer.id]) continue;
    const tab = tabs.find(t => t.id === buffer.id)!;
    const session = registry.get(tab.docId);
    const view = tab.kind === "untitled" ? bufferEditor(tab.id) : tab.kind === "document" ? session?.docEditor
      : tab.kind === "generated" ? session?.openOutputs.get(tab.target) : plainSearchEditor(tab.target);
    const key = tab.kind === "untitled" ? bufferTarget(tab.id) : tab.kind === "document" ? `doc:${tab.docId}`
      : tab.kind === "generated" ? `output:${tab.docId}:${tab.target}` : `file:${tab.target}`;
    documents.push(await projectDocument(buffer.id, buffer.content, view ?? retainedReadingState(key), selection[buffer.id] === "pinned"));
  }
  const observations = [...new Map(buffers.flatMap(buffer => buffer.observations).map(record => [JSON.stringify(record), record])).values()];
  const projection: ReadingProjection = { version: 1, documents, observations, estimate: projectionEstimate({ documents, observations }) };
  return { live: true, buffers, ...(originatingWindow ? { ui_window: originatingWindow } : {}), ...(selection ? { projection } : {}) };
}

</storycode:copy>
<a id="f-c910953d9e7f-WorkspaceChat"></a>
### `WorkspaceChat`

A conversation belongs to the window, independent of which editor is active.

<storycode:copy id="f-c910953d9e7f-WorkspaceChat">
/** A conversation belongs to the window, independent of which editor is active. */
export function WorkspaceChat({ layout, registry, untitledSources, plainSources, folder, focusedEditor, onOpenSession }: {
  layout: Layout;
  registry: SessionRegistry;
  untitledSources: MutableRefObject<Record<string, string>>;
  plainSources: MutableRefObject<Map<string, string>>;
  folder: string | null;
  focusedEditor: string | null;
  onOpenSession: (path: string) => void;
}) {
  const [projectionOpen, setProjectionOpen] = useState(false);
  const projectionEpoch = useRef(0);
  const [selection, setSelection] = useState<Record<string, "expanded" | "pinned">>({});
  const [preview, setPreview] = useState<AgentEditorContext | null>(null);
  const [previewError, setPreviewError] = useState("");
  const tabs = editorTabs(layout);
  const focused = tabs.some(tab => tab.id === focusedEditor) ? focusedEditor : tabs[0]?.id ?? null;
  const realtime = useMemo(() => getWorkspaceRealtime() ?? new LocalRealtime(), []);
  const latest = useRef({ tabs, focused });
  latest.current = { tabs, focused };
  const getContext = async () => editorContext(latest.current.tabs, latest.current.focused,
    registry, untitledSources.current, plainSources.current, selection);
  function chooseContext(id: string, mode: "off" | "expanded" | "pinned") {
    projectionEpoch.current++;
    setSelection(current => { const next = { ...current }; if (mode === "off") delete next[id]; else next[id] = mode; return next; });
    setPreview(null); setPreviewError("");
  }
  async function refreshProjection() {
    const epoch = ++projectionEpoch.current;
    const captureTabs = JSON.stringify(latest.current);
    setPreview(null); setPreviewError("");
    try {
      const captured = await getContext();
      if (projectionEpoch.current === epoch && captureTabs === JSON.stringify(latest.current)) setPreview(captured);
    } catch (error) { if (projectionEpoch.current === epoch) setPreviewError(String(error)); }
  }
  const projectionActions = useContextProjectionActions({ open: projectionOpen, setOpen: setProjectionOpen, tabs, selection, choose: chooseContext, preview: preview?.projection ?? null, error: previewError, refresh: refreshProjection });
  const applyAgentChange = (change: AgentChange) => {
    const target = latest.current.tabs.find(tab => tab.id === (change.buffer ?? change.name));
    if (!target) throw new Error("The target editor was closed. Reopen it and ask the agent again.");
    if ((target.kind === "untitled" ? null : target.target) !== change.path) throw new Error("The target editor was saved or renamed. Nothing was applied to that editor; preserve the revision and review its destination.");
    const session = registry.get(target.docId);
    const view = target.kind === "untitled" ? bufferEditor(target.id)
      : target.kind === "document" ? session?.docEditor
      : target.kind === "generated" ? session?.openOutputs.get(target.target)
      : plainSearchEditor(target.target);
    applyAgentEdit(view, change);
  };
  return <div className="workspace-chat"><details className="chat-note" open={projectionOpen} onToggle={event => setProjectionOpen(event.currentTarget.open)}><summary onClick={event => { event.preventDefault(); void projectionActions.invoke("visibility", { open: !projectionOpen }); }}>Next context projection</summary>
    <p>Select documents explicitly. Expanded text excludes folds; pin includes the complete document. Unavailable editor fold state omits text until reopened or pinned. Collapsing changes future selection; previous exposure remains in history.</p>
    {tabs.map(tab => <div key={tab.id}><label><input type="checkbox" checked={!!selection[tab.id]} onChange={event => void projectionActions.invoke("choose", { id: tab.id, mode: event.target.checked ? "expanded" : "off" })} />{tab.title ?? tab.target}</label>
      <label><input type="checkbox" disabled={!selection[tab.id]} checked={selection[tab.id] === "pinned"} onChange={event => void projectionActions.invoke("choose", { id: tab.id, mode: event.target.checked ? "pinned" : "expanded" })} />Pin full text</label></div>)}
    <button onClick={() => void projectionActions.invoke("refresh")}>Refresh preview</button>
    {previewError && <p role="alert">{previewError}</p>}
    <p>Preview is a capture, not a delivery receipt. Send captures selection again. Request history, instructions and tools add context; actual requests are recorded in the session.</p>
    {preview?.projection && <pre aria-label="Context projection preview">{JSON.stringify(preview.projection, null, 2)}</pre>}
  </details><ChatDock key={folder ?? "window"} docId="workspace" recoveryWorkspace={folder ?? "window"} applyAgentChange={applyAgentChange} realtime={realtime} getContext={getContext}
    contextLabel={[folder, ...tabs.map(tab => `${tab.title ?? tab.target}${tab.kind === "untitled" ? " (unsaved)" : ""}`)].filter(Boolean).join(" · ") || "No open editors or folder"}
    onOpenSession={onOpenSession}
    onAgentFinished={() => {
      registry.all().forEach(session => session.refresh());
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    }} /></div>;
}
</storycode:copy>

### Assemble `apps/web/src/views/WorkspaceChat.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/WorkspaceChat.tsx"><storycode:paste select="#f-c910953d9e7f-editorTabs"/><storycode:paste select="#f-c910953d9e7f-editorContext"/><storycode:paste select="#f-c910953d9e7f-WorkspaceChat"/></storycode:file>

<a id="output-e31b18afaad0"></a>
## `crates/hickory-cli/src/serve/workspace.rs`

Related contracts: [Unsent responses survive restart](contract.md).

<a id="f-e31b18afaad0-store"></a>
### `store`

What you had open, and what you had not saved.
Five routes over [`hickory_workspace::WorkspaceStore`], which writes under
the user's own data directory rather than inside the project. The reason is
in that crate's header and worth repeating here, because this is the file
somebody would reach for when they wanted to "just put it in `.hickory/`":
a draft is unfinished work its author has not decided to keep, and a
`.gitignore` entry is a promise this tool cannot keep on somebody else's
machine.
Nothing here is required for the app to work. A store that cannot be
opened — a read-only home directory, a platform with no data directory —
degrades to "the window forgets its layout", never to a startup failure.
See `.instructions/config-and-environments.md`.

<storycode:copy id="f-e31b18afaad0-store">
//! What you had open, and what you had not saved.
//!
//! Five routes over [`hickory_workspace::WorkspaceStore`], which writes under
//! the user's own data directory rather than inside the project. The reason is
//! in that crate's header and worth repeating here, because this is the file
//! somebody would reach for when they wanted to "just put it in `.hickory/`":
//! a draft is unfinished work its author has not decided to keep, and a
//! `.gitignore` entry is a promise this tool cannot keep on somebody else's
//! machine.
//!
//! Nothing here is required for the app to work. A store that cannot be
//! opened — a read-only home directory, a platform with no data directory —
//! degrades to "the window forgets its layout", never to a startup failure.
//! See `.instructions/config-and-environments.md`.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use hickory_workspace::{Draft, WorkspaceStore};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// The store for the served folder.
///
/// Opened per request rather than held on [`LocalState`]: it is a path and a
/// `create_dir_all`, these routes fire a handful of times a session, and a
/// handle cached at boot would keep pointing at the old folder after the
/// desktop shell switched projects.
fn store(state: &LocalState) -> ApiResult<WorkspaceStore> {
    WorkspaceStore::for_project(state.index.root())
        .and_then(|store| match &state.window_slot {
            Some(slot) => store.window(slot),
            None => Ok(store),
        })
        .map_err(|e| {
            ApiError::unavailable(format!(
                "{e:#}\n  The app works without this — it will just forget which tabs \
             were open the next time it starts."
            ))
        })
}

</storycode:copy>
<a id="f-e31b18afaad0-get_ui"></a>
### `get_ui`

`GET /api/workspace/ui` — the stored layout, or null.

<storycode:copy id="f-e31b18afaad0-get_ui">
/// `GET /api/workspace/ui` — the stored layout, or null.
pub async fn get_ui(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let stored = store(&state)?.load_ui();
    Ok(Json(json!({ "state": stored })))
}

</storycode:copy>
<a id="f-e31b18afaad0-PutUi"></a>
### `PutUi`

<storycode:copy id="f-e31b18afaad0-PutUi">
#[derive(Deserialize)]
pub struct PutUi {
    /// Opaque to the server: which tabs sit in which panes is a shape the UI
    /// owns, and a second definition here would be one more thing to keep in
    /// step for no benefit.
    pub state: Value,
}

</storycode:copy>
<a id="f-e31b18afaad0-put_ui"></a>
### `put_ui`

`PUT /api/workspace/ui` — replace the stored layout.

<storycode:copy id="f-e31b18afaad0-put_ui">
/// `PUT /api/workspace/ui` — replace the stored layout.
pub async fn put_ui(
    State(state): State<LocalState>,
    Json(body): Json<PutUi>,
) -> ApiResult<Json<Value>> {
    store(&state)?
        .save_ui(&body.state)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

</storycode:copy>
<a id="f-e31b18afaad0-list_drafts"></a>
### `list_drafts`

`GET /api/workspace/drafts` — every buffer with unsaved changes.
Each carries the bytes it was taken from, so the page can tell "the file
is untouched, just restore it" from "the file moved on, this needs a
merge" without a second round trip.

<storycode:copy id="f-e31b18afaad0-list_drafts">
/// `GET /api/workspace/drafts` — every buffer with unsaved changes.
///
/// Each carries the bytes it was taken from, so the page can tell "the file
/// is untouched, just restore it" from "the file moved on, this needs a
/// merge" without a second round trip.
pub async fn list_drafts(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "drafts": store(&state)?.list_drafts() })))
}

</storycode:copy>
<a id="f-e31b18afaad0-PutDraft"></a>
### `PutDraft`

<storycode:copy id="f-e31b18afaad0-PutDraft">
#[derive(Deserialize)]
pub struct PutDraft {
    pub path: String,
    pub contents: String,
    /// The file's contents when this editing session began.
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub saved_at: u64,
}

</storycode:copy>
<a id="f-e31b18afaad0-put_draft"></a>
### `put_draft`

`PUT /api/workspace/drafts` — write one buffer down.

<storycode:copy id="f-e31b18afaad0-put_draft">
/// `PUT /api/workspace/drafts` — write one buffer down.
pub async fn put_draft(
    State(state): State<LocalState>,
    Json(body): Json<PutDraft>,
) -> ApiResult<Json<Value>> {
    store(&state)?
        .save_draft(&Draft {
            path: body.path,
            contents: body.contents,
            base: body.base,
            saved_at: body.saved_at,
        })
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

</storycode:copy>
<a id="f-e31b18afaad0-DraftQuery"></a>
### `DraftQuery`

<storycode:copy id="f-e31b18afaad0-DraftQuery">
#[derive(Deserialize)]
pub struct DraftQuery {
    pub path: String,
}

</storycode:copy>
<a id="f-e31b18afaad0-discard_draft"></a>
### `discard_draft`

`DELETE /api/workspace/drafts?path=…` — the buffer was saved, or the
reader threw the draft away.

<storycode:copy id="f-e31b18afaad0-discard_draft">
/// `DELETE /api/workspace/drafts?path=…` — the buffer was saved, or the
/// reader threw the draft away.
pub async fn discard_draft(
    State(state): State<LocalState>,
    Query(q): Query<DraftQuery>,
) -> ApiResult<Json<Value>> {
    store(&state)?.discard_draft(&q.path)?;
    Ok(Json(json!({ "ok": true })))
}

</storycode:copy>
<a id="f-e31b18afaad0-ResponseQuery"></a>
### `ResponseQuery`

<storycode:copy id="f-e31b18afaad0-ResponseQuery">
#[derive(Deserialize)]
pub struct ResponseQuery {
    pub doc: String,
}

</storycode:copy>
<a id="f-e31b18afaad0-response_store"></a>
### `response_store`

<storycode:copy id="f-e31b18afaad0-response_store">
fn response_store(state: &LocalState, doc: &str) -> ApiResult<(WorkspaceStore, String, String)> {
    let store = store(state)?.responses()?;
    let subject = super::agent_context::subject(state, doc)?
        .to_string_lossy()
        .into_owned();
    // Reject a queued write if a blank window attached a different folder.
    let scope = serde_json::to_string(&(store.dir(), &subject)).map_err(anyhow::Error::from)?;
    Ok((store, subject, scope))
}

</storycode:copy>
<a id="f-e31b18afaad0-get_response"></a>
### `get_response`

<storycode:copy id="f-e31b18afaad0-get_response">
pub async fn get_response(
    State(state): State<LocalState>,
    Query(q): Query<ResponseQuery>,
) -> ApiResult<Json<Value>> {
    let (store, subject, scope) = response_store(&state, &q.doc)?;
    let draft = store.response(&subject)?;
    let saved = draft
        .map(|draft| serde_json::from_str::<Value>(&draft.contents))
        .transpose()
        .map_err(|e| ApiError::unprocessable(format!("Response recovery is damaged: {e}")))?;
    Ok(Json(json!({ "scope": scope, "state": saved })))
}

</storycode:copy>
<a id="f-e31b18afaad0-PutResponse"></a>
### `PutResponse`

<storycode:copy id="f-e31b18afaad0-PutResponse">
#[derive(Deserialize)]
pub struct PutResponse {
    pub scope: String,
    pub state: Value,
}

</storycode:copy>
<a id="f-e31b18afaad0-put_response"></a>
### `put_response`

<storycode:copy id="f-e31b18afaad0-put_response">
pub async fn put_response(
    State(state): State<LocalState>,
    Query(q): Query<ResponseQuery>,
    Json(body): Json<PutResponse>,
) -> ApiResult<Json<Value>> {
    let (store, subject, scope) = response_store(&state, &q.doc)?;
    if body.scope != scope {
        return Err(ApiError::conflict(
            "The workspace changed. Response recovery was not written.",
        ));
    }
    store.save_draft(&Draft {
        path: subject,
        contents: serde_json::to_string(&body.state).map_err(anyhow::Error::from)?,
        base: String::new(),
        saved_at: 0,
    })?;
    Ok(Json(json!({ "ok": true })))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/workspace.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/workspace.rs"><storycode:paste select="#f-e31b18afaad0-store"/><storycode:paste select="#f-e31b18afaad0-get_ui"/><storycode:paste select="#f-e31b18afaad0-PutUi"/><storycode:paste select="#f-e31b18afaad0-put_ui"/><storycode:paste select="#f-e31b18afaad0-list_drafts"/><storycode:paste select="#f-e31b18afaad0-PutDraft"/><storycode:paste select="#f-e31b18afaad0-put_draft"/><storycode:paste select="#f-e31b18afaad0-DraftQuery"/><storycode:paste select="#f-e31b18afaad0-discard_draft"/><storycode:paste select="#f-e31b18afaad0-ResponseQuery"/><storycode:paste select="#f-e31b18afaad0-response_store"/><storycode:paste select="#f-e31b18afaad0-get_response"/><storycode:paste select="#f-e31b18afaad0-PutResponse"/><storycode:paste select="#f-e31b18afaad0-put_response"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/folder_session.rs](../../../mechanisms/apps/desktop/src-tauri/src/folder_session.rs.md#output-46ac251403d6)
- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/lib/responseRecovery.ts](contract.md#output-9fc35081631a)
- [apps/web/src/views/WorkspaceChat.tsx](contract.md#output-c910953d9e7f)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-0a84f897aa4c)
- [crates/hickory-cli/src/serve/agent_context.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent_context.rs.md#output-792dad4f8311)
- [crates/hickory-cli/src/serve/workspace.rs](contract.md#output-e31b18afaad0)
- [crates/hickory-workspace/src/lib.rs](../../../mechanisms/crates/hickory-workspace/src/lib.rs.md#output-a9e0f815de73)

### Tests and fixtures

- [apps/web/e2e/responseRecovery.spec.ts](contract.md#output-4d7fe59844bb)
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [apps/web/src/lib/responseRecovery.test.ts](contract.md#output-8203da095c26)
- [apps/web/src/views/WorkspaceChat.test.tsx](../../../mechanisms/apps/web/src/views/WorkspaceChat.test.tsx.md#output-41ee32c19327)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
