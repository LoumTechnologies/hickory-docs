<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/session/evidence.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Historical evidence stays historical](../../../../../../../guarantees/agent/historical-evidence-stays-historical/contract.md)
- [Captured reading surfaces share owned native actions](../../../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md)
- [Review investigations link observed agent work without resolving human review](../../../../../../../guarantees/authoring/review-investigations-link-observed-agent-work/contract.md)

Captured SHA-256: `2ae0b7ac09830db3f799a498f147facdd1c5ba73e885cce2ed0e778bb64d11dc`.

<book:entry path="apps/web/src/elements/session/evidence.test.tsx">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
// Guarantee: docs/guarantees/agent/historical-evidence-stays-historical.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { SourceChips } from "./view";
import { openReading, publishReading } from "../../lib/readingViews";
import { openLocation } from "../../lib/revealLine";
vi.mock("../../lib/readingViews", () => ({ publishReading: vi.fn(), openReading: vi.fn() }));
vi.mock("../../lib/revealLine", () => ({ openLocation: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it("opens the captured historical passage with its citing paragraph and comparison facts", () => {
  render(<SourceChips links={[{ family: "declared", span: [10,40], lines: [3,3], to: {path:"meeting.md",lines:[2,2]},title:"Declared reference",historical: {record_kind:"read-snapshot",session_line:2,path:"meeting.md",content:"Approval pending",sha256:"old",first_line:2,last_line:2,current_state:"changed",claim:'Legal said “approved”.',quoted_literals:[{literal:"approved",present_in_recorded_passage:false}]} }]} />);
  fireEvent.click(screen.getByRole("button"));
  const view = vi.mocked(publishReading).mock.calls[0][1];
  expect(view.after).toContain("Approval pending");
  expect(view.after).toContain('Legal said “approved”.');
  expect(view.after).toContain('"approved": absent from recorded passage');
  expect(view.status).toContain("semantic support not assessed");
  expect(view.after).toBe(view.before);
  expect(openReading).toHaveBeenCalled();
  expect(openLocation).not.toHaveBeenCalled();
});
it("uses a report label for adapter assertions", () => {
  render(<SourceChips links={[{family:"declared",span:[1,2],lines:[1,1],to:{path:"file.md"},title:"ACP-reported file edit"}]} />);
  expect(screen.getByText("reports")).toBeTruthy();
});

// Guarantee: docs/guarantees/authoring/review-investigations-link-observed-agent-work.md
it("captures source-chip relationship reviews with historical bytes via keyboard", () => {
  const received = vi.fn();
  window.addEventListener("hickory:review-target", received);
  try {
    render(<SourceChips links={[{ family: "declared", span: [10, 40], lines: [3, 3],
      to: { path: "meeting.md", lines: [2, 2] }, title: "Declared reference",
      historical: { record_kind: "read-snapshot", session_line: 2, path: "meeting.md",
        content: "Approval pending", sha256: "b".repeat(64), first_line: 2, last_line: 2,
        current_state: "changed", claim: "Legal approved" } }]} />);
    fireEvent.keyDown(screen.getByRole("button"), { key: "Enter", altKey: true });
    expect(received).toHaveBeenCalledTimes(1);
    const target = (received.mock.calls[0][0] as CustomEvent).detail;
    expect(target.quote).toBe("Approval pending");
    expect(target.source_hash).toBe("b".repeat(64));
    expect(target.relationship.claim).toBe("Legal approved");
    expect(openLocation).not.toHaveBeenCalled();
  } finally { window.removeEventListener("hickory:review-target", received); }
});
it("shares historical-chip navigation and retires same-label replacement handles", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const links = (content: string) => [{ family: "declared" as const, span: [1, 2] as [number, number], lines: [1, 1] as [number, number], to: { path: "file.md" }, title: "Recorded source",
    historical: { record_kind: "read-snapshot" as const, session_line: 1, path: "file.md", content, sha256: "a".repeat(64), first_line: 1, last_line: 1 } }];
  const renderChips = (content: string) => <UiActionContext.Provider value={registry}><SourceChips links={links(content)} /></UiActionContext.Provider>;
  const view = render(renderChips("old bytes"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = () => actions().find(action => action.label === "Open recorded source file.md")!.handle;
  const run = async (target: string) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: target, arguments: {}, request_id: crypto.randomUUID() }); });
    return result;
  };
  const stale = handle();
  view.rerender(renderChips("new captured bytes"));
  expect(await run(stale)).not.toMatchObject({ status: "result" });
  expect(publishReading).not.toHaveBeenCalled();
  expect(await run(handle())).toMatchObject({ status: "result", outcome: { value: { navigation_requested: true, source_verified: false } } });
  expect(vi.mocked(publishReading).mock.calls[0][1].after).toContain("new captured bytes");
  expect(openLocation).not.toHaveBeenCalled();
});
it("keeps recorded reasoning behind shared disclosure and retires its byte handle on collapse", async () => {
  const { act } = await import("@testing-library/react");
  const { sessionReasoningView } = await import("./view");
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const slot = { key: "r", text: "private recorded reasoning", at: 0, span: [0, 26], index: 0 } as import("../../editor/rendered").RenderedSlot;
  const cx = { sessionBlocks: [] } as unknown as import("../types").SlotContext;
  render(<UiActionContext.Provider value={registry}>{sessionReasoningView.render(slot, cx)}</UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (target: string, args: Record<string, boolean | number>) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: target, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(actions().some(action => action.label === "Read disclosed evidence bytes")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("private recorded reasoning");
  await run(actions().find(action => action.label === "Set evidence visibility: reasoning")!.handle, { open: true });
  const bytes = actions().find(action => action.label === "Read disclosed evidence bytes")!.handle;
  const result = await run(bytes, { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(result.outcome.value.bytes))).toContain("private recorded reasoning");
  const fold = document.querySelector("details")!;
  fold.open = false; fireEvent(fold, new Event("toggle"));
  expect(await run(bytes, { offset: 0, limit: 8192 })).not.toMatchObject({ status: "result" });
});
</book:entry>

</book:doc>
