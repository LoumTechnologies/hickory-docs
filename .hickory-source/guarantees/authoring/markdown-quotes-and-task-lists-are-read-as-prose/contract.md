<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Markdown Quotes And Task Lists Are Read As Prose

For: Authors editing executable documents and the files those documents produce.

Given any buffer this app styles as markdown — a `.md` document's prose, a
woven `.md` output pane, a plain markdown file — when a line opens with one or
more `>` markers, then that line is drawn as a quote at its nesting depth; and
when a list item's marker is followed by `[ ]` or `[x]`, then its box is drawn
as a checkbox that a single click toggles in the source.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/markdown-quotes-and-task-lists-are-read-as-prose.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-59145de2de8b"></a>
## `apps/web/src/editor/markdownStyling.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Quotes And Task Lists Are Read As Prose](contract.md).

<a id="f-59145de2de8b-classOf"></a>
### `classOf`

<storycode:copy id="f-59145de2de8b-classOf">
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import type { Range } from "@codemirror/state";
import type { Decoration } from "@codemirror/view";
import {
  fencedCodeRanges,
  isMarkdownPath,
  markdownDecorations,
  parseMarkdownProse,
  proseDecorationRanges,
} from "./markdownStyling";

const classOf = (r: Range<Decoration>) =>
  (r.value.spec as { class?: string }).class ?? "";

</storycode:copy>
<a id="f-59145de2de8b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-59145de2de8b-expression_statement">
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

</storycode:copy>
<a id="f-59145de2de8b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-59145de2de8b-expression_statement-2">
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

</storycode:copy>
<a id="f-59145de2de8b-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-59145de2de8b-expression_statement-3">
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

</storycode:copy>
<a id="f-59145de2de8b-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-59145de2de8b-expression_statement-4">
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

</storycode:copy>
<a id="f-59145de2de8b-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-59145de2de8b-expression_statement-5">
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

</storycode:copy>
<a id="f-59145de2de8b-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-59145de2de8b-expression_statement-6">
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
</storycode:copy>

### Assemble `apps/web/src/editor/markdownStyling.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/markdownStyling.test.ts"><storycode:paste select="#f-59145de2de8b-classOf"/><storycode:paste select="#f-59145de2de8b-expression_statement"/><storycode:paste select="#f-59145de2de8b-expression_statement-2"/><storycode:paste select="#f-59145de2de8b-expression_statement-3"/><storycode:paste select="#f-59145de2de8b-expression_statement-4"/><storycode:paste select="#f-59145de2de8b-expression_statement-5"/><storycode:paste select="#f-59145de2de8b-expression_statement-6"/></storycode:file>

<a id="output-432a956d4c21"></a>
## `apps/web/src/editor/markdownStyling.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Quotes And Task Lists Are Read As Prose](contract.md).

<a id="f-432a956d4c21-headingLineDecos"></a>
### `headingLineDecos`

