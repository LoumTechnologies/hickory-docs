import { EditorSelection, Prec, RangeSet, StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, gutter, keymap, WidgetType } from "@codemirror/view";

interface Gap { pos: number; height: number; side?: number }
const spacesEffect = StateEffect.define<Gap[]>();
class FileSpace extends WidgetType {
  constructor(readonly height: number) { super(); }
  eq(other: FileSpace) { return this.height === other.height; }
  toDOM() { const el = document.createElement("div"); el.className = "spatial-file-space"; el.style.height = `${this.height}px`; el.setAttribute("aria-hidden", "true"); return el; }
}
const spacesField = StateField.define<ReturnType<typeof Decoration.set>>({
  create: () => Decoration.none,
  update(value, tr) { let next = value.map(tr.changes); for (const effect of tr.effects) if (effect.is(spacesEffect)) next = Decoration.set(effect.value.map(gap => Decoration.widget({ block: true, side: gap.side ?? -1, widget: new FileSpace(gap.height) }).range(gap.pos)), true); return next; },
  provide: field => EditorView.decorations.from(field),
});

export interface FilePreviewRow { path: string; key: string; top: number; pinned: boolean; height?: number; below?: boolean }
const previewEffect = StateEffect.define<ReturnType<typeof Decoration.set>>();
const previewField = StateField.define<ReturnType<typeof Decoration.set>>({
  create: () => Decoration.none,
  update(value, tr) { for (const effect of tr.effects) if (effect.is(previewEffect)) return effect.value; return value.map(tr.changes); },
  provide: field => EditorView.decorations.from(field),
});

class PreviewGrip extends GutterMarker {
  constructor(readonly key: string, readonly pinned: boolean, readonly offset: number) { super(); }
  eq(other: PreviewGrip) { return this.key === other.key && this.pinned === other.pinned && Math.abs(this.offset - other.offset) < .1; }
  toDOM() {
    const el = document.createElement("span"); el.textContent = this.pinned ? "◆" : "◇";
    el.dataset.spatialDrag = this.key; el.dataset.previewPinned = String(this.pinned); el.className = "spatial-file-grip";
    el.style.transform = `translateY(${this.offset}px)`;
    el.dataset.tip = "Drag preview; click to pin or unpin"; return el;
  }
}
const previewGutter = gutter({ class: "spatial-preview-gutter", markers: view => {
  const marks = [];
  for (const cursor = view.state.field(previewField).iter(); cursor.value; cursor.next()) {
    const attrs = cursor.value.spec.attributes;
    if (attrs?.["data-spatial-row"]) marks.push(new PreviewGrip(attrs["data-spatial-row"], attrs["data-preview-pinned"] === "true", Number(attrs["data-spatial-offset"] ?? 0)).range(cursor.from));
  }
  return RangeSet.of(marks, true);
} });

/** CodeMirror's height map describes tree order. The browser's caret hit-test
 * describes the displayed text after a row passes another row. */
const previewMouseSelection = EditorView.mouseSelectionStyle.of((view, startEvent) => {
  if (startEvent.button !== 0 || startEvent.altKey || !view.state.field(previewField).size || !view.contentDOM.contains(startEvent.target as Node)) return null;
  const at = (event: MouseEvent) => {
    const doc = view.contentDOM.ownerDocument;
    const caret = doc.caretPositionFromPoint?.(event.clientX, event.clientY);
    if (caret && view.contentDOM.contains(caret.offsetNode)) return view.posAtDOM(caret.offsetNode, caret.offset);
    const range = doc.caretRangeFromPoint?.(event.clientX, event.clientY);
    if (range && view.contentDOM.contains(range.startContainer)) return view.posAtDOM(range.startContainer, range.startOffset);
    const row = doc.elementFromPoint(event.clientX, event.clientY)?.closest(".cm-line");
    if (row && view.contentDOM.contains(row)) return view.posAtDOM(row);
    return view.posAtCoords({ x: event.clientX, y: event.clientY }, false);
  };
  let start = at(startEvent), initial = view.state.selection;
  return {
    update(update) {
      if (update.docChanged) { start = update.changes.mapPos(start); initial = initial.map(update.changes); }
    },
    get(event, extend, multiple) {
      const current = at(event), line = view.state.doc.lineAt(current);
      const range = startEvent.detail >= 2 ? EditorSelection.range(line.from, line.to)
        : EditorSelection.range(extend ? initial.main.anchor : start, current);
      if (extend) return initial.replaceRange(range);
      if (multiple) return initial.addRange(range);
      return EditorSelection.create([range]);
    },
  };
});

/** Move through the rows as they are drawn. Pinned filenames are translated
 * without changing document order, so CodeMirror's height map no longer
 * describes the row directly below the caret. */
const previewVerticalNavigation = Prec.highest(keymap.of([
  { key: "Shift-ArrowDown", run: (view) => moveToVisibleRow(view, 1, true) },
  { key: "Shift-ArrowUp", run: (view) => moveToVisibleRow(view, -1, true) },
  { key: "ArrowDown", run: (view) => moveToVisibleRow(view, 1, false) },
  { key: "ArrowUp", run: (view) => moveToVisibleRow(view, -1, false) },
]));

