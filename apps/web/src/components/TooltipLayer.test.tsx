import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { TooltipLayer } from "./TooltipLayer";

// The show is a timer, so these tests drive it with `act` — which React only
// treats as a test boundary when this flag is set.
(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  vi.useRealTimers();
  cleanup();
});

/** A control with a tooltip, rendered next to the layer that draws it. */
function scene() {
  render(
    <div>
      <button data-tip="Split right" aria-label="Split right">
        <span data-testid="glyph">⇥</span>
      </button>
      <span data-testid="plain">no tooltip here</span>
      <TooltipLayer />
    </div>,
  );
  return screen.getByTestId("glyph");
}

const rest = (target: Element) => {
  fireEvent.pointerOver(target, { bubbles: true });
  act(() => void vi.advanceTimersByTime(600));
};

describe("TooltipLayer", () => {
  it("shows the owner's tooltip when the pointer rests on a child of it", () => {
    // Delegation is the point: the pointer is over the glyph, the tooltip
    // belongs to the button.
    const glyph = scene();
    fireEvent.pointerOver(glyph, { bubbles: true });
    expect(document.querySelector(".tip")).toBeNull();
    act(() => void vi.advanceTimersByTime(600));
    expect(document.querySelector(".tip")?.textContent).toBe("Split right");
  });

  it("hides on Escape, on a click, and on leaving for untooltipped ground", () => {
    const glyph = scene();
    rest(glyph);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(document.querySelector(".tip")).toBeNull();

    rest(glyph);
    fireEvent.pointerDown(glyph, { bubbles: true });
    expect(document.querySelector(".tip")).toBeNull();

    rest(glyph);
    fireEvent.pointerOver(screen.getByTestId("plain"), { bubbles: true });
    expect(document.querySelector(".tip")).toBeNull();
  });

  it("does not draw a tooltip for an element that was removed while waiting", () => {
    // The delay outlives plenty of rows: a re-render during it must not leave
    // a card pointing at nothing.
    const glyph = scene();
    fireEvent.pointerOver(glyph, { bubbles: true });
    glyph.closest("button")!.remove();
    act(() => void vi.advanceTimersByTime(600));
    expect(document.querySelector(".tip")).toBeNull();
  });

  it("leaves no listeners behind when it unmounts", () => {
    const glyph = scene();
    cleanup();
    fireEvent.pointerOver(glyph, { bubbles: true });
    act(() => void vi.advanceTimersByTime(600));
    expect(document.querySelector(".tip")).toBeNull();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("explicitly discloses visible tooltips, uses the native show controller and refuses replaced owners", async () => {
  vi.useRealTimers();
  const registry = new UiActions({ workspace: "tooltips", window: "one" }, () => true, async () => {});
  const activated = vi.fn();
  render(<UiActionContext.Provider value={registry}><button aria-label="Help" onClick={activated} data-tip="Private detailed help">?</button><button aria-label="Hidden" data-tip="hidden body" hidden>hidden</button><TooltipLayer /></UiActionContext.Provider>);
  const target = screen.getByRole("button", { name: "Help" });
  vi.spyOn(target, "getBoundingClientRect").mockReturnValue({ x: 10, y: 10, left: 10, top: 10, right: 70, bottom: 30, width: 60, height: 20, toJSON: () => ({}) });
  const group = registry.groups().groups.find(group => group.startsWith("tooltips:"))!;
  let sequence = 0;
  const invoke = async (label: string, args = {}) => {
    const command = registry.observe(group).actions.find(action => action.label === label)!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: command.handle, arguments: args, request_id: String(++sequence) }); });
    return result;
  };
  expect(JSON.stringify(registry.observe(group))).not.toContain("Private detailed help");
  expect(await invoke("Disclose visible tooltip targets", { offset: 0, limit: 64 })).toMatchObject({ outcome: { value: { disclosed: 1 } } });
  expect(JSON.stringify(registry.observe(group))).not.toContain("Private detailed help");
  expect(await invoke("Show tooltip: Help")).toMatchObject({ outcome: { value: { shown: true, activated_control: false } } });
  expect(document.querySelector(".tip")?.textContent).toBe("Private detailed help");
  expect(activated).not.toHaveBeenCalled();
  await invoke("Dismiss shown tooltip");
  expect(document.querySelector(".tip")).toBeNull();
  target.setAttribute("data-tip", "Replacement help");
  expect(await invoke("Show tooltip: Help")).toMatchObject({ status: "disabled" });
  expect(document.querySelector(".tip")).toBeNull();
});
