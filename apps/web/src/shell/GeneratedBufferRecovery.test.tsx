import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { createOutputSaver } from "../lib/outputSave";
import { viewBuffers } from "../lib/viewBuffers";
import { GeneratedBufferRecovery } from "./GeneratedBufferRecovery";
afterEach(cleanup);
// Guarantees: docs/guarantees/agent/ui-action-dispatch.md;
// docs/guarantees/authoring/an-output-edit-lands-in-its-document.md.
it("retains a real failed save, rejects stale discard and shares native recovery without a new write", async () => {
  const post = vi.fn().mockRejectedValue(new Error("unmapped edit"));
  const saver = createOutputSaver({ post, onError: () => {}, debounceMs: 60000 });
  saver.load("base");
  let source = "draft", current = true;
  saver.changed(source); expect(await saver.flushNow()).toBe(false);
  const discard = vi.fn(async () => { source = await saver.discardPending(source); });
  viewBuffers.set("fixture-generated", { dirty: true, save: () => saver.flushNow(), discard });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const rendered = render(<UiActionContext.Provider value={registry}><GeneratedBufferRecovery bufferId="fixture-generated" saver={saver} error="unmapped edit" isCurrent={() => current} source={() => source} /></UiActionContext.Provider>);
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  try {
    const retry = action("Retry generated buffer save");
    await act(async () => expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: retry.handle, arguments: {}, request_id: "retry" })).toMatchObject({ outcome: { value: { saved: false, verification: "not requested" } } }));
    expect(post).toHaveBeenCalledTimes(2); expect(saver.hasPendingEdits()).toBe(true);
    const old = action("Discard generated buffer and reload"); source = "newer draft"; saver.changed(source);
    expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
    expect(source).toBe("newer draft"); expect(discard).not.toHaveBeenCalled();
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Discard this buffer and reload" })));
    expect(discard).toHaveBeenCalledTimes(1); expect(source).toBe("base"); expect(saver.hasPendingEdits()).toBe(false); expect(post).toHaveBeenCalledTimes(2);
    const retired = action("Retry generated buffer save"); current = false;
    expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: retired.handle, arguments: {}, request_id: "retired" })).toMatchObject({ status: "stale" });
    expect(action("Retry generated buffer save").enabled).toBe(false);
  } finally { rendered.unmount(); saver.dispose(); viewBuffers.forget("fixture-generated"); }
});
