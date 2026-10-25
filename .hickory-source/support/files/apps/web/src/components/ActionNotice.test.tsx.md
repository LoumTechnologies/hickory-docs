<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ActionNotice.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `55e3fd4d34a36b15c1b7b32eb068f43849d0d13737cc54c72b4597bef2208a63`.

<book:entry path="apps/web/src/components/ActionNotice.test.tsx">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { ActionNotice } from "./ActionNotice";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);

it("shares recovery callbacks and refuses a stale callback with the same visible label", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const retry = vi.fn(), replacement = vi.fn(), dismiss = vi.fn();
  const mount = (run: () => void) => <UiActionContext.Provider value={registry}><ActionNotice kind="fail"
    retry={{ label: "Retry Save", run }} onDismiss={dismiss}>Save not confirmed for <strong>notes.md</strong>.</ActionNotice></UiActionContext.Provider>;
  const view = render(mount(retry));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const old = actions().find(action => action.label === "Retry Save")!.handle;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old, arguments: {}, request_id: "retry" }); });
  expect(result).toMatchObject({ status: "result", outcome: { value: { callback_requested: true } } });
  expect(retry).toHaveBeenCalledOnce();
  fireEvent.click(screen.getByRole("button", { name: "Retry Save" }));
  await act(async () => {});
  expect(retry).toHaveBeenCalledTimes(2);
  view.rerender(mount(replacement));
  expect(await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, { handle: old, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(replacement).not.toHaveBeenCalled();
  await act(async () => { await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Dismiss action notice")!.handle, arguments: {}, request_id: "dismiss",
  }); });
  expect(dismiss).toHaveBeenCalledOnce();
});
</book:entry>

</book:doc>
