<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/StaticUiSurface.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b72f7c25d76c"></a>
## `apps/web/src/embed/StaticUiSurface.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-b72f7c25d76c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b72f7c25d76c-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { connectStaticUiParent, STATIC_UI_CHANNEL } from "./StaticUiSurface";
import { createEmbedUiSession } from "./uiHostSession";
import { UiActionContext, useUiCommands } from "../lib/uiActionBridge";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-b72f7c25d76c-Control"></a>
### `Control`

<storycode:copy id="f-b72f7c25d76c-Control">
function Control() {
  useUiCommands([{ key: "test", command: { label: "Read fixture", parent: "test", subject: "test", parameters: {}, prerequisites: [], read: () => ({ revision: "v1", enabled: true }), execute: async () => ({ status: "completed", value: "public fixture" }) } }]);
  return null;
}
</storycode:copy>
<a id="f-b72f7c25d76c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b72f7c25d76c-expression_statement-2">
it("binds static parent requests to exact origin, source and scope and retires the connection", async () => {
  const target = new EventTarget(), postMessage = vi.fn(), parent = { postMessage }, session = createEmbedUiSession(() => {});
  Object.assign(target, { parent });
  render(<UiActionContext.Provider value={session.registry}><Control /></UiActionContext.Provider>);
  const bridge = connectStaticUiParent(target as unknown as Window, "https://host.example", session);
  expect(postMessage).toHaveBeenCalledWith(expect.objectContaining({ event: "ready", data: expect.objectContaining({ scope: session.scope }) }), "https://host.example");
  postMessage.mockClear();
  const send = (origin: string, source: unknown, window = session.scope.window) => target.dispatchEvent(new MessageEvent("message", { origin, source: source as Window, data: { channel: STATIC_UI_CHANNEL, version: 1, op: "ui", id: "r1", data: { window, method: "ui_observe", arguments: {} } } }));
  await act(async () => { send("https://wrong.example", parent); send("https://host.example", {}); });
  expect(postMessage).not.toHaveBeenCalled();
  await act(async () => { send("https://host.example", parent, "other-window"); });
  expect(postMessage).toHaveBeenLastCalledWith(expect.objectContaining({ event: "error", data: expect.objectContaining({ message: expect.stringContaining("another document scope") }) }), "https://host.example");
  await act(async () => { send("https://host.example", parent); });
  expect(postMessage).toHaveBeenLastCalledWith(expect.objectContaining({ event: "ui-result", data: expect.objectContaining({ result: expect.objectContaining({ groups: ["test"] }) }) }), "https://host.example");
  bridge.dispose(); session.dispose(); postMessage.mockClear();
  await act(async () => { send("https://host.example", parent); }); expect(postMessage).not.toHaveBeenCalled();
  expect(() => connectStaticUiParent(target as unknown as Window, "*", session)).toThrow();
});
</storycode:copy>

### Assemble `apps/web/src/embed/StaticUiSurface.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/StaticUiSurface.test.tsx"><storycode:paste select="#f-b72f7c25d76c-expression_statement"/><storycode:paste select="#f-b72f7c25d76c-Control"/><storycode:paste select="#f-b72f7c25d76c-expression_statement-2"/></storycode:file>

</storycode:doc>
