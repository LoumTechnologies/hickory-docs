import { describe, expect, it } from "vitest";
import {
  cellAt,
  clearCells,
  columnCount,
  filledSize,
  guessDelimiter,
  insertColumn,
  insertRow,
  needsQuoting,
  parseCsv,
  pasteBlock,
  removeColumn,
  removeColumns,
  removeRow,
  removeRows,
  resizeTable,
  toMarkdown,
  withCell,
  writeCsv,
  writeField,
} from "./csv";

const rows = (text: string, delimiter?: string) => parseCsv(text, delimiter).rows;

describe("reading CSV", () => {
  it("reads plain rows and fields", () => {
    expect(rows("a,b\n1,2\n")).toEqual([
      ["a", "b"],
      ["1", "2"],
    ]);
  });

  it("reads a quoted field holding the delimiter", () => {
    expect(rows('name,note\nAda,"one, two"\n')).toEqual([
      ["name", "note"],
      ["Ada", "one, two"],
    ]);
  });

  it("reads a doubled quote as one quote", () => {
    expect(rows('a\n"she said ""hi"""\n')).toEqual([["a"], ['she said "hi"']]);
  });

  it("reads a newline inside a quoted field", () => {
    expect(rows('a,b\n"line one\nline two",x\n')).toEqual([
      ["a", "b"],
      ["line one\nline two", "x"],
    ]);
  });

  it("reads a bare quote in the middle of a field literally", () => {
    // What a spreadsheet export produces, and what RFC 4180 leaves to the
    // reader. Refusing it would be refusing a real file.
    expect(rows('a\n5" pipe\n')).toEqual([["a"], ['5" pipe']]);
  });

  it("keeps a ragged row ragged", () => {
    // Padding on read would write four fields back out and claim the file
    // said something it did not. The GRID pads for display.
    expect(rows("a,b,c\n1,2\n")).toEqual([
      ["a", "b", "c"],
      ["1", "2"],
    ]);
  });

  it("does not invent a row for the trailing newline", () => {
    expect(rows("a\nb\n")).toEqual([["a"], ["b"]]);
    expect(rows("a\nb")).toEqual([["a"], ["b"]]);
  });

  it("reads an empty text as an empty table", () => {
    expect(rows("")).toEqual([[""]]);
  });

  it("never throws on an unterminated quote", () => {
    // A table editor that refuses to open a file is useless exactly when it
    // is most needed.
    expect(() => parseCsv('a,"unterminated\n')).not.toThrow();
    expect(rows('a,"unterminated\n')).toEqual([["a", "unterminated\n"]]);
  });
});

describe("guessing the delimiter", () => {
  it("finds tabs, semicolons and pipes", () => {
    expect(guessDelimiter("a\tb\n1\t2\n")).toBe("\t");
    expect(guessDelimiter("a;b\n1;2\n")).toBe(";");
    expect(guessDelimiter("a|b\n1|2\n")).toBe("|");
  });

  it("prefers the one that is consistent per row", () => {
    // A comma inside prose is not a delimiter; a real one appears the same
    // number of times on every line.
    expect(guessDelimiter("a;b;c\none, two;three;four\nx;y;z\n")).toBe(";");
  });

  it("ignores delimiters inside quotes", () => {
    expect(guessDelimiter('a;b\n"one;two";x\n')).toBe(";");
  });

  it("falls back to a comma when there is nothing to go on", () => {
    expect(guessDelimiter("")).toBe(",");
    expect(guessDelimiter("just one column\n")).toBe(",");
  });
});

