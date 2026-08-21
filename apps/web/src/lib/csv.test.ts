import { describe, expect, it } from "vitest";
import {
  cellAt,
  columnCount,
  guessDelimiter,
  insertColumn,
  insertRow,
  needsQuoting,
  parseCsv,
  removeColumn,
  removeRow,
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
