<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useFleetPairingControls.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-73bda5132042"></a>
## `apps/web/src/views/useFleetPairingControls.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-73bda5132042-value"></a>
### `value`

<storycode:copy id="f-73bda5132042-value">
import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import { pairingPending, type FleetPairingJob } from "../api/fleetPairings";
import { useUiCommands, useUiRevision, useUiScope } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome, UiValue } from "../lib/uiActions";

const value = (job: FleetPairingJob): UiValue => JSON.parse(JSON.stringify(job));
</storycode:copy>
<a id="f-73bda5132042-Attempt"></a>
### `Attempt`

<storycode:copy id="f-73bda5132042-Attempt">
type Attempt = { id: string; mode: "host" | "dial"; phrase: string; cancel: boolean };
</storycode:copy>
<a id="f-73bda5132042-useFleetPairingControls"></a>
### `useFleetPairingControls`

A transport failure keeps the exact retry identity; it never invents nonexecution.

<storycode:copy id="f-73bda5132042-useFleetPairingControls">
/** A transport failure keeps the exact retry identity; it never invents nonexecution. */
export function useFleetPairingControls(parent: string, onPaired: () => void) {
  const scope = useUiScope();
  const owner = scope?.window ?? null;
  const [jobs, setJobs] = useState<FleetPairingJob[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [uncertain, setUncertain] = useState(false);
  const epoch = useRef(0);
  const lock = useRef(false), attempt = useRef<Attempt | null>(null);
  const latest = useRef(jobs); latest.current = jobs;
  const lifetime = useRef<{ owner: string | null; live: boolean }>({ owner, live: true });
  const paired = useRef(onPaired); paired.current = onPaired;
  const adopt = (job: FleetPairingJob) => {
    const previous = latest.current.find(item => item.id === job.id);
    if (previous && !pairingPending(previous) && pairingPending(job)) return;
    if (previous?.cancel_requested) job = { ...job, cancel_requested: true };
    const next = [...latest.current.filter(item => item.id !== job.id), job];
    latest.current = next; setJobs(next);
    if (job.status === "paired" && previous?.status !== "paired") paired.current();
  };
  useEffect(() => {
    const life = { owner, live: true }; lifetime.current = life;
    latest.current = []; setJobs([]); attempt.current = null; lock.current = false;
    setPending(false); setUncertain(false); setError(null);
    const refresh = async () => {
      if (!owner) return;
      try {
        const response = await api.fleetPairings(owner);
        if (!life.live) return;
        response.jobs.filter(job => !latest.current.some(existing => existing.id === job.id)).forEach(adopt); setError(null);
      } catch (cause) { if (life.live) setError(String(cause)); }
    };
    void refresh();
    return () => {
      life.live = false; epoch.current++;
      if (attempt.current) attempt.current.cancel = true;
      if (owner) for (const job of latest.current.filter(pairingPending)) void api.cancelFleetPairing(owner, job.id).catch(() => {});
    };
  }, [owner]);
  useEffect(() => {
    const life = lifetime.current;
    const active = jobs.filter(pairingPending);
    if (!owner || !active.length) return;
    const timer = setTimeout(async () => {
      const version = epoch.current;
      try {
        for (const job of active) {
          const response = await api.fleetPairing(owner, job.id);
          if (!life.live || epoch.current !== version) return;
          adopt(response);
        }
      } catch (cause) { if (life.live) setError(String(cause)); }
    }, 800);
    return () => clearTimeout(timer);
  }, [owner, jobs]);
  const start = async (mode: "host" | "dial", phrase: string, signal?: AbortSignal): Promise<UiOutcome> => {
    const life = lifetime.current;
    if (signal?.aborted) return { status: "unavailable", reason: "Pairing start cancelled before submission" };
    if (!owner || life.owner !== owner || !life.live) return { status: "unavailable", reason: "Connect this UI window before pairing; no legacy fallback is used" };
    if (lock.current || latest.current.some(pairingPending)) return { status: "unavailable", reason: "Inspect or cancel the active pairing request first" };
    let captured = attempt.current;
    if (captured && (captured.mode !== mode || captured.phrase !== phrase)) return { status: "requires_input", request: `${parent}:pairing-retry`, reason: "The previous start outcome is unknown. Retry the same phrase or refresh and inspect its request before changing arguments" };
    if (!captured) { captured = { id: crypto.randomUUID(), mode, phrase, cancel: false }; attempt.current = captured; }
    const cancelStart = () => { captured!.cancel = true; };
    signal?.addEventListener("abort", cancelStart, { once: true });
    lock.current = true; epoch.current++; setPending(true);
    try {
      const response = await api.startFleetPairing(owner, captured.id, mode, phrase);
      if (!life.live || captured.cancel) {
        if (pairingPending(response.job)) void api.cancelFleetPairing(owner, response.job.id).catch(() => {});
      }
      if (life.live) { adopt(response.job); setError(null); setUncertain(false); attempt.current = null; }
      return pairingPending(response.job) ? { status: response.job.status as "queued" | "running", job: response.job.id, value: value(response.job) } : { status: "completed", value: value(response.job) };
    } catch (cause) {
      const status = (cause as { status?: number })?.status;
      const refused = typeof status === "number" && status >= 400 && status < 500;
      if (life.live) {
        if (refused) attempt.current = null;
        setError(refused ? String(cause) : `${String(cause)}. Start outcome unknown; retry uses the same request ID.`); setUncertain(!refused);
      }
      return { status: "unavailable", reason: `${String(cause)}; start outcome unknown, inspect before changing the request` };
    } finally { signal?.removeEventListener("abort", cancelStart); if (life.live) { lock.current = false; setPending(false); } }
  };
  const refresh = async (): Promise<UiOutcome> => {
    const life = lifetime.current;
    if (!owner || !life.live) return { status: "unavailable", reason: "Originating window is unavailable" };
    try {
      const version = ++epoch.current;
      const response = await api.fleetPairings(owner);
      if (!life.live || version !== epoch.current) return { status: "unavailable", reason: "Originating window closed" };
      const previouslyPaired = new Set(latest.current.filter(job => job.status === "paired").map(job => job.id));
      latest.current = response.jobs; setJobs(response.jobs);
      if (response.jobs.some(job => job.status === "paired" && !previouslyPaired.has(job.id))) paired.current();
      const found = response.jobs.find(job => job.request_id === attempt.current?.id);
      if (found) { attempt.current = null; setUncertain(false); }
      setError(null);
      return { status: "completed", value: { refreshed: true, requests: response.jobs.length } };
    } catch (cause) { setError(String(cause)); return { status: "unavailable", reason: String(cause) }; }
  };
  const revision = useUiRevision(() => [jobs, pending, uncertain, error]);
  const commands = [{ id: "refresh", label: "Refresh fleet pairing requests", enabled: !pending, execute: refresh },
    { id: "cancel-start", label: "Request cancellation of pending fleet pairing start", enabled: pending || uncertain, execute: async (): Promise<UiOutcome> => {
      if (!attempt.current) return { status: "unavailable", reason: "No pending pairing start" };
      attempt.current.cancel = true;
      return { status: "completed", value: { cancellation_requested: true, scope: "Forwarded when the host job identity is recovered; this does not confirm stopped work", rollback: false } };
    } }];
  for (const job of jobs) {
    commands.push({ id: `cancel:${job.id}`, label: `Cancel fleet pairing ${job.id}`, enabled: pairingPending(job), execute: async () => {
      const life = lifetime.current;
      try {
        const version = ++epoch.current;
        const response = await api.cancelFleetPairing(owner!, job.id);
        if (life.live && version === epoch.current) adopt(response.job);
        return { status: "completed", value: { cancellation_requested: true, job: value(response.job), rollback: false } };
      } catch (cause) { return { status: "unavailable", reason: String(cause) }; }
    } });
    commands.push({ id: `forget:${job.id}`, label: `Forget fleet pairing result ${job.id}`, enabled: !pairingPending(job), execute: async () => {
      const life = lifetime.current;
      try {
        const version = ++epoch.current;
        const response = await api.forgetFleetPairing(owner!, job.id);
        if (life.live && version === epoch.current) { latest.current = latest.current.filter(item => item.id !== job.id); setJobs(latest.current); }
        return { status: "completed", value: { forgotten: response.forgotten, rollback: false } };
      } catch (cause) { return { status: "unavailable", reason: String(cause) }; }
    } });
  }
  const invoke = useUiCommands(commands.map(command => ({ key: `${parent}:pairing:${command.id}`, command: {
    parent, subject: `${parent}:pairing:${command.id}`, label: command.label, parameters: {}, prerequisites: ["Actual requests owned by this local UI window"],
    read: () => ({ revision: revision(), enabled: !!owner && command.enabled }), execute: command.execute,
  } })));
  useUiReading("Fleet pairing request status", { connected: !!owner, pending, uncertain_start: uncertain, error,
    jobs: jobs.map(job => ({ id: job.id, mode: job.mode, status: job.status, phase: job.phase, cancel_requested: job.cancel_requested })),
    scope: "Last host-observed in-memory requests for this window. Cancellation does not roll back enrollment. Full replies require disclosure." });
  return { start, jobs, pending, uncertain, error, active: jobs.some(pairingPending), human: (id: string) => void invoke(`${parent}:pairing:${id}`).catch(cause => setError(String(cause))) };
}
</storycode:copy>

### Assemble `apps/web/src/views/useFleetPairingControls.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useFleetPairingControls.ts"><storycode:paste select="#f-73bda5132042-value"/><storycode:paste select="#f-73bda5132042-Attempt"/><storycode:paste select="#f-73bda5132042-useFleetPairingControls"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-19c258066692"></a>
## `apps/web/src/views/useFleetPairingControls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-19c258066692-job"></a>
### `job`

<storycode:copy id="f-19c258066692-job">
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { api } from "../api/client";
import type { FleetPairingJob } from "../api/fleetPairings";
import { UiActionContext, useUiCommands } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { useFleetPairingControls } from "./useFleetPairingControls";
const job = (status: FleetPairingJob["status"] = "running"): FleetPairingJob => ({
  id: "actual-job", request_id: "request", mode: "dial", status, phase: "waiting_for_peer", cancel_requested: status === "cancelled", rollback: false,
});
</storycode:copy>
<a id="f-19c258066692-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement">
beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(api, "fleetPairings").mockResolvedValue({ jobs: [] });
  vi.spyOn(api, "cancelFleetPairing").mockResolvedValue({ job: job("cancelled") });
});
</storycode:copy>
<a id="f-19c258066692-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement-2">
afterEach(cleanup);
</storycode:copy>
<a id="f-19c258066692-harness"></a>
### `harness`

