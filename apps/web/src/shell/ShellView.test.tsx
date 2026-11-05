// @vitest-environment jsdom
//
// The tab strip's pointer gestures, at the level only a real DOM answers:
// middle-click closes the tab it lands on, whatever kind of tab it is, and
// the button that closes must not also start a drag.

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { ShellView } from "./ShellView";
import { activate, freeform, open, tab, type Layout } from "./layout";

afterEach(cleanup);

/** Renders a shell over `layout` and reports the layout it hands back. */
function shell(layout: Layout) {
  const seen: Layout[] = [];
  const { container } = render(
    <ShellView
      layout={layout}
      onLayout={(next) => seen.push(next)}
      render={(t) => <div>{t.target}</div>}
    />,
  );
  /** The tab in the strip whose file is `target`. */
  const strip = (target: string) =>
    container.querySelector<HTMLElement>(`.shell-tab[data-shell-tab-target="${target}"]`)!;
  return { seen, strip };
}

/** jsdom has no PointerEvent; a MouseEvent carries every field read here. */
function pointer(el: Element, type: "pointerdown" | "pointerup", button: number) {
  fireEvent(el, new MouseEvent(type, { bubbles: true, cancelable: true, button }));
}

describe("middle-click closes a tab", () => {
  it.each([
    ["a document", tab("document", "notes.md")],
    ["a terminal", tab("terminal", "term:1")],
    ["a plain file", tab("file", "README.md")],
  ])("closes %s", (_name, closing) => {
    const layout = open(open(freeform(), tab("document", "keep.md")), closing);
    const { seen, strip } = shell(layout);

    const el = strip(closing.target);
    pointer(el, "pointerdown", 1);
    pointer(el, "pointerup", 1);

    const next = seen.at(-1)!;
    expect(next.root.type === "pane" && next.root.tabs.map((t) => t.target)).toEqual([
      "keep.md",
    ]);
  });

  it("takes no notice of the right button", () => {
    const layout = open(open(freeform(), tab("document", "a.md")), tab("document", "b.md"));
    const { seen, strip } = shell(layout);

    pointer(strip("a.md"), "pointerdown", 2);
    pointer(strip("a.md"), "pointerup", 2);
    expect(seen).toHaveLength(0);
  });

  it("does not close the tab a press merely drifted onto", () => {
    const layout = open(open(freeform(), tab("document", "a.md")), tab("document", "b.md"));
    const { seen, strip } = shell(layout);

    pointer(strip("a.md"), "pointerdown", 1);
    pointer(strip("b.md"), "pointerup", 1);
    expect(seen).toHaveLength(0);
  });
});

// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
describe("unsaved tabs", () => {
  it("draws the data-driven asterisk and asks its owner before closing", () => {
    const dirty = tab("untitled", "untitled", "Untitled");
    const layout = open(freeform(), dirty);
    const request = vi.fn();
    const { getByRole } = render(
      <ShellView
        layout={layout}
        onLayout={() => {}}
        render={() => null}
        dirtyTabIds={new Set([dirty.id])}
        onRequestCloseTab={request}
      />,
    );
    expect(getByRole("tab").textContent).toBe("Untitled *");
    fireEvent.click(getByRole("button", { name: "Close Untitled" }));
    expect(request).toHaveBeenCalledWith(layout.focus, dirty.id);
  });
});


describe("active tab visibility", () => {
  it.each(["top", "side"] as const)("reveals %s tabs with strip-relative geometry and preserves deliberate scrolling", tabStyle => {
    const vertical = tabStyle === "side";
    let resized = () => {};
    const disconnect = vi.fn();
    vi.stubGlobal("ResizeObserver", class {
      constructor(callback: () => void) { resized = callback; }
      observe() {}
      disconnect = disconnect;
    });
    let size = 100;
    const dimension = vi.spyOn(HTMLElement.prototype, vertical ? "clientHeight" : "clientWidth", "get").mockImplementation(() => size);
    const geometry = vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (this: HTMLElement) {
      const strip = this.closest<HTMLElement>('[role="tablist"]');
      const scroll = strip ? (vertical ? strip.scrollTop : strip.scrollLeft) : 0;
      const offset = this.classList.contains("shell-tab") ? (this.dataset.shellTabIndex === "1" ? 80 : 0) - scroll : 0;
      return { left: 200 + (vertical ? 0 : offset), top: 150 + (vertical ? offset : 0), right: 200 + (vertical ? size : offset + 80), bottom: 150 + (vertical ? offset + 80 : size), width: vertical ? size : 80, height: vertical ? 80 : size, x: 0, y: 0, toJSON() {} };
    });
    try {
      const layout = open(open(freeform(), tab("file", "a.md")), tab("file", "b.md"));
      const props = { onLayout: () => {}, render: () => <input defaultValue="unsaved" />, tabStyle };
      const ui = render(<ShellView {...props} layout={layout} />);
      const strip = ui.container.querySelector<HTMLElement>('[role="tablist"]')!;
      const position = () => vertical ? strip.scrollTop : strip.scrollLeft;
      expect(position()).toBe(60);
      const first = activate(layout, layout.focus, 0);
      ui.rerender(<ShellView {...props} layout={first} />);
      expect(position()).toBe(0);
      if (vertical) strip.scrollTop = 20; else strip.scrollLeft = 20;
      ui.rerender(<ShellView {...props} layout={{ ...first }} />);
      expect(position()).toBe(20);
      ui.rerender(<ShellView {...props} layout={layout} />);
      expect(position()).toBe(60);
      size = 90; resized();
      expect(position()).toBe(70);
      ui.unmount();
      expect(disconnect).toHaveBeenCalled();
    } finally {
      geometry.mockRestore(); dimension.mockRestore(); vi.unstubAllGlobals();
    }
  });
});
