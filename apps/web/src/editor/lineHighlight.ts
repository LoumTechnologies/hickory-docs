// Line-number tinting for a hovered lineage connection, shared by BOTH rails
// of a pane: CodeMirror's left gutter (through the lineNumberMarkers facet)
// and the right rail (editor/RightRail.tsx reads the same field). One state,
// two renderings — the tint can never disagree with itself across a pane.
//
// The ribbon overlay dispatches `setLineHighlight` at the involved editors on
// hover, with the ribbon's palette index; `null` clears it on leave.

import { RangeSet, RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import { GutterMarker, lineNumberMarkers, type EditorView } from "@codemirror/view";

/** Char range of the involved text, plus the ribbon's palette index. */
export interface LineHighlight {
  from: number;
  to: number;
  color: number;
}

export const setLineHighlight = StateEffect.define<LineHighlight | null>();

/**
 * Fired on window after a highlight dispatch: the right rails redraw on
 * scroll and resize, which a highlight change is neither, so it announces
 * itself.
 */
export const RAIL_SYNC_EVENT = "hickory:rail-sync";

class Tint extends GutterMarker {
  constructor(color: number) {
    super();
    this.elementClass = `cm-linehl cm-linehl-c${color}`;
  }
}

export const lineHighlightField = StateField.define<LineHighlight | null>({
  create: () => null,
  update(value, tr) {
    if (value && tr.docChanged) {
      value = {
        ...value,
        from: tr.changes.mapPos(value.from, 1),
        to: tr.changes.mapPos(value.to, -1),
      };
    }
    for (const e of tr.effects) if (e.is(setLineHighlight)) value = e.value;
    return value;
  },
  provide: (field) =>
    lineNumberMarkers.compute([field], (state) => {
      const hl = state.field(field);
      if (!hl) return RangeSet.empty;
      const doc = state.doc;
      const clamp = (pos: number) => Math.max(0, Math.min(pos, doc.length));
      const first = doc.lineAt(clamp(hl.from)).number;
      const last = doc.lineAt(clamp(Math.max(hl.from, hl.to))).number;
      const marker = new Tint(hl.color);
      const builder = new RangeSetBuilder<GutterMarker>();
      for (let n = first; n <= last; n++) {
        const at = doc.line(n).from;
        builder.add(at, at, marker);
      }
      return builder.finish();
    }),
});

/** Set (or clear, with null) the tinted line range on one editor. */
export function highlightLines(view: EditorView, hl: LineHighlight | null): void {
  // A pane can close mid-hover; a dispatch at a destroyed view throws.
  if (!view.dom.isConnected) return;
  view.dispatch({ effects: setLineHighlight.of(hl) });
  window.dispatchEvent(new Event(RAIL_SYNC_EVENT));
}
