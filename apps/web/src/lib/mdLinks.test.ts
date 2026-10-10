import { describe, expect, it } from "vitest";
import {
  findLinks,
  imageMarkdown,
  isLocalTarget,
  isUrl,
  linkOverSelection,
  normalizeUrl,
  resolveTarget,
  splitFragment,
  wovenTarget,
} from "./mdLinks";

describe("findLinks", () => {
  it("finds a link and its parts", () => {
    const text = "see [the notes](notes/plan.md) for more";
    const [link] = findLinks(text);
    expect(link.image).toBe(false);
    expect(text.slice(link.from, link.to)).toBe("[the notes](notes/plan.md)");
    expect(link.text).toBe("the notes");
    expect(link.target).toBe("notes/plan.md");
    expect(text.slice(link.targetFrom, link.targetTo)).toBe("notes/plan.md");
  });

  it("marks an image as one, bang included in its range", () => {
    const text = "![a chart](assets/chart.png)";
    const [image] = findLinks(text);
    expect(image.image).toBe(true);
    expect(text.slice(image.from, image.to)).toBe(text);
  });

  it("balances parentheses inside a destination", () => {
    const [link] = findLinks("[x](https://en.wikipedia.org/wiki/Foo_(bar))");
    expect(link.target).toBe("https://en.wikipedia.org/wiki/Foo_(bar)");
  });

  it("leaves a title out of the destination", () => {
    const [link] = findLinks('[x](a.md "why")');
    expect(link.target).toBe("a.md");
  });

  it("is not fooled by an escaped bracket or an unclosed one", () => {
    expect(findLinks("\\[not a link](x)")).toEqual([]);
    expect(findLinks("[open\nclosed](x)")).toEqual([]);
    expect(findLinks("[label] (spaced)")).toEqual([]);
  });

  it("finds several on one line", () => {
    expect(findLinks("[a](1.md) and [b](2.md)").map((l) => l.target)).toEqual([
      "1.md",
      "2.md",
    ]);
  });
});

describe("isUrl", () => {
  it("accepts what a browser or a mail client puts on the clipboard", () => {
    expect(isUrl("https://example.com/a?b=c#d")).toBe(true);
    expect(isUrl("http://localhost:3000")).toBe(true);
    expect(isUrl("mailto:nate@example.com")).toBe(true);
    expect(isUrl("www.example.com")).toBe(true);
  });

  it("refuses prose that merely contains one", () => {
    expect(isUrl("see https://example.com for more")).toBe(false);
    expect(isUrl("")).toBe(false);
    expect(isUrl("notes.md")).toBe(false);
  });
});

describe("linkOverSelection", () => {
  it("wraps the selected words in a link", () => {
    expect(linkOverSelection("the spec", "https://example.com")).toBe(
      "[the spec](https://example.com)",
    );
  });

  it("gives a schemeless address a scheme", () => {
    expect(linkOverSelection("here", "www.example.com")).toBe(
      "[here](https://www.example.com)",
    );
  });

  it("re-points a link rather than nesting one inside it", () => {
    expect(linkOverSelection("[the spec](old.md)", "https://new.example.com")).toBe(
      "[the spec](https://new.example.com)",
    );
  });

  it("stays out of the way when the paste is not that act", () => {
    expect(linkOverSelection("", "https://example.com")).toBeNull();
    expect(linkOverSelection("two\nlines", "https://example.com")).toBeNull();
    expect(linkOverSelection("words", "other words")).toBeNull();
  });
});

describe("normalizeUrl and imageMarkdown", () => {
  it("encodes what would end a destination early", () => {
    expect(normalizeUrl("https://example.com/a b(c)")).toBe(
      "https://example.com/a%20b%28c%29",
    );
    expect(imageMarkdown("assets/my shot.png", "my shot")).toBe(
      "![my shot](assets/my%20shot.png)",
    );
  });
});

describe("wovenTarget", () => {
  it("points a document link at the markdown that document weaves", () => {
    expect(wovenTarget("notes/plan.md")).toBe("notes/plan.md");
    expect(wovenTarget("plan.md#risks")).toBe("plan.md#risks");
  });

  it("leaves everything else exactly as written", () => {
    expect(wovenTarget("https://example.com/a.md")).toBe("https://example.com/a.md");
    expect(wovenTarget("assets/chart.png")).toBe("assets/chart.png");
    expect(wovenTarget("#section")).toBe("#section");
  });
});

describe("isLocalTarget and splitFragment", () => {
  it("separates the folder from the web and from a bare fragment", () => {
    expect(isLocalTarget("notes/a.md")).toBe(true);
    expect(isLocalTarget("https://example.com")).toBe(false);
    expect(isLocalTarget("mailto:a@b.c")).toBe(false);
    expect(isLocalTarget("#here")).toBe(false);
    expect(isLocalTarget("//example.com/a")).toBe(false);
  });

  it("splits the fragment off", () => {
    expect(splitFragment("a.md#b")).toEqual({ path: "a.md", fragment: "#b" });
    expect(splitFragment("a.md")).toEqual({ path: "a.md", fragment: "" });
  });
});

describe("resolveTarget", () => {
  it("resolves against the linking document's own directory", () => {
    expect(resolveTarget("notes/weekly/mon.md", "tue.md")).toBe("notes/weekly/tue.md");
    expect(resolveTarget("notes/weekly/mon.md", "../plan.md")).toBe("notes/plan.md");
    expect(resolveTarget("notes/weekly/mon.md", "/top.md")).toBe("top.md");
    expect(resolveTarget(null, "top.md")).toBe("top.md");
  });

  it("decodes what a destination had to encode", () => {
    expect(resolveTarget("a.md", "my%20shot.png")).toBe("my shot.png");
  });

  it("has nowhere to go for a URL or a bare fragment", () => {
    expect(resolveTarget("a.md", "https://example.com")).toBeNull();
    expect(resolveTarget("a.md", "#top")).toBeNull();
  });
});
