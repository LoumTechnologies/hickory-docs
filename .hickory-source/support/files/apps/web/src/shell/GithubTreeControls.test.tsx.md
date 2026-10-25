<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/GithubTreeControls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Workspace Tree Is An Editor Buffer](../../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md)

Captured SHA-256: `69d324190ce760fb8a0c5b84bd4ca1b191b20692eb36425e5918bb0758305043`.

<book:entry path="apps/web/src/shell/GithubTreeControls.test.tsx">
// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { GithubTreeLink } from "./GithubTreeControls";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
it("refuses provider links outside HTTPS GitHub for both actors", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const open = vi.spyOn(window, "open").mockReturnValue(null);
  render(<UiActionContext.Provider value={registry}><GithubTreeLink url="javascript:alert(1)" label="Unsafe provider link" /></UiActionContext.Provider>);
  expect(screen.queryByRole("link")).toBeNull();
  const action = registry.groups().groups.flatMap(group => registry.observe(group).actions)[0];
  expect(action.enabled).toBe(false);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
    handle: action.handle, arguments: {}, request_id: "unsafe",
  })).toMatchObject({ status: "disabled" });
  expect(open).not.toHaveBeenCalled(); open.mockRestore();
});
</book:entry>

</book:doc>
