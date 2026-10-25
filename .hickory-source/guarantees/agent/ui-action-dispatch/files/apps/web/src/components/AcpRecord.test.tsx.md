<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/AcpRecord.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `0dea42dbd98f6a21c1d8c2db96b6e5188165a30c1eb6c99838b0b8b47e035172`.

<book:entry path="apps/web/src/components/AcpRecord.test.tsx">
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { AcpRecord } from "./AcpRecord";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses recorded tool evidence explicitly and retires nested readings when collapsed", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const body = JSON.stringify({ toolCallId: "tool", title: "Read file", status: "completed", rawOutput: "PRIVATE EVIDENCE λ" });
  render(<UiActionContext.Provider value={registry}><AcpRecord kind="acp-activity" body={body} /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  expect(registry.groups().groups).toEqual([parent]);
  expect(JSON.stringify(registry.observe(parent))).not.toContain("PRIVATE EVIDENCE");
  const open = registry.observe(parent).actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: open.handle, arguments: { open: true }, request_id: "open" }); });
  expect(registry.groups().groups.some(g => g.startsWith("adapter-tool-reading:"))).toBe(true);
  const read = registry.observe(parent).actions.find(a => a.label === "Read disclosed evidence bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain("PRIVATE EVIDENCE λ");
  const close = registry.observe(parent).actions.find(a => a.label.startsWith("Set evidence visibility"))!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: close.handle, arguments: { open: false }, request_id: "close" }); });
  expect(registry.groups().groups).toEqual([parent]);
  expect(registry.observe(parent).actions).toHaveLength(1);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "old" })).toMatchObject({ status: "unavailable" });
});
it("treats malformed/non-object recorded payloads as inspectable text and labels stopped neutrally", () => {
  const view = render(<AcpRecord kind="acp-activity" body="null" />);
  expect(screen.getByText("Agent activity")).toBeTruthy();
  view.rerender(<AcpRecord kind="acp-turn-status" body='{"status":"stopped"}' />);
  expect(screen.getByText("Turn stopped")).toBeTruthy();
  expect(screen.queryByText("Stopped by you")).toBeNull();
});
</book:entry>

</book:doc>
