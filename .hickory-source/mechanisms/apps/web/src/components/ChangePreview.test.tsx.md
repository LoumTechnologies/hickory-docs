<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ChangePreview.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0c17a4d8dd9b"></a>
## `apps/web/src/components/ChangePreview.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0c17a4d8dd9b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0c17a4d8dd9b-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useEffect } from "react";
import { ChangePreview } from "./ChangePreview";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
afterEach(cleanup);
</storycode:copy>
<a id="f-0c17a4d8dd9b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0c17a4d8dd9b-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/components/ChangePreview.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ChangePreview.test.tsx"><storycode:paste select="#f-0c17a4d8dd9b-expression_statement"/><storycode:paste select="#f-0c17a4d8dd9b-expression_statement-2"/></storycode:file>

</storycode:doc>
