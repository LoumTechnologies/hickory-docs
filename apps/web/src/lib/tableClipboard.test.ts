// Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
// — "the clipboard is somebody else's spreadsheet".

import { describe, expect, it } from "vitest";

import {
  parseClipboardTable,
  parseHtmlTable,
  toHtmlTable,
  toTabSeparated,
} from "./tableClipboard";

describe("what goes on the clipboard", () => {
  it("is tab separated, because that is what a spreadsheet reads", () => {
    // Comma-separated text pasted into Excel lands in a single column.
    expect(toTabSeparated([["a", "b"], ["1", "2"]])).toBe("a\tb\r\n1\t2");
  });

  it("quotes a field only because it has to be quoted", () => {
    expect(toTabSeparated([["plain", "two\twords", 'a "quote"', "two\nlines"]])).toBe(
      'plain\t"two\twords"\t"a ""quote"""\t"two\nlines"',
    );
  });

  it("is also a real table, for a program that wants structure", () => {
    expect(toHtmlTable([["a", "<b>"]])).toBe("<table><tr><td>a</td><td>&lt;b&gt;</td></tr></table>");
  });

  it("keeps a newline inside a cell as a break, which is why the HTML exists", () => {
    expect(toHtmlTable([["two\nlines"]])).toContain("two<br>lines");
  });
});

describe("what comes off it", () => {
  it("reads a table pasted out of a web page", () => {
    expect(
      parseHtmlTable("<table><tr><th>name</th><th>age</th></tr><tr><td>Ada</td><td>36</td></tr>"),
    ).toEqual([
      ["name", "age"],
      ["Ada", "36"],
    ]);
  });

  it("reads the table out of the wrapper Excel puts around it", () => {
    const excel = `<html><head><style>td {}</style></head><body><!--StartFragment-->
      <table border=0 cellpadding=0><colgroup><col width=64></colgroup>
      <tr height=20><td>north</td><td>120</td></tr></table><!--EndFragment--></body></html>`;
    expect(parseHtmlTable(excel)).toEqual([["north", "120"]]);
  });

  it("pads a colspan so the columns after it still line up", () => {
    expect(parseHtmlTable('<table><tr><td colspan="2">wide</td><td>after</td></tr></table>')).toEqual(
      [["wide", "", "after"]],
    );
  });

  it("is nothing when the HTML holds no table at all", () => {
    expect(parseHtmlTable("<p>just a paragraph</p>")).toBeNull();
    expect(parseHtmlTable("")).toBeNull();
  });

  it("prefers the table to the text, because the table survived the newlines", () => {
    const block = parseClipboardTable("<table><tr><td>a<br>b</td></tr></table>", "a b");
    expect(block).toEqual([["a\nb"]]);
  });

  it("reads a spreadsheet's plain text as tab separated", () => {
    expect(parseClipboardTable(null, "north\t120\r\nsouth\t90\r\n")).toEqual([
      ["north", "120"],
      ["south", "90"],
    ]);
  });

  it("reads pasted CSV as CSV, quotes and all", () => {
    expect(parseClipboardTable(null, 'name,age\n"Lovelace, Ada",36\n')).toEqual([
      ["name", "age"],
      ["Lovelace, Ada", "36"],
    ]);
  });

  it("reads one word as the 1 × 1 rectangle it is", () => {
    // The commonest paste there is; it must not need a table to work.
    expect(parseClipboardTable(null, "hello")).toEqual([["hello"]]);
  });

  it("does not read a trailing newline as an empty row that would wipe cells", () => {
    expect(parseClipboardTable(null, "a\tb\r\n")).toEqual([["a", "b"]]);
  });

  it("is nothing when the clipboard holds nothing", () => {
    expect(parseClipboardTable(null, "")).toBeNull();
    expect(parseClipboardTable("", null)).toBeNull();
  });
});
