<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/mdLinks.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b86c148dc6ce"></a>
## `apps/web/src/lib/mdLinks.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Link Between Documents Lands In The Weave](../../../../../guarantees/authoring/a-link-between-documents-lands-in-the-weave/contract.md), [Pasting A URL Over Words Makes A Link](../../../../../guarantees/authoring/pasting-a-url-over-words-makes-a-link/contract.md).

<a id="f-b86c148dc6ce-MdLink"></a>
### `MdLink`

Markdown links and images, as text — found, made, and pointed at the right
file.
Everything here is a pure function over a string. The CodeMirror side
(editor/mdLinks.ts) decorates and opens what this finds, and the paste side
(editor/mdPaste.ts) writes what this makes; keeping the string work here is
what lets both be tested without an editor.
## Why the link syntax is markdown and not a hick tag
A `.md` document is markdown first (`bare-documents.md`), and a link is
the one piece of markup every reader of the woven `.md` already
understands. A `<hick:link>` would have to weave to `…` anyway, and
would be unreadable in the source in the meantime. So a cross-document link
is written the way a reader would write it, and the WEAVER is what makes it
land — see `crates/hick-literate/src/links.rs`, which rewrites a `.md`
target to the `.md` that document weaves.
A markdown inline link or image, located in some text.

<storycode:copy id="f-b86c148dc6ce-MdLink">
// Markdown links and images, as text — found, made, and pointed at the right
// file.
//
// Everything here is a pure function over a string. The CodeMirror side
// (editor/mdLinks.ts) decorates and opens what this finds, and the paste side
// (editor/mdPaste.ts) writes what this makes; keeping the string work here is
// what lets both be tested without an editor.
//
// ## Why the link syntax is markdown and not a hick tag
//
// A `.md` document is markdown first (`bare-documents.md`), and a link is
// the one piece of markup every reader of the woven `.md` already
// understands. A `<hick:link>` would have to weave to `[…](…)` anyway, and
// would be unreadable in the source in the meantime. So a cross-document link
// is written the way a reader would write it, and the WEAVER is what makes it
// land — see `crates/hick-literate/src/links.rs`, which rewrites a `.md`
// target to the `.md` that document weaves.

/** A markdown inline link or image, located in some text. */
export interface MdLink {
  /** An image (`![…](…)`) rather than a link (`[…](…)`). */
  image: boolean;
  /** The whole construct, brackets and parentheses included. */
  from: number;
  to: number;
  /** The label between the brackets. */
  textFrom: number;
  textTo: number;
  /** The destination inside the parentheses, title excluded. */
  targetFrom: number;
  targetTo: number;
  /** The destination, as written. */
  target: string;
  /** The label, as written. */
  text: string;
}

</storycode:copy>
<a id="f-b86c148dc6ce-findLinks"></a>
### `findLinks`

Every inline link and image in `text`.
A deliberately small scanner rather than a markdown parser: it matches the
bracket/parenthesis pair, balances nested parentheses inside the
destination (Wikipedia URLs have them), and stops at a newline, because an
unclosed `[` at the end of a paragraph is a typo and not a link that runs
to the end of the document.

<storycode:copy id="f-b86c148dc6ce-findLinks">
/**
 * Every inline link and image in `text`.
 *
 * A deliberately small scanner rather than a markdown parser: it matches the
 * bracket/parenthesis pair, balances nested parentheses inside the
 * destination (Wikipedia URLs have them), and stops at a newline, because an
 * unclosed `[` at the end of a paragraph is a typo and not a link that runs
 * to the end of the document.
 */
export function findLinks(text: string): MdLink[] {
  const found: MdLink[] = [];
  for (let i = 0; i < text.length; i++) {
    if (text[i] !== "[") continue;
    // An escaped bracket is a bracket.
    if (i > 0 && text[i - 1] === "\\") continue;
    const image = i > 0 && text[i - 1] === "!" && !(i > 1 && text[i - 2] === "\\");
    const textFrom = i + 1;
    const textTo = closingBracket(text, textFrom);
    if (textTo < 0) continue;
    if (text[textTo + 1] !== "(") continue;
    const targetFrom = textTo + 2;
    const close = closingParen(text, targetFrom);
    if (close < 0) continue;
    // `[a](b "title")` — the title is not part of the destination.
    const inner = text.slice(targetFrom, close);
    const titleAt = titleStart(inner);
    const targetTo = targetFrom + (titleAt < 0 ? inner.length : titleAt);
    found.push({
      image,
      from: image ? i - 1 : i,
      to: close + 1,
      textFrom,
      textTo,
      targetFrom,
      targetTo,
      target: text.slice(targetFrom, targetTo).trim(),
      text: text.slice(textFrom, textTo),
    });
    i = close;
  }
  return found;
}

