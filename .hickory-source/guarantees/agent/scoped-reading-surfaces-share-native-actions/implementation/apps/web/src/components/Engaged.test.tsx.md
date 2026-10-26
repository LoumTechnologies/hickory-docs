<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/Engaged.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-45bf37888c2a"></a>
## `apps/web/src/components/Engaged.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../contract.md).

<a id="f-45bf37888c2a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-45bf37888c2a-expression_statement">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
// @vitest-environment jsdom
// The map-in-a-page rule's shared signal: a widget is scenery until clicked
// into, and Escape (outside a field) or clicking away disengages it.
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { Engaged } from "./Engaged";

afterEach(cleanup);

</storycode:copy>
<a id="f-45bf37888c2a-gate"></a>
### `gate`

<storycode:copy id="f-45bf37888c2a-gate">
function gate() {
  const { container } = render(
    <Engaged className="widget">
      <input aria-label="field" />
      <div data-testid="body" />
    </Engaged>,
  );
  return { container, root: container.firstElementChild as HTMLElement };
}

</storycode:copy>
<a id="f-45bf37888c2a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-45bf37888c2a-expression_statement-2">
describe("the engage gate", () => {
  it("engages on click-in, disengages on Escape and on focus leaving", () => {
    const { root } = gate();
    expect(root.className).not.toContain("engaged-gate--on");
    fireEvent.pointerDown(root);
    fireEvent.focus(root);
    expect(root.className).toContain("engaged-gate--on");
    fireEvent.keyDown(root, { key: "Escape" });
    fireEvent.blur(root);
    expect(root.className).not.toContain("engaged-gate--on");
  });

  it("leaves Escape alone inside a field — that one means 'undo my typing'", () => {
    const { container, root } = gate();
    const field = container.querySelector("input")!;
    field.focus();
    fireEvent.focus(root);
    expect(root.className).toContain("engaged-gate--on");
    fireEvent.keyDown(field, { key: "Escape" });
    // No blur was forced: still engaged, still editing.
    expect(root.className).toContain("engaged-gate--on");
    expect(document.activeElement).toBe(field);
  });
});

</storycode:copy>
<a id="f-45bf37888c2a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-45bf37888c2a-expression_statement-3">
it("shares focus/wheel engagement and releases a focused descendant on Escape", async () => {
  const registry = new UiActions({ workspace: "p", window: "one" }, () => true, async () => {});
  const view = render(<UiActionContext.Provider value={registry}><Engaged><button>Grid control</button></Engaged></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(group => group.startsWith("widget-engagement:"))!;
  const action = registry.observe(parent).actions.find(action => action.label === "Set embedded widget engagement")!;
  let outcome;
  await act(async () => { outcome = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action.handle, arguments: { engaged: true }, request_id: "engage" }); });
  expect(outcome).toMatchObject({ outcome: { status: "completed", value: { engaged: true, focus_inside: true } } });
  expect(view.container.firstElementChild?.classList.contains("engaged-gate--on")).toBe(true);
  const button = screen.getByRole("button"); act(() => button.focus());
  fireEvent.keyDown(button, { key: "Escape" });
  await act(async () => {});
  expect(document.activeElement).not.toBe(button);
  expect(view.container.firstElementChild?.classList.contains("engaged-gate--on")).toBe(false);
});
</storycode:copy>
<a id="f-45bf37888c2a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-45bf37888c2a-expression_statement-4">
it("does not blur or commit an active field when an agent asks to release the widget", async () => {
  const registry = new UiActions({ workspace: "p", window: "two" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><input aria-label="Other draft" /><Engaged><input aria-label="Grid draft" /></Engaged></UiActionContext.Provider>);
  const other = screen.getByRole("textbox", { name: "Other draft" }); act(() => other.focus());
  const beforeParent = registry.groups().groups.find(group => group.startsWith("widget-engagement:"))!;
  const before = registry.observe(beforeParent).actions.find(action => action.label === "Set embedded widget engagement")!;
  let first;
  await act(async () => { first = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: before.handle, arguments: { engaged: true }, request_id: "enter" }); });
  expect(first).toMatchObject({ outcome: { status: "requires_input" } });
  expect(document.activeElement).toBe(other);
  const field = screen.getByRole("textbox", { name: "Grid draft" }); act(() => field.focus());
  const parent = registry.groups().groups.find(group => group.startsWith("widget-engagement:"))!;
  const action = registry.observe(parent).actions.find(action => action.label === "Set embedded widget engagement")!;
  let outcome;
  await act(async () => { outcome = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action.handle, arguments: { engaged: false }, request_id: "release" }); });
  expect(outcome).toMatchObject({ outcome: { status: "requires_input" } });
  expect(document.activeElement).toBe(field);
});
</storycode:copy>

### Assemble `apps/web/src/components/Engaged.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/Engaged.test.tsx"><storycode:paste select="#f-45bf37888c2a-expression_statement"/><storycode:paste select="#f-45bf37888c2a-gate"/><storycode:paste select="#f-45bf37888c2a-expression_statement-2"/><storycode:paste select="#f-45bf37888c2a-expression_statement-3"/><storycode:paste select="#f-45bf37888c2a-expression_statement-4"/></storycode:file>

</storycode:doc>
