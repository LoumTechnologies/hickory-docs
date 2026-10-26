<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/lineHighlight.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b13cbbb02ce2"></a>
## `apps/web/src/editor/lineHighlight.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b13cbbb02ce2-LineHighlight"></a>
### `LineHighlight`

Line-number tinting for a hovered lineage connection, shared by BOTH rails
of a pane: CodeMirror's left gutter (through the lineNumberMarkers facet)
and the right rail (editor/RightRail.tsx reads the same field). One state,
two renderings — the tint can never disagree with itself across a pane.
The ribbon overlay dispatches `setLineHighlight` at the involved editors on
hover, with the ribbon's palette index; `null` clears it on leave.

<storycode:copy id="f-b13cbbb02ce2-LineHighlight">
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

</storycode:copy>
<a id="f-b13cbbb02ce2-setLineHighlight"></a>
### `setLineHighlight`

<storycode:copy id="f-b13cbbb02ce2-setLineHighlight">
export const setLineHighlight = StateEffect.define<LineHighlight | null>();

</storycode:copy>
<a id="f-b13cbbb02ce2-RAIL_SYNC_EVENT"></a>
### `RAIL_SYNC_EVENT`

Fired on window after a highlight dispatch: the right rails redraw on
scroll and resize, which a highlight change is neither, so it announces
itself.

<storycode:copy id="f-b13cbbb02ce2-RAIL_SYNC_EVENT">
/**
 * Fired on window after a highlight dispatch: the right rails redraw on
 * scroll and resize, which a highlight change is neither, so it announces
 * itself.
 */
export const RAIL_SYNC_EVENT = "hickory:rail-sync";

</storycode:copy>
<a id="f-b13cbbb02ce2-Tint"></a>
### `Tint`

<storycode:copy id="f-b13cbbb02ce2-Tint">
class Tint extends GutterMarker {
  constructor(color: number) {
    super();
    this.elementClass = `cm-linehl cm-linehl-c${color}`;
  }
}

</storycode:copy>
<a id="f-b13cbbb02ce2-lineHighlightField"></a>
### `lineHighlightField`

<storycode:copy id="f-b13cbbb02ce2-lineHighlightField">
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

</storycode:copy>
<a id="f-b13cbbb02ce2-highlightLines"></a>
### `highlightLines`

Set (or clear, with null) the tinted line range on one editor.

<storycode:copy id="f-b13cbbb02ce2-highlightLines">
/** Set (or clear, with null) the tinted line range on one editor. */
export function highlightLines(view: EditorView, hl: LineHighlight | null): void {
  // A pane can close mid-hover; a dispatch at a destroyed view throws.
  if (!view.dom.isConnected) return;
  view.dispatch({ effects: setLineHighlight.of(hl) });
  window.dispatchEvent(new Event(RAIL_SYNC_EVENT));
}
</storycode:copy>

### Assemble `apps/web/src/editor/lineHighlight.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/lineHighlight.ts"><storycode:paste select="#f-b13cbbb02ce2-LineHighlight"/><storycode:paste select="#f-b13cbbb02ce2-setLineHighlight"/><storycode:paste select="#f-b13cbbb02ce2-RAIL_SYNC_EVENT"/><storycode:paste select="#f-b13cbbb02ce2-Tint"/><storycode:paste select="#f-b13cbbb02ce2-lineHighlightField"/><storycode:paste select="#f-b13cbbb02ce2-highlightLines"/></storycode:file>

</storycode:doc>
