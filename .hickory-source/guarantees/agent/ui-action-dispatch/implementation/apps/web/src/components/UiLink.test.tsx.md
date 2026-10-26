<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/UiLink.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9503a258f59f"></a>
## `apps/web/src/components/UiLink.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-9503a258f59f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9503a258f59f-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiLink, requestLinkNavigation } from "./UiLink";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-9503a258f59f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9503a258f59f-expression_statement-2">
it("uses the same isolated browser navigation for native and agent links and rejects script URLs", async () => {
  const open = vi.spyOn(window, "open").mockReturnValue({} as Window);
  const registry = new UiActions({ workspace: "public", window: "links" }, () => true, async () => {});
  const app = render(<UiActionContext.Provider value={registry}><UiLink href="https://example.com/evidence" target="_blank">Inspect evidence</UiLink></UiActionContext.Provider>);
  fireEvent.click(app.getByText("Inspect evidence"));
  expect(open).toHaveBeenCalledWith("https://example.com/evidence", "_blank", "noopener");
  const descriptor = registry.observe("browser-links", 32).actions[0];
  expect(descriptor.value).toMatchObject({ href: "https://example.com/evidence" });
  await act(async () => { expect(await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "navigate" })).toMatchObject({ outcome: { value: { navigation_requested: true, loaded: "not observed" } } }); });
  expect(open).toHaveBeenCalledTimes(2);
  expect(requestLinkNavigation("javascript:alert(1)")).toMatchObject({ status: "unavailable" }); expect(open).toHaveBeenCalledTimes(2);
  open.mockReturnValue(null);
  expect(requestLinkNavigation("https://example.com/evidence", "_blank")).toMatchObject({ status: "requires_input" });
});
</storycode:copy>

### Assemble `apps/web/src/components/UiLink.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/UiLink.test.tsx"><storycode:paste select="#f-9503a258f59f-expression_statement"/><storycode:paste select="#f-9503a258f59f-expression_statement-2"/></storycode:file>

</storycode:doc>
