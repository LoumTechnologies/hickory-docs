// Markdown presentation shared by source and output panes. Decorations can
// conceal a prefix or replace its glyphs while source and selections keep
// their original offsets.
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
import type { Extension, Range, Transaction } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { scanMarkdownProse, yamlFrontmatter } from "./hickDoc";
import type { Heading, InlineMark, ListMarker, ProseLine, QuoteLine, TaskItem } from "./hickDoc";

// Reuse each line decoration so the editor can keep existing DOM on updates.
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

export function proseCarets(state: EditorState): number[] {
  return state.selection.ranges.filter(range => range.empty).map(range => range.head);
}

export function proseSourceVisible(line: ProseLine, carets: readonly number[]): boolean {
  const { from, to } = line.quote ?? line;
  return carets.some(caret => caret >= from && caret <= to);
}

class ListLabel extends WidgetType {
  constructor(private readonly label: string) { super(); }
  eq(other: ListLabel) { return this.label === other.label; }
  toDOM() {
    const marker = document.createElement("span");
    marker.textContent = this.label;
    marker.style.color = "var(--fg-muted, currentColor)";
    return marker;
  }
  ignoreEvent() { return false; }
}

export function quoteClipboard(quotes: (state: EditorState) => readonly QuoteLine[]): Extension {
  return EditorView.domEventHandlers({
    copy(event, view) {
      const selection = view.state.selection;
      if (selection.ranges.length !== 1 || selection.main.empty || !event.clipboardData) return false;
      const { from, to } = selection.main;
      const lines = quotes(view.state);
      const block = lines.find(line => from >= line.from && from <= line.to)?.quote;
      if (!block || to > block.to + 1) return false;
      let text = "", offset = from;
      for (const line of lines) {
        if (line.quote !== block || line.markTo <= from || line.markFrom >= to) continue;
        text += view.state.sliceDoc(offset, Math.max(offset, line.markFrom));
        offset = Math.min(to, line.markTo);
      }
      if (offset === from) return false;
      text += view.state.sliceDoc(offset, to);
      event.clipboardData.setData("text/plain", text);
      return true;
    },
  });
}

