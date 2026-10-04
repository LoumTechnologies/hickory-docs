import { Annotation, StateEffect, StateField } from "@codemirror/state";
import { EditorState } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";

/** Only the owner of a live reading can replace its protected prefix. */
export const replaceReading = Annotation.define<boolean>();
export const setResponseStart = StateEffect.define<number>();
export const responseStart = StateField.define<number>({
  create: () => 0,
  update: (value, tr) => {
    for (const effect of tr.effects) if (effect.is(setResponseStart)) return effect.value;
    return value;
  },
});
export const protectedPrefix = [responseStart,
  EditorState.transactionFilter.of(tr => {
    if (!tr.docChanged || tr.annotation(replaceReading)) return tr;
    let protectedChange = false;
    tr.changes.iterChangedRanges(from => { if (from < tr.startState.field(responseStart)) protectedChange = true; });
    return protectedChange ? [] : tr;
  }),
  EditorView.decorations.compute([responseStart], state => {
    const end = state.field(responseStart);
    return end ? Decoration.set([Decoration.mark({ class: "conversation-protected", attributes: { "aria-readonly": "true" } }).range(0, end)]) : Decoration.none;
  }),
];
