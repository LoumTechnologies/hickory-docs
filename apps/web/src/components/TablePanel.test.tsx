// Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md and
// docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md,
// and the grid half of
// docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import { api } from "../api/client";

import { TablePanel } from "./TablePanel";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const CSV = "name,age\nAda,36\nGrace,45\n";

const grid = (over: Partial<React.ComponentProps<typeof TablePanel>> = {}) =>
  render(<TablePanel source={CSV} onChange={() => {}} {...over} />);

const cellText = () =>
  [...document.querySelectorAll(".table-panel__cell")].map((el) => el.textContent);

/** Click a cell, then open it — the spreadsheet two-step. A single click
 * selects and does not open, which is what leaves the toolbar something to
 * act on once focus has moved to a button. */
/** The one cell showing a value its text does not equal. */
const computedCell = () =>
  document.querySelector(".table-panel__cell--computed") as HTMLElement;

const open = (text: string) => {
  const cell = screen.getByText(text);
  fireEvent.click(cell);
  fireEvent.doubleClick(cell);
  return document.querySelector(".table-panel__input") as HTMLInputElement;
};

describe("what the grid shows", () => {
  it("draws the A1 furniture: letters across the top, numbers down the side", () => {
    // The reference a formula needs is the thing already on screen, rather
    // than something the author counts out.
    grid();
    const head = document.querySelector("thead")! as HTMLElement;
    expect([...head.querySelectorAll(".table-panel__head")].map((el) => el.textContent)).toEqual([
      "A",
      "B",
    ]);
    expect(
      [...document.querySelectorAll(".table-panel__gutter")].map((el) => el.textContent),
    ).toEqual(["1", "2", "3"]);
  });

  it("keeps the CSV's own header row in the body, marked as names", () => {
    // The letter row is the grid's furniture and is not in the file; the
    // first line of the CSV is row 1, exactly as a formula counts it.
    grid();
    const names = document.querySelector(".table-panel__names")!.closest("tr")!;
    expect([...names.querySelectorAll(".table-panel__cell")].map((el) => el.textContent)).toEqual([
      "name",
      "age",
    ]);
  });

  it("shows every body row", () => {
    grid();
    const body = document.querySelector("tbody")!;
    expect(within(body as HTMLElement).getByText("Ada")).toBeTruthy();
    expect(within(body as HTMLElement).getByText("Grace")).toBeTruthy();
  });

  it("pads a ragged row with empty cells for display", () => {
    // The file keeps its ragged row; the grid is what pads.
    grid({ source: "a,b,c\n1\n" });
    const rows = document.querySelectorAll("tbody tr");
    expect(rows).toHaveLength(2);
    expect(rows[1].querySelectorAll("td")).toHaveLength(3);
  });

  it("treats the first row as data when there is no header", () => {
    // The letter row is always there — it is the grid's own furniture. What
    // goes away is the styling that claims the first line names the columns.
    grid({ source: "1,2\n", header: false });
    expect(document.querySelector(".table-panel__names")).toBeNull();
    expect(cellText()).toEqual(["1", "2"]);
  });
});

