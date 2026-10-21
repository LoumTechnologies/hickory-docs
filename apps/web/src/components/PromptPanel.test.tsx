import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { PromptPanel, usePrompt } from "./PromptPanel";
afterEach(cleanup);
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
it("requires a human for a caller-owned decision and resolves the exact observed choice", async () => {
  const app = setup(); let answer!: Promise<string | null>; let resolved = false;
  act(() => { answer = app.controller.askChoice("Unsaved work", [{ label: "Discard", value: "discard" }]); void answer.then(() => { resolved = true; }); });
  expect(await app.invoke("Choose prompt option", { index: 0 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(resolved).toBe(false);
  fireEvent.click(screen.getByText("Discard"));
  await act(async () => {});
  expect(await answer).toBe("discard");
});
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
