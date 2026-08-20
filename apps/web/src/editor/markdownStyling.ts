// Typora-style markdown DISPLAY styling, shared between the document editor
// (wysiwyg.ts, over .hick prose) and the output panes (OutputEditorPane, over
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
export const mdMark = Decoration.mark({ class: "cm-md-mark" });
export const mdStrong = Decoration.mark({ class: "cm-md-strong" });
export const mdEm = Decoration.mark({ class: "cm-md-em" });
export const mdCode = Decoration.mark({ class: "cm-md-code" });

/** Quote lines, by nesting depth. Deeper than three reuses the third tint:
 * the point of the indent is "this is quoted", and a fourth shade of the same
 * grey carries no information a reader could act on. */
export const QUOTE_DEPTHS = 3;
export const mdQuoteLines = [1, 2, 3].map((depth) =>
  Decoration.line({ class: `cm-md-quote cm-md-quote-${depth}` }),
);
export const mdTaskLine = Decoration.line({ class: "cm-md-task" });
export const mdTaskDoneLine = Decoration.line({
  class: "cm-md-task cm-md-task--done",
});

export interface MarkdownProse {
  headings: Heading[];
  inline: InlineMark[];
  quotes: QuoteLine[];
  tasks: TaskItem[];
}

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

/** Headings + inline marks of a plain markdown text (fences are verbatim). */
export function parseMarkdownProse(text: string): MarkdownProse {
  return scanMarkdownProse(text, fencedCodeRanges(text), []);
}

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

const markdownField = StateField.define<DecorationSet>({
  create: markdownDecorations,
  update: (deco, tr) => (tr.docChanged ? markdownDecorations(tr.state) : deco),
  provide: (f) => EditorView.decorations.from(f),
});

/**
 * The output panes' markdown styling extension: same look as the document
 * editor's prose, display-only, composes with whatever else the pane loads.
 */
export function markdownStyling(): Extension {
  return markdownField;
}

/** Whether a file path names a markdown file. */
export function isMarkdownPath(path: string): boolean {
  return /\.(md|markdown)$/i.test(path);
}