Typora-style markdown DISPLAY styling, shared between the document editor
(wysiwyg.ts, over .md prose) and the output panes (OutputEditorPane, over
a generated .md file). Decorations only — line classes and marks; nothing
here ever replaces, hides, or edits text.
Two layers:
- `proseDecorationRanges` turns already-parsed headings/inline marks into
decoration ranges. wysiwyg.ts calls this with the hick structure parse.
- `markdownStyling()` is a self-contained extension for a plain markdown
buffer: it parses the whole buffer with the same heading/inline scanner
the document editor uses (`scanMarkdownProse`), treating fenced code
blocks as verbatim, and provides the decorations from a StateField so
heading heights are part of the same update (CodeMirror's layout rule).

<storycode:copy id="f-432a956d4c21-headingLineDecos">
// Typora-style markdown DISPLAY styling, shared between the document editor
// (wysiwyg.ts, over .md prose) and the output panes (OutputEditorPane, over
// a generated .md file). Decorations only — line classes and marks; nothing
// here ever replaces, hides, or edits text.
//
// Two layers:
//  - `proseDecorationRanges` turns already-parsed headings/inline marks into
//    decoration ranges. wysiwyg.ts calls this with the hick structure parse.
//  - `markdownStyling()` is a self-contained extension for a plain markdown
//    buffer: it parses the whole buffer with the same heading/inline scanner
//    the document editor uses (`scanMarkdownProse`), treating fenced code
//    blocks as verbatim, and provides the decorations from a StateField so
//    heading heights are part of the same update (CodeMirror's layout rule).

import { EditorState, StateField } from "@codemirror/state";
import type { Extension, Range } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { scanMarkdownProse } from "./hickDoc";
import type { Heading, InlineMark, QuoteLine, TaskItem } from "./hickDoc";

// One Decoration instance per class, shared by both editors so the CSS in
// styles.css (`.cm-md-*`) is the single definition of the look.
export const headingLineDecos = [1, 2, 3, 4, 5, 6].map((level) =>
  Decoration.line({ class: `cm-md-heading cm-md-h${level}` }),
);
</storycode:copy>
<a id="f-432a956d4c21-mdMark"></a>
### `mdMark`

<storycode:copy id="f-432a956d4c21-mdMark">
export const mdMark = Decoration.mark({ class: "cm-md-mark" });
</storycode:copy>
<a id="f-432a956d4c21-mdStrong"></a>
### `mdStrong`

<storycode:copy id="f-432a956d4c21-mdStrong">
export const mdStrong = Decoration.mark({ class: "cm-md-strong" });
</storycode:copy>
<a id="f-432a956d4c21-mdEm"></a>
### `mdEm`

<storycode:copy id="f-432a956d4c21-mdEm">
export const mdEm = Decoration.mark({ class: "cm-md-em" });
</storycode:copy>
<a id="f-432a956d4c21-mdCode"></a>
### `mdCode`

<storycode:copy id="f-432a956d4c21-mdCode">
export const mdCode = Decoration.mark({ class: "cm-md-code" });

</storycode:copy>
<a id="f-432a956d4c21-QUOTE_DEPTHS"></a>
### `QUOTE_DEPTHS`

Quote lines, by nesting depth. Deeper than three reuses the third tint:
the point of the indent is "this is quoted", and a fourth shade of the same
grey carries no information a reader could act on.

<storycode:copy id="f-432a956d4c21-QUOTE_DEPTHS">
/** Quote lines, by nesting depth. Deeper than three reuses the third tint:
 * the point of the indent is "this is quoted", and a fourth shade of the same
 * grey carries no information a reader could act on. */
export const QUOTE_DEPTHS = 3;
</storycode:copy>
<a id="f-432a956d4c21-mdQuoteLines"></a>
### `mdQuoteLines`

<storycode:copy id="f-432a956d4c21-mdQuoteLines">
export const mdQuoteLines = [1, 2, 3].map((depth) =>
  Decoration.line({ class: `cm-md-quote cm-md-quote-${depth}` }),
);
</storycode:copy>
<a id="f-432a956d4c21-mdTaskLine"></a>
### `mdTaskLine`

<storycode:copy id="f-432a956d4c21-mdTaskLine">
export const mdTaskLine = Decoration.line({ class: "cm-md-task" });
</storycode:copy>
<a id="f-432a956d4c21-mdTaskDoneLine"></a>
### `mdTaskDoneLine`

<storycode:copy id="f-432a956d4c21-mdTaskDoneLine">
export const mdTaskDoneLine = Decoration.line({
  class: "cm-md-task cm-md-task--done",
});

</storycode:copy>
<a id="f-432a956d4c21-MarkdownProse"></a>
### `MarkdownProse`

<storycode:copy id="f-432a956d4c21-MarkdownProse">
export interface MarkdownProse {
  headings: Heading[];
  inline: InlineMark[];
  quotes: QuoteLine[];
  tasks: TaskItem[];
}

</storycode:copy>
<a id="f-432a956d4c21-proseDecorationRanges"></a>
### `proseDecorationRanges`

Decoration ranges for parsed markdown prose: heading lines get big type
with their `#` marks kept but dimmed; inline strong/em/code get styled
content with visible-but-dimmed delimiters. Unsorted — callers that merge
these with other ranges sort once at the end.

<storycode:copy id="f-432a956d4c21-proseDecorationRanges">
/**
 * Decoration ranges for parsed markdown prose: heading lines get big type
 * with their `#` marks kept but dimmed; inline strong/em/code get styled
 * content with visible-but-dimmed delimiters. Unsorted — callers that merge
 * these with other ranges sort once at the end.
 */
export function proseDecorationRanges(
  prose: MarkdownProse,
  docLen: number,
): Range<Decoration>[] {
  const ranges: Range<Decoration>[] = [];
  const clamp = (n: number) => Math.max(0, Math.min(n, docLen));
  for (const h of prose.headings) {
    ranges.push(headingLineDecos[Math.min(h.level, 6) - 1].range(clamp(h.from)));
    if (h.markTo > h.markFrom)
      ranges.push(mdMark.range(clamp(h.markFrom), clamp(h.markTo)));
  }
  // Quotes are a line tint plus a dimmed `>` run: the marker stays in the
  // text (this module never hides anything) but stops competing with the
  // words it introduces.
  for (const quote of prose.quotes) {
    const depth = Math.min(Math.max(quote.depth, 1), QUOTE_DEPTHS);
    ranges.push(mdQuoteLines[depth - 1].range(clamp(quote.from)));
    if (quote.markTo > quote.markFrom)
      ranges.push(mdMark.range(clamp(quote.markFrom), clamp(quote.markTo)));
  }
  for (const task of prose.tasks) {
    ranges.push(
      (task.checked ? mdTaskDoneLine : mdTaskLine).range(clamp(task.from)),
    );
    ranges.push(mdMark.range(clamp(task.boxFrom), clamp(task.boxTo)));
  }
  for (const mark of prose.inline) {
    const style =
      mark.kind === "strong" ? mdStrong : mark.kind === "em" ? mdEm : mdCode;
    ranges.push(mdMark.range(clamp(mark.openFrom), clamp(mark.openTo)));
    if (mark.to > mark.from)
      ranges.push(style.range(clamp(mark.from), clamp(mark.to)));
    ranges.push(mdMark.range(clamp(mark.closeFrom), clamp(mark.closeTo)));
  }
  return ranges;
}

</storycode:copy>
<a id="f-432a956d4c21-fencedCodeRanges"></a>
### `fencedCodeRanges`

Fenced-code ranges (``` / ~~~) of a markdown text, fence lines included.
Inside a fence nothing is prose: a `# comment` in fenced shell must not
become an h1. An unclosed fence runs to the end of the text.

<storycode:copy id="f-432a956d4c21-fencedCodeRanges">
/**
 * Fenced-code ranges (``` / ~~~) of a markdown text, fence lines included.
 * Inside a fence nothing is prose: a `# comment` in fenced shell must not
 * become an h1. An unclosed fence runs to the end of the text.
 */
export function fencedCodeRanges(text: string): [number, number][] {
  const ranges: [number, number][] = [];
  let openFrom = -1;
  let fence = "";
  let lineFrom = 0;
  while (lineFrom <= text.length) {
    let lineTo = text.indexOf("\n", lineFrom);
    if (lineTo < 0) lineTo = text.length;
    const m = /^ {0,3}(```|~~~)/.exec(text.slice(lineFrom, lineTo));
    if (m) {
      if (openFrom < 0) {
        openFrom = lineFrom;
        fence = m[1];
      } else if (m[1] === fence) {
        ranges.push([openFrom, lineTo]);
        openFrom = -1;
      }
    }
    lineFrom = lineTo + 1;
  }
  if (openFrom >= 0) ranges.push([openFrom, text.length]);
  return ranges;
}

</storycode:copy>
<a id="f-432a956d4c21-parseMarkdownProse"></a>
### `parseMarkdownProse`

Headings + inline marks of a plain markdown text (fences are verbatim).

<storycode:copy id="f-432a956d4c21-parseMarkdownProse">
/** Headings + inline marks of a plain markdown text (fences are verbatim). */
export function parseMarkdownProse(text: string): MarkdownProse {
  return scanMarkdownProse(text, fencedCodeRanges(text), []);
}

</storycode:copy>
<a id="f-432a956d4c21-markdownDecorations"></a>
### `markdownDecorations`

All markdown display decorations for a plain markdown buffer.

<storycode:copy id="f-432a956d4c21-markdownDecorations">
/** All markdown display decorations for a plain markdown buffer. */
export function markdownDecorations(state: EditorState): DecorationSet {
  const ranges = proseDecorationRanges(
    parseMarkdownProse(state.doc.toString()),
    state.doc.length,
  );
  ranges.sort(
    (a, b) => a.from - b.from || a.value.startSide - b.value.startSide,
  );
  return Decoration.set(ranges, true);
}

</storycode:copy>
<a id="f-432a956d4c21-markdownField"></a>
### `markdownField`

<storycode:copy id="f-432a956d4c21-markdownField">
const markdownField = StateField.define<DecorationSet>({
  create: markdownDecorations,
  update: (deco, tr) => (tr.docChanged ? markdownDecorations(tr.state) : deco),
  provide: (f) => EditorView.decorations.from(f),
});

</storycode:copy>
<a id="f-432a956d4c21-markdownStyling"></a>
### `markdownStyling`

The output panes' markdown styling extension: same look as the document
editor's prose, display-only, composes with whatever else the pane loads.

<storycode:copy id="f-432a956d4c21-markdownStyling">
/**
 * The output panes' markdown styling extension: same look as the document
 * editor's prose, display-only, composes with whatever else the pane loads.
 */
export function markdownStyling(): Extension {
  return markdownField;
}

</storycode:copy>
<a id="f-432a956d4c21-isMarkdownPath"></a>
### `isMarkdownPath`

<storycode:copy id="f-432a956d4c21-isMarkdownPath">
/** Whether a file path names a markdown file. */
export function isMarkdownPath(path: string): boolean {
  return /\.(md|markdown)$/i.test(path);
}
</storycode:copy>

### Assemble `apps/web/src/editor/markdownStyling.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/markdownStyling.ts"><storycode:paste select="#f-432a956d4c21-headingLineDecos"/><storycode:paste select="#f-432a956d4c21-mdMark"/><storycode:paste select="#f-432a956d4c21-mdStrong"/><storycode:paste select="#f-432a956d4c21-mdEm"/><storycode:paste select="#f-432a956d4c21-mdCode"/><storycode:paste select="#f-432a956d4c21-QUOTE_DEPTHS"/><storycode:paste select="#f-432a956d4c21-mdQuoteLines"/><storycode:paste select="#f-432a956d4c21-mdTaskLine"/><storycode:paste select="#f-432a956d4c21-mdTaskDoneLine"/><storycode:paste select="#f-432a956d4c21-MarkdownProse"/><storycode:paste select="#f-432a956d4c21-proseDecorationRanges"/><storycode:paste select="#f-432a956d4c21-fencedCodeRanges"/><storycode:paste select="#f-432a956d4c21-parseMarkdownProse"/><storycode:paste select="#f-432a956d4c21-markdownDecorations"/><storycode:paste select="#f-432a956d4c21-markdownField"/><storycode:paste select="#f-432a956d4c21-markdownStyling"/><storycode:paste select="#f-432a956d4c21-isMarkdownPath"/></storycode:file>

<a id="output-4284e1f70acf"></a>
## `apps/web/src/editor/taskList.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Quotes And Task Lists Are Read As Prose](contract.md).

<a id="f-4284e1f70acf-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4284e1f70acf-expression_statement">
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { parseMarkdownProse } from "./markdownStyling";
import { parseHickDoc } from "./hickDoc";
import { taskCheckboxes, toggleTaskAt } from "./taskList";

describe("task lists — the scan", () => {
  it("finds bullet and ordered items, checked and not", () => {
    const src = "- [ ] open\n* [x] closed\n1. [X] numbered\n";
    const { tasks } = parseMarkdownProse(src);
    expect(tasks.map((t) => t.checked)).toEqual([false, true, true]);
    expect(src.slice(tasks[0].boxFrom, tasks[0].boxTo)).toBe("[ ]");
    expect(src.slice(tasks[2].boxFrom, tasks[2].boxTo)).toBe("[X]");
  });

  it("ignores a bare bracket pair that is prose about brackets", () => {
    // Without the bullet in the match, a sentence mentioning `[ ]` would grow
    // a checkbox in the middle of a paragraph.
    expect(parseMarkdownProse("the empty box [ ] means nothing here\n").tasks).toEqual([]);
  });

  it("wants whitespace after the box, so `[x]y` is a link-ish literal", () => {
    expect(parseMarkdownProse("- [x]done\n").tasks).toEqual([]);
  });

  it("reads a task nested inside a quote", () => {
    const src = "> - [ ] quoted todo\n";
    const { tasks, quotes } = parseMarkdownProse(src);
    expect(quotes).toHaveLength(1);
    expect(tasks).toHaveLength(1);
    expect(src.slice(tasks[0].boxFrom, tasks[0].boxTo)).toBe("[ ]");
  });

  it("does not see a checkbox inside an exec cell's payload", () => {
    // The payload of a cell is a command, and `- [ ]` in it is an argument.
    const src = '<hick:exec>\n- [ ] not a checkbox\n</hick:exec>\n\n- [ ] a checkbox\n';
    const { tasks } = parseHickDoc(src);
    expect(tasks).toHaveLength(1);
    expect(src.slice(tasks[0].from, tasks[0].to)).toBe("- [ ] a checkbox");
  });
});

</storycode:copy>
<a id="f-4284e1f70acf-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4284e1f70acf-expression_statement-2">
describe("task lists — toggling", () => {
  const src = "- [ ] one\n- [x] two\n";

  it("checks an empty box with a lowercase x", () => {
    expect(toggleTaskAt(src, 3)).toEqual({ from: 3, to: 4, insert: "x" });
  });

  it("clears a checked box, wherever on the line the click landed", () => {
    // Position 14 is inside the word "two", not on the box itself.
    expect(toggleTaskAt(src, 14)).toEqual({ from: 13, to: 14, insert: " " });
  });

  it("normalises a capital X to an empty box rather than to lowercase", () => {
    expect(toggleTaskAt("- [X] shout\n", 0)).toEqual({ from: 3, to: 4, insert: " " });
  });

  it("answers null off a task line, so a stray click edits nothing", () => {
    expect(toggleTaskAt("just prose\n", 4)).toBeNull();
  });
});

</storycode:copy>
<a id="f-4284e1f70acf-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4284e1f70acf-expression_statement-3">
describe("task lists — the checkbox decoration", () => {
  const mount = (doc: string, cursor: number) => {
    const view = new EditorView({
      state: EditorState.create({
        doc,
        selection: { anchor: cursor },
        extensions: [taskCheckboxes()],
      }),
      parent: document.body,
    });
    return view;
  };

  it("replaces the box everywhere except the line holding the cursor", () => {
    // A person who put the caret on a task line is editing its text, and a
    // box they cannot see is a box they cannot fix.
    const view = mount("- [ ] one\n- [x] two\n", 2);
    const boxes = view.dom.querySelectorAll(".cm-md-checkbox");
    expect(boxes).toHaveLength(1);
    expect(boxes[0].getAttribute("aria-checked")).toBe("true");
    expect(view.state.doc.toString()).toContain("- [ ] one");
    view.destroy();
  });

  it("draws an unchecked box with no glyph and a checked one with a tick", () => {
    const view = mount("- [ ] one\n- [x] two\n", 20);
    const boxes = [...view.dom.querySelectorAll(".cm-md-checkbox")];
    expect(boxes).toHaveLength(2);
    expect(boxes[0].textContent).toBe("");
    expect(boxes[1].textContent).toBe("\u2713");
    view.destroy();
  });

  it("edits the document when a box is pressed", () => {
    const view = mount("- [ ] one\n", 10);
    const box = view.dom.querySelector(".cm-md-checkbox") as HTMLElement;
    box.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(view.state.doc.toString()).toBe("- [x] one\n");
    view.destroy();
  });

  it("offers no checkbox in a buffer that has no tasks", () => {
    const view = mount("# just prose\n", 0);
    expect(view.dom.querySelectorAll(".cm-md-checkbox")).toHaveLength(0);
    view.destroy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/taskList.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/taskList.test.ts"><storycode:paste select="#f-4284e1f70acf-expression_statement"/><storycode:paste select="#f-4284e1f70acf-expression_statement-2"/><storycode:paste select="#f-4284e1f70acf-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/hickDoc.ts](../../../mechanisms/apps/web/src/editor/hickDoc.ts.md#output-c2f7bc821e36)
- [apps/web/src/editor/markdownStyling.ts](contract.md#output-432a956d4c21)
- [apps/web/src/editor/taskList.ts](../../../mechanisms/apps/web/src/editor/taskList.ts.md#output-529a1371d56c)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)

### Tests and fixtures

- [apps/web/src/editor/markdownStyling.test.ts](contract.md#output-59145de2de8b)
- [apps/web/src/editor/taskList.test.ts](contract.md#output-4284e1f70acf)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