describe("editing a cell", () => {
  it("writes back a CSV that changed only the line it touched", () => {
    // The file underneath is still a file somebody reads in a diff.
    const onChange = vi.fn();
    grid({ onChange });
    const input = open("Ada");
    fireEvent.change(input, { target: { value: "Ada L" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
  });

  it("quotes a value only because it has to be quoted", () => {
    const onChange = vi.fn();
    grid({ onChange });
    const input = open("Ada");
    fireEvent.change(input, { target: { value: "Lovelace, Ada" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith('name,age\n"Lovelace, Ada",36\nGrace,45\n');
  });

  it("writes nothing when the value did not change", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.blur(open("Ada"));
    expect(onChange).not.toHaveBeenCalled();
  });

  it("commits and drops a row on Enter, as every spreadsheet does", () => {
    const onChange = vi.fn();
    grid({ onChange });
    const input = open("Ada");
    fireEvent.change(input, { target: { value: "Ada L" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
  });

  it("commits and moves right on Tab", () => {
    const onChange = vi.fn();
    grid({ onChange });
    const input = open("Ada");
    fireEvent.change(input, { target: { value: "X" } });
    fireEvent.keyDown(input, { key: "Tab" });
    expect(onChange).toHaveBeenCalledWith("name,age\nX,36\nGrace,45\n");
    // ...and the selection is now on the next cell, which holds 36.
    expect(document.querySelector(".table-panel__cell--selected")!.textContent).toBe("36");
  });

  it("moves between cells with the arrows rather than inside the text", () => {
    // A grid where Up puts the caret at the start of the field is a grid
    // nobody can navigate.
    grid();
    fireEvent.keyDown(open("Ada"), { key: "ArrowDown" });
    // Committing and stepping leaves the arrived-at cell SELECTED, not open:
    // an editor that opened it would swallow the next keystroke as a
    // replacement of what was already there.
    expect(document.querySelector(".table-panel__input")).toBeNull();
    expect(document.querySelector(".table-panel__cell--selected")!.textContent).toBe("Grace");
  });
});

describe("changing the shape", () => {
  it("adds a row", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByRole("button", { name: "+ Row" }));
    expect(onChange).toHaveBeenCalledWith("name,age\nAda,36\nGrace,45\n,\n");
  });

  it("adds a column", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByRole("button", { name: "+ Column" }));
    expect(onChange).toHaveBeenCalledWith("name,age,\nAda,36,\nGrace,45,\n");
  });

  it("will not remove a row until one is chosen", () => {
    grid();
    expect(screen.getByRole("button", { name: "− Row" })).toHaveProperty("disabled", true);
  });

  it("removes the row the cursor is in", () => {
    const onChange = vi.fn();
    grid({ onChange });
    // One click, not two: removing the row a cell is in must work from the
    // SELECTED state, which is the state the toolbar can be reached from.
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.click(screen.getByRole("button", { name: "− Row" }));
    expect(onChange).toHaveBeenCalledWith("name,age\nGrace,45\n");
  });
});

describe("a table the reader may not edit", () => {
  it("says where to change it instead of silently discarding keystrokes", () => {
    grid({ onChange: undefined });
    expect(screen.getByText(/written by a document/i)).toBeTruthy();
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.doubleClick(screen.getByText("Ada"));
    expect(document.querySelector(".table-panel__input")).toBeNull();
  });
});

describe("formulas", () => {
  it("asks for nothing when the table names no language", () => {
    // A cell beginning with `=` is then just text, which is what a table of
    // shell snippets needs it to be.
    const evaluate = vi.spyOn(api, "evaluateFormulas");
    grid({ source: "a\n=1+1\n" });
    expect(evaluate).not.toHaveBeenCalled();
  });

  it("asks for nothing when the table has no formulas", () => {
    const evaluate = vi.spyOn(api, "evaluateFormulas");
    grid({ source: "a\n1\n", language: "python" });
    expect(evaluate).not.toHaveBeenCalled();
  });

  it("shows a formula's value, not its text", async () => {
    vi.spyOn(api, "evaluateFormulas").mockResolvedValue({
      values: { A2: "2" },
      errors: {},
    });
    grid({ source: "a\n=1+1\n", language: "python" });
    await waitFor(() => expect(computedCell().textContent).toBe("2"));
    expect(cellText()).toEqual(["a", "2"]);
  });

  it("keeps the formula in the FILE, which is what is reviewed", async () => {
    // The value is a view. What is written down is the expression, which is
    // the thing worth reading in a diff and the thing that still works on a
    // machine with no interpreter.
    const onChange = vi.fn();
    vi.spyOn(api, "evaluateFormulas").mockResolvedValue({
      values: { A2: "2" },
      errors: {},
    });
    grid({ source: "a\n=1+1\n", language: "python", onChange });
    await waitFor(() => expect(computedCell().textContent).toBe("2"));
    // Entering the cell reveals the formula, as every spreadsheet does.
    fireEvent.click(computedCell());
    fireEvent.doubleClick(computedCell());
    expect((document.querySelector(".table-panel__input") as HTMLInputElement).value).toBe("=1+1");
  });

  it("shows the language's own message on a broken formula", async () => {
    vi.spyOn(api, "evaluateFormulas").mockResolvedValue({
      values: {},
      errors: { A2: "NameError: name 'nope' is not defined" },
    });
    grid({ source: "a\n=nope\n", language: "python" });
    await waitFor(() => {
      const bad = document.querySelector(".table-panel__cell--bad") as HTMLElement;
      expect(bad).toBeTruthy();
      expect(bad.dataset.tip).toContain("NameError");
    });
  });

  it("still renders and still edits when nothing can evaluate", async () => {
    // A missing interpreter is the common case and not a fault.
    vi.spyOn(api, "evaluateFormulas").mockRejectedValue(
      new Error("formulas in `python` need one of: python3, python"),
    );
    grid({ source: "a\n=1+1\n", language: "python" });
    await waitFor(() => expect(screen.getByRole("status").textContent).toMatch(/python3/));
    // The formula's own text is shown rather than a blank cell, which would
    // be a lie about there being nothing there.
    expect(screen.getByText("=1+1")).toBeTruthy();
  });
});

describe("selection, which is what a toolbar can act on", () => {
  it("selects on one click and opens on a second act, as a spreadsheet does", () => {
    grid();
    fireEvent.click(screen.getByText("Ada"));
    expect(document.querySelector(".table-panel__input")).toBeNull();
    expect(document.querySelector(".table-panel__cell--selected")!.textContent).toBe("Ada");
    fireEvent.doubleClick(screen.getByText("Ada"));
    expect(document.querySelector(".table-panel__input")).toBeTruthy();
  });

  it("keeps the cell selected after its input loses focus", () => {
    // The bug this exists to stop: "− Row" blurs the input, the blur clears
    // the cursor, the button disables, and the click never lands.
    grid();
    const input = open("Ada");
    fireEvent.blur(input);
    expect(screen.getByRole("button", { name: "− Row" })).toHaveProperty("disabled", false);
  });

  it("removes a row that was being EDITED, not merely selected", () => {
    const onChange = vi.fn();
    grid({ onChange });
    open("Ada");
    fireEvent.click(screen.getByRole("button", { name: "− Row" }));
    expect(onChange).toHaveBeenCalledWith("name,age\nGrace,45\n");
  });

  it("names the selected cell in A1 notation", () => {
    grid();
    expect(screen.getByTestId("table-panel-name").textContent).toBe("—");
    fireEvent.click(screen.getByText("45"));
    expect(screen.getByTestId("table-panel-name").textContent).toBe("B3");
  });

  it("selects a WHOLE column from its letter, and a whole row from its number", () => {
    // Not merely the cursor moved into the column: `B:B` is what the name box
    // of a spreadsheet says, and it is what "− Column" then acts on.
    grid();
    fireEvent.click(screen.getByText("B"));
    expect(screen.getByTestId("table-panel-name").textContent).toBe("B:B");
    fireEvent.click(screen.getByText("3"));
    expect(screen.getByTestId("table-panel-name").textContent).toBe("3:3");
  });
});

describe("moving around without opening a cell", () => {
  const selected = () => document.querySelector(".table-panel__cell--selected")!.textContent;

  it("moves with the arrows", () => {
    grid();
    const cell = screen.getByText("Ada");
    fireEvent.click(cell);
    fireEvent.keyDown(cell, { key: "ArrowRight" });
    expect(selected()).toBe("36");
  });

  it("opens the cell on Enter, and on F2", () => {
    grid();
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.keyDown(screen.getByText("Ada"), { key: "Enter" });
    expect((document.querySelector(".table-panel__input") as HTMLInputElement).value).toBe("Ada");
  });

  it("replaces the cell when a printable character is typed", () => {
    // The fastest thing a spreadsheet does, and the one people miss most.
    grid();
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.keyDown(screen.getByText("Ada"), { key: "X" });
    expect((document.querySelector(".table-panel__input") as HTMLInputElement).value).toBe("X");
  });

  it("clears the cell on Delete", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.keyDown(screen.getByText("Ada"), { key: "Delete" });
    expect(onChange).toHaveBeenCalledWith("name,age\n,36\nGrace,45\n");
  });

  it("abandons an edit on Escape rather than saving it", () => {
    const onChange = vi.fn();
    grid({ onChange });
    const input = open("Ada");
    fireEvent.change(input, { target: { value: "wrong" } });
    fireEvent.keyDown(input, { key: "Escape" });
    expect(onChange).not.toHaveBeenCalled();
  });
});

describe("the formula bar", () => {
  const bar = () => screen.getByLabelText("Cell contents") as HTMLInputElement;

  it("shows the selected cell's own text, not what it came to", async () => {
    vi.spyOn(api, "evaluateFormulas").mockResolvedValue({ values: { A2: "2" }, errors: {} });
    grid({ source: "a\n=1+1\n", language: "python" });
    await waitFor(() => expect(computedCell().textContent).toBe("2"));
    fireEvent.click(computedCell());
    expect(bar().value).toBe("=1+1");
  });

  it("says a formula begins with `=`, in the language the table names", () => {
    // The only place that says so. A table whose formulas silently stay text
    // is a table whose author never learns why.
    grid({ language: "python" });
    fireEvent.click(screen.getByText("Ada"));
    expect(bar().placeholder).toBe("Text, or = for a python expression");
  });

  it("says so differently when the table names no language at all", () => {
    grid();
    fireEvent.click(screen.getByText("Ada"));
    expect(bar().placeholder).toBe("Text — this table names no formula language");
  });

  it("edits the cell it names", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.change(bar(), { target: { value: "=1+1" } });
    fireEvent.keyDown(bar(), { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith("name,age\n=1+1,36\nGrace,45\n");
  });
});

describe("showing the formulas instead of their values", () => {
  it("swaps every formula for its text, and back", async () => {
    // Ctrl+` — how a sheet somebody else built is read rather than used.
    vi.spyOn(api, "evaluateFormulas").mockResolvedValue({ values: { A2: "2" }, errors: {} });
    grid({ source: "a\n=1+1\n", language: "python" });
    await waitFor(() => expect(computedCell().textContent).toBe("2"));
    fireEvent.click(screen.getByRole("button", { name: "Formulas" }));
    expect(cellText()).toEqual(["a", "=1+1"]);
    fireEvent.click(screen.getByRole("button", { name: "Formulas" }));
    expect(cellText()).toEqual(["a", "2"]);
  });

  it("offers the toggle only where formulas are possible", () => {
    grid();
    expect(screen.queryByRole("button", { name: "Formulas" })).toBeNull();
  });
});

describe("a size the table keeps", () => {
  it("opens at the size it was left", () => {
    grid({ layout: { height: 300, widths: { "0": 220 } } });
    const scroll = document.querySelector(".table-panel__scroll") as HTMLElement;
    expect(scroll.style.height).toBe("300px");
    const cols = document.querySelectorAll("colgroup col");
    // The first col is the row-number gutter, so column A is the second.
    expect((cols[1] as HTMLElement).style.width).toBe("220px");
  });

  it("reports a new height, so the workspace can remember it", () => {
    const onLayout = vi.fn();
    grid({ layout: { height: 200 }, onLayout });
    const bar = screen.getByTestId("height-resizer");
    fireEvent.mouseDown(bar, { clientY: 500 });
    fireEvent.mouseMove(window, { clientY: 560 });
    expect(onLayout).toHaveBeenLastCalledWith({ height: 260 });
    fireEvent.mouseUp(window);
  });

  it("will not be dragged smaller than its own furniture", () => {
    const onLayout = vi.fn();
    grid({ layout: { height: 100 }, onLayout });
    fireEvent.mouseDown(screen.getByTestId("height-resizer"), { clientY: 500 });
    fireEvent.mouseMove(window, { clientY: 0 });
    expect(onLayout).toHaveBeenLastCalledWith({ height: 64 });
    fireEvent.mouseUp(window);
  });

  it("keeps the widths when the height changes, and the height when a width does", () => {
    const onLayout = vi.fn();
    grid({ layout: { height: 200, widths: { "1": 150 } }, onLayout });
    fireEvent.mouseDown(screen.getByTestId("height-resizer"), { clientY: 0 });
    fireEvent.mouseMove(window, { clientY: 40 });
    expect(onLayout).toHaveBeenLastCalledWith({ height: 240, widths: { "1": 150 } });
    fireEvent.mouseUp(window);
  });

  it("never writes a size into the CSV — the file is a dataset, not a layout", () => {
    const onChange = vi.fn();
    grid({ onChange, onLayout: () => {} });
    fireEvent.mouseDown(screen.getByTestId("height-resizer"), { clientY: 0 });
    fireEvent.mouseMove(window, { clientY: 40 });
    fireEvent.mouseUp(window);
    expect(onChange).not.toHaveBeenCalled();
  });
});

// A body cell and a row number can read the same — row 1 and a cell holding
// "1" — so these say which they mean rather than relying on the text alone.
const cell = (text: string) => screen.getByText(text, { selector: ".table-panel__cell" });
const gutter = (row: string) => screen.getByText(row, { selector: ".table-panel__gutter" });
const head = (letter: string) => screen.getByText(letter, { selector: ".table-panel__head" });
const name = () => screen.getByTestId("table-panel-name").textContent;

describe("selecting more than one cell", () => {
  const inRange = () =>
    [...document.querySelectorAll(".table-panel__cell--selected, .table-panel__cell--within")].map(
      (el) => el.textContent,
    );

  it("sweeps a rectangle with a drag", () => {
    grid({ source: "a,b,c\nd,e,f\ng,h,i\n" });
    fireEvent.mouseDown(cell("d"));
    fireEvent.mouseEnter(cell("h"));
    fireEvent.mouseUp(window);
    expect(name()).toBe("A2:B3");
    expect(inRange()).toEqual(["d", "e", "g", "h"]);
  });

  it("keeps the anchor still when the drag goes backwards", () => {
    // The anchor is what the formula bar edits; a drag that flipped it would
    // move the edit somewhere the person did not click.
    grid({ source: "a,b\nc,d\ne,f\n" });
    fireEvent.mouseDown(cell("f"));
    fireEvent.mouseEnter(cell("c"));
    expect(name()).toBe("A2:B3");
    expect(document.querySelector(".table-panel__cell--selected")!.textContent).toBe("f");
  });

  it("stops sweeping once the button is up, even outside the grid", () => {
    // Releasing over the toolbar must not leave the grid selecting forever.
    grid({ source: "a,b\nc,d\ne,f\n" });
    fireEvent.mouseDown(cell("c"));
    fireEvent.mouseUp(window);
    fireEvent.mouseEnter(cell("f"));
    expect(name()).toBe("A2");
  });

  it("extends to a shift-click without a drag at all", () => {
    grid({ source: "a,b\nc,d\ne,f\n" });
    fireEvent.click(cell("c"));
    fireEvent.click(cell("f"), { shiftKey: true });
    expect(name()).toBe("A2:B3");
  });

  it("extends with shift and an arrow, which is the only way on a keyboard", () => {
    grid({ source: "a,b\nc,d\ne,f\n" });
    fireEvent.click(cell("c"));
    fireEvent.keyDown(cell("c"), { key: "ArrowDown", shiftKey: true });
    expect(name()).toBe("A2:A3");
  });

  it("drags across the letters for whole columns, never half of one", () => {
    grid({ source: "a,b,c\nd,e,f\n" });
    fireEvent.mouseDown(head("A"));
    fireEvent.mouseEnter(head("B"));
    expect(name()).toBe("A:B");
    expect(inRange()).toEqual(["a", "b", "d", "e"]);
  });

  it("drags down the numbers for whole rows", () => {
    grid({ source: "a,b\nc,d\ne,f\n" });
    fireEvent.mouseDown(gutter("1"));
    fireEvent.mouseEnter(gutter("2"));
    expect(name()).toBe("1:2");
    expect(inRange()).toEqual(["a", "b", "c", "d"]);
  });

  it("takes the whole table from the corner", () => {
    grid({ source: "a,b\nc,d\n" });
    fireEvent.click(screen.getByLabelText("Select the whole table"));
    expect(name()).toBe("A1:B2");
  });

  it("takes the whole table on Ctrl+A", () => {
    grid({ source: "a,b\nc,d\n" });
    fireEvent.click(cell("a"));
    fireEvent.keyDown(cell("a"), { key: "a", ctrlKey: true });
    expect(name()).toBe("A1:B2");
  });
});

describe("acting on what is selected", () => {
  it("empties every cell in the range on Delete, not just the anchor", () => {
    const onChange = vi.fn();
    grid({ source: "a,b,c\nd,e,f\n", onChange });
    fireEvent.mouseDown(cell("d"));
    fireEvent.mouseEnter(cell("e"));
    fireEvent.keyDown(cell("d"), { key: "Delete" });
    expect(onChange).toHaveBeenCalledWith("a,b,c\n,,f\n");
  });

  it("removes every row in the range, and says how many", () => {
    const onChange = vi.fn();
    grid({ source: "a\nb\nc\nd\n", onChange });
    fireEvent.mouseDown(cell("b"));
    fireEvent.mouseEnter(cell("c"));
    fireEvent.click(screen.getByRole("button", { name: "− 2 Rows" }));
    expect(onChange).toHaveBeenCalledWith("a\nd\n");
  });

  it("writes nothing when Delete empties cells that were already empty", () => {
    // A write that changes nothing still marks the document dirty and still
    // lands in the undo history.
    const onChange = vi.fn();
    grid({ source: "a,b\n,\n", onChange });
    fireEvent.click(gutter("2"));
    fireEvent.keyDown(cell("a"), { key: "Delete" });
    expect(onChange).toHaveBeenCalledTimes(0);
  });

  it("removes a whole column chosen by its letter", () => {
    const onChange = vi.fn();
    grid({ source: "a,b,c\nd,e,f\n", onChange });
    fireEvent.click(head("B"));
    fireEvent.click(screen.getByRole("button", { name: "− Column" }));
    expect(onChange).toHaveBeenCalledWith("a,c\nd,f\n");
  });

  it("refuses to remove EVERY row, which is not a table any more", () => {
    // What selecting a whole column would otherwise do.
    grid({ source: "a,b\nc,d\n" });
    fireEvent.click(head("A"));
    const button = screen.getByRole("button", { name: "− 2 Rows" });
    expect(button).toHaveProperty("disabled", true);
    expect(button.dataset.tip).toContain("every row");
  });
});

describe("pointing at a cell while writing a formula", () => {
  const typing = () => document.querySelector(".table-panel__input") as HTMLInputElement;
  const openCell = (text: string) => {
    fireEvent.click(cell(text));
    fireEvent.doubleClick(cell(text));
    return typing();
  };

  it("writes the clicked cell's reference into the formula", () => {
    // The feature that makes formulas usable without counting rows.
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange });
    fireEvent.change(openCell("e"), { target: { value: "=" } });
    fireEvent.mouseDown(cell("d"));
    fireEvent.click(cell("d"));
    expect(typing().value).toBe("=B2");
    // Still editing: a click that meant a reference is not a click that left.
    expect(onChange).not.toHaveBeenCalled();
  });

  it("REPLACES the last reference when another cell is clicked", () => {
    // Clicking around to find the right cell leaves one reference, not five.
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange: () => {} });
    fireEvent.change(openCell("e"), { target: { value: "=" } });
    fireEvent.click(cell("d"));
    fireEvent.click(cell("c"));
    expect(typing().value).toBe("=A2");
  });

  it("keeps both when the person typed an operator between the clicks", () => {
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange: () => {} });
    fireEvent.change(openCell("e"), { target: { value: "=" } });
    fireEvent.click(cell("c"));
    fireEvent.change(typing(), { target: { value: "=A2+" } });
    fireEvent.click(cell("d"));
    expect(typing().value).toBe("=A2+B2");
  });

  it("leaves the cell as an ordinary click would when the formula wants no operand", () => {
    // After `42` the expression is not asking for anything, so a click is a
    // click — which is how you still get out of a formula cell.
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange });
    fireEvent.change(openCell("e"), { target: { value: "=42" } });
    fireEvent.click(cell("d"));
    expect(onChange).toHaveBeenCalledWith("a,b\nc,d\n=42,f\n");
    expect(name()).toBe("B2");
  });

  it("Down points at the cell beneath, and Down again walks one further", () => {
    // Excel's point mode from the keyboard: the first arrow leaves from the
    // cell being edited, the next from where the pointer already is, and the
    // reference is replaced rather than stacked.
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange: () => {} });
    const input = openCell("a");
    fireEvent.change(input, { target: { value: "=" } });
    input.setSelectionRange(1, 1);
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(typing().value).toBe("=A2");
    expect(cell("c").className).toContain("table-panel__cell--pointed");
    fireEvent.keyDown(typing(), { key: "ArrowDown" });
    expect(typing().value).toBe("=A3");
    expect(cell("e").className).toContain("table-panel__cell--pointed");
    expect(cell("c").className).not.toContain("table-panel__cell--pointed");
    // Still editing, still A1 selected: the pointer is not the selection.
    expect(name()).toBe("A1");
  });

  it("Right points only from the end of the text; in the middle it moves the caret", () => {
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange: () => {} });
    const input = openCell("a");
    fireEvent.change(input, { target: { value: "=1+2" } });
    input.setSelectionRange(2, 2);
    fireEvent.keyDown(input, { key: "ArrowRight" });
    expect(typing().value).toBe("=1+2"); // a caret move, not a reference
    fireEvent.change(typing(), { target: { value: "=1+" } });
    typing().setSelectionRange(3, 3);
    fireEvent.keyDown(typing(), { key: "ArrowRight" });
    expect(typing().value).toBe("=1+B1");
    // At the grid's edge the pointer stays put — you are still pointing.
    fireEvent.keyDown(typing(), { key: "ArrowRight" });
    expect(typing().value).toBe("=1+B1");
    // And Left walks it back.
    fireEvent.keyDown(typing(), { key: "ArrowLeft" });
    expect(typing().value).toBe("=1+A1");
  });

  it("typing after pointing keeps the reference, and the next arrow leaves from the edited cell", () => {
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange: () => {} });
    const input = openCell("a");
    fireEvent.change(input, { target: { value: "=" } });
    input.setSelectionRange(1, 1);
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(typing().value).toBe("=A2");
    fireEvent.change(typing(), { target: { value: "=A2+" } });
    expect(document.querySelector(".table-panel__cell--pointed")).toBeNull();
    typing().setSelectionRange(4, 4);
    fireEvent.keyDown(typing(), { key: "ArrowRight" });
    expect(typing().value).toBe("=A2+B1");
  });

  it("Down still moves between cells when the formula wants no operand", () => {
    // After `42` there is nothing to point for, so the arrow is the arrow it
    // always was: commit and step down.
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange });
    const input = openCell("a");
    fireEvent.change(input, { target: { value: "=42" } });
    input.setSelectionRange(3, 3);
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(onChange).toHaveBeenCalledWith("=42,b\nc,d\ne,f\n");
    expect(name()).toBe("A2");
  });

  it("does not point from a cell that is not a formula", () => {
    // Where there is no `=` there is nothing to write a reference into.
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\ne,f\n", language: "python", onChange });
    fireEvent.change(openCell("e"), { target: { value: "hello" } });
    fireEvent.mouseDown(cell("d"));
    fireEvent.click(cell("d"));
    // Committed and left, rather than a reference written into it.
    expect(onChange).toHaveBeenCalledWith("a,b\nc,d\nhello,f\n");
    expect(name()).toBe("B2");
  });

  it("does not point in a table that names no formula language", () => {
    // `=` is text there, so there are no references for a click to mean.
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\ne,f\n", onChange });
    fireEvent.change(openCell("e"), { target: { value: "=" } });
    fireEvent.mouseDown(cell("d"));
    fireEvent.click(cell("d"));
    expect(onChange).toHaveBeenCalledWith("a,b\nc,d\n=,f\n");
  });
});

