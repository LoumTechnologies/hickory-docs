// Guarantee: docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { KeyboardSection } from "./KeyboardSection";
import { ACTIONS, DEFAULT_SETTINGS, keymapSettings, setKeymapSettings } from "../lib/keymap";
import { api } from "../api/client";
afterEach(() => { cleanup(); vi.restoreAllMocks(); setKeymapSettings(DEFAULT_SETTINGS); });
it("shares persistent profiles and individual bindings with native controls, rejects malformed chords and retires handles", async () => {
  const save = vi.spyOn(api, "saveSettingsUi").mockResolvedValue({ window_title: null, format_on_save: false });
  setKeymapSettings(DEFAULT_SETTINGS);
  const registry = new UiActions({ workspace: "settings", window: "keyboard" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><KeyboardSection /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => {
    const total = registry.observe(group).coverage.total;
    return Array.from({ length: Math.ceil(total / 64) }, (_, index) => registry.observe(group, 64, index * 64).actions).flat();
  });
  let serial = 0;
  const invoke = async (label: string, args: Record<string, string>) => {
    const action = actions().find(action => action.label === label)!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, {kind:"agent", id:"keyboard-agent"}, { handle: action.handle, arguments: args, request_id: `key-${serial++}` }); });
    return result;
  };
  const stale = actions().find(action => action.label === "Set keyboard profile")!;
  await invoke("Set keyboard profile", { profile: "vscode" });
  expect(screen.getByLabelText("Profile")).toHaveProperty("value", "vscode");
  expect(save).toHaveBeenCalled();
  const spec = ACTIONS.find(spec => spec.scope === "editor")!;
  await invoke(`Change key for ${spec.label}`, { binding: "Mod+Shift+K" });
  expect(keymapSettings().overrides[spec.id]).toBe("Mod+Shift+K");
  const menu = ACTIONS.find(spec => spec.scope === "menu")!;
  const before = save.mock.calls.length;
  await invoke(`Change key for ${menu.label}`, { binding: "Mod+K Mod+C" });
  expect(save).toHaveBeenCalledTimes(before);
  await invoke(`Reset key for ${spec.label}`, {});
  expect(keymapSettings().overrides[spec.id]).toBeUndefined();
  fireEvent.change(screen.getByLabelText("Profile"), { target: { value: "jetbrains" } });
  await waitFor(() => expect(keymapSettings().profile).toBe("jetbrains"));
  expect(await registry.act(registry.scope, { kind:"agent", id:"keyboard-agent" }, { handle: stale.handle, arguments: { profile:"hickory" }, request_id:"stale" })).toMatchObject({status:"unavailable"});
});
