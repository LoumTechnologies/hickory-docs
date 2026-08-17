import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

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
