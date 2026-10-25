<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/UiLink.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `c7eccfafc843e51bfc9af06d14705b4c33e2da94fc02adf81ed8d2896f5da2eb`.

<book:entry path="apps/web/src/components/UiLink.test.tsx">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiLink, requestLinkNavigation } from "./UiLink";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
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
</book:entry>

</book:doc>
