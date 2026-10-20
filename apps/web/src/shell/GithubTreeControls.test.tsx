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
