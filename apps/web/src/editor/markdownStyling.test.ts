import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import type { Range } from "@codemirror/state";
import { EditorView, type Decoration } from "@codemirror/view";
import {
  fencedCodeRanges,
  isMarkdownPath,
  markdownDecorations,
  horizontalRuleStyling,
  parseMarkdownProse,
  proseDecorationRanges,
} from "./markdownStyling";

const classOf = (r: Range<Decoration>) =>
  (r.value.spec as { class?: string }).class ?? "";

it("colours YAML frontmatter and keeps its comments out of Markdown prose", async () => {
  const { Decoration } = await import("@codemirror/view");
  const { yamlFrontmatter, parseHickDoc } = await import("./hickDoc");
  const { fenceHighlightRanges } = await import("./markdownFences");
  const source = "\uFEFF---\r\n# metadata comment\r\ntitle: [broken\r\n...\r\n# Visible heading\n";
  const metadata = yamlFrontmatter(source)!;
  expect(source.slice(metadata.from, metadata.to)).toBe("# metadata comment\r\ntitle: [broken\r\n");
  const structure = parseHickDoc(source);
  expect(structure.headings.map(h => source.slice(h.markTo, h.to))).toEqual(["Visible heading"]);
  const state = EditorState.create({ doc: source });
  const original = state.doc.toString();
  const drawnMetadata = yamlFrontmatter(original)!;
  const ranges = fenceHighlightRanges(state, structure, [{ from: 0, to: source.length }], cls => Decoration.mark({ class: cls }));
  expect(ranges.length).toBeGreaterThan(0);
  expect(ranges.every(range => range.from >= drawnMetadata.from && range.to <= drawnMetadata.to)).toBe(true);
  expect(state.doc.toString()).toBe(original);
  expect(yamlFrontmatter("# Body\n---\na: b\n---\n")).toBeNull();
  expect(yamlFrontmatter("---\na: b\n")).toBeNull();
});


describe("markdownStyling — parseMarkdownProse", () => {
  it("finds headings with their level and dimmed `#` mark range", () => {
    const src = "# Title\n\nprose\n\n### Deep\n";
    const { headings } = parseMarkdownProse(src);
    expect(headings.map((h) => h.level)).toEqual([1, 3]);
    expect(src.slice(headings[0].markFrom, headings[0].markTo)).toBe("# ");
    expect(src.slice(headings[0].from, headings[0].to)).toBe("# Title");
  });

  it("finds strong, em, and inline code with delimiter ranges", () => {
    const src = "some **bold** and *em* and `code` here";
    const { inline } = parseMarkdownProse(src);
    expect(inline.map((m) => m.kind).sort()).toEqual(["code", "em", "strong"]);
    const strong = inline.find((m) => m.kind === "strong")!;
    expect(src.slice(strong.from, strong.to)).toBe("bold");
    expect(src.slice(strong.openFrom, strong.openTo)).toBe("**");
    expect(src.slice(strong.closeFrom, strong.closeTo)).toBe("**");
  });

  it("does not style prose inside fenced code blocks", () => {
    const src = "# Real\n```sh\n# comment, not a heading\n**not bold**\n```\n**bold**\n";
    const { headings, inline } = parseMarkdownProse(src);
    expect(headings).toHaveLength(1);
    expect(src.slice(headings[0].from, headings[0].to)).toBe("# Real");
    expect(inline).toHaveLength(1);
    expect(src.slice(inline[0].from, inline[0].to)).toBe("bold");
  });
});

describe("markdownStyling — fencedCodeRanges", () => {
  it("pairs matching fences, fence lines included", () => {
    const src = "a\n```\ncode\n```\nb\n";
    const ranges = fencedCodeRanges(src);
    expect(ranges).toHaveLength(1);
    expect(src.slice(ranges[0][0], ranges[0][1])).toBe("```\ncode\n```");
  });

  it("runs an unclosed fence to the end of the text", () => {
    const src = "```\nnever closed\n# not a heading";
    expect(fencedCodeRanges(src)).toEqual([[0, src.length]]);
    expect(parseMarkdownProse(src).headings).toHaveLength(0);
  });

  it("does not close ``` with ~~~", () => {
    const src = "```\n~~~\nstill code\n```\n";
    const ranges = fencedCodeRanges(src);
    expect(ranges).toHaveLength(1);
    expect(src.slice(ranges[0][0], ranges[0][1])).toBe("```\n~~~\nstill code\n```");
  });
});

describe("markdownStyling — block quotes", () => {
  it("marks each quoted LINE with its nesting depth and marker run", () => {
    const src = "> one\n> > two\nplain\n";
    const { quotes } = parseMarkdownProse(src);
    expect(quotes.map((q) => q.depth)).toEqual([1, 2]);
    expect(src.slice(quotes[0].markFrom, quotes[0].markTo)).toBe("> ");
    expect(src.slice(quotes[1].markFrom, quotes[1].markTo)).toBe("> > ");
  });

  it("styles prose inside a quote, so `> **bold**` is both", () => {
    const src = "> a **bold** word\n";
    const { quotes, inline } = parseMarkdownProse(src);
    expect(quotes).toHaveLength(1);
    const strong = inline.find((m) => m.kind === "strong")!;
    expect(src.slice(strong.from, strong.to)).toBe("bold");
  });

  it("reads a heading inside a quote as a heading, not as a hash", () => {
    const src = "> ## Quoted heading\n";
    const { headings } = parseMarkdownProse(src);
    expect(headings.map((h) => h.level)).toEqual([2]);
    expect(src.slice(headings[0].markFrom, headings[0].markTo)).toBe("## ");
  });

  it("does not quote a line inside a fenced code block", () => {
    const src = "```sh\n> not a quote\n```\n";
    expect(parseMarkdownProse(src).quotes).toEqual([]);
  });

  it("gives a quote line a tinted line class and dims its marker", () => {
    const src = "> quoted\n";
    const classes = proseDecorationRanges(parseMarkdownProse(src), src.length).map(classOf);
    expect(classes).toContain("cm-md-quote cm-md-quote-1");
    expect(classes).toContain("cm-md-mark");
  });

  it("caps the tint at three levels rather than inventing a fourth grey", () => {
    const src = "> > > > deep\n";
    const classes = proseDecorationRanges(parseMarkdownProse(src), src.length).map(classOf);
    expect(classes).toContain("cm-md-quote cm-md-quote-3");
  });
});

