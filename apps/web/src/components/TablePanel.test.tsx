import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";

import { TablePanel } from "./TablePanel";

afterEach(() => {
  cleanup();
});

const CSV = "name,age\nAda,36\nGrace,45\n";

const grid = (over: Partial<React.ComponentProps<typeof TablePanel>> = {}) =>
  render(<TablePanel source={CSV} onChange={() => {}} {...over} />);

const cellText = () =>
  [...document.querySelectorAll(".table-panel__cell")].map((el) => el.textContent);

describe("what the grid shows", () => {
  it("puts the first row in the header when it names the columns", () => {
    grid();
    const head = document.querySelector("thead")!;
    expect(within(head as HTMLElement).getByText("name")).toBeTruthy();
    expect(within(head as HTMLElement).getByText("age")).toBeTruthy();
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
    expect(document.querySelectorAll("tbody td")).toHaveLength(3);
  });

  it("treats the first row as data when there is no header", () => {
    grid({ source: "1,2\n", header: false });
    expect(document.querySelector("thead")).toBeNull();
    expect(cellText()).toEqual(["1", "2"]);
  });
});

describe("editing a cell", () => {
  it("writes back a CSV that changed only the line it touched", () => {
    // The file underneath is still a file somebody reads in a diff.
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    const input = document.querySelector(".table-panel__input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Ada L" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
  });

  it("quotes a value only because it has to be quoted", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    const input = document.querySelector(".table-panel__input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Lovelace, Ada" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith('name,age\n"Lovelace, Ada",36\nGrace,45\n');
  });

  it("writes nothing when the value did not change", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.blur(document.querySelector(".table-panel__input")!);
    expect(onChange).not.toHaveBeenCalled();
  });

  it("commits and drops a row on Enter, as every spreadsheet does", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    const input = document.querySelector(".table-panel__input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Ada L" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith("name,age\nAda L,36\nGrace,45\n");
  });

  it("commits and moves right on Tab", () => {
    const onChange = vi.fn();
    grid({ onChange });
    fireEvent.click(screen.getByText("Ada"));
    const input = document.querySelector(".table-panel__input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "X" } });
    fireEvent.keyDown(input, { key: "Tab" });
    expect(onChange).toHaveBeenCalledWith("name,age\nX,36\nGrace,45\n");
    // ...and the cursor is now in the next cell, which holds 36.
    const moved = document.querySelector(".table-panel__input") as HTMLInputElement;
    expect(moved.value).toBe("36");
  });

  it("moves between cells with the arrows rather than inside the text", () => {
    // A grid where Up puts the caret at the start of the field is a grid
    // nobody can navigate.
    grid();
    fireEvent.click(screen.getByText("Ada"));
    fireEvent.keyDown(document.querySelector(".table-panel__input")!, { key: "ArrowDown" });
    const moved = document.querySelector(".table-panel__input") as HTMLInputElement;
    expect(moved.value).toBe("Grace");
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
    expect(document.querySelector(".table-panel__input")).toBeNull();
  });
});