<storycode:copy id="f-19c258066692-harness">
function harness() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const paired = vi.fn();
  const hook = renderHook(() => {
    const controls = useFleetPairingControls("fleet-fixture", paired);
    useUiCommands([{ key: "fixture-start", command: { parent: "fleet-fixture", subject: "fleet-fixture", label: "Start fixture pairing through UI registry", parameters: {}, prerequisites: [],
      read: () => ({ revision: "fixture", enabled: true }), execute: (_args, _actor, signal) => controls.start("host", "fixture", signal),
    } }]);
    return controls;
  }, {
    wrapper: ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>,
  });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (label: string) => {
    let outcome: unknown;
    await act(async () => { outcome = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
      handle: actions().find(action => action.label === label)!.handle, arguments: {}, request_id: crypto.randomUUID(),
    }); });
    return outcome;
  };
  return { ...hook, registry, paired, run };
}
</storycode:copy>
<a id="f-19c258066692-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement-3">
it("shares real job identity and agent cancellation without a false stopped claim", async () => {
  vi.spyOn(api, "startFleetPairing").mockResolvedValue({ started: true, job: job() });
  vi.mocked(api.cancelFleetPairing).mockResolvedValue({ job: { ...job(), cancel_requested: true } });
  const hook = harness();
  await act(async () => { expect(await hook.result.current.start("dial", "fixture-phrase")).toMatchObject({ status: "running", job: "actual-job" }); });
  expect(await hook.run("Cancel fleet pairing actual-job")).toMatchObject({ status: "result", outcome: { value: { cancellation_requested: true, job: { status: "running" }, rollback: false } } });
  expect(api.cancelFleetPairing).toHaveBeenCalledWith("w", "actual-job");
  expect(hook.result.current.active).toBe(true);
  const response = { ...job("paired"), result: { machine: { name: "peer", public_key: "fixture", kind: "desktop" as const, grants: [], added: "fixture" }, their_fingerprint: "them", our_fingerprint: "us", note: "Compare" } };
  vi.mocked(api.fleetPairings).mockResolvedValue({ jobs: [response] });
  await hook.run("Refresh fleet pairing requests");
  expect(hook.result.current.active).toBe(false);
  const forget = vi.spyOn(api, "forgetFleetPairing").mockResolvedValue({ forgotten: "actual-job" });
  await hook.run("Forget fleet pairing result actual-job");
  expect(forget).toHaveBeenCalledWith("w", "actual-job");
  expect(hook.result.current.jobs).toEqual([]);
});
</storycode:copy>
<a id="f-19c258066692-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement-4">
it("reuses an uncertain start identity and refuses changed-argument retry", async () => {
  const start = vi.spyOn(api, "startFleetPairing").mockRejectedValueOnce(Error("response lost"))
    .mockResolvedValueOnce({ started: false, job: job() });
  const hook = harness();
  await act(async () => { await hook.result.current.start("dial", "same"); });
  expect(hook.result.current.uncertain).toBe(true);
  await act(async () => { expect(await hook.result.current.start("dial", "different")).toMatchObject({ status: "requires_input" }); });
  expect(start).toHaveBeenCalledTimes(1);
  await act(async () => { await hook.result.current.start("dial", "same"); });
  expect(start.mock.calls[1]).toEqual(start.mock.calls[0]);
  expect(hook.result.current.jobs[0].id).toBe("actual-job");
});
</storycode:copy>
<a id="f-19c258066692-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement-5">
it("forwards cancellation after a late start reply from a closed pane", async () => {
  let complete!: (reply: { started: boolean; job: FleetPairingJob }) => void;
  vi.spyOn(api, "startFleetPairing").mockImplementation(() => new Promise(resolve => { complete = resolve; }));
  const hook = harness();
  let pending!: ReturnType<typeof hook.result.current.start>;
  await act(async () => { pending = hook.result.current.start("host", "fixture"); });
  hook.unmount();
  await act(async () => { complete({ started: true, job: job() }); await pending; });
  await waitFor(() => expect(api.cancelFleetPairing).toHaveBeenCalledWith("w", "actual-job"));
  expect(hook.paired).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-19c258066692-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement-6">
it("requests cancellation before the host identity arrives and reports the actual later phase", async () => {
  let complete!: (reply: { started: boolean; job: FleetPairingJob }) => void;
  vi.spyOn(api, "startFleetPairing").mockImplementation(() => new Promise(resolve => { complete = resolve; }));
  vi.spyOn(api, "fleetPairing").mockResolvedValue({ ...job("cancelled"), phase: "closed" });
  const hook = harness();
  let pending!: ReturnType<typeof hook.result.current.start>;
  await act(async () => { pending = hook.result.current.start("host", "fixture"); });
  expect(await hook.run("Request cancellation of pending fleet pairing start")).toMatchObject({ status: "result", outcome: { value: { cancellation_requested: true, rollback: false } } });
  expect(api.cancelFleetPairing).not.toHaveBeenCalled();
  await act(async () => { complete({ started: true, job: job() }); await pending; });
  expect(api.cancelFleetPairing).toHaveBeenCalledWith("w", "actual-job");
  expect(hook.result.current.active).toBe(true);
  await waitFor(() => expect(hook.result.current.jobs[0]).toMatchObject({ status: "cancelled", phase: "closed" }), { timeout: 1800 });
  expect(hook.result.current.active).toBe(false);
});
</storycode:copy>
<a id="f-19c258066692-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-19c258066692-expression_statement-7">
it("keeps pending-start cancellation addressable while the start action holds its registry lock", async () => {
  let complete!: (reply: { started: boolean; job: FleetPairingJob }) => void;
  vi.spyOn(api, "startFleetPairing").mockImplementation(() => new Promise(resolve => { complete = resolve; }));
  const hook = harness();
  const action = hook.registry.observe("fleet-fixture", 64).actions.find(item => item.label === "Start fixture pairing through UI registry")!;
  let pending!: ReturnType<typeof hook.registry.act>;
  await act(async () => { pending = hook.registry.act(hook.registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action.handle, arguments: {}, request_id: "start-lock" }); });
  expect(await hook.run("Request cancellation of pending fleet pairing start")).toMatchObject({ status: "result", outcome: { value: { cancellation_requested: true, rollback: false } } });
  expect(api.cancelFleetPairing).not.toHaveBeenCalled();
  await act(async () => { complete({ started: true, job: job() }); expect(await pending).toMatchObject({ status: "result", outcome: { status: "running", job: "actual-job" } }); });
  expect(api.cancelFleetPairing).toHaveBeenCalledWith("w", "actual-job");
});
</storycode:copy>

### Assemble `apps/web/src/views/useFleetPairingControls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useFleetPairingControls.test.tsx"><storycode:paste select="#f-19c258066692-job"/><storycode:paste select="#f-19c258066692-expression_statement"/><storycode:paste select="#f-19c258066692-expression_statement-2"/><storycode:paste select="#f-19c258066692-harness"/><storycode:paste select="#f-19c258066692-expression_statement-3"/><storycode:paste select="#f-19c258066692-expression_statement-4"/><storycode:paste select="#f-19c258066692-expression_statement-5"/><storycode:paste select="#f-19c258066692-expression_statement-6"/><storycode:paste select="#f-19c258066692-expression_statement-7"/></storycode:file>


</storycode:doc>
