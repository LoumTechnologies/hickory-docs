import { describe, expect, it } from "vitest";
import { columnsPhrase, rowsPhrase, tableMenuItems } from "./tableMenu";

const at = (rows: number[], columns: number[], height = 5, width = 4) => ({
  rows,
  columns,
  height,
  width,
});

const byId = (items: ReturnType<typeof tableMenuItems>, id: string) =>
  items.find((item) => item.id === id)!;

describe("what a right-click offers", () => {
  it("names the rows and columns the click actually landed on", () => {
    const items = tableMenuItems(at([2], [1]));
    expect(byId(items, "delete-rows").label).toBe("Delete row");
    expect(byId(items, "delete-rows").tip).toBe("Remove row 3");
    expect(byId(items, "delete-columns").tip).toBe("Remove column B");
  });

  it("counts a sweep, in the numbering a person reads off the grid", () => {
    const items = tableMenuItems(at([1, 2, 3], [0, 1]));
    expect(byId(items, "delete-rows").label).toBe("Delete 3 rows");
    expect(byId(items, "delete-rows").tip).toBe("Remove rows 2–4");
    expect(byId(items, "delete-columns").label).toBe("Delete 2 columns");
    expect(byId(items, "delete-columns").tip).toBe("Remove columns A–B");
  });

  it("inserts as many as are selected, which is what a spreadsheet does", () => {
    const items = tableMenuItems(at([1, 2], [0]));
    expect(byId(items, "insert-rows-above").label).toBe("Insert 2 rows above");
    expect(byId(items, "insert-columns-left").label).toBe("Insert a column left");
  });

  it("offers and disables the delete that would empty the table", () => {
    const items = tableMenuItems(at([0, 1, 2], [0, 1], 3, 2));
    expect(byId(items, "delete-rows").disabled).toBe(true);
    expect(byId(items, "delete-rows").tip).toContain("every row");
    expect(byId(items, "delete-columns").disabled).toBe(true);
    expect(byId(items, "delete-columns").tip).toContain("every column");
  });

  it("marks the destructive half so the hand slows down", () => {
    const items = tableMenuItems(at([0], [0]));
    expect(items.filter((i) => i.danger).map((i) => i.id)).toEqual([
      "delete-rows",
      "delete-columns",
    ]);
  });
});

describe("phrases", () => {
  it("reads the way a person would say it", () => {
    expect(rowsPhrase([0])).toBe("row 1");
    expect(rowsPhrase([0, 1, 2])).toBe("rows 1–3");
    expect(rowsPhrase([])).toBe("no rows");
    expect(columnsPhrase([25, 26])).toBe("columns Z–AA");
  });
});