function moveToVisibleRow(view: EditorView, direction: -1 | 1, extend: boolean): boolean {
  if (!view.state.field(previewField).size) return false;
  const selection = view.state.selection.main;
  const head = selection.head;
  const currentLine = view.state.doc.lineAt(head).number;
  const rows = [...view.contentDOM.querySelectorAll<HTMLElement>(".cm-line")]
    .map(element => {
      try {
        const line = view.state.doc.lineAt(view.posAtDOM(element)).number;
        return { element, line, rect: element.getBoundingClientRect() };
      } catch { return null; }
    })
    .filter((row): row is { element: HTMLElement; line: number; rect: DOMRect } => !!row && row.rect.height > 0)
    .sort((a, b) => a.rect.top - b.rect.top);
  const index = rows.findIndex(row => row.line === currentLine);
  const target = rows[index + direction];
  if (index < 0 || !target) return false;

  const caret = view.coordsAtPos(head);
  const x = caret?.left ?? target.rect.left;
  const y = target.rect.top + Math.min(target.rect.height / 2, 12);
  const doc = view.contentDOM.ownerDocument;
  const caretPosition = doc.caretPositionFromPoint?.(x, y);
  const caretRange = doc.caretRangeFromPoint?.(x, y);
  let position: number | null = null;
  try {
    if (caretPosition && view.contentDOM.contains(caretPosition.offsetNode)) {
      position = view.posAtDOM(caretPosition.offsetNode, caretPosition.offset);
    } else if (caretRange && view.contentDOM.contains(caretRange.startContainer)) {
      position = view.posAtDOM(caretRange.startContainer, caretRange.startOffset);
    }
  } catch { /* A stale DOM hit falls back to the target row start. */ }
  if (position === null || view.state.doc.lineAt(position).number !== target.line) {
    position = view.state.doc.line(target.line).from;
  }
  view.dispatch({ selection: extend ? { anchor: selection.anchor, head: position } : { anchor: position, head: position } });
  return true;
}

/** The actual editable filename lines follow screen placements independently of
 * tree order. Reserve normal rows around them; no row feeds placement back into
 * the window, and a flowing filename can pass either side of a pinned one. */
export function alignSpatialFileRows(view: EditorView, previews: readonly FilePreviewRow[]) {
  if (!view.state.field(spacesField, false)) view.dispatch({ effects: StateEffect.appendConfig.of([spacesField, previewField, previewGutter, previewMouseSelection, previewVerticalNavigation]) });
  const old: Gap[] = [], next: Gap[] = [], ranges = [];
  for (const cursor = view.state.field(spacesField).iter(); cursor.value; cursor.next()) {
    const widget = cursor.value.spec.widget;
    if (widget instanceof FileSpace) old.push({ pos: cursor.from, height: widget.height, side: cursor.value.spec.side });
  }
  let oldAdded = 0, gapIndex = 0;
  const rows = [...view.contentDOM.querySelectorAll<HTMLElement>(".cm-line")].flatMap(row => {
    let pos: number;
    try { pos = view.state.doc.lineAt(view.posAtDOM(row)).from; } catch { return []; }
    while (gapIndex < old.length && old[gapIndex].pos <= pos) {
      if ((old[gapIndex].side ?? -1) < 0) oldAdded += old[gapIndex].height;
      gapIndex++;
    }
    const rect = row.getBoundingClientRect(), offset = Number(row.dataset.spatialOffset ?? 0);
    return [{ pos, base: rect.top - offset - oldAdded, height: rect.height,
      preview: previews.find(item => item.path === row.dataset.treePath) }];
  });
  const headers = previews.map(preview => ({ top: preview.top, bottom: preview.top + (preview.below ? Math.max(36, preview.height ?? 36) : 36) })).sort((a, b) => a.top - b.top);
  let added = 0, bottom = view.contentDOM.getBoundingClientRect().top;
  for (const { pos, base, height, preview } of rows) {
    let top = base + added;
    if (!preview) {
      for (const header of headers) if (top < header.bottom && top + height > header.top) top = header.bottom;
      const gap = Math.max(0, Math.ceil(top - base - added));
      if (gap) { next.push({ pos, height: gap }); added += gap; }
    } else {
      const offset = Math.round((preview.top - top) * 10) / 10;
      ranges.push(Decoration.line({ class: `spatial-file-row${preview.pinned ? " spatial-file-row--pinned" : ""}${preview.below ? " spatial-file-row--below" : ""}`,
        attributes: { "data-spatial-row": preview.key, "data-preview-pinned": String(preview.pinned),
          "data-spatial-offset": String(offset), style: `transform: translateY(${offset}px); position: relative; z-index: 1` } }).range(pos));
    }
    bottom = Math.max(bottom, top + height);
  }
  // A translated final filename still belongs inside the editor's scrollport.
  // Keep room for it in the document layout rather than clipping it at EOF.
  const end = Math.max(bottom, ...headers.map(header => header.bottom));
  if (end > bottom) next.push({ pos: view.state.doc.length, height: Math.ceil(end - bottom), side: 1 });
  const signature = (gaps: Gap[]) => gaps.map(gap => `${gap.pos}:${gap.height}:${gap.side ?? -1}`).join(",");
  const decorations = Decoration.set(ranges, true);
  if (signature(old) !== signature(next) || !RangeSet.eq([view.state.field(previewField)], [decorations])) {
    view.dispatch({ effects: [spacesEffect.of(next), previewEffect.of(decorations)] });
    return true;
  }
  return false;
}

/** Snap to measured text rows, including folding, zoom and Files scrolling. */
export function snapSpatialFileRow(view: EditorView, y: number): number | undefined {
  const clip = view.scrollDOM.getBoundingClientRect();
  const rows = [...view.dom.querySelectorAll<HTMLElement>("[data-tree-path]")]
    .map(row => row.getBoundingClientRect().top).filter(top => top >= clip.top && top < clip.bottom);
  const nearest = rows.sort((a, b) => Math.abs(a - y) - Math.abs(b - y))[0];
  if (nearest === undefined) return;
  // The filename row itself can occupy a new slot below the last entry. Use
  // the editor's measured line rhythm there; alignment moves that real row
  // into the slot without inventing another filename or editing tree bytes.
  const height = view.defaultLineHeight;
  return nearest + Math.round((y - nearest) / height) * height;
}
