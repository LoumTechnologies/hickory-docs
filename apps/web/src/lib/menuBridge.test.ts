import { afterEach, describe, expect, it, vi } from "vitest";

import { DEDUPE_MS, MENU_EVENT, insertTarget, onMenuAction, type MenuAction } from "./menuBridge";

const fire = (detail: unknown) =>
  window.dispatchEvent(new CustomEvent(MENU_EVENT, { detail }));

describe("the native menu bridge", () => {
  let unsubscribe: (() => void) | null = null;
  afterEach(() => {
    unsubscribe?.();
    unsubscribe = null;
  });

  it("delivers each menu action, typed", () => {
    const seen: MenuAction[] = [];
    let clock = 0;
    unsubscribe = onMenuAction((a) => seen.push(a), () => (clock += DEDUPE_MS));
    fire("new");
    fire("save");
    fire("save-as");
    fire("settings");
    expect(seen).toEqual(["new", "save", "save-as", "settings"]);
  });

  it("handles the same action arriving twice within the window once", () => {
    // The guard against an accelerator reaching the page by two routes: the
    // duplicate is one keypress, not two intents to save.
    const handler = vi.fn();
    let clock = 1000;
    unsubscribe = onMenuAction(handler, () => clock);
    fire("save");
    clock += DEDUPE_MS - 1;
    fire("save");
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("treats the same action after the window as a second keypress", () => {
    const handler = vi.fn();
    let clock = 1000;
    unsubscribe = onMenuAction(handler, () => clock);
    fire("save");
    clock += DEDUPE_MS;
    fire("save");
    expect(handler).toHaveBeenCalledTimes(2);
  });

  it("does not dedupe two different actions in quick succession", () => {
    // Save then immediately Save As is two intents; only repeats collapse.
    const seen: MenuAction[] = [];
    const clock = 1000;
    unsubscribe = onMenuAction((a) => seen.push(a), () => clock);
    fire("save");
    fire("save-as");
    expect(seen).toEqual(["save", "save-as"]);
  });

  it("ignores details it does not know", () => {
    // A newer shell talking to an older page does nothing, not something odd.
    const handler = vi.fn();
    unsubscribe = onMenuAction(handler);
    fire("quit");
    fire(42);
    fire(undefined);
    fire("insert:");
    fire("insert:not a name");
    expect(handler).not.toHaveBeenCalled();
  });

  it("carries the element one Insert menu item names", () => {
    const seen: MenuAction[] = [];
    const clock = 1000;
    unsubscribe = onMenuAction((a) => seen.push(a), () => clock);
    fire("insert");
    fire("insert:allow-network");
    expect(seen).toEqual(["insert", "insert:allow-network"]);
    expect(insertTarget("insert:allow-network")).toBe("allow-network");
    expect(insertTarget("insert")).toBeNull();
  });

  it("stops delivering after unsubscribe", () => {
    const handler = vi.fn();
    const off = onMenuAction(handler);
    off();
    fire("new");
    expect(handler).not.toHaveBeenCalled();
  });
});