describe("Enter, while a cell is open", () => {
  it("finalizes the edit and moves to the cell beneath", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(cell("Ada"));
    fireEvent.doubleClick(cell("Ada"));
    const input = document.querySelector(".table-panel__input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Ada L" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
    // Selected, not open: an editor that opened the next cell would swallow
    // the next keystroke as a replacement of what was already there.
    expect(name()).toBe("A3");
    expect(document.querySelector(".table-panel__input")).toBeNull();
  });

  it("goes UP on shift-Enter, as every spreadsheet does", () => {
    grid({ onChange: () => {} });
    fireEvent.click(cell("Grace"));
    fireEvent.doubleClick(cell("Grace"));
    fireEvent.keyDown(document.querySelector(".table-panel__input")!, {
      key: "Enter",
      shiftKey: true,
    });
    expect(name()).toBe("A2");
  });

  it("stays put at the bottom rather than falling off the table", () => {
    grid({ onChange: () => {} });
    fireEvent.click(cell("Grace"));
    fireEvent.doubleClick(cell("Grace"));
    fireEvent.keyDown(document.querySelector(".table-panel__input")!, { key: "Enter" });
    expect(name()).toBe("A3");
  });
});

describe("an edit that is interrupted by a click somewhere else", () => {
  it("is committed, not thrown away, when a drag begins on another cell", () => {
    // The input unmounts the moment editing ends and React sends no blur to a
    // field that is no longer there, so nothing else would catch this.
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(cell("Ada"));
    fireEvent.doubleClick(cell("Ada"));
    fireEvent.change(document.querySelector(".table-panel__input")!, {
      target: { value: "Ada L" },
    });
    fireEvent.mouseDown(cell("45"));
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
  });

  it("is committed when a column is taken by its letter", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(cell("Ada"));
    fireEvent.doubleClick(cell("Ada"));
    fireEvent.change(document.querySelector(".table-panel__input")!, {
      target: { value: "Ada L" },
    });
    fireEvent.mouseDown(head("B"));
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
  });
});

