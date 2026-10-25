<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ComparisonEditor.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Comparison edit permissions do not establish replay evidence](../../../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md)

Captured SHA-256: `3eeb073faa456ee23a7a2f583d9e963506ffa781f7ce8282e06ec81fa9ea064f`.

<book:entry path="apps/web/src/components/ComparisonEditor.test.tsx">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { ComparisonEditor } from "./ComparisonEditor";
import type { ComparisonModel } from "../lib/projectComparison";
afterEach(cleanup);

// Guarantee: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
it("switches layouts, exposes independent permissions and invalidates a verified proposal on typing", () => {
  const model: ComparisonModel = { sources: [
    { id: "a", label: "Old project", kind: "git", text: "old\n" },
    { id: "b", label: "New proposal", kind: "authored", target: "app.ts", text: "new\n" },
  ], old: "a", new: "b", policy: "new", evidence: "verified" };
  const change = vi.fn();
  render(<ComparisonEditor path="app.ts" model={model} onChange={change} />);
  expect((screen.getByLabelText("Comparison layout") as HTMLSelectElement).value).toBe("side-by-side");
  const editors = document.querySelectorAll<HTMLElement>(".cm-editor");
  expect(editors).toHaveLength(2);
  expect(editors[0].querySelector(".cm-content")?.getAttribute("contenteditable")).toBe("false");
  expect(editors[1].querySelector(".cm-content")?.getAttribute("contenteditable")).toBe("true");
  EditorView.findFromDOM(editors[1])!.dispatch({ changes: { from: 0, to: 3, insert: "edited" } });
  expect(change.mock.lastCall?.[0].evidence).toBe("unverified");
  expect(change.mock.lastCall?.[0].sources[0].text).toBe("old\n");
  expect(change.mock.lastCall?.[0].sources[1].text).toBe("edited\n");
});

it("an immutable reading forks without a save path or execution action", () => {
  const model: ComparisonModel = { sources: [
    { id: "a", label: "Old", kind: "git", text: "old" },
    { id: "b", label: "New", kind: "git", text: "new" },
  ], old: "a", new: "b", policy: "neither", evidence: "verified" };
  const change = vi.fn();
  render(<ComparisonEditor path="app.ts" model={model} onChange={change} allowFork />);
  fireEvent.click(screen.getByText("Fork editable comparison"));
  expect(change.mock.lastCall?.[0].sources.every((s: {kind: string}) => s.kind === "authored")).toBe(true);
  expect(change.mock.lastCall?.[0].evidence).toBe("unverified");
  expect(screen.queryByText("Run")).toBeNull();
});


it("highlights changed lines on both sides by default without inserting historical rows", () => {
  const model: ComparisonModel = { sources: [
    { id: "old", label: "Before", kind: "git", text: "same\nremoved\nend\n" },
    { id: "new", label: "After", kind: "authored", target: "app.py", text: "same\nadded\nend\n" },
  ], old: "old", new: "new", policy: "new", evidence: "unverified" };
  const mounted = render(<ComparisonEditor path="app.py" model={model} onChange={vi.fn()} />);
  const panes = mounted.container.querySelectorAll(".comparison-side");
  expect(panes[0].querySelector(".comparison-deleted")?.textContent).toBe("removed");
  expect(panes[1].querySelector(".comparison-added")?.textContent).toBe("added");
  expect(mounted.container.querySelectorAll(".comparison-added, .comparison-deleted")).toHaveLength(2);
  expect(mounted.container.querySelector(".comparison-deletion")).toBeNull();
  const next = EditorView.findFromDOM(panes[1].querySelector<HTMLElement>(".cm-editor")!)!;
  next.dispatch({ changes: { from: 5, to: 10, insert: "removed" } });
  expect(panes[1].querySelector(".comparison-added")).toBeNull();
  fireEvent.change(screen.getByLabelText("Comparison layout"), { target: { value: "inline" } });
  expect(mounted.container.querySelectorAll(".comparison-side")).toHaveLength(1);
  expect(mounted.container.querySelector(".comparison-deletion")).not.toBeNull();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses compact choices, guards live draft edits, and keeps evidence unverified", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const model: ComparisonModel = { sources: [
    { id: "old", label: "Old", kind: "git", text: "PRIVATE OLD" },
    { id: "new", label: "New", kind: "authored", target: "app.py", text: "PRIVATE NEW" },
  ], old: "old", new: "new", policy: "new", evidence: "verified" };
  const change = vi.fn();
  render(<UiActionContext.Provider value={registry}><ComparisonEditor path="app.py" model={model} onChange={change} compact /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  expect(actions().some(a => a.label === "Set comparison layout")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("PRIVATE NEW");
  let id = 0;
  const run = async (action: ReturnType<typeof actions>[number], args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run(actions().find(a => a.label === "Set Patch options visibility")!, { open: true });
  expect(actions().some(a => a.label === "Set comparison layout")).toBe(true);
  const write = actions().find(a => a.label === "Replace comparison draft range" && a.enabled)!;
  expect(await run(write, { from: 0, to: 7, text: "Changed" })).toMatchObject({ status: "result", outcome: { status: "completed", value: { saved: false, approved: false, verified: false } } });
  expect(change.mock.lastCall?.[0].sources[1].text).toBe("Changed NEW");
  expect(change.mock.lastCall?.[0].evidence).toBe("unverified");
  expect(await run(write, { from: 0, to: 7, text: "Replay!" })).toMatchObject({ status: "stale" });
  const immutable = actions().find(a => a.label === "Replace comparison draft range" && !a.enabled)!;
  expect(await run(immutable, { from: 0, to: 1, text: "x" })).toMatchObject({ status: "disabled" });
  const layout = actions().find(a => a.label === "Set comparison layout")!;
  await run(actions().find(a => a.label === "Set Patch options visibility")!, { open: false });
  expect(await run(layout, { layout: "patch" })).toMatchObject({ status: "unavailable" });
});
</book:entry>

</book:doc>
