<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/pasteClick.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b73cd44e0061"></a>
## `apps/web/src/editor/pasteClick.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b73cd44e0061-pasteTagAt"></a>
### `pasteTagAt`

<storycode:copy id="f-b73cd44e0061-pasteTagAt">
import type { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { resolvePasteTarget } from "./hickDoc";
import type { HickDocStructure } from "./hickDoc";

// ---------------------------------------------------------------------------
// Paste chips are clickable: clicking a `hick:paste` selects the source of
// the copy/cut fragment its selector refers to (selection only — the text is
// never touched).
// ---------------------------------------------------------------------------

/** The paste tag at `pos`, or null. Exported for the click handler + tests. */
export function pasteTagAt(structure: HickDocStructure, pos: number) {
  for (const t of structure.tags) {
    if (!t.closing && t.name === "paste" && pos >= t.from && pos < t.to)
      return t;
  }
  return null;
}

</storycode:copy>
<a id="f-b73cd44e0061-pasteClick"></a>
### `pasteClick`

<storycode:copy id="f-b73cd44e0061-pasteClick">
export function pasteClick(structureOf: (state: EditorState) => HickDocStructure) {
  return EditorView.domEventHandlers({
  mousedown(event, view) {
    const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
    if (pos === null) return false;
    const structure = structureOf(view.state);
    const tag = pasteTagAt(structure, pos);
    if (!tag) return false;
    const target = resolvePasteTarget(structure, tag.attrs.select);
    if (!target) return false;
    const max = view.state.doc.length;
    view.dispatch({
      selection: {
        anchor: Math.min(target.contentFrom, max),
        head: Math.min(target.contentTo, max),
      },
      scrollIntoView: true,
    });
    event.preventDefault();
    return true;
  },
  });
}

</storycode:copy>

### Assemble `apps/web/src/editor/pasteClick.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/pasteClick.ts"><storycode:paste select="#f-b73cd44e0061-pasteTagAt"/><storycode:paste select="#f-b73cd44e0061-pasteClick"/></storycode:file>

</storycode:doc>