describe("stepping through the formulas from the grid", () => {
  const trace = (steps: import("../api/types").FormulaStep[]) =>
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps, values: {}, errors: {} });

  it("offers stepping only where there are formulas to step through", () => {
    grid({ source: "a\n1\n", language: "python", onChange: () => {} });
    expect(screen.queryByRole("button", { name: "Step" })).toBeNull();
    cleanup();
    grid({ source: "a\n=1+1\n", language: "python", onChange: () => {} });
    expect(screen.getByRole("button", { name: "Step" })).toBeTruthy();
  });

  it("does not offer it where `=` is only text", () => {
    // No language means no formulas, so there is no order to step through.
    grid({ source: "a\n=1+1\n", onChange: () => {} });
    expect(screen.queryByRole("button", { name: "Step" })).toBeNull();
  });

  it("marks the cell whose turn it is, and the cells it read", async () => {
    vi.spyOn(api, "evaluateFormulas").mockResolvedValue({ values: { B1: "20" }, errors: {} });
    trace([
      {
        cell: "B1",
        level: 0,
        expression: "A1*2",
        bindings: [{ cell: "A1", text: "10", kind: "number" }],
        value: "20",
        error: null,
      },
    ]);
    grid({ source: "10,=A1*2\n", header: false, language: "python", onChange: () => {} });
    fireEvent.click(screen.getByRole("button", { name: "Step" }));
    await waitFor(() =>
      expect(document.querySelector(".table-panel__cell--stepping")).toBeTruthy(),
    );
    expect(document.querySelector(".table-panel__cell--read")!.textContent).toBe("10");
  });

  it("takes the marks away when stepping stops", async () => {
    trace([
      { cell: "A1", level: 0, expression: "1+1", bindings: [], value: "2", error: null },
    ]);
    grid({ source: "=1+1\n", header: false, language: "python", onChange: () => {} });
    fireEvent.click(screen.getByRole("button", { name: "Step" }));
    await waitFor(() =>
      expect(document.querySelector(".table-panel__cell--stepping")).toBeTruthy(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() =>
      expect(document.querySelector(".table-panel__cell--stepping")).toBeNull(),
    );
    expect(screen.queryByTestId("formula-debug")).toBeNull();
  });
});

