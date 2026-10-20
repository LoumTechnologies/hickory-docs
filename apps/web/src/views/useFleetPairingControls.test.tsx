import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { api } from "../api/client";
import type { FleetPairingJob } from "../api/fleetPairings";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { useFleetPairingControls } from "./useFleetPairingControls";
const job = (status: FleetPairingJob["status"] = "running"): FleetPairingJob => ({
  id: "actual-job", request_id: "request", mode: "dial", status, phase: "waiting_for_peer", cancel_requested: status === "cancelled", rollback: false,
});
beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(api, "fleetPairings").mockResolvedValue({ jobs: [] });
  vi.spyOn(api, "cancelFleetPairing").mockResolvedValue({ job: job("cancelled") });
});
afterEach(cleanup);
function harness() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const paired = vi.fn();
  const hook = renderHook(() => useFleetPairingControls("fleet-fixture", paired), {
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
