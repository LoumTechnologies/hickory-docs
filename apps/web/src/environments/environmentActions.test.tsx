import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EnvironmentNotice, EnvironmentPanel } from "./EnvironmentPanel";
import type { Environments } from "./useEnvironments";
import type { EnvironmentFinding } from "../api/environments";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";

afterEach(cleanup);
const finding: EnvironmentFinding = {
  project: "/practice", manager: "uv", target: "host", profile: "defaults", revision: "lock:one",
  state: "environment-missing", message: "Dependencies missing", details: "Current lockfile", actions: [{ id: "sync", label: "Sync dependencies", argv: ["uv", "sync", "--locked"], effects: "Install locked dependencies" }],
  install_url: null, interpreter: null, manager_choices: [],
};

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("returns the host setup job and rejects changed/disabled environment actions", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const start = vi.fn().mockResolvedValue({ session: { id: "real-returned-terminal", title: "uv sync" } });
  const environments: Environments = { findings: [finding], notices: [finding], running: {}, pending: null, error: null, refresh: vi.fn(), act: start, choose: vi.fn(), dismiss: vi.fn() };
  const { rerender } = render(<UiActionContext.Provider value={registry}><EnvironmentNotice environments={environments} /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  const descriptor = registry.observe(parent).actions[0];
  expect(descriptor.value).toMatchObject({ argv: ["uv", "sync", "--locked"], state: "environment-missing" });
  const actor = { id: "python:setup", kind: "agent" as const };
  await act(async () => {
    expect(await registry.act(registry.scope, actor, { handle: descriptor.handle, arguments: {}, request_id: "setup" })).toMatchObject({ status: "result", outcome: { status: "queued", job: "real-returned-terminal", value: { dependency_setup: "not yet verified" } } });
  });
  expect(start).toHaveBeenCalledExactlyOnceWith(finding, "sync");
  rerender(<UiActionContext.Provider value={registry}><EnvironmentNotice environments={{ ...environments, pending: "/practice" }} /></UiActionContext.Provider>);
  expect(await registry.act(registry.scope, actor, { handle: descriptor.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  const disabled = registry.observe(parent).actions[0];
  expect(await registry.act(registry.scope, actor, { handle: disabled.handle, arguments: {}, request_id: "disabled" })).toMatchObject({ status: "disabled" });
  expect(start).toHaveBeenCalledOnce();
});

it("reports setup failure instead of inventing a completed job", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const environments: Environments = { findings: [finding], notices: [finding], running: {}, pending: null, error: null, refresh: vi.fn(), act: vi.fn().mockResolvedValue({ error: "Lockfile changed" }), choose: vi.fn(), dismiss: vi.fn() };
  render(<UiActionContext.Provider value={registry}><EnvironmentNotice environments={environments} /></UiActionContext.Provider>);
  const descriptor = registry.observe(registry.groups().groups[0]).actions[0];
  let result;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:setup", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "failed" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "Lockfile changed" } });
});

it("discloses closed environment children only after the real group opens", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const ready = { ...finding, state: "ready" as const };
  const environments: Environments = { findings: [ready], notices: [], running: {}, pending: null, error: null, refresh: vi.fn(), act: vi.fn(), choose: vi.fn(), dismiss: vi.fn() };
  render(<UiActionContext.Provider value={registry}><EnvironmentPanel environments={environments} /></UiActionContext.Provider>);
  expect(registry.groups().groups).toEqual(["environments"]);
  const actor = { id: "python:setup", kind: "agent" as const };
  const disclosure = registry.observe("environments").actions[0];
  expect(disclosure.value).toMatchObject({ open: false });
  await act(async () => { await registry.act(registry.scope, actor, { handle: disclosure.handle, arguments: {}, request_id: "open" }); });
  const children = registry.groups().groups.find(group => group !== "environments")!;
  expect(registry.observe(children).actions[0].label).toBe("Sync dependencies");
  const child = registry.observe(children).actions[0];
  const close = registry.observe("environments").actions[0];
  await act(async () => { await registry.act(registry.scope, actor, { handle: close.handle, arguments: {}, request_id: "close" }); });
  expect(registry.groups().groups).toEqual(["environments"]);
  expect(await registry.act(registry.scope, actor, { handle: child.handle, arguments: {}, request_id: "hidden" })).toMatchObject({ status: "unavailable" });
});