describe("the row numbers, on both sides", () => {
  const gutters = (row: HTMLElement) => [...row.querySelectorAll(".table-panel__gutter")];

  it("numbers every row beside the first column, always", () => {
    grid({ source: "a,b\nc,d\n" });
    const rows = [...document.querySelectorAll("tbody tr")] as HTMLElement[];
    expect(rows.map((row) => row.firstElementChild!.textContent)).toEqual(["1", "2"]);
  });

  it("adds a lane at the far right as well, where the editor's rail is", () => {
    // As well as, never instead of: the left-hand numbers are where a
    // spreadsheet puts them and where a hand goes to grab a row.
    grid({ source: "a,b\nc,d\n", laneRight: true });
    const rows = [...document.querySelectorAll("tbody tr")] as HTMLElement[];
    for (const [index, row] of rows.entries()) {
      expect(gutters(row).map((el) => el.textContent)).toEqual([
        String(index + 1),
        String(index + 1),
      ]);
      expect(row.lastElementChild!.className).toContain("table-panel__gutter--lane");
    }
  });

  it("gives the whole-table corner one accessible name, not two", () => {
    grid({ source: "a,b\nc,d\n", laneRight: true });
    expect(document.querySelectorAll(".table-panel__corner")).toHaveLength(2);
    expect(screen.getAllByLabelText("Select the whole table")).toHaveLength(1);
  });

  it("selects a row from either lane", () => {
    grid({ source: "a,b\nc,d\n", laneRight: true });
    const [left, right] = screen.getAllByText("2", { selector: ".table-panel__gutter" });
    fireEvent.click(right);
    expect(name()).toBe("2:2");
    fireEvent.click(screen.getByLabelText("Select the whole table"));
    fireEvent.click(left);
    expect(name()).toBe("2:2");
  });
});

describe("how much of the table is on screen", () => {
  const scroller = () => screen.getByTestId("table-panel-scroll");
  const rowsOf = (n: number) =>
    Array.from({ length: n }, (_, i) => `row ${i + 1}`).join("\n") + "\n";

  it("lets a short table be its own height", () => {
    grid({ source: rowsOf(4) });
    expect(scroller().style.height).toBe("");
  });

  it("stops growing at nine rows and scrolls instead", () => {
    // A table in a document is a paragraph. A hundred-row dataset that pushes
    // the prose after it off the screen has stopped being one.
    grid({ source: rowsOf(40) });
    // The letter row, then nine rows of 24px.
    expect(scroller().style.height).toBe("238px");
    expect(scroller().style.maxHeight).toBe("none");
    // Every row is still THERE — it is a viewport, not a truncation.
    expect(document.querySelectorAll("tbody tr")).toHaveLength(40);
  });

  it("keeps a remembered height instead, however many rows there are", () => {
    // It is what this person decided this table is worth.
    grid({ source: rowsOf(40), layout: { height: 500 } });
    expect(scroller().style.height).toBe("500px");
  });

  it("can be dragged past the nine rows to see more at a time", () => {
    const onLayout = vi.fn();
    grid({ source: rowsOf(40), onLayout });
    const handle = screen.getByTestId("height-resizer");
    fireEvent.mouseDown(handle, { clientY: 0 });
    fireEvent.mouseMove(window, { clientY: 200 });
    fireEvent.mouseUp(window);
    // Dragged down from the nine-row height, not from a default.
    expect(onLayout).toHaveBeenCalledWith({ height: 438 });
    expect(scroller().style.height).toBe("438px");
  });
});

