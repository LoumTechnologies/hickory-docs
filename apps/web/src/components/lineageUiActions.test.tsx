import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { LineageSettings } from "./LineageSettings";
import { loadRibbonStyle } from "../lib/ribbonStyle";
afterEach(() => { cleanup(); localStorage.clear(); });
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("mounts preference commands only after the real feature-settings disclosure", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><LineageSettings /></UiActionContext.Provider>);
  expect(registry.groups().groups.some(group => group.startsWith("lineage-settings:"))).toBe(false);
  let n = 0;
  const invoke = async (parent: string, label: string, arguments_: Record<string, string | boolean>) => {
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); });
  };
  await invoke("feature-settings", "Set Lineage settings visibility", { open: true });
  const parent = registry.groups().groups.find(group => group.startsWith("lineage-settings:"))!;
  await invoke(parent, "Set lineage drawing style", { style: "bands" });
  expect(loadRibbonStyle()).toBe("bands"); expect((screen.getByLabelText("Lineage style") as HTMLSelectElement).value).toBe("bands");
  await invoke("feature-settings", "Set Lineage settings visibility", { open: false });
  expect(registry.observe(parent).actions).toEqual([]);
});