export interface MarkdownProse {
  headings: Heading[];
  inline: InlineMark[];
  quotes: QuoteLine[];
  lists: ListMarker[];
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
  carets: readonly number[] = [],
): Range<Decoration>[] {
  const ranges: Range<Decoration>[] = [];
  const clamp = (n: number) => Math.max(0, Math.min(n, docLen));
  const raw = [...prose.quotes, ...prose.lists].filter(line => proseSourceVisible(line, carets))
    .sort((a, b) => a.from - b.from);
  const revealed = new Set(raw.map(line => line.from));
  let rawIndex = 0;
  for (const h of prose.headings) {
    if (revealed.has(h.from)) continue;
    ranges.push(headingLineDecos[Math.min(h.level, 6) - 1].range(clamp(h.from)));
    if (h.markTo > h.markFrom)
      ranges.push(mdMark.range(clamp(h.markFrom), clamp(h.markTo)));
  }
  for (const quote of prose.quotes) {
    if (revealed.has(quote.from)) continue;
    const depth = Math.min(Math.max(quote.depth, 1), QUOTE_DEPTHS);
    ranges.push(mdQuoteLines[depth - 1].range(clamp(quote.from)));
    if (quote.markTo > quote.markFrom)
      ranges.push(Decoration.replace({}).range(clamp(quote.markFrom), clamp(quote.markTo)));
  }
  for (const item of prose.lists) {
    if (!revealed.has(item.from) && clamp(item.markTo) > clamp(item.markFrom))
      ranges.push(Decoration.replace({ widget: item.label ? new ListLabel(item.label) : undefined })
        .range(clamp(item.markFrom), clamp(item.markTo)));
  }
  for (const task of prose.tasks) {
    if (revealed.has(task.from)) continue;
    ranges.push(
      (task.checked ? mdTaskDoneLine : mdTaskLine).range(clamp(task.from)),
    );
    ranges.push(mdMark.range(clamp(task.boxFrom), clamp(task.boxTo)));
  }
  for (const mark of prose.inline) {
    while (rawIndex < raw.length && raw[rawIndex].to < mark.openFrom) rawIndex++;
    if (rawIndex < raw.length && mark.openFrom >= raw[rawIndex].from) continue;
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
  return scanMarkdownProse(text, [], []);
}

/** All markdown display decorations for a plain markdown buffer. */
const plainProseCache = new WeakMap<EditorState["doc"], MarkdownProse>();
export function markdownProseOf(state: EditorState): MarkdownProse {
  let prose = plainProseCache.get(state.doc);
  if (!prose) {
    prose = parseMarkdownProse(state.doc.toString());
    plainProseCache.set(state.doc, prose);
  }
  return prose;
}

export function markdownDecorations(state: EditorState, prose = markdownProseOf(state)): DecorationSet {
  const ranges = proseDecorationRanges(
    prose,
    state.doc.length,
    proseCarets(state),
  );
  ranges.sort(
    (a, b) => a.from - b.from || a.value.startSide - b.value.startSide,
  );
  return Decoration.set(ranges, true);
}

export function proseStyling(
  source: (state: EditorState) => MarkdownProse,
  refresh?: (transaction: Transaction) => boolean,
): Extension {
  const field = StateField.define<DecorationSet>({
    create: state => markdownDecorations(state, source(state)),
    update: (value, tr) => tr.docChanged || tr.selection || refresh?.(tr)
      ? markdownDecorations(tr.state, source(tr.state)) : value,
    provide: field => EditorView.decorations.from(field),
  });
  return [field, quoteClipboard(state => source(state).quotes), EditorView.baseTheme({
    ".cm-line.cm-md-quote": {
      color: "var(--fg-muted, CanvasText)", borderLeft: "3px solid var(--border, GrayText)",
      paddingLeft: "0.7em", marginLeft: "0.1em",
    },
    ".cm-line.cm-md-quote-2": { boxShadow: "inset 6px 0 0 -3px var(--border, GrayText)", paddingLeft: "1.3em" },
    ".cm-line.cm-md-quote-3": {
      boxShadow: "inset 6px 0 0 -3px var(--border, GrayText), inset 12px 0 0 -9px var(--border, GrayText)",
      paddingLeft: "1.9em",
    },
    ".cm-md-task--done": { color: "var(--fg-muted, GrayText)" },
    ".cm-md-checkbox": {
      display: "inline-block", width: "1.05em", height: "1.05em", lineHeight: "1.05em",
      textAlign: "center", verticalAlign: "-0.15em", marginRight: "0.15em",
      border: "1px solid var(--fg-muted, GrayText)", borderRadius: "3px", fontSize: "0.85em",
      cursor: "pointer", color: "var(--bg, Canvas)", userSelect: "none",
    },
    ".cm-md-checkbox--on": { background: "var(--accent, Highlight)", borderColor: "var(--accent, Highlight)" },
    '.cm-md-checkbox[aria-disabled="true"]': { cursor: "inherit" },
  })];
}

/**
 * The output panes' markdown styling extension: same look as the document
 * editor's prose, display-only, composes with whatever else the pane loads.
 */
export function markdownStyling(): Extension {
  return [proseStyling(markdownProseOf), horizontalRuleStyling()];
}


/** Render dash rules except on lines containing a cursor. */
export function horizontalRuleStyling(
  excluded: (state: EditorState) => readonly (readonly [number, number])[] = () => [],
  refresh?: (transaction: Transaction) => boolean,
): Extension {
  const build = (state: EditorState): DecorationSet => {
    const text = state.doc.toString();
    const metadata = yamlFrontmatter(text);
    const blocked = [...fencedCodeRanges(text), ...excluded(state), ...(metadata ? [[0, metadata.end] as const] : [])];
    const active = new Set(state.selection.ranges.map((r) => state.doc.lineAt(r.head).number));
    const ranges: Range<Decoration>[] = [];
    for (let n = 1; n <= state.doc.lines; n++) {
      const line = state.doc.line(n);
      if (active.has(n) || !/^ {0,3}(?:-[ \t]*){3,}$/.test(line.text)) continue;
      if (blocked.some(([from, to]) => line.from < to && line.to > from)) continue;
      // A nonblank preceding paragraph makes --- a heading underline.
      if (n > 1 && state.doc.line(n - 1).text.trim() &&
          !/^ {0,3}(?:-[ \t]*){3,}$/.test(state.doc.line(n - 1).text)) continue;
      ranges.push(Decoration.line({ class: "cm-md-rule" }).range(line.from));
    }
    return Decoration.set(ranges);
  };
  return [
    StateField.define<DecorationSet>({
      create: build,
      update: (value, tr) => tr.docChanged || tr.selection || refresh?.(tr) ? build(tr.state) : value,
      provide: (field) => EditorView.decorations.from(field),
    }),
    EditorView.baseTheme({
      ".cm-line.cm-md-rule": { position: "relative", color: "transparent" },
      ".cm-line.cm-md-rule::after": {
        content: '\'""\'', position: "absolute", left: "0", top: "50%",
        width: "var(--prose-wrap, 100%)", borderTop: "1px solid var(--fg, currentColor)",
        opacity: "0.35", pointerEvents: "none",
      },
    }),
  ];
}
/** Whether a file path names a markdown file. */
export function isMarkdownPath(path: string): boolean {
  return /\.(md|markdown)$/i.test(path);
}