describe("typing a size into the indicator", () => {
  const chip = () => screen.getByRole("button", { name: "2 × 2" });
  const rowsField = () => screen.getByLabelText("Rows") as HTMLInputElement;
  const columnsField = () => screen.getByLabelText("Columns") as HTMLInputElement;

  it("opens on the indicator and starts at the size it is", () => {
    grid({ source: "a,b\nc,d\n", onChange: () => {} });
    fireEvent.click(chip());
    expect(screen.getByRole("dialog", { name: "Table size" })).toBeTruthy();
    expect(rowsField().value).toBe("2");
    expect(columnsField().value).toBe("2");
  });

  it("grows the table to what was typed", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(chip());
    fireEvent.change(rowsField(), { target: { value: "3" } });
    fireEvent.change(columnsField(), { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(onChange).toHaveBeenCalledWith("a,b,\nc,d,\n,,\n");
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("says what a shrink is about to take, while the number is being typed", () => {
    // Not a confirm-then-do step, which trains people to click through.
    grid({ source: "a,b\nc,d\n", onChange: () => {} });
    fireEvent.click(chip());
    fireEvent.change(rowsField(), { target: { value: "1" } });
    expect(screen.getByRole("status").textContent).toContain("Removes 1 row");
    fireEvent.change(columnsField(), { target: { value: "1" } });
    expect(screen.getByRole("status").textContent).toContain("1 row and 1 column");
  });

  it("says nothing about loss when the cells it would drop are empty", () => {
    grid({ source: "a,b\n,\n", onChange: () => {} });
    fireEvent.click(chip());
    fireEvent.change(rowsField(), { target: { value: "1" } });
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("refuses a grid that is a dataset rather than a spreadsheet, and says what to do", () => {
    grid({ source: "a,b\nc,d\n", onChange: () => {} });
    fireEvent.click(chip());
    fireEvent.change(rowsField(), { target: { value: "9000" } });
    fireEvent.change(columnsField(), { target: { value: "9" } });
    expect(screen.getByRole("button", { name: "Apply" })).toHaveProperty("disabled", true);
    expect(screen.getByRole("status").textContent).toContain("exec cell");
  });

  it("refuses a table with no rows at all", () => {
    grid({ source: "a,b\nc,d\n", onChange: () => {} });
    fireEvent.click(chip());
    fireEvent.change(rowsField(), { target: { value: "0" } });
    expect(screen.getByRole("button", { name: "Apply" })).toHaveProperty("disabled", true);
    expect(screen.getByRole("status").textContent).toContain("at least one row");
  });

  it("closes on Escape without changing anything", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(chip());
    fireEvent.change(rowsField(), { target: { value: "9" } });
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(onChange).not.toHaveBeenCalled();
  });
});

describe("the clipboard, over a selection", () => {
  /** A stand-in for the real thing: the event carries every flavour that was
   * copied, and the grid reads and writes several. */
  const clipboard = (held: Record<string, string> = {}) => ({
    data: held,
    getData: (type: string) => held[type] ?? "",
    setData: (type: string, value: string) => {
      held[type] = value;
    },
  });

  const panel = () => screen.getByTestId("table-panel");

  it("copies the selected rectangle as tab-separated text and as a table", () => {
    grid({ source: "a,b,c\nd,e,f\n" });
    fireEvent.mouseDown(cell("a"));
    fireEvent.mouseEnter(cell("e"));
    const board = clipboard();
    fireEvent.copy(panel(), { clipboardData: board });
    expect(board.data["text/plain"]).toBe("a\tb\r\nd\te");
    expect(board.data["text/html"]).toBe(
      "<table><tr><td>a</td><td>b</td></tr><tr><td>d</td><td>e</td></tr></table>",
    );
  });

  it("copies a whole column taken by its letter", () => {
    grid({ source: "a,b\nc,d\n" });
    fireEvent.click(head("B"));
    const board = clipboard();
    fireEvent.copy(panel(), { clipboardData: board });
    expect(board.data["text/plain"]).toBe("b\r\nd");
  });

  it("leaves the clipboard to the input while a cell is open", () => {
    // Copying the characters you selected inside the field is exactly right
    // there, and it is the browser's own behaviour.
    grid({ onChange: () => {} });
    fireEvent.click(cell("Ada"));
    fireEvent.doubleClick(cell("Ada"));
    const board = clipboard();
    fireEvent.copy(document.querySelector(".table-panel__input")!, { clipboardData: board });
    expect(board.data["text/plain"]).toBeUndefined();
  });

  it("leaves it to the formula bar too, where the same thing is meant", () => {
    grid({ onChange: () => {} });
    fireEvent.click(cell("Ada"));
    const board = clipboard();
    fireEvent.copy(screen.getByLabelText("Cell contents"), { clipboardData: board });
    expect(board.data["text/plain"]).toBeUndefined();
  });

  it("cuts by copying and then emptying what it copied", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.mouseDown(cell("a"));
    fireEvent.mouseEnter(cell("b"));
    const board = clipboard();
    fireEvent.cut(panel(), { clipboardData: board });
    expect(board.data["text/plain"]).toBe("a\tb");
    expect(onChange).toHaveBeenCalledWith(",\nc,d\n");
  });

  it("pastes a spreadsheet's tab-separated text at the anchor", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(cell("a"));
    fireEvent.paste(panel(), {
      clipboardData: clipboard({ "text/plain": "north\t120\r\nsouth\t90\r\n" }),
    });
    expect(onChange).toHaveBeenCalledWith("north,120\nsouth,90\n");
  });

  it("pastes an HTML table copied out of a web page", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(cell("c"));
    fireEvent.paste(panel(), {
      clipboardData: clipboard({
        "text/html": "<table><tr><td>x</td><td>y</td></tr></table>",
        "text/plain": "x y",
      }),
    });
    // The table won, which is what keeps a cell holding a newline intact.
    expect(onChange).toHaveBeenCalledWith("a,b\nx,y\n");
  });

  it("pastes plain CSV as CSV", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(cell("a"));
    fireEvent.paste(panel(), {
      clipboardData: clipboard({ "text/plain": 'name,age\n"Lovelace, Ada",36\n' }),
    });
    expect(onChange).toHaveBeenCalledWith('name,age\n"Lovelace, Ada",36\n');
  });

  it("grows the table rather than dropping what does not fit", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(cell("c"));
    fireEvent.paste(panel(), {
      clipboardData: clipboard({ "text/plain": "1\t2\t3\r\n4\t5\t6" }),
    });
    // The row above is not touched and so is not padded: the file keeps what
    // it had until somebody edits it.
    expect(onChange).toHaveBeenCalledWith("a,b\n1,2,3\n4,5,6\n");
  });

  it("selects exactly what it wrote, so one Delete puts it back", () => {
    grid({ source: "a,b\nc,d\n", onChange: () => {} });
    fireEvent.click(cell("a"));
    fireEvent.paste(panel(), { clipboardData: clipboard({ "text/plain": "1\t2\r\n3\t4" }) });
    expect(name()).toBe("A1:B2");
  });

  it("pastes one word into one cell", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(cell("d"));
    fireEvent.paste(panel(), { clipboardData: clipboard({ "text/plain": "hello" }) });
    expect(onChange).toHaveBeenCalledWith("a,b\nc,hello\n");
  });

  it("writes nothing when the clipboard holds nothing this grid can use", () => {
    const onChange = vi.fn();
    grid({ source: "a,b\nc,d\n", onChange });
    fireEvent.click(cell("a"));
    fireEvent.paste(panel(), {
      clipboardData: clipboard({ "text/html": "<p>a paragraph</p>", "text/plain": "" }),
    });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("refuses to paste into a table the reader may not edit, but still copies", () => {
    grid({ source: "a,b\nc,d\n", onChange: undefined });
    fireEvent.click(head("A"));
    const board = clipboard({ "text/plain": "x" });
    fireEvent.paste(panel(), { clipboardData: board });
    fireEvent.copy(panel(), { clipboardData: board });
    expect(board.data["text/plain"]).toBe("a\r\nc");
  });
});

