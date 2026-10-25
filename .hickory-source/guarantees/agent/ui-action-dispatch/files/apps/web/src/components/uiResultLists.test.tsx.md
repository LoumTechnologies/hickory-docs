<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/uiResultLists.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `1cbd7f161852b9e87e4e7e4f51a577fd3d7e3c09f213e69ce373d2335ebf34c6`.

<book:entry path="apps/web/src/components/uiResultLists.test.tsx">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { ReferencesPanel } from "./ReferencesPanel";
import { ProblemsPanel } from "./ProblemsPanel";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses reference values explicitly and refuses retargeting an old result index", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onPick = vi.fn(), onClose = vi.fn();
  const first = { uri: "hick:///private.md", range: { start: { line: 2, character: 4 }, end: { line: 2, character: 7 } } };
  const content = (uri: string) => <UiActionContext.Provider value={registry}><ReferencesPanel query="secret λ" locations={[{ ...first, uri }]} onPick={onPick} onClose={onClose} /></UiActionContext.Provider>;
  const { rerender } = render(content(first.uri));
  const parent = registry.groups().groups[0];
  const descriptors = registry.observe(parent).actions;
  expect(JSON.stringify(descriptors)).not.toContain("private.md");
  const reader = descriptors.find(a => a.label === "Read References evidence bytes")!;
  const result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: reader.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (result.status !== "result" || result.outcome.status !== "completed") throw Error("Expected reading");
  expect(new TextDecoder().decode(new Uint8Array((result.outcome.value as { bytes: number[] }).bytes))).toContain("secret λ");
  const old = descriptors.find(a => a.label === "Open References result")!;
  rerender(content("hick:///changed.md"));
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: { index: 0 }, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(onPick).not.toHaveBeenCalled();
  const fresh = registry.observe(parent).actions.find(a => a.label === "Open References result")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: fresh.handle, arguments: { index: 0 }, request_id: "open" }); });
  expect(onPick).toHaveBeenLastCalledWith({ ...first, uri: "hick:///changed.md" });
  fireEvent.click(screen.getByLabelText("Close references"));
  await act(async () => {});
  expect(onClose).toHaveBeenCalledOnce();
});
it("uses the same problem row for human and agent navigation", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onPick = vi.fn();
  const row = { docId: "doc", path: "a.md", diagnostic: { range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } }, severity: 1, message: "Broken requirement" } };
  const { unmount } = render(<UiActionContext.Provider value={registry}><ProblemsPanel rows={[row]} onPick={onPick} onClose={() => {}} /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  const action = registry.observe(parent).actions.find(a => a.label === "Open Problems result")!;
  await act(async () => { await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { index: 0 }, request_id: "open" }); });
  fireEvent.click(screen.getByText("Broken requirement"));
  await act(async () => {});
  expect(onPick.mock.calls).toEqual([[row], [row]]);
  unmount(); expect(registry.observe(parent).actions).toEqual([]);
});
</book:entry>

</book:doc>
