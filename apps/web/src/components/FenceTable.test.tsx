import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { FenceTable, tableElementFor } from "./FenceTable";

afterEach(() => {
  cleanup();
});

describe("the element a fence becomes", () => {
  it("carries the CSV across untouched", () => {
    // A promotion that reformatted the data would be a promotion nobody
    // could check.
    expect(tableElementFor("a,b\n1,2\n", "data/x.csv")).toBe(
      '<hick:table path="data/x.csv">\na,b\n1,2\n</hick:table>',
    );
  });

  it("writes no path attribute when no file was asked for", () => {
    expect(tableElementFor("a\n", "")).toBe("<hick:table>\na\n</hick:table>");
  });

  it("escapes a quote in the path rather than breaking the tag", () => {
    expect(tableElementFor("a\n", 'we"ird.csv')).toContain('path="we&quot;ird.csv"');
  });
});

describe("editing a fence as a grid", () => {
  it("writes the cells back as CSV", () => {
    const onChange = vi.fn();
    render(<FenceTable body="a,b\n1,2\n" onChange={onChange} onPromote={() => {}} />);
    expect(screen.getByTestId("table-panel")).toBeTruthy();
  });

  it("promotes with the path that was typed", () => {
    const onPromote = vi.fn();
    render(
      <FenceTable body={"a,b\n1,2\n"} onChange={() => {}} onPromote={onPromote} />,
    );
    fireEvent.change(screen.getByLabelText(/file to write the csv to/i), {
      target: { value: " data/sales.csv " },
    });
    fireEvent.click(screen.getByRole("button", { name: /make it a table/i }));
    expect(onPromote).toHaveBeenCalledWith("data/sales.csv");
  });

  it("promotes with no path at all, which is a table that writes nothing", () => {
    const onPromote = vi.fn();
    render(<FenceTable body={"a\n"} onChange={() => {}} onPromote={onPromote} />);
    fireEvent.click(screen.getByRole("button", { name: /make it a table/i }));
    expect(onPromote).toHaveBeenCalledWith("");
  });
});
