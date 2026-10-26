<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/PromptPanel.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6e5d4425a8d2"></a>
## `apps/web/src/components/PromptPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md).

<a id="f-6e5d4425a8d2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-6e5d4425a8d2-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { PromptPanel, usePrompt } from "./PromptPanel";
afterEach(cleanup);
</storycode:copy>
<a id="f-6e5d4425a8d2-setup"></a>
### `setup`

<storycode:copy id="f-6e5d4425a8d2-setup">
function setup() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let controller!: ReturnType<typeof usePrompt>;
  function Harness() { controller = usePrompt(); return <PromptPanel prompt={controller.prompt} onSettle={controller.settle} />; }
  const view = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  let n = 0;
  return { registry, view, get controller() { return controller; }, invoke: async (label: string, args: Record<string, UiValue>) => {
    const parent = registry.groups().groups.find(g => g.startsWith("prompt:"))!;
    const action = registry.observe(parent).actions.find(a => a.label === label)!;
    let result: UiResult | undefined;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); });
    return result;
  } };
}
</storycode:copy>
<a id="f-6e5d4425a8d2-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-6e5d4425a8d2-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares allowed draft/submit with the actual resolver and keeps private text out of discovery", async () => {
  const app = setup(); let answer!: Promise<string | null>;
  act(() => { answer = app.controller.askText("Rename", "PRIVATE λ", { agent: true }); });
  const parent = app.registry.groups().groups[0];
  expect(JSON.stringify(app.registry.observe(parent))).not.toContain("PRIVATE λ");
  await app.invoke("Set prompt text draft", { text: "new λ" });
  expect((screen.getByRole("textbox", { name: "Rename" }) as HTMLInputElement).value).toBe("new λ");
  await app.invoke("Apply prompt text", {});
  expect(await answer).toBe("new λ");
  expect(app.registry.groups().groups).toEqual([]);
});
</storycode:copy>
<a id="f-6e5d4425a8d2-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-6e5d4425a8d2-expression_statement-3">
it("requires a human for a caller-owned decision and resolves the exact observed choice", async () => {
  const app = setup(); let answer!: Promise<string | null>; let resolved = false;
  act(() => { answer = app.controller.askChoice("Unsaved work", [{ label: "Discard", value: "discard" }]); void answer.then(() => { resolved = true; }); });
  expect(await app.invoke("Choose prompt option", { index: 0 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(resolved).toBe(false);
  fireEvent.click(screen.getByText("Discard"));
  await act(async () => {});
  expect(await answer).toBe("discard");
});
</storycode:copy>
<a id="f-6e5d4425a8d2-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-6e5d4425a8d2-expression_statement-4">
it("refuses changed-source input and permits cancellation without applying it", async () => {
  const app = setup(); let source = {}; let answer!: Promise<string | null>;
  act(() => { answer = app.controller.askText("Rename", "old", { agent: true, currentRevision: () => source }); });
  const parent = app.registry.groups().groups[0];
  const old = app.registry.observe(parent).actions.find(a => a.label === "Apply prompt text")!;
  source = {};
  expect(await app.registry.act(app.registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(app.registry.observe(parent).actions.find(a => a.label === "Apply prompt text")!.enabled).toBe(false);
  await app.invoke("Cancel active prompt", {});
  expect(await answer).toBeNull();
});
</storycode:copy>
<a id="f-6e5d4425a8d2-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-6e5d4425a8d2-expression_statement-5">
it("retires identically labelled replaced prompts and settles replacement/unmount promises", async () => {
  const app = setup(); let first!: Promise<string | null>, second!: Promise<string | null>;
  act(() => { first = app.controller.askText("Same title", "same", { agent: true }); });
  const parent = app.registry.groups().groups[0];
  const old = app.registry.observe(parent).actions.find(a => a.label === "Apply prompt text")!;
  act(() => { second = app.controller.askText("Same title", "same", { agent: true }); });
  expect(await first).toBeNull();
  expect(app.registry.groups().groups[0]).not.toBe(parent);
  expect(await app.registry.act(app.registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "replaced" })).toMatchObject({ status: "unavailable" });
  app.view.unmount(); expect(await second).toBeNull();
});
</storycode:copy>
<a id="f-6e5d4425a8d2-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-6e5d4425a8d2-expression_statement-6">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("cancels only the exact abort-owned prompt and leaves a replacement untouched", async () => {
  const app = setup(), first = new AbortController(), second = new AbortController(), already = new AbortController();
  let old!: Promise<string | null>, current!: Promise<string | null>, refused!: Promise<string | null>;
  act(() => { old = app.controller.askText("First", "one", { signal: first.signal }); });
  act(() => { current = app.controller.askText("Replacement", "two", { signal: second.signal }); });
  expect(await old).toBeNull();
  act(() => first.abort());
  expect(screen.getByRole("textbox", { name: "Replacement" })).toBeTruthy();
  already.abort();
  act(() => { refused = app.controller.askChoice("Already cancelled", [{ label: "Discard", value: "discard" }], { signal: already.signal }); });
  expect(await refused).toBeNull();
  expect(screen.getByRole("textbox", { name: "Replacement" })).toBeTruthy();
  act(() => second.abort());
  expect(await current).toBeNull(); expect(app.registry.groups().groups).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/components/PromptPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PromptPanel.test.tsx"><storycode:paste select="#f-6e5d4425a8d2-expression_statement"/><storycode:paste select="#f-6e5d4425a8d2-setup"/><storycode:paste select="#f-6e5d4425a8d2-expression_statement-2"/><storycode:paste select="#f-6e5d4425a8d2-expression_statement-3"/><storycode:paste select="#f-6e5d4425a8d2-expression_statement-4"/><storycode:paste select="#f-6e5d4425a8d2-expression_statement-5"/><storycode:paste select="#f-6e5d4425a8d2-expression_statement-6"/></storycode:file>

</storycode:doc>
