// Typora-style WYSIWYG decorations over raw .hick source.
//
// The user always edits the real source text — every decoration here is a
// mark, line class, or block *widget*; nothing replaces or hides text, so the
// cursor can enter any styled region without corrupting the document.
//
// Split per CodeMirror's rules:
//  - mark/line decorations (headings, inline markdown, tag chrome, cell
//    frames) come from a ViewPlugin that materialises them only for the
//    visible ranges;
//  - block widgets (the exec-cell panel below each cell, the file-path chip
//    above each file block) come from a StateField, because plugin
//    decorations may not affect vertical layout.

import { EditorState, StateField } from "@codemirror/state";
import type { Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType } from "@codemirror/view";
import type { DecorationSet, ViewUpdate } from "@codemirror/view";
import { parseHickDoc, execBlocksOf, fileBlocksOf } from "./hickDoc";
import type { HickDocStructure } from "./hickDoc";

// ---------------------------------------------------------------------------
// Structure cache: one parse per document version, shared by field + plugin.
// ---------------------------------------------------------------------------

const structureCache = new WeakMap<object, HickDocStructure>();

export function structureOf(state: EditorState): HickDocStructure {
  const key = state.doc as unknown as object;
  let s = structureCache.get(key);
  if (!s) {
    s = parseHickDoc(state.doc.toString());
    structureCache.set(key, s);
  }
  return s;
}

// ---------------------------------------------------------------------------
// Cell widget registry — the bridge to React. Widgets mount bare container
// elements; DocumentEditor renders CellPanel components into them via portals.
// ---------------------------------------------------------------------------

export interface CellSlot {
  key: string;
  el: HTMLElement;
  /** Source span of the exec block this panel belongs to. */
  span: [number, number];
  /** Ordinal among exec blocks in the document. */
  index: number;
}

export class CellRegistry {
  private slots = new Map<string, CellSlot>();
  private listeners = new Set<() => void>();

  register(slot: CellSlot) {
    this.slots.set(slot.key, slot);
    queueMicrotask(() => this.notify());
  }

  unregister(key: string, el: HTMLElement) {
    // Only remove if the registered element is still ours (CM may create the
    // replacement widget before destroying the old one).
    if (this.slots.get(key)?.el === el) {
      this.slots.delete(key);
      queueMicrotask(() => this.notify());
    }
  }

  list(): CellSlot[] {
    return [...this.slots.values()].sort((a, b) => a.index - b.index);
  }

  subscribe(fn: () => void): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  private notify() {
    for (const fn of this.listeners) fn();
  }
}

class CellPanelWidget extends WidgetType {
  constructor(
    private registry: CellRegistry,
    private key: string,
    private span: [number, number],
    private index: number,
  ) {
    super();
  }

  eq(other: CellPanelWidget) {
    // Same key → reuse DOM; the panel's content is owned by React and updates
    // through the portal regardless of CM redraws. Span updates re-register.
    if (other.key !== this.key) return false;
    if (other.span[0] !== this.span[0] || other.span[1] !== this.span[1]) {
      return false;
    }
    return true;
  }

  toDOM() {
    const el = document.createElement("div");
    el.className = "cm-cell-panel";
    this.registry.register({ key: this.key, el, span: this.span, index: this.index });
    return el;
  }

  destroy(el: HTMLElement) {
    this.registry.unregister(this.key, el);
  }

  get estimatedHeight() {
    return 40;
  }

  ignoreEvent() {
    // The panel is interactive React UI; the editor must not treat clicks in
    // it as editing gestures.
    return true;
  }
}

class FileChipWidget extends WidgetType {
  constructor(
    private path: string,
    private language: string | undefined,
  ) {
    super();
  }

  eq(other: FileChipWidget) {
    return other.path === this.path && other.language === this.language;
  }

  toDOM() {
    const el = document.createElement("div");
    el.className = "cm-file-chip";
    const path = document.createElement("span");
    path.className = "cm-file-chip-path";
    path.textContent = this.path || "(unnamed file)";
    el.appendChild(path);
    if (this.language) {
      const lang = document.createElement("span");
      lang.className = "cm-file-chip-lang";
      lang.textContent = this.language;
      el.appendChild(lang);
    }
    return el;
  }

  ignoreEvent() {
    return false;
  }
}

function buildWidgets(state: EditorState, registry: CellRegistry): DecorationSet {
  const structure = structureOf(state);
  const ranges: Range<Decoration>[] = [];
  const docLen = state.doc.length;
  execBlocksOf(structure).forEach((block, i) => {
    const at = Math.min(block.to, docLen);
    ranges.push(
      Decoration.widget({
        widget: new CellPanelWidget(registry, `exec-${i}`, [block.from, block.to], i),
        side: 1,
        block: true,
      }).range(at),
    );
  });
  for (const block of fileBlocksOf(structure)) {
    const at = Math.min(block.from, docLen);
    ranges.push(
      Decoration.widget({
        widget: new FileChipWidget(block.attrs.path ?? "", block.attrs.language),
        side: -1,
        block: true,
      }).range(at),
    );
  }
  ranges.sort((a, b) => a.from - b.from || (a.value.spec.side ?? 0) - (b.value.spec.side ?? 0));
  return Decoration.set(ranges, true);
}

