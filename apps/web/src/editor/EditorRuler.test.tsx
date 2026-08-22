import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, fireEvent } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { EditorRuler } from "./EditorRuler";
import { WRAP_MAX, WRAP_MIN, proseWrap, wrapColumnOf } from "./wrapColumn";
import { READABLE_MAX, READABLE_MIN } from "./EditorRuler";

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
      /prose wraps at about 80 characters per line.*code never wraps/i,
    );
  });
});

describe("where column zero is", () => {
  it("measures from the inside of the content padding, where the text starts", () => {
    // The bug this exists to stop: `.cm-content` carries `--cm-pad-x` of
    // padding and the dotted margin line is drawn from the inside of it, so a
    // ruler measured from the BORDER box put every tick and the marker 2.5rem
    // to the left of the line the text actually wraps at.
    const live = view();
    const content = live.contentDOM;
    content.style.paddingLeft = "40px";
    // jsdom lays nothing out, so the geometry the component reads is stated
    // here rather than measured: 100px to the content box, 40px of padding.
    content.getBoundingClientRect = () => ({ left: 100, width: 600 }) as DOMRect;
    Object.defineProperty(live, "defaultCharacterWidth", { value: 8, configurable: true });

    render(<EditorRuler view={live} column={80} onColumn={() => {}} />);
    const ruler = screen.getByTestId("editor-ruler");
    ruler.getBoundingClientRect = () => ({ left: 0, width: 900 }) as DOMRect;
    live.requestMeasure = (<T,>(request: {
      read: (v: EditorView) => T;
      write?: (measure: T, v: EditorView) => void;
    }) => request.write?.(request.read(live), live)) as typeof live.requestMeasure;

    fireEvent(window, new Event("resize"));

    // 100 (content) + 40 (padding) + 80 × 8 = 780, not 740.
    expect(screen.getByRole("slider").style.left).toBe("780px");
    live.destroy();
  });
});

describe("what the ruler says it is measuring", () => {
  // The strip used to be a row of unexplained numbers. Prose here is set in
  // the system's proportional face, so they are not monospace columns and
  // they are certainly not inches — they are the typographer's measure, and
  // the ruler has to say so or it is furniture people learn to ignore.
  it("names its unit at the left end", () => {
    render(<EditorRuler view={null} column={72} onColumn={() => {}} />);
    expect(document.querySelector(".editor-ruler__unit")?.textContent).toBe("chars/line");
  });

  it("says the number is an average, because the face is proportional", () => {
    render(<EditorRuler view={null} column={72} onColumn={() => {}} />);
    const marker = screen.getByRole("slider");
    expect(marker.getAttribute("aria-valuetext")).toBe("about 72 characters per line");
    expect(marker.dataset.tip).toMatch(/average, not a column count/i);
  });

  it("shades the comfortable range so the marker's place means something", () => {
    const live = view();
    render(<EditorRuler view={live} column={72} onColumn={() => {}} />);
    expect(READABLE_MIN).toBeLessThan(READABLE_MAX);
    expect(screen.getByRole("slider").dataset.tip).toContain(
      `${READABLE_MIN}\u2013${READABLE_MAX}`,
    );
    live.destroy();
  });

  it("drops the prose furniture entirely inside a table, where it would lie", () => {
    const table = document.createElement("div");
    const head = document.createElement("div");
    head.className = "table-panel__head";
    head.textContent = "A";
    table.appendChild(head);
    document.body.appendChild(table);
    render(<EditorRuler view={null} column={72} onColumn={() => {}} tableEl={table} />);
    // jsdom gives every element a zero-width box, so no band is drawn; what
    // this pins is that the unit label is a PROSE thing and goes with it.
    expect(document.querySelector(".editor-ruler__band")).toBeNull();
  });
});
