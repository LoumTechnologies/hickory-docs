// Shared ribbon-hover highlight: a StateField of background-only MARK
// decorations (never widgets/line styles — marks cannot change vertical
// layout, and being a StateField they'd be safe even if they could). Used by
// both panes of the Split view; the Document editor gets it appended via
// StateEffect.appendConfig once its view exists.

import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

export const setRangeHighlights = StateEffect.define<{ from: number; to: number }[]>();

const hlMark = Decoration.mark({ class: "cm-ribbon-hl" });

export const rangeHighlightField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(setRangeHighlights)) {
        const builder = new RangeSetBuilder<Decoration>();
        const max = tr.state.doc.length;
        for (const r of [...e.value].sort((a, b) => a.from - b.from)) {
          const from = Math.max(0, Math.min(r.from, max));
          const to = Math.max(0, Math.min(r.to, max));
          if (to > from) builder.add(from, to, hlMark);
        }
        deco = builder.finish();
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});
