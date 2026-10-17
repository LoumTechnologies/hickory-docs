import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
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