describe("dragging a grid line", () => {
  /** The handle on one of a cell's two edges. */
  const edge = (text: string, axis: "row" | "column") =>
    cell(text).parentElement!.querySelector(`[data-testid="${axis}-resizer"]`)!;

  const drag = (handle: Element, by: { x?: number; y?: number }) => {
    fireEvent.mouseDown(handle, { clientX: 0, clientY: 0 });
    fireEvent.mouseMove(window, { clientX: by.x ?? 0, clientY: by.y ?? 0 });
    fireEvent.mouseUp(window);
  };

  it("resizes the column from a line inside the table, not only from the header", () => {
    // The line you want to move is usually the one your pointer is next to.
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    drag(edge("d", "column"), { x: 30 });
    expect(onLayout).toHaveBeenCalledWith({ widths: { "1": 134 } });
  });

  it("resizes the row from the line along its bottom", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    drag(edge("c", "row"), { y: 20 });
    expect(onLayout).toHaveBeenCalledWith({ heights: { "1": 44 } });
  });

  it("gives the row its height, so a row is never two heights at once", () => {
    grid({ source: "a,b\nc,d\n", layout: { heights: { "1": 60 } } });
    const rows = document.querySelectorAll("tbody tr") as NodeListOf<HTMLElement>;
    expect(rows[1].style.getPropertyValue("--table-row-height")).toBe("60px");
    expect(rows[0].style.getPropertyValue("--table-row-height")).toBe("24px");
  });

  it("will not drag a column or a row away to nothing", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    drag(edge("d", "column"), { x: -500 });
    expect(onLayout).toHaveBeenCalledWith({ widths: { "1": 40 } });
    drag(edge("d", "row"), { y: -500 });
    expect(onLayout).toHaveBeenLastCalledWith({ widths: { "1": 40 }, heights: { "1": 16 } });
  });

  it("hands a press back to the cell when it never became a drag", () => {
    // The strip runs along the edge of a cell people also want to select. A
    // five-pixel dead zone would make the bottom of every row unclickable.
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    fireEvent.mouseDown(edge("d", "row"), { clientX: 0, clientY: 0 });
    fireEvent.mouseUp(window);
    expect(name()).toBe("B2");
    expect(onLayout).not.toHaveBeenCalled();
  });

  it("ignores a wobble, which is what a click on a five-pixel strip is", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    fireEvent.mouseDown(edge("d", "column"), { clientX: 0, clientY: 0 });
    fireEvent.mouseMove(window, { clientX: 2 });
    fireEvent.mouseUp(window);
    expect(onLayout).not.toHaveBeenCalled();
    expect(name()).toBe("B2");
  });

  it("resizes a row from its number, where a spreadsheet puts the handle", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    const handle = gutter("2").querySelector('[data-testid="row-resizer"]')!;
    drag(handle, { y: 16 });
    expect(onLayout).toHaveBeenCalledWith({ heights: { "1": 40 } });
  });

  it("takes the whole row when its number's line is pressed and not dragged", () => {
    grid({ source: "a,b\nc,d\n" });
    fireEvent.mouseDown(gutter("2").querySelector('[data-testid="row-resizer"]')!);
    fireEvent.mouseUp(window);
    expect(name()).toBe("2:2");
  });

  it("counts the nine visible rows at the heights they actually are", () => {
    const rows = Array.from({ length: 20 }, (_, i) => `row ${i + 1}`).join("\n") + "\n";
    grid({ source: rows, layout: { heights: { "0": 100 } } });
    // The letter row, one row of 100, and eight of 24.
    expect(screen.getByTestId("table-panel-scroll").style.height).toBe("314px");
  });
});

