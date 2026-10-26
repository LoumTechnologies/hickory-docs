<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/RefactorBadge.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c40aef0d3c52"></a>
## `apps/web/src/components/RefactorBadge.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Refactor Baseline Reports The Moment An Output Would Move](../../../../../guarantees/authoring/a-refactor-baseline-reports-drift/contract.md), [Run and test from the pane menu without adding a toolbar row](../../../../../guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows/contract.md).

<a id="f-c40aef0d3c52-POLL_MS"></a>
### `POLL_MS`

The output baseline, as document-toolbar chrome.
What it does: pins the document's current woven outputs on the server
(POST /refactor/begin), then polls the live verdict — weave-only, no cell
executes — and says either "Outputs match" or which outputs moved.
Dropping the baseline makes the current outputs the new truth. It is the
same check `hick equiv` runs.
**Not called "Refactor" any more, and that was worth changing.** In every
IDE a developer has used, "Refactor" opens a menu of rename / extract /
inline — code actions on a symbol. Here it started a mode. A word that
means something specific and different in every other editor is a word
that costs a person one wrong click and a paragraph of documentation to
recover from, and the thing it names is better described by what it
produces: a baseline.
The baseline is session state on the server, so the badge resumes after a
tab close and re-open by asking for status on mount.

<storycode:copy id="f-c40aef0d3c52-POLL_MS">
// The output baseline, as document-toolbar chrome.
//
// What it does: pins the document's current woven outputs on the server
// (POST /refactor/begin), then polls the live verdict — weave-only, no cell
// executes — and says either "Outputs match" or which outputs moved.
// Dropping the baseline makes the current outputs the new truth. It is the
// same check `hick equiv` runs.
//
// **Not called "Refactor" any more, and that was worth changing.** In every
// IDE a developer has used, "Refactor" opens a menu of rename / extract /
// inline — code actions on a symbol. Here it started a mode. A word that
// means something specific and different in every other editor is a word
// that costs a person one wrong click and a paragraph of documentation to
// recover from, and the thing it names is better described by what it
// produces: a baseline.
//
// The baseline is session state on the server, so the badge resumes after a
// tab close and re-open by asking for status on mount.

import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome, UiValue } from "../lib/uiActions";
import { api } from "../api/client";
import type { RefactorStatus } from "../api/types";
import { refactorDetail, refactorSummary } from "../lib/refactorSummary";

/** How often the live verdict refreshes while a baseline is pinned. The
 * check weaves the document server-side (cached, never executing), so this
 * is cheap — and a badge that lags keystrokes by a couple of seconds is
 * still telling the truth about the last settled text. */
const POLL_MS = 2500;

</storycode:copy>
<a id="f-c40aef0d3c52-Baseline"></a>
### `Baseline`

<storycode:copy id="f-c40aef0d3c52-Baseline">
export interface Baseline {
  scope: string;
  status: RefactorStatus | null;
  active: boolean;
  busy: boolean;
  error: string | null;
  begin: () => Promise<UiOutcome>;
  end: () => Promise<UiOutcome>;
  refresh: () => Promise<UiOutcome>;
}

</storycode:copy>
<a id="f-c40aef0d3c52-useBaseline"></a>
### `useBaseline`

The baseline for one document: its live verdict, and the two verbs.
A hook rather than state inside the badge, because the badge is no longer
the only thing that needs it — the toolbar's overflow menu offers the verb
while the badge shows only the verdict, and both have to be looking at the
same baseline.

<storycode:copy id="f-c40aef0d3c52-useBaseline">
/**
 * The baseline for one document: its live verdict, and the two verbs.
 *
 * A hook rather than state inside the badge, because the badge is no longer
 * the only thing that needs it — the toolbar's overflow menu offers the verb
 * while the badge shows only the verdict, and both have to be looking at the
 * same baseline.
 */
