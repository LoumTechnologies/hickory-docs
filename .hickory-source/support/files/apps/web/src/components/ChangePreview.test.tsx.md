<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ChangePreview.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cb84955fc7d267eef2a0364dc21598753701e61dbb121e53bd2da19bae5cb35a`.

<book:entry path="apps/web/src/components/ChangePreview.test.tsx">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useEffect } from "react";
import { ChangePreview } from "./ChangePreview";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
afterEach(cleanup);
it("keeps preview child readers unmounted until actual disclosure and retires them on close", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const mounted = vi.fn();
  function Child() {
    useEffect(() => { mounted(); }, []);
    useUiReading("child preview", { value: "Actual child preview" });
    return <pre>Actual child preview</pre>;
  }
  render(<UiActionContext.Provider value={registry}><ChangePreview heading="Review proposed bytes" collapsible><Child /></ChangePreview></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const set = async (open: boolean) => act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Set evidence visibility: Review proposed bytes")!.handle,
    arguments: { open }, request_id: String(open),
  }); });
  expect(mounted).not.toHaveBeenCalled();
  expect(actions().some(action => action.label === "Read child preview bytes")).toBe(false);
  await set(true);
  expect(mounted).toHaveBeenCalledOnce();
  expect(actions().some(action => action.label === "Read child preview bytes")).toBe(true);
  await set(false);
  expect(actions().some(action => action.label === "Read child preview bytes")).toBe(false);
  expect(screen.queryByText("Actual child preview")).toBeNull();
  fireEvent.click(screen.getByText("Review proposed bytes", { selector: "summary" }));
  await waitFor(() => expect(mounted).toHaveBeenCalledTimes(2));
});
</book:entry>

</book:doc>
