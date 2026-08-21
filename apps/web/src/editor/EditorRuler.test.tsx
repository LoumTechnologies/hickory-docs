import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, fireEvent } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { EditorRuler } from "./EditorRuler";
import { WRAP_MAX, WRAP_MIN, proseWrap, wrapColumnOf } from "./wrapColumn";

// vitest runs without `globals`, so Testing Library's automatic cleanup
// never registers; without this every render stacks up in one document.
afterEach(() => {
  cleanup();
});

const view = () =>
  new EditorView({
    state: EditorState.create({ doc: "prose\n", extensions: [proseWrap(() => [])] }),
    parent: document.body,
  });

describe("the ruler's margin marker", () => {
  it("is a slider with a value, not just something to drag", () => {
    // A margin you can only set by dragging is a margin nobody can set
    // precisely, and nobody without a mouse can set at all.
    render(<EditorRuler view={null} column={72} onColumn={() => {}} />);
    const marker = screen.getByRole("slider", { name: /where prose wraps/i });
    expect(marker.getAttribute("aria-valuenow")).toBe("72");
    expect(marker.getAttribute("aria-valuemin")).toBe(String(WRAP_MIN));
    expect(marker.getAttribute("aria-valuemax")).toBe(String(WRAP_MAX));
  });

  it("moves a column at a time with the arrows, ten with shift", () => {
    const onColumn = vi.fn();
    render(<EditorRuler view={null} column={80} onColumn={onColumn} />);
    const marker = screen.getByRole("slider");
    fireEvent.keyDown(marker, { key: "ArrowLeft" });
    expect(onColumn).toHaveBeenLastCalledWith(79);
    fireEvent.keyDown(marker, { key: "ArrowRight", shiftKey: true });
    expect(onColumn).toHaveBeenLastCalledWith(90);
  });

  it("will not be driven past the measures that still lay out", () => {
    const onColumn = vi.fn();
    render(<EditorRuler view={null} column={WRAP_MIN} onColumn={onColumn} />);
    fireEvent.keyDown(screen.getByRole("slider"), { key: "ArrowLeft" });
    expect(onColumn).toHaveBeenLastCalledWith(WRAP_MIN);
  });

  it("tells the editor the tab's measure, so a restored session lays out right", () => {
    const live = view();
    render(<EditorRuler view={live} column={64} onColumn={() => {}} />);
    expect(wrapColumnOf(live.state)).toBe(64);
    live.destroy();
  });

  it("says in words what it does, in the app's own tooltip layer", () => {
    render(<EditorRuler view={null} column={80} onColumn={() => {}} />);
    expect(screen.getByRole("slider").dataset.tip).toMatch(
      /prose wraps at 80 columns.*code never wraps/i,
    );
  });
});