describe("writing CSV back", () => {
  it("quotes only what has to be quoted", () => {
    // Quoting everything is valid and unreadable, and it turns a one-cell
    // edit into a whole-file diff.
    expect(writeField("plain", ",")).toBe("plain");
    expect(writeField("one, two", ",")).toBe('"one, two"');
    expect(writeField('say "hi"', ",")).toBe('"say ""hi"""');
    expect(writeField("two\nlines", ",")).toBe('"two\nlines"');
    expect(writeField(" padded ", ",")).toBe('" padded "');
    expect(needsQuoting("plain", ",")).toBe(false);
  });

  it("round-trips a file byte for byte", () => {
    for (const text of [
      "a,b\n1,2\n",
      "a,b\n1,2",
      'name,note\nAda,"one, two"\n',
      "a\tb\n1\t2\n",
      'x\n"multi\nline"\n',
    ]) {
      expect(writeCsv(parseCsv(text))).toBe(text);
    }
  });

  it("keeps the line ending the file came with", () => {
    // A Windows CSV silently becoming a Unix one shows up as every line
    // changed.
    const text = "a,b\r\n1,2\r\n";
    expect(writeCsv(parseCsv(text))).toBe(text);
  });

  it("changes exactly one line when one cell is edited", () => {
    const before = "a,b\n1,2\n3,4\n";
    const after = writeCsv(withCell(parseCsv(before), 1, 1, "TWO"));
    expect(after).toBe("a,b\n1,TWO\n3,4\n");
  });
});

describe("editing the grid", () => {
  const table = parseCsv("a,b,c\n1,2,3\n");

  it("pads only as far as it must when a ragged row is edited", () => {
    const ragged = parseCsv("a,b,c\n1\n");
    expect(withCell(ragged, 1, 2, "z").rows[1]).toEqual(["1", "", "z"]);
  });

  it("inserts and removes rows", () => {
    expect(insertRow(table, 1).rows).toEqual([["a", "b", "c"], ["", "", ""], ["1", "2", "3"]]);
    expect(removeRow(table, 0).rows).toEqual([["1", "2", "3"]]);
  });

  it("inserts and removes columns in every row", () => {
    expect(insertColumn(table, 1).rows).toEqual([
      ["a", "", "b", "c"],
      ["1", "", "2", "3"],
    ]);
    expect(removeColumn(table, 1).rows).toEqual([
      ["a", "c"],
      ["1", "3"],
    ]);
  });

  it("survives removing the last row", () => {
    expect(removeRow(parseCsv("only\n"), 0).rows).toEqual([]);
  });

  it("counts columns from the widest row", () => {
    expect(columnCount(parseCsv("a,b,c\n1\n"))).toBe(3);
  });

  it("shows a short row's missing cells as empty", () => {
    const ragged = parseCsv("a,b,c\n1\n");
    expect(cellAt(ragged, 1, 0)).toBe("1");
    expect(cellAt(ragged, 1, 2)).toBe("");
    expect(cellAt(ragged, 9, 9)).toBe("");
  });
});

describe("the markdown a table weaves to", () => {
  it("writes a header row and a rule", () => {
    expect(toMarkdown(parseCsv("name,age\nAda,36\n"))).toBe(
      "| name | age |\n| --- | --- |\n| Ada | 36 |",
    );
  });

  it("escapes a pipe, which markdown has no quoting for", () => {
    // A cell containing `|` would silently become two cells.
    expect(toMarkdown(parseCsv('a\n"x|y"\n'))).toContain("x\\|y");
  });

  it("pads a ragged row out to the table's width", () => {
    expect(toMarkdown(parseCsv("a,b\n1\n"))).toBe("| a | b |\n| --- | --- |\n| 1 |  |");
  });

  it("gives a headerless table an empty header rather than inventing names", () => {
    const out = toMarkdown(parseCsv("1,2\n"), false);
    expect(out.split("\n")[0]).toBe("|  |  |");
    expect(out).toContain("| 1 | 2 |");
  });

  it("writes nothing for an empty table", () => {
    expect(toMarkdown({ rows: [] })).toBe("");
  });
});