describe("markdownStyling — proseDecorationRanges", () => {
  it("emits a heading line class per level and a dimmed mark", () => {
    const src = "## Second\n";
    const ranges = proseDecorationRanges(parseMarkdownProse(src), src.length);
    const line = ranges.find((r) => classOf(r).includes("cm-md-heading"))!;
    expect(classOf(line)).toBe("cm-md-heading cm-md-h2");
    expect(line.from).toBe(0);
    const mark = ranges.find((r) => classOf(r) === "cm-md-mark")!;
    expect([mark.from, mark.to]).toEqual([0, 3]); // "## "
  });

  it("styles inline content and dims both delimiters", () => {
    const src = "a `x` b";
    const ranges = proseDecorationRanges(parseMarkdownProse(src), src.length);
    const byClass = (cls: string) => ranges.filter((r) => classOf(r) === cls);
    const code = byClass("cm-md-code");
    expect(code).toHaveLength(1);
    expect(src.slice(code[0].from, code[0].to)).toBe("x");
    // Opening and closing backtick each dimmed.
    expect(byClass("cm-md-mark").map((r) => src.slice(r.from, r.to))).toEqual([
      "`",
      "`",
    ]);
  });

  it("clamps ranges to the document length instead of throwing", () => {
    const src = "# Title";
    const prose = parseMarkdownProse(src);
    // Simulate a stale parse over a shorter doc.
    expect(() => proseDecorationRanges(prose, 3)).not.toThrow();
  });
});

describe("markdownStyling — markdownDecorations (extension-level)", () => {
  it("builds a sorted decoration set over a whole buffer without changing it", () => {
    const src = "# Title\n\nSome **bold** prose.\n\n```py\n# not a heading\n```\n";
    const state = EditorState.create({ doc: src });
    const deco = markdownDecorations(state);
    const classes: string[] = [];
    deco.between(0, src.length, (_from, _to, d) => {
      classes.push((d.spec as { class?: string }).class ?? "");
    });
    expect(classes).toContain("cm-md-heading cm-md-h1");
    expect(classes).toContain("cm-md-strong");
    expect(classes.join(" ")).not.toContain("cm-md-h2");
    // Display-only: the buffer text is untouched.
    expect(state.doc.toString()).toBe(src);
  });
});

describe("cursor-aware horizontal rules", () => {
  const rules = (state: EditorState) => {
    const found: number[] = [];
    for (const set of state.facet(EditorView.decorations)) {
      if (typeof set === "function") continue;
      set.between(0, state.doc.length, (from, _to, deco) => {
        if (deco.spec.class === "cm-md-rule") found.push(from);
      });
    }
    return found;
  };
  it("reveals source on entry, restores the rule on exit, and responds to edits", () => {
    const src = "text\n\n---\n\nend";
    let state = EditorState.create({ doc: src, extensions: horizontalRuleStyling() });
    expect(rules(state)).toEqual([6]);
    state = state.update({ selection: { anchor: 7 } }).state;
    expect(rules(state)).toEqual([]);
    expect(state.doc.line(3).text).toBe("---");
    state = state.update({ selection: { anchor: 0 } }).state;
    expect(rules(state)).toEqual([6]);
    state = state.update({ changes: { from: 6, to: 7, insert: "x" } }).state;
    expect(rules(state)).toEqual([]);
  });
  it("keeps YAML delimiters literal when the cursor is in prose", () => {
    const src = "---\ntitle: hello\n\n---\n\nbody";
    const state = EditorState.create({ doc: src, selection: { anchor: src.length }, extensions: horizontalRuleStyling() });
    expect(rules(state)).toEqual([]);
  });
  it("leaves fences, payloads, four-space code and heading underlines literal", () => {
    const src = "text\n---\n\n```\n---\n```\n\n    ---\n\n---\n";
    const from = src.lastIndexOf("---");
    const state = EditorState.create({ doc: src,
      extensions: horizontalRuleStyling(() => [[from, from + 3]]) });
    expect(rules(state)).toEqual([]);
  });
});

describe("markdownStyling — isMarkdownPath", () => {
  it("matches .md and .markdown (any case), nothing else", () => {
    expect(isMarkdownPath("README.md")).toBe(true);
    expect(isMarkdownPath("docs/guide.markdown")).toBe(true);
    expect(isMarkdownPath("NOTES.MD")).toBe(true);
    expect(isMarkdownPath("main.py")).toBe(false);
    expect(isMarkdownPath("md")).toBe(false);
    expect(isMarkdownPath("archive.md.gz")).toBe(false);
  });
});
