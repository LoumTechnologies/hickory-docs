// @vitest-environment jsdom
// The map-in-a-page rule's shared signal: a widget is scenery until clicked
// into, and Escape (outside a field) or clicking away disengages it.
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { Engaged } from "./Engaged";

afterEach(cleanup);

function gate() {
  const { container } = render(
    <Engaged className="widget">
      <input aria-label="field" />
      <div data-testid="body" />
    </Engaged>,
  );
  return { container, root: container.firstElementChild as HTMLElement };
}

describe("the engage gate", () => {
  it("engages on click-in, disengages on Escape and on focus leaving", () => {
    const { root } = gate();
    expect(root.className).not.toContain("engaged-gate--on");
    fireEvent.pointerDown(root);
    fireEvent.focus(root);
    expect(root.className).toContain("engaged-gate--on");
    fireEvent.keyDown(root, { key: "Escape" });
    fireEvent.blur(root);
    expect(root.className).not.toContain("engaged-gate--on");
  });

  it("leaves Escape alone inside a field — that one means 'undo my typing'", () => {
    const { container, root } = gate();
    const field = container.querySelector("input")!;
    field.focus();
    fireEvent.focus(root);
    expect(root.className).toContain("engaged-gate--on");
    fireEvent.keyDown(field, { key: "Escape" });
    // No blur was forced: still engaged, still editing.
    expect(root.className).toContain("engaged-gate--on");
    expect(document.activeElement).toBe(field);
  });
});