export function useBaseline(docId: string): Baseline {
  const scope = useMemo(() => crypto.randomUUID(), [docId]);
  const [snapshot, setSnapshot] = useState<{ owner: string; status: RefactorStatus } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const currentDoc = useRef(docId); currentDoc.current = docId;
  const life = useRef({ owner: docId, live: false, sequence: 0, pending: false });
  const status = snapshot?.owner === docId ? snapshot.status : null;
  const request = useCallback(async (kind: "status" | "begin" | "end"): Promise<UiOutcome> => {
    const captured = life.current;
    if (!captured.live || captured.owner !== docId || currentDoc.current !== docId || captured.pending)
      return { status: "unavailable", reason: "Baseline owner is closed, changed or busy" };
    const ticket = ++captured.sequence;
    if (kind !== "status") { captured.pending = true; setBusy(true); }
    try {
      const response = await (kind === "status" ? api.refactorStatus(docId) : kind === "begin" ? api.refactorBegin(docId) : api.refactorEnd(docId));
      if (captured.live && currentDoc.current === docId && captured.sequence === ticket) { setSnapshot({ owner: docId, status: response }); setError(null); }
      return { status: "completed", value: { doc_id: docId, response: JSON.parse(JSON.stringify(response)) as UiValue,
        scope: "Actual baseline API reply. Weave-only comparison; no cell execution or independent output verification." } };
    } catch (cause) {
      const reason = `${String(cause)}. Inspect this document's baseline before retrying; no rollback is asserted.`;
      if (captured.live && currentDoc.current === docId && captured.sequence === ticket) setError(reason);
      return { status: "unavailable", reason };
    } finally { if (kind !== "status") { captured.pending = false; if (captured.live && currentDoc.current === docId) setBusy(false); } }
  }, [docId]);
  useEffect(() => {
    const captured = { owner: docId, live: true, sequence: 0, pending: false }; life.current = captured;
    setSnapshot(null); setBusy(false); setError(null);
    void request("status");
    return () => { captured.live = false; captured.sequence++; };
  }, [docId, request]);
  const active = status?.active === true;
  useEffect(() => {
    if (!active) return;
    const timer = setInterval(() => { void request("status"); }, POLL_MS);
    return () => clearInterval(timer);
  }, [active, request]);
  return { scope, status, active, busy: life.current.owner === docId && busy, error: life.current.owner === docId ? error : null, begin: () => request("begin"), end: () => request("end"), refresh: () => request("status") };
}

</storycode:copy>
<a id="f-c40aef0d3c52-RefactorBadge"></a>
### `RefactorBadge`

The verdict, while a baseline is pinned. Nothing at all when none is.
An idle mode used to advertise itself with a button in the toolbar; now it
lives in the overflow menu, and this slot stays empty until there is
something true to say in it.

<storycode:copy id="f-c40aef0d3c52-RefactorBadge">
/**
 * The verdict, while a baseline is pinned. Nothing at all when none is.
 *
 * An idle mode used to advertise itself with a button in the toolbar; now it
 * lives in the overflow menu, and this slot stays empty until there is
 * something true to say in it.
 */