describe("acting on more than one cell", () => {
  it("empties every cell it is given and leaves the rest alone", () => {
    const out = clearCells(parseCsv("a,b,c\n1,2,3\n"), [
      { row: 1, column: 0 },
      { row: 1, column: 2 },
    ]);
    expect(writeCsv(out)).toBe("a,b,c\n,2,\n");
  });

  it("will not pad a ragged row to clear a cell that was never in the file", () => {
    // The file keeps its ragged rows until somebody edits one.
    const out = clearCells(parseCsv("a,b,c\n1\n"), [{ row: 1, column: 2 }]);
    expect(writeCsv(out)).toBe("a,b,c\n1\n");
  });

  it("removes several rows without the earlier ones shifting the later ones", () => {
    expect(writeCsv(removeRows(parseCsv("1\n2\n3\n4\n"), [0, 2]))).toBe("2\n4\n");
  });

  it("removes several columns the same way", () => {
    expect(writeCsv(removeColumns(parseCsv("a,b,c,d\n1,2,3,4\n"), [1, 3]))).toBe("a,c\n1,3\n");
  });
});

describe("setting the size by hand", () => {
  it("grows with empty cells", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b\n1,2\n"), 3, 3))).toBe("a,b,\n1,2,\n,,\n");
  });

  it("trims from the end, which is what a smaller number means", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b,c\n1,2,3\n4,5,6\n"), 2, 2))).toBe("a,b\n1,2\n");
  });

  it("squares up a ragged row, because an explicit size is an instruction", () => {
    // The one place padding is right: everywhere else it would claim the file
    // said something it did not.
    expect(writeCsv(resizeTable(parseCsv("a,b,c\n1\n"), 2, 3))).toBe("a,b,c\n1,,\n");
  });

  it("keeps the line ending and the trailing newline the file came with", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b\r\n1,2\r\n"), 1, 2))).toBe("a,b\r\n");
  });

  it("will not make a table with no rows or no columns", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b\n1,2\n"), 0, 0))).toBe("a\n");
  });
});

describe("how much of a table holds something", () => {
  it("is smaller than the shape when the end is empty", () => {
    // What a shrink would really take, so "removes 4 rows" is said only when
    // there is something in them.
    expect(filledSize(parseCsv("a,b,\n1,2,\n,,\n"))).toEqual({ rows: 2, columns: 2 });
  });

  it("counts a cell of only spaces as empty, as the grid draws it", () => {
    expect(filledSize(parseCsv("a\n\" \"\n"))).toEqual({ rows: 1, columns: 1 });
  });

  it("is zero for a table with nothing in it", () => {
    expect(filledSize(parseCsv(",\n,\n"))).toEqual({ rows: 0, columns: 0 });
  });
});

describe("writing a block of cells in", () => {
  it("lands it with its corner where it was asked for", () => {
    const out = pasteBlock(parseCsv("a,b,c\n1,2,3\n4,5,6\n"), 1, 1, [
      ["x", "y"],
      ["z", "w"],
    ]);
    expect(writeCsv(out)).toBe("a,b,c\n1,x,y\n4,z,w\n");
  });

  it("grows the table rather than dropping what does not fit", () => {
    // A paste is an instruction: four rows pasted into the last row means the
    // table now has three more.
    const out = pasteBlock(parseCsv("a\n1\n"), 1, 0, [["x"], ["y"], ["z"]]);
    expect(writeCsv(out)).toBe("a\n x\ny\nz\n".replace(" ", ""));
  });

  it("grows sideways the same way", () => {
    expect(writeCsv(pasteBlock(parseCsv("a\n1\n"), 0, 1, [["b", "c"]]))).toBe("a,b,c\n1\n");
  });

  it("leaves a ragged row it did not touch exactly as ragged", () => {
    expect(writeCsv(pasteBlock(parseCsv("a,b,c\n1\n2,3,4\n"), 2, 0, [["x"]]))).toBe(
      "a,b,c\n1\nx,3,4\n",
    );
  });
});