function cellWidgetField(registry: CellRegistry) {
  return StateField.define<DecorationSet>({
    create: (state) => buildWidgets(state, registry),
    update: (widgets, tr) => (tr.docChanged ? buildWidgets(tr.state, registry) : widgets),
    provide: (f) => EditorView.decorations.from(f),
  });
}

// ---------------------------------------------------------------------------
// Mark + line decorations (visible ranges only).
// ---------------------------------------------------------------------------

const headingLine = [1, 2, 3, 4, 5, 6].map((level) =>
  Decoration.line({ class: `cm-md-heading cm-md-h${level}` }),
);
const mdMark = Decoration.mark({ class: "cm-md-mark" });
const mdStrong = Decoration.mark({ class: "cm-md-strong" });
const mdEm = Decoration.mark({ class: "cm-md-em" });
const mdCode = Decoration.mark({ class: "cm-md-code" });
const tagChrome = Decoration.mark({ class: "cm-hick-chrome" });
const tagAttr = Decoration.mark({ class: "cm-hick-attr" });

const cellLine = Decoration.line({ class: "cm-cell-line" });
const cellLineFirst = Decoration.line({ class: "cm-cell-line cm-cell-first" });
const cellLineLast = Decoration.line({ class: "cm-cell-line cm-cell-last" });
const cellLineOnly = Decoration.line({ class: "cm-cell-line cm-cell-first cm-cell-last" });
const fileLine = Decoration.line({ class: "cm-file-line" });
const fileLineFirst = Decoration.line({ class: "cm-file-line cm-file-first" });
const fileLineLast = Decoration.line({ class: "cm-file-line cm-file-last" });
const fileLineOnly = Decoration.line({ class: "cm-file-line cm-file-first cm-file-last" });

function lineDecosForBlock(
  view: EditorView,
  from: number,
  to: number,
  decos: [Decoration, Decoration, Decoration, Decoration],
  out: Range<Decoration>[],
) {
  const doc = view.state.doc;
  const first = doc.lineAt(Math.min(from, doc.length)).number;
  const last = doc.lineAt(Math.min(Math.max(to - 1, from), doc.length)).number;
  const [mid, firstD, lastD, onlyD] = decos;
  for (let n = first; n <= last; n++) {
    const line = doc.line(n);
    const deco =
      first === last ? onlyD : n === first ? firstD : n === last ? lastD : mid;
    out.push(deco.range(line.from));
  }
}

function buildMarks(view: EditorView): DecorationSet {
  const structure = structureOf(view.state);
  const ranges: Range<Decoration>[] = [];
  const docLen = view.state.doc.length;
  const clamp = (n: number) => Math.max(0, Math.min(n, docLen));

  for (const { from, to } of view.visibleRanges) {
    // Cell / file frames (line decorations).
    for (const block of structure.blocks) {
      if (block.to < from || block.from > to) continue;
      if (block.name === "exec") {
        lineDecosForBlock(
          view,
          block.from,
          block.to,
          [cellLine, cellLineFirst, cellLineLast, cellLineOnly],
          ranges,
        );
      } else if (block.name === "file") {
        lineDecosForBlock(
          view,
          block.from,
          block.to,
          [fileLine, fileLineFirst, fileLineLast, fileLineOnly],
          ranges,
        );
      }
    }
    // Headings: big type, `#` marks kept but dimmed.
    for (const h of structure.headings) {
      if (h.to < from || h.from > to) continue;
      ranges.push(headingLine[Math.min(h.level, 6) - 1].range(clamp(h.from)));
      if (h.markTo > h.markFrom) ranges.push(mdMark.range(clamp(h.markFrom), clamp(h.markTo)));
    }
    // Inline markdown: content styled, delimiters visible but dimmed.
    for (const mark of structure.inline) {
      if (mark.closeTo < from || mark.openFrom > to) continue;
      const style = mark.kind === "strong" ? mdStrong : mark.kind === "em" ? mdEm : mdCode;
      ranges.push(mdMark.range(clamp(mark.openFrom), clamp(mark.openTo)));
      if (mark.to > mark.from) ranges.push(style.range(clamp(mark.from), clamp(mark.to)));
      ranges.push(mdMark.range(clamp(mark.closeFrom), clamp(mark.closeTo)));
    }
    // Hick tags as subtle chrome; attribute names a shade different.
    for (const tag of structure.tags) {
      if (tag.to < from || tag.from > to) continue;
      ranges.push(tagChrome.range(clamp(tag.from), clamp(tag.to)));
      for (const a of tag.attrNames) {
        if (a.to > a.from) ranges.push(tagAttr.range(clamp(a.from), clamp(a.to)));
      }
    }
  }

  ranges.sort((a, b) => a.from - b.from || a.value.startSide - b.value.startSide);
  return Decoration.set(ranges, true);
}

const markPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = buildMarks(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = buildMarks(update.view);
      }
    }
  },
  { decorations: (v) => v.decorations },
);

/** The Document-view WYSIWYG extension set. */
export function wysiwyg(registry: CellRegistry): Extension {
  return [cellWidgetField(registry), markPlugin];
}
