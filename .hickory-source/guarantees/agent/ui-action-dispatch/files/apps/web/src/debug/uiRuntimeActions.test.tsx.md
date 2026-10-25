<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/uiRuntimeActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `5127a1b453b7518acf0dc51a280f4e3d1add2ddca7d98aaef849c1f2aa6bf485`.

<book:entry path="apps/web/src/debug/uiRuntimeActions.test.tsx">
import { cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useRuntimeActions } from "./uiRuntimeActions";
import type { RuntimeRun } from "./runtimeObservations";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("pages exact retained source and profile events without merging different runs", async () => {
  const run = { id: "profile-source-a", status: "completed", started_unix_ms: 1, pinned: false, sources: [{ id: "source", uri: "hotspot.md", revision: "sha256:a", bytes: [...new TextEncoder().encode("λ😀")] }], events: [{ id: "profile-real-event", observation: { kind: "profile", metrics: [{ name: "calls", value: 12 }] } }] } as unknown as RuntimeRun;
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const controls = { select: vi.fn(), occurrence: vi.fn(), compare: vi.fn(), pin: vi.fn(), remove: vi.fn() };
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  renderHook(() => useRuntimeActions([run], run, "", "", true, controls), { wrapper });
  const descriptor = (label: string) => registry.observe(registry.groups().groups[0]).actions.find(item => item.label === label)!;
  const actor = { id: "python:profile", kind: "agent" as const };
  const source = descriptor("Read retained runtime source bytes");
  expect(await registry.act(registry.scope, actor, { handle: source.handle, arguments: { source: 0, offset: 0, limit: 2 }, request_id: "source-page" })).toMatchObject({ status: "result", outcome: { value: { bytes: [206,187], source: { revision: "sha256:a" }, next: 2, edit_baseline: false } } });
  const events = descriptor("Read retained runtime events");
  expect(await registry.act(registry.scope, actor, { handle: events.handle, arguments: { offset: 0, limit: 1 }, request_id: "events" })).toMatchObject({ status: "result", outcome: { value: { run: run.id, source_changed: true, events: [{ id: "profile-real-event" }], next: null } } });
  expect(await registry.act(registry.scope, actor, { handle: events.handle, arguments: { offset: 0, limit: 65 }, request_id: "too-many" })).toMatchObject({ status: "invalid" });
  expect(controls.select).not.toHaveBeenCalled(); expect(controls.pin).not.toHaveBeenCalled();
});
</book:entry>

</book:doc>
