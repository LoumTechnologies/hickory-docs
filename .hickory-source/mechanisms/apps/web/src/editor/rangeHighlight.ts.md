<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/rangeHighlight.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7a905e7442f4"></a>
## `apps/web/src/editor/rangeHighlight.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7a905e7442f4-setRangeHighlights"></a>
### `setRangeHighlights`

Shared ribbon-hover highlight: a StateField of background-only MARK
decorations (never widgets/line styles — marks cannot change vertical
layout, and being a StateField they'd be safe even if they could). Used by
both panes of the Split view; the Document editor gets it appended via
StateEffect.appendConfig once its view exists.

<storycode:copy id="f-7a905e7442f4-setRangeHighlights">
// Shared ribbon-hover highlight: a StateField of background-only MARK
// decorations (never widgets/line styles — marks cannot change vertical
// layout, and being a StateField they'd be safe even if they could). Used by
// both panes of the Split view; the Document editor gets it appended via
// StateEffect.appendConfig once its view exists.

import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

export const setRangeHighlights = StateEffect.define<{ from: number; to: number }[]>();

</storycode:copy>
<a id="f-7a905e7442f4-hlMark"></a>
### `hlMark`

<storycode:copy id="f-7a905e7442f4-hlMark">
const hlMark = Decoration.mark({ class: "cm-ribbon-hl" });

</storycode:copy>
<a id="f-7a905e7442f4-rangeHighlightField"></a>
### `rangeHighlightField`

<storycode:copy id="f-7a905e7442f4-rangeHighlightField">
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
</storycode:copy>

### Assemble `apps/web/src/editor/rangeHighlight.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/rangeHighlight.ts"><storycode:paste select="#f-7a905e7442f4-setRangeHighlights"/><storycode:paste select="#f-7a905e7442f4-hlMark"/><storycode:paste select="#f-7a905e7442f4-rangeHighlightField"/></storycode:file>

</storycode:doc>
