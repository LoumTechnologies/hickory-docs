import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { bandBetween } from "../shell/Ribbons";

interface Gap { pos: number; height: number }
const spacesEffect = StateEffect.define<Gap[]>();
class FileSpace extends WidgetType {
  constructor(readonly height: number) { super(); }
  eq(other: FileSpace) { return this.height === other.height; }
  toDOM() { const el = document.createElement("div"); el.className = "spatial-file-space"; el.style.height = `${this.height}px`; el.setAttribute("aria-hidden", "true"); return el; }
}
const spacesField = StateField.define<ReturnType<typeof Decoration.set>>({
  create: () => Decoration.none,
  update(value, tr) { let next = value.map(tr.changes); for (const effect of tr.effects) if (effect.is(spacesEffect)) next = Decoration.set(effect.value.map(gap => Decoration.widget({ block: true, side: -1, widget: new FileSpace(gap.height) }).range(gap.pos)), true); return next; },
  provide: field => EditorView.decorations.from(field),
});

/** Native block widgets reserve complete filename rows without touching tree bytes. */
export function reserveSpatialFileRows(view: EditorView, headers: readonly { top: number; bottom: number }[]) {
  if (!view.state.field(spacesField, false)) view.dispatch({ effects: StateEffect.appendConfig.of(spacesField) });
  const old: Gap[] = [], next: Gap[] = [], visible = view.scrollDOM.getBoundingClientRect();
  for (const cursor = view.state.field(spacesField).iter(); cursor.value; cursor.next()) { const widget = cursor.value.spec.widget; if (widget instanceof FileSpace) old.push({ pos: cursor.from, height: widget.height }); }
  let oldAdded = 0, added = 0, gapIndex = 0;
  for (let n = 1; n <= view.state.doc.lines; n++) {
    const line = view.state.doc.line(n), band = bandBetween(view, line.from, line.to);
    if (!band) continue;
    while (gapIndex < old.length && old[gapIndex].pos <= line.from) oldAdded += old[gapIndex++].height;
    if (view.lineBlockAt(line.from).from !== line.from) continue;
    const base = band[0] - oldAdded, h = Math.max(36, band[1] - band[0]); let top = base + added;
    for (const header of headers) if (top < header.bottom && top + h > header.top && header.bottom > visible.top && header.top < visible.bottom) top = header.bottom;
    const height = Math.round(top - base - added);
    if (height > 0) { next.push({ pos: line.from, height }); added += height; }
  }
  const signature = (gaps: Gap[]) => gaps.map(gap => `${gap.pos}:${gap.height}`).join(",");
  if (signature(old) === signature(next)) return;
  view.dispatch({ effects: spacesEffect.of(next) });
}
