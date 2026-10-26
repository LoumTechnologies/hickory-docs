<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/GeneratedBufferRecovery.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-57ffaede010a"></a>
## `apps/web/src/shell/GeneratedBufferRecovery.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-57ffaede010a-owners"></a>
### `owners`

<storycode:copy id="f-57ffaede010a-owners">
import { useId, useState } from "react";
import { ActionNotice } from "../components/ActionNotice";
import { viewBuffers } from "../lib/viewBuffers";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { OutputSaver } from "../lib/outputSave";
import type { UiOutcome } from "../lib/uiActions";
const owners = new WeakMap<object, string>();
</storycode:copy>
<a id="f-57ffaede010a-identity"></a>
### `identity`

<storycode:copy id="f-57ffaede010a-identity">
function identity(owner: object | undefined) { if (!owner) return null; let id = owners.get(owner); if (!id) { id = crypto.randomUUID(); owners.set(owner, id); } return id; }
</storycode:copy>
<a id="f-57ffaede010a-GeneratedBufferRecovery"></a>
### `GeneratedBufferRecovery`

Recovery stays with the exact failed generated buffer and ordinary saver.

<storycode:copy id="f-57ffaede010a-GeneratedBufferRecovery">
/** Recovery stays with the exact failed generated buffer and ordinary saver. */
export function GeneratedBufferRecovery({ bufferId, saver, error, isCurrent, source }: {
  bufferId: string; saver: OutputSaver; error: string; isCurrent: () => boolean; source: () => unknown;
}) {
  const parent = `generated-buffer-recovery:${useId()}`;
  const [failure, setFailure] = useState("");
  const revision = useUiRevision(() => [bufferId, identity(saver), identity(viewBuffers.get(bufferId)), error, source(), saver.baselineContent(), isCurrent(), failure]);
  const invoke = useUiCommands((["retry", "discard"] as const).map(operation => ({ key: `${parent}:${operation}`, command: {
    label: operation === "retry" ? "Retry generated buffer save" : "Discard generated buffer and reload", parent, subject: parent, parameters: {}, prerequisites: ["The current failed generated buffer; discard removes its unsaved changes through the existing recovery handler"],
    read: () => ({ revision: revision(), enabled: isCurrent() && !!viewBuffers.get(bufferId), reason: "The failed buffer owner changed or was closed" }),
    execute: async (): Promise<UiOutcome> => {
      const buffer = viewBuffers.get(bufferId);
      if (!buffer || !isCurrent()) return { status: "unavailable", reason: "The failed generated buffer is no longer current" };
      if (operation === "retry") {
        const saved = await saver.flushNow();
        if (!isCurrent()) return { status: "unavailable", reason: "The buffer owner changed during save; earlier effects may have occurred" };
        return { status: "completed", value: { saved, verification: "not requested" } };
      }
      await buffer.discard();
      if (!isCurrent()) return { status: "unavailable", reason: "The buffer owner changed during discard; earlier effects may have occurred" };
      return { status: "completed", value: { discarded: true, verification: "not requested" } };
    },
  } })));
  const act = (operation: string) => { void invoke(`${parent}:${operation}`).catch(cause => setFailure(cause instanceof Error ? cause.message : String(cause))); };
  return <ActionNotice kind="fail">
    Could not save into the owning document. Your edits remain in this buffer: {error}
    <button type="button" onClick={() => act("retry")}>Retry Save</button>
    <button type="button" onClick={() => act("discard")}>Discard this buffer and reload</button>
    {failure && <span role="alert">{failure}</span>}
  </ActionNotice>;
}
</storycode:copy>

### Assemble `apps/web/src/shell/GeneratedBufferRecovery.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GeneratedBufferRecovery.tsx"><storycode:paste select="#f-57ffaede010a-owners"/><storycode:paste select="#f-57ffaede010a-identity"/><storycode:paste select="#f-57ffaede010a-GeneratedBufferRecovery"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-9cef161ebf1f"></a>
## `apps/web/src/shell/GeneratedBufferRecovery.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Save a generated edit once and retain it when saving fails](../../../../../guarantees/authoring/an-output-edit-lands-in-its-document/contract.md).

<a id="f-9cef161ebf1f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9cef161ebf1f-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { createOutputSaver } from "../lib/outputSave";
import { viewBuffers } from "../lib/viewBuffers";
import { GeneratedBufferRecovery } from "./GeneratedBufferRecovery";
afterEach(cleanup);
</storycode:copy>
<a id="f-9cef161ebf1f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9cef161ebf1f-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/shell/GeneratedBufferRecovery.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GeneratedBufferRecovery.test.tsx"><storycode:paste select="#f-9cef161ebf1f-expression_statement"/><storycode:paste select="#f-9cef161ebf1f-expression_statement-2"/></storycode:file>


</storycode:doc>
