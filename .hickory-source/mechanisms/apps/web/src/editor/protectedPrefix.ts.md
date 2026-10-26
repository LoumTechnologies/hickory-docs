<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/protectedPrefix.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b8cdb5acce29"></a>
## `apps/web/src/editor/protectedPrefix.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b8cdb5acce29-replaceReading"></a>
### `replaceReading`

<storycode:copy id="f-b8cdb5acce29-replaceReading">
import { Annotation, StateEffect, StateField } from "@codemirror/state";
import { EditorState } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";

/** Only the owner of a live reading can replace its protected prefix. */
export const replaceReading = Annotation.define<boolean>();
</storycode:copy>
<a id="f-b8cdb5acce29-setResponseStart"></a>
### `setResponseStart`

<storycode:copy id="f-b8cdb5acce29-setResponseStart">
export const setResponseStart = StateEffect.define<number>();
</storycode:copy>
<a id="f-b8cdb5acce29-responseStart"></a>
### `responseStart`

<storycode:copy id="f-b8cdb5acce29-responseStart">
export const responseStart = StateField.define<number>({
  create: () => 0,
  update: (value, tr) => {
    for (const effect of tr.effects) if (effect.is(setResponseStart)) return effect.value;
    return value;
  },
});
</storycode:copy>
<a id="f-b8cdb5acce29-protectedPrefix"></a>
### `protectedPrefix`

<storycode:copy id="f-b8cdb5acce29-protectedPrefix">
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
</storycode:copy>

### Assemble `apps/web/src/editor/protectedPrefix.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/protectedPrefix.ts"><storycode:paste select="#f-b8cdb5acce29-replaceReading"/><storycode:paste select="#f-b8cdb5acce29-setResponseStart"/><storycode:paste select="#f-b8cdb5acce29-responseStart"/><storycode:paste select="#f-b8cdb5acce29-protectedPrefix"/></storycode:file>

</storycode:doc>
