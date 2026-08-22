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
    const text = "see [the notes](notes/plan.hick) for more";
    const [link] = findLinks(text);
    expect(link.image).toBe(false);
    expect(text.slice(link.from, link.to)).toBe("[the notes](notes/plan.hick)");
    expect(link.text).toBe("the notes");
    expect(link.target).toBe("notes/plan.hick");
    expect(text.slice(link.targetFrom, link.targetTo)).toBe("notes/plan.hick");
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
    const [link] = findLinks('[x](a.hick "why")');
    expect(link.target).toBe("a.hick");
  });

  it("is not fooled by an escaped bracket or an unclosed one", () => {
    expect(findLinks("\\[not a link](x)")).toEqual([]);
    expect(findLinks("[open\nclosed](x)")).toEqual([]);
    expect(findLinks("[label] (spaced)")).toEqual([]);
  });

  it("finds several on one line", () => {
    expect(findLinks("[a](1.hick) and [b](2.hick)").map((l) => l.target)).toEqual([
      "1.hick",
      "2.hick",
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
    expect(isUrl("notes.hick")).toBe(false);
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
    expect(wovenTarget("notes/plan.hick")).toBe("notes/plan.md");
    expect(wovenTarget("plan.hick#risks")).toBe("plan.md#risks");
  });

  it("leaves everything else exactly as written", () => {
    expect(wovenTarget("https://example.com/a.hick")).toBe("https://example.com/a.hick");
    expect(wovenTarget("assets/chart.png")).toBe("assets/chart.png");
    expect(wovenTarget("#section")).toBe("#section");
  });
});

describe("isLocalTarget and splitFragment", () => {
  it("separates the folder from the web and from a bare fragment", () => {
    expect(isLocalTarget("notes/a.hick")).toBe(true);
    expect(isLocalTarget("https://example.com")).toBe(false);
    expect(isLocalTarget("mailto:a@b.c")).toBe(false);
    expect(isLocalTarget("#here")).toBe(false);
    expect(isLocalTarget("//example.com/a")).toBe(false);
  });

  it("splits the fragment off", () => {
    expect(splitFragment("a.hick#b")).toEqual({ path: "a.hick", fragment: "#b" });
    expect(splitFragment("a.hick")).toEqual({ path: "a.hick", fragment: "" });
  });
});

describe("resolveTarget", () => {
  it("resolves against the linking document's own directory", () => {
    expect(resolveTarget("notes/weekly/mon.hick", "tue.hick")).toBe("notes/weekly/tue.hick");
    expect(resolveTarget("notes/weekly/mon.hick", "../plan.hick")).toBe("notes/plan.hick");
    expect(resolveTarget("notes/weekly/mon.hick", "/top.hick")).toBe("top.hick");
    expect(resolveTarget(null, "top.hick")).toBe("top.hick");
  });

  it("decodes what a destination had to encode", () => {
    expect(resolveTarget("a.hick", "my%20shot.png")).toBe("my shot.png");
  });

  it("has nowhere to go for a URL or a bare fragment", () => {
    expect(resolveTarget("a.hick", "https://example.com")).toBeNull();
    expect(resolveTarget("a.hick", "#top")).toBeNull();
  });
});
