<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/uiRuntimeActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1790b7a5242a"></a>
## `apps/web/src/debug/uiRuntimeActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-1790b7a5242a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1790b7a5242a-expression_statement">
import { cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useRuntimeActions } from "./uiRuntimeActions";
import type { RuntimeRun } from "./runtimeObservations";
afterEach(cleanup);
</storycode:copy>
<a id="f-1790b7a5242a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1790b7a5242a-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/debug/uiRuntimeActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/uiRuntimeActions.test.tsx"><storycode:paste select="#f-1790b7a5242a-expression_statement"/><storycode:paste select="#f-1790b7a5242a-expression_statement-2"/></storycode:file>

</storycode:doc>
