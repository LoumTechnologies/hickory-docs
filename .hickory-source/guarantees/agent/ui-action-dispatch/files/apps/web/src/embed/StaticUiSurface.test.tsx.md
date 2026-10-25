<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/StaticUiSurface.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `bb2662491f266d189c8416b8e077d1ae189486178047641fe605acd7834e333c`.

<book:entry path="apps/web/src/embed/StaticUiSurface.test.tsx">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { connectStaticUiParent, STATIC_UI_CHANNEL } from "./StaticUiSurface";
import { createEmbedUiSession } from "./uiHostSession";
import { UiActionContext, useUiCommands } from "../lib/uiActionBridge";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
function Control() {
  useUiCommands([{ key: "test", command: { label: "Read fixture", parent: "test", subject: "test", parameters: {}, prerequisites: [], read: () => ({ revision: "v1", enabled: true }), execute: async () => ({ status: "completed", value: "public fixture" }) } }]);
  return null;
}
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
</book:entry>

</book:doc>
