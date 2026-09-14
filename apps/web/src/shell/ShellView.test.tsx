// @vitest-environment jsdom
//
// The tab strip's pointer gestures, at the level only a real DOM answers:
// middle-click closes the tab it lands on, whatever kind of tab it is, and
// the button that closes must not also start a drag.

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { ShellView } from "./ShellView";
import { freeform, open, tab, type Layout } from "./layout";

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
    ["a document", tab("document", "notes.hick")],
    ["a terminal", tab("terminal", "term:1")],
    ["a plain file", tab("file", "README.md")],
  ])("closes %s", (_name, closing) => {
    const layout = open(open(freeform(), tab("document", "keep.hick")), closing);
    const { seen, strip } = shell(layout);

    const el = strip(closing.target);
    pointer(el, "pointerdown", 1);
    pointer(el, "pointerup", 1);

    const next = seen.at(-1)!;
    expect(next.root.type === "pane" && next.root.tabs.map((t) => t.target)).toEqual([
      "keep.hick",
    ]);
  });

  it("takes no notice of the right button", () => {
    const layout = open(open(freeform(), tab("document", "a.hick")), tab("document", "b.hick"));
    const { seen, strip } = shell(layout);

    pointer(strip("a.hick"), "pointerdown", 2);
    pointer(strip("a.hick"), "pointerup", 2);
    expect(seen).toHaveLength(0);
  });

  it("does not close the tab a press merely drifted onto", () => {
    const layout = open(open(freeform(), tab("document", "a.hick")), tab("document", "b.hick"));
    const { seen, strip } = shell(layout);

    pointer(strip("a.hick"), "pointerdown", 1);
    pointer(strip("b.hick"), "pointerup", 1);
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