export function RefactorBadge({ baseline }: { baseline: Baseline }) {
  const { status, busy, error, end, refresh } = baseline;
  const parent = `output-baseline:${useId()}`;
  const revision = useUiRevision(() => [baseline.scope, status, busy, error]);
  useUiCommands([
    ...(status?.active ? [{ key: `${parent}:end`, command: { parent, subject: parent, label: "Stop comparing outputs to baseline", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy }), execute: end } }] : []),
    ...(status?.active || error ? [{ key: `${parent}:refresh`, command: { parent, subject: parent, label: "Refresh output baseline verdict", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy }), execute: refresh } }] : []),
  ]);
  useUiReading("output baseline verdict", { active: status?.active ?? null, busy, error, summary: refactorSummary(status), detail: refactorDetail(status),
    scope: "Last host-observed weave-only comparison; no cell execution or independent verification" });
  if (!status || !status.active) {
    return error ? (
      <span className="error refactor-badge__error" role="status">
        {error}<button type="button" disabled={busy} onClick={() => void refresh()}>Refresh baseline verdict</button>
      </span>
    ) : null;
  }

  const summary = refactorSummary(status);
  const detail = refactorDetail(status);
  return (
    <>
      <span
        className={`refactor-badge ${status.clean ? "refactor-badge--clean" : "refactor-badge--dirty"}`}
        role="status"
        {...(detail ? { "data-tip": detail } : {})}
      >
        {status.clean ? "✓ " : "△ "}
        {summary}
      </span>
      <button className="btn" disabled={busy} onClick={() => void refresh()}>Refresh baseline verdict</button>
      <button
        className="btn"
        disabled={busy}
        onClick={() => void end()}
        data-tip="Stop comparing. The outputs as they stand become the new baseline."
      >
        Done
      </button>
      {error && (
        <span className="error refactor-badge__error" role="status">
          {error}
        </span>
      )}
    </>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/RefactorBadge.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/RefactorBadge.tsx"><storycode:paste select="#f-c40aef0d3c52-POLL_MS"/><storycode:paste select="#f-c40aef0d3c52-Baseline"/><storycode:paste select="#f-c40aef0d3c52-useBaseline"/><storycode:paste select="#f-c40aef0d3c52-RefactorBadge"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-3825e5aa0fc1"></a>
## `apps/web/src/components/RefactorBadge.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3825e5aa0fc1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3825e5aa0fc1-expression_statement">
import { act, cleanup, fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import type { RefactorStatus } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { RefactorBadge, useBaseline } from "./RefactorBadge";
afterEach(cleanup);
</storycode:copy>
<a id="f-3825e5aa0fc1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3825e5aa0fc1-expression_statement-2">
beforeEach(() => { vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-3825e5aa0fc1-active"></a>
### `active`

<storycode:copy id="f-3825e5aa0fc1-active">
const active: RefactorStatus = { active: true, started_at: "fixture", clean: false, diffs: [{ path: "out.txt", kind: "changed" }] };
</storycode:copy>
<a id="f-3825e5aa0fc1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-3825e5aa0fc1-expression_statement-3">
it("refuses old-document replies and cached callbacks after changing owners", async () => {
  let finishOld!: (status: RefactorStatus) => void;
  vi.spyOn(api, "refactorStatus").mockImplementation(doc => doc === "a" ? new Promise(resolve => { finishOld = resolve; }) : Promise.resolve({ active: false }));
  const begin = vi.spyOn(api, "refactorBegin").mockResolvedValue(active);
  const hook = renderHook(({ doc }) => useBaseline(doc), { initialProps: { doc: "a" } });
  const oldBegin = hook.result.current.begin, oldScope = hook.result.current.scope;
  hook.rerender({ doc: "b" });
  await waitFor(() => expect(hook.result.current.status).toEqual({ active: false }));
  await act(async () => { finishOld(active); });
  expect(hook.result.current.status).toEqual({ active: false });
  expect(hook.result.current.scope).not.toBe(oldScope);
  await act(async () => { expect(await oldBegin()).toMatchObject({ status: "unavailable" }); });
  expect(begin).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-3825e5aa0fc1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-3825e5aa0fc1-expression_statement-4">
it("excludes duplicate writes and prevents an old status poll from overwriting a mutation", async () => {
  let finishStatus!: (status: RefactorStatus) => void;
  let finishBegin!: (status: RefactorStatus) => void;
  vi.spyOn(api, "refactorStatus").mockImplementation(() => new Promise(resolve => { finishStatus = resolve; }));
  const begin = vi.spyOn(api, "refactorBegin").mockImplementation(() => new Promise(resolve => { finishBegin = resolve; }));
  const end = vi.spyOn(api, "refactorEnd");
  const hook = renderHook(() => useBaseline("a"));
  let pending!: ReturnType<typeof hook.result.current.begin>;
  await act(async () => { pending = hook.result.current.begin(); });
  await act(async () => { expect(await hook.result.current.end()).toMatchObject({ status: "unavailable" }); });
  expect(begin).toHaveBeenCalledTimes(1); expect(end).not.toHaveBeenCalled();
  await act(async () => { finishBegin(active); expect(await pending).toMatchObject({ status: "completed", value: { doc_id: "a", response: active } }); });
  await act(async () => { finishStatus({ active: false }); });
  expect(hook.result.current.status).toEqual(active);
});
</storycode:copy>
<a id="f-3825e5aa0fc1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-3825e5aa0fc1-expression_statement-5">
it("shares visible verdict reading and stop/refresh with actual mocked replies and failures", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.spyOn(api, "refactorStatus").mockResolvedValue(active);
  const end = vi.spyOn(api, "refactorEnd").mockRejectedValueOnce(Error("actual refusal")).mockResolvedValueOnce({ active: false });
  function Harness() { const baseline = useBaseline("a"); return <RefactorBadge baseline={baseline} />; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  await screen.findByText(/1 output differs/);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  const reading = await run("Read output baseline verdict bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes))).toContain("out.txt (changed)");
  expect(await run("Stop comparing outputs to baseline")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("actual refusal") } });
  expect(screen.getByText(/actual refusal/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Done" }));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Done" })).toBeNull());
  expect(end).toHaveBeenCalledTimes(2);
  expect(actions().some(action => action.label === "Stop comparing outputs to baseline")).toBe(false);
});
</storycode:copy>

### Assemble `apps/web/src/components/RefactorBadge.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/RefactorBadge.test.tsx"><storycode:paste select="#f-3825e5aa0fc1-expression_statement"/><storycode:paste select="#f-3825e5aa0fc1-expression_statement-2"/><storycode:paste select="#f-3825e5aa0fc1-active"/><storycode:paste select="#f-3825e5aa0fc1-expression_statement-3"/><storycode:paste select="#f-3825e5aa0fc1-expression_statement-4"/><storycode:paste select="#f-3825e5aa0fc1-expression_statement-5"/></storycode:file>


</storycode:doc>
