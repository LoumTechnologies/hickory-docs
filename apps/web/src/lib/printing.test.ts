import { afterEach, describe, expect, it } from "vitest";
import { printText, printTitleFor } from "./printing";

afterEach(() => {
  document.getElementById("hickory-print-sheet")?.remove();
  document.title = "";
});

describe("printing a buffer", () => {
  it("prints the WHOLE text, not the screenful CodeMirror has rendered", () => {
    // The bug this module exists to avoid: a virtualised editor prints the
    // viewport and the paper looks fine.
    let printed = "";
    const win = {
      document,
      print: () => {
        printed = document.getElementById("hickory-print-sheet")?.textContent ?? "";
      },
    } as unknown as Window;
    const text = Array.from({ length: 500 }, (_, i) => `line ${i}`).join("\n");
    printText({ title: "notes.md", text }, win);
    expect(printed).toContain("line 0");
    expect(printed).toContain("line 499");
  });

  it("names the page after the file, because that names the saved PDF", () => {
    let titleAtPrint = "";
    const win = {
      document,
      print: () => {
        titleAtPrint = document.title;
      },
    } as unknown as Window;
    document.title = "Hickory Docs";
    printText({ title: "quarterly.md", text: "x" }, win);
    expect(titleAtPrint).toBe("quarterly.md");
    expect(document.title).toBe("Hickory Docs");
  });

  it("puts the page back even when printing throws", () => {
    // A webview with printing disabled must not leave the app wearing a
    // printout's title and a hidden element full of somebody's notes.
    const win = {
      document,
      print: () => {
        throw new Error("no printer");
      },
    } as unknown as Window;
    document.title = "Hickory Docs";
    expect(() => printText({ title: "n.md", text: "secret" }, win)).toThrow();
    expect(document.getElementById("hickory-print-sheet")).toBeNull();
    expect(document.title).toBe("Hickory Docs");
  });

  it("writes the text as text, so a note about scripts stays one", () => {
    let html = "";
    const win = {
      document,
      print: () => {
        html = document.getElementById("hickory-print-sheet")?.innerHTML ?? "";
      },
    } as unknown as Window;
    printText({ title: "t", text: "<script>alert(1)</script>" }, win);
    expect(html).toContain("&lt;script&gt;");
  });
});

describe("what the printout is called", () => {
  it("uses the file's own name", () => {
    expect(printTitleFor("notes/2026/today.hick")).toBe("today.hick");
    expect(printTitleFor("README.md")).toBe("README.md");
  });

  it("falls back to the whole path rather than to nothing", () => {
    expect(printTitleFor("")).toBe("");
    expect(printTitleFor("/")).toBe("/");
  });
});
