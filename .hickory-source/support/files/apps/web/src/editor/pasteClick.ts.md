<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/pasteClick.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cd3ecfee9d1430b3f85a2a14cb0b771c618cf6410bef39165e0cae65899068fe`.

<book:entry path="apps/web/src/editor/pasteClick.ts">
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

</book:entry>

</book:doc>