describe("double-clicking a grid line", () => {
  /**
   * jsdom lays nothing out, so this stands in for the two things a browser
   * reports about a cell: the box it is being held to, and the size its
   * content would take if it were not.
   *
   * Faithful on purpose about `scrollWidth` being the LARGER of the two —
   * that is what made a fit only ever grow, and the shrink case below is what
   * fails if anything goes back to reading it.
   */
  const layout = (
    selector: string,
    axis: "width" | "height",
    { box, content }: { box: number; content: number },
  ) => {
    for (const node of document.querySelectorAll(selector)) {
      const el = node as HTMLElement;
      Object.defineProperty(el, "getBoundingClientRect", {
        configurable: true,
        value: () => {
          // A browser answers with the content's own size only while the
          // element is not being held to a width or a height.
          const size = el.style[axis] === "max-content" ? content : box;
          return axis === "width" ? { width: size, height: 20 } : { width: 100, height: size };
        },
      });
      Object.defineProperty(el, axis === "width" ? "scrollWidth" : "scrollHeight", {
        configurable: true,
        value: Math.max(box, content),
      });
    }
  };

  const inColumn = (column: number, sizes: { box: number; content: number }) =>
    layout(`[data-column="${column}"]`, "width", sizes);
  const inRow = (row: number, sizes: { box: number; content: number }) =>
    layout(`[data-row="${row}"]`, "height", sizes);

  const edge = (text: string, axis: "row" | "column") =>
    cell(text).parentElement!.querySelector(`[data-testid="${axis}-resizer"]`)!;

  /**
   * What a browser actually sends: two presses and then the dblclick, the
   * second press carrying `detail: 2`.
   *
   * `fireEvent.doubleClick` alone sends none of that, and the presses are
   * half the behaviour here — the first one selects, the second must not, and
   * the fit has to give the first one back.
   */
  const doubleClick = (handle: Element) => {
    fireEvent.mouseDown(handle, { detail: 1 });
    fireEvent.mouseUp(window, { detail: 1 });
    fireEvent.click(handle, { detail: 1 });
    fireEvent.mouseDown(handle, { detail: 2 });
    fireEvent.mouseUp(window, { detail: 2 });
    fireEvent.click(handle, { detail: 2 });
    fireEvent.doubleClick(handle);
  };

  it("fits the column to the widest thing in it", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    inColumn(1, { box: 104, content: 180 });
    doubleClick(edge("d", "column"));
    // The cell's own border, and a pixel so rounding up never lands on the
    // text and clips it into an ellipsis.
    expect(onLayout).toHaveBeenCalledWith({ widths: { "1": 182 } });
  });

  it("measures what the text needs, not the box the cell is already in", () => {
    // The bug this replaced: a cell fills its column, so its scroll size is
    // the size it ALREADY has whenever the text is smaller — and a fit built
    // on that could only ever grow. Double-clicking anything expanded it
    // slightly, the slack being the only thing that changed.
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout, layout: { widths: { "1": 600 } } });
    inColumn(1, { box: 600, content: 70 });
    doubleClick(edge("d", "column"));
    expect(onLayout).toHaveBeenCalledWith({ widths: { "1": 72 } });
  });

  it("leaves the cell laid out exactly as it found it", () => {
    // The constraint is lifted only for the length of the measurement; a fit
    // that left it off would leave one cell sized unlike the ones beside it.
    grid({ source: "a,b\nc,d\n" });
    inColumn(1, { box: 104, content: 180 });
    doubleClick(edge("d", "column"));
    expect((cell("d") as HTMLElement).style.width).toBe("");
  });

  it("fits from the line between two column letters, where a spreadsheet has it", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    inColumn(0, { box: 104, content: 300 });
    doubleClick(head("A").querySelector('[data-testid="column-resizer"]')!);
    expect(onLayout).toHaveBeenCalledWith({ widths: { "0": 302 } });
  });

  it("fits the row to the tallest thing in it", () => {
    // A cell holding a newline — which a paste from a web page produces — is
    // several lines tall, and this is what makes room for it.
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    inRow(1, { box: 24, content: 64 });
    doubleClick(edge("c", "row"));
    // A height's only slack is the cell's own border: there is no ellipsis on
    // this axis to guard against.
    expect(onLayout).toHaveBeenCalledWith({ heights: { "1": 65 } });
  });

  it("shrinks a row that was dragged too tall, back to its one line", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout, layout: { heights: { "1": 300 } } });
    inRow(1, { box: 300, content: 23 });
    doubleClick(edge("c", "row"));
    expect(onLayout).toHaveBeenCalledWith({ heights: { "1": 24 } });
  });

  it("fits from the line between two row numbers too", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    inRow(0, { box: 24, content: 40 });
    doubleClick(gutter("1").querySelector('[data-testid="row-resizer"]')!);
    expect(onLayout).toHaveBeenCalledWith({ heights: { "0": 41 } });
  });

  it("fits an empty column to the smallest a column may be", () => {
    // The honest answer to how much room nothing needs.
    const onLayout = vi.fn();
    grid({ source: "a,\nc,\n", onLayout });
    inColumn(1, { box: 104, content: 13 });
    const bothInRow = cell("a")
      .parentElement!.parentElement!.querySelectorAll('[data-testid="column-resizer"]');
    doubleClick(bothInRow[1]!);
    expect(onLayout).toHaveBeenCalledWith({ widths: { "1": 40 } });
  });

  it("will not let one enormous cell make a column nobody can scroll past", () => {
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    inColumn(1, { box: 104, content: 40_000 });
    doubleClick(edge("d", "column"));
    expect(onLayout).toHaveBeenCalledWith({ widths: { "1": 2000 } });
  });

  it("leaves a row that already fits exactly where it was", () => {
    // Measured in a browser: one line of this font is 23px, so a border and
    // nothing else puts a fitted row back at the 24 it started at. Slack for
    // an ellipsis that a height cannot have would nudge every row a pixel
    // taller on every double-click.
    const onLayout = vi.fn();
    grid({ source: "a,b\nc,d\n", onLayout });
    inRow(1, { box: 24, content: 23 });
    doubleClick(edge("c", "row"));
    expect(onLayout).toHaveBeenCalledWith({ heights: { "1": 24 } });
  });

  it("leaves the selection alone — resizing is not a way of choosing something", () => {
    // A double-click is two clicks, and the first cannot know the second is
    // coming, so its tap has already moved the selection by the time "fit" is
    // the answer. The fit puts it back.
    grid({ source: "a,b\nc,d\n", onLayout: () => {} });
    fireEvent.click(cell("a"));
    inColumn(1, { box: 104, content: 180 });
    doubleClick(edge("d", "column"));
    expect(name()).toBe("A1");
    expect(document.querySelector(".table-panel__cell--selected")!.textContent).toBe("a");
  });

  it("leaves a range alone, not just a single cell", () => {
    grid({ source: "a,b\nc,d\n" });
    fireEvent.mouseDown(cell("a"));
    fireEvent.mouseEnter(cell("d"));
    fireEvent.mouseUp(window);
    inRow(1, { box: 24, content: 40 });
    doubleClick(gutter("2").querySelector('[data-testid="row-resizer"]')!);
    expect(name()).toBe("A1:B2");
  });

  it("selects nothing when nothing was selected before", () => {
    grid({ source: "a,b\nc,d\n" });
    inColumn(1, { box: 104, content: 180 });
    doubleClick(edge("d", "column"));
    expect(name()).toBe("—");
  });

  it("still selects on a press that is only ever one click", () => {
    // The tap is not delayed behind a double-click timer; that would make an
    // ordinary press on a line feel broken.
    grid({ source: "a,b\nc,d\n" });
    fireEvent.mouseDown(edge("d", "column"));
    fireEvent.mouseUp(window, { detail: 1 });
    expect(name()).toBe("B2");
  });

  it("does not open the cell behind the line for editing", () => {
    // A double-click on a cell means "edit this"; on the line it means fit.
    grid({ source: "a,b\nc,d\n", onChange: () => {} });
    inColumn(1, { box: 104, content: 90 });
    doubleClick(edge("d", "column"));
    expect(document.querySelector(".table-panel__input")).toBeNull();
  });
});

describe("the right-click menu", () => {
  /** Open the menu over a cell by its text. */
  const menuOver = (text: string) => {
    fireEvent.contextMenu(screen.getByText(text));
    return document.querySelector('[role="menu"]') as HTMLElement;
  };
  const pick = (id: string) =>
    fireEvent.click(document.querySelector(`[data-menu-item="${id}"]`)!);

  it("selects the cell it was opened over, so the items name that row", () => {
    grid();
    const menu = menuOver("Ada");
    expect(menu).not.toBeNull();
    expect(menu.getAttribute("aria-label")).toContain("row 2");
    expect(menu.getAttribute("aria-label")).toContain("column A");
  });

  it("removes the row it was opened over", () => {
    const onChange = vi.fn();
    grid({ onChange });
    menuOver("Ada");
    pick("delete-rows");
    expect(onChange).toHaveBeenCalledWith("name,age\nGrace,45\n");
  });

  it("removes the column it was opened over", () => {
    const onChange = vi.fn();
    grid({ onChange });
    menuOver("36");
    pick("delete-columns");
    expect(onChange).toHaveBeenCalledWith("name\nAda\nGrace\n");
  });

  it("inserts a row above and below the cell, which the toolbar cannot", () => {
    const onChange = vi.fn();
    grid({ onChange });
    menuOver("Ada");
    pick("insert-rows-above");
    expect(onChange).toHaveBeenCalledWith("name,age\n,\nAda,36\nGrace,45\n");
    onChange.mockClear();
    menuOver("Ada");
    pick("insert-rows-below");
    expect(onChange).toHaveBeenCalledWith("name,age\nAda,36\n,\nGrace,45\n");
  });

  it("inserts a column beside the cell", () => {
    const onChange = vi.fn();
    grid({ onChange });
    menuOver("36");
    pick("insert-columns-left");
    expect(onChange).toHaveBeenCalledWith("name,,age\nAda,,36\nGrace,,45\n");
  });

  it("keeps a sweep intact when the click lands inside it", () => {
    const onChange = vi.fn();
    grid({ onChange });
    // Sweep the two data rows, then right-click inside them: the menu must
    // act on both, not on the one cell under the pointer.
    fireEvent.mouseDown(screen.getByText("Ada"));
    fireEvent.mouseEnter(screen.getByText("45"));
    fireEvent.mouseUp(window);
    menuOver("36");
    pick("delete-rows");
    expect(onChange).toHaveBeenCalledWith("name,age\n");
  });

  it("offers the delete that would empty the table, and refuses to run it", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(document.querySelector(".table-panel__corner")!);
    fireEvent.contextMenu(screen.getByText("Ada"));
    const item = document.querySelector<HTMLButtonElement>('[data-menu-item="delete-rows"]')!;
    expect(item.disabled).toBe(true);
    fireEvent.click(item);
    expect(onChange).not.toHaveBeenCalled();
  });

  it("is not offered at all on a table with a document behind it", () => {
    render(<TablePanel source={CSV} />);
    fireEvent.contextMenu(screen.getByText("Ada"));
    expect(document.querySelector('[role="menu"]')).toBeNull();
  });
});