</storycode:copy>
<a id="f-b86c148dc6ce-closingBracket"></a>
### `closingBracket`

Index of the `]` closing a `[` at `from`, or -1. Nesting counts.

<storycode:copy id="f-b86c148dc6ce-closingBracket">
/** Index of the `]` closing a `[` at `from`, or -1. Nesting counts. */
function closingBracket(text: string, from: number): number {
  let depth = 0;
  for (let i = from; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\n") return -1;
    if (ch === "\\") {
      i++;
      continue;
    }
    if (ch === "[") depth++;
    else if (ch === "]") {
      if (depth === 0) return i;
      depth--;
    }
  }
  return -1;
}

</storycode:copy>
<a id="f-b86c148dc6ce-closingParen"></a>
### `closingParen`

<storycode:copy id="f-b86c148dc6ce-closingParen">
/** Index of the `)` closing a `(` before `from`, or -1. */
function closingParen(text: string, from: number): number {
  let depth = 0;
  for (let i = from; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\n") return -1;
    if (ch === "\\") {
      i++;
      continue;
    }
    if (ch === "(") depth++;
    else if (ch === ")") {
      if (depth === 0) return i;
      depth--;
    }
  }
  return -1;
}

</storycode:copy>
<a id="f-b86c148dc6ce-titleStart"></a>
### `titleStart`

Where a `"…"` / `'…'` title begins inside a destination, or -1.

<storycode:copy id="f-b86c148dc6ce-titleStart">
/** Where a `"…"` / `'…'` title begins inside a destination, or -1. */
function titleStart(inner: string): number {
  const match = /\s+["']/.exec(inner);
  return match ? match.index : -1;
}

</storycode:copy>
<a id="f-b86c148dc6ce-isUrl"></a>
### `isUrl`

Does this text name somewhere to go?
Used to decide whether a paste over a selection is a link being applied to
words rather than text replacing them, so it is deliberately strict: a
scheme and something after it, on one line, with no whitespace. "See
https://example.com for more" is prose being pasted, not a URL, and
swallowing it into a link would be the wrong guess.

<storycode:copy id="f-b86c148dc6ce-isUrl">
/**
 * Does this text name somewhere to go?
 *
 * Used to decide whether a paste over a selection is a link being applied to
 * words rather than text replacing them, so it is deliberately strict: a
 * scheme and something after it, on one line, with no whitespace. "See
 * https://example.com for more" is prose being pasted, not a URL, and
 * swallowing it into a link would be the wrong guess.
 */
export function isUrl(text: string): boolean {
  const trimmed = text.trim();
  if (trimmed.length === 0 || /\s/.test(trimmed)) return false;
  if (/^(https?|ftp|mailto|tel|file):/i.test(trimmed)) return true;
  // `www.example.com` with no scheme is the other thing people copy out of a
  // browser's address bar and out of an email.
  return /^www\.[^.]+\.[^.]+$/i.test(trimmed);
}

</storycode:copy>
<a id="f-b86c148dc6ce-normalizeUrl"></a>
### `normalizeUrl`

A URL that a markdown destination can hold: schemeless `www.` gets one,
and spaces (which would end the destination) get encoded.

<storycode:copy id="f-b86c148dc6ce-normalizeUrl">
/** A URL that a markdown destination can hold: schemeless `www.` gets one,
 * and spaces (which would end the destination) get encoded. */
export function normalizeUrl(text: string): string {
  const trimmed = text.trim();
  const url = /^www\./i.test(trimmed) ? `https://${trimmed}` : trimmed;
  return url.replace(/ /g, "%20").replace(/\(/g, "%28").replace(/\)/g, "%29");
}

</storycode:copy>
<a id="f-b86c148dc6ce-linkOverSelection"></a>
### `linkOverSelection`

The markdown for pasting `url` over `selected`, or null when this paste is
not that act.
Null — rather than a link with an empty label — when nothing is selected:
pasting a URL at the caret should give you the URL you copied, which is
what every other editor does and what you get by doing nothing here.

<storycode:copy id="f-b86c148dc6ce-linkOverSelection">
/**
 * The markdown for pasting `url` over `selected`, or null when this paste is
 * not that act.
 *
 * Null — rather than a link with an empty label — when nothing is selected:
 * pasting a URL at the caret should give you the URL you copied, which is
 * what every other editor does and what you get by doing nothing here.
 */
export function linkOverSelection(selected: string, pasted: string): string | null {
  if (!isUrl(pasted)) return null;
  if (selected.length === 0 || selected.includes("\n")) return null;
  // Selecting a link and pasting a URL over it means "point this somewhere
  // else", not "nest a link inside a label".
  const links = findLinks(selected);
  const whole = links.find((l) => l.from === 0 && l.to === selected.length);
  const label = whole ? whole.text : selected;
  return `[${label}](${normalizeUrl(pasted)})`;
}

</storycode:copy>
<a id="f-b86c148dc6ce-encodeTarget"></a>
### `encodeTarget`

Characters a markdown destination cannot hold bare.

<storycode:copy id="f-b86c148dc6ce-encodeTarget">
/** Characters a markdown destination cannot hold bare. */
export function encodeTarget(path: string): string {
  return path.replace(/ /g, "%20").replace(/\(/g, "%28").replace(/\)/g, "%29");
}

</storycode:copy>
<a id="f-b86c148dc6ce-imageMarkdown"></a>
### `imageMarkdown`

The markdown for an image at `path`, labelled `alt`.

<storycode:copy id="f-b86c148dc6ce-imageMarkdown">
/** The markdown for an image at `path`, labelled `alt`. */
export function imageMarkdown(path: string, alt: string): string {
  return `![${alt.replace(/[[\]]/g, "")}](${encodeTarget(path)})`;
}

</storycode:copy>
<a id="f-b86c148dc6ce-isLocalTarget"></a>
### `isLocalTarget`

Is this destination somewhere in the notes folder rather than on the web?
A fragment alone (`#section`) is inside the document that holds it, which
is neither — it is already correct in the woven markdown and wants no
rewriting and no file to open.

<storycode:copy id="f-b86c148dc6ce-isLocalTarget">
/**
 * Is this destination somewhere in the notes folder rather than on the web?
 *
 * A fragment alone (`#section`) is inside the document that holds it, which
 * is neither — it is already correct in the woven markdown and wants no
 * rewriting and no file to open.
 */
export function isLocalTarget(target: string): boolean {
  if (target.length === 0) return false;
  if (target.startsWith("#")) return false;
  if (/^[a-z][a-z0-9+.-]*:/i.test(target)) return false;
  return !target.startsWith("//");
}

</storycode:copy>
<a id="f-b86c148dc6ce-splitFragment"></a>
### `splitFragment`

A destination split into the path and the `#fragment` after it.

<storycode:copy id="f-b86c148dc6ce-splitFragment">
/** A destination split into the path and the `#fragment` after it. */
export function splitFragment(target: string): { path: string; fragment: string } {
  const hash = target.indexOf("#");
  return hash < 0
    ? { path: target, fragment: "" }
    : { path: target.slice(0, hash), fragment: target.slice(hash) };
}

</storycode:copy>
<a id="f-b86c148dc6ce-wovenTarget"></a>
### `wovenTarget`

The destination as it should read in woven markdown: a link to a document
points at the markdown that document weaves, because that is the file the
reader of the weave actually has. Anything else is left alone.

<storycode:copy id="f-b86c148dc6ce-wovenTarget">
/** The destination as it should read in woven markdown: a link to a document
 * points at the markdown that document weaves, because that is the file the
 * reader of the weave actually has. Anything else is left alone. */
export function wovenTarget(target: string): string {
  if (!isLocalTarget(target)) return target;
  const { path, fragment } = splitFragment(target);
  if (!path.toLowerCase().endsWith(".md")) return target;
  return `${path.slice(0, -".md".length)}.md${fragment}`;
}

</storycode:copy>
<a id="f-b86c148dc6ce-resolveTarget"></a>
### `resolveTarget`

Where a link in the document at `docPath` points, as a folder-relative
path — or null when it points nowhere this app opens.
Relative destinations resolve against the LINKING document's directory,
which is what a relative link means everywhere else and what keeps a folder
of notes movable.

<storycode:copy id="f-b86c148dc6ce-resolveTarget">
/**
 * Where a link in the document at `docPath` points, as a folder-relative
 * path — or null when it points nowhere this app opens.
 *
 * Relative destinations resolve against the LINKING document's directory,
 * which is what a relative link means everywhere else and what keeps a folder
 * of notes movable.
 */
export function resolveTarget(docPath: string | null, target: string): string | null {
  if (!isLocalTarget(target)) return null;
  const { path } = splitFragment(decodeURI(target));
  if (path.length === 0) return null;
  const base = path.startsWith("/")
    ? []
    : (docPath ?? "").split("/").slice(0, -1).filter(Boolean);
  const parts = path.replace(/^\//, "").split("/");
  const out = [...base];
  for (const part of parts) {
    if (part === "" || part === ".") continue;
    if (part === "..") out.pop();
    else out.push(part);
  }
  return out.length > 0 ? out.join("/") : null;
}
</storycode:copy>

### Assemble `apps/web/src/lib/mdLinks.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/mdLinks.ts"><storycode:paste select="#f-b86c148dc6ce-MdLink"/><storycode:paste select="#f-b86c148dc6ce-findLinks"/><storycode:paste select="#f-b86c148dc6ce-closingBracket"/><storycode:paste select="#f-b86c148dc6ce-closingParen"/><storycode:paste select="#f-b86c148dc6ce-titleStart"/><storycode:paste select="#f-b86c148dc6ce-isUrl"/><storycode:paste select="#f-b86c148dc6ce-normalizeUrl"/><storycode:paste select="#f-b86c148dc6ce-linkOverSelection"/><storycode:paste select="#f-b86c148dc6ce-encodeTarget"/><storycode:paste select="#f-b86c148dc6ce-imageMarkdown"/><storycode:paste select="#f-b86c148dc6ce-isLocalTarget"/><storycode:paste select="#f-b86c148dc6ce-splitFragment"/><storycode:paste select="#f-b86c148dc6ce-wovenTarget"/><storycode:paste select="#f-b86c148dc6ce-resolveTarget"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-9cdf9b2037e6"></a>
## `apps/web/src/lib/mdLinks.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Link Between Documents Lands In The Weave](../../../../../guarantees/authoring/a-link-between-documents-lands-in-the-weave/contract.md), [Pasting A URL Over Words Makes A Link](../../../../../guarantees/authoring/pasting-a-url-over-words-makes-a-link/contract.md).

<a id="f-9cdf9b2037e6-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement">
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

</storycode:copy>
<a id="f-9cdf9b2037e6-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement-2">
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

</storycode:copy>
<a id="f-9cdf9b2037e6-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement-3">
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

</storycode:copy>
<a id="f-9cdf9b2037e6-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement-4">
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

</storycode:copy>
<a id="f-9cdf9b2037e6-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement-5">
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

</storycode:copy>
<a id="f-9cdf9b2037e6-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement-6">
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

</storycode:copy>
<a id="f-9cdf9b2037e6-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-9cdf9b2037e6-expression_statement-7">
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
</storycode:copy>

### Assemble `apps/web/src/lib/mdLinks.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/mdLinks.test.ts"><storycode:paste select="#f-9cdf9b2037e6-expression_statement"/><storycode:paste select="#f-9cdf9b2037e6-expression_statement-2"/><storycode:paste select="#f-9cdf9b2037e6-expression_statement-3"/><storycode:paste select="#f-9cdf9b2037e6-expression_statement-4"/><storycode:paste select="#f-9cdf9b2037e6-expression_statement-5"/><storycode:paste select="#f-9cdf9b2037e6-expression_statement-6"/><storycode:paste select="#f-9cdf9b2037e6-expression_statement-7"/></storycode:file>


</storycode:doc>
