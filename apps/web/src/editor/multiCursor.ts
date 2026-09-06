// Several cursors, one keystroke.
//
// CodeMirror can hold any number of selections but does not by default: the
// facet is off, and the browser's own selection can draw only one. So the
// three parts arrive together — allow many, draw them ourselves, and let a
// dragged rectangle become a column of cursors — because any one without the
// others is a feature that looks broken.
//
// The gestures are the ones every editor shares: Alt+click adds a cursor,
// Alt+drag selects a column, Ctrl+D (Cmd+D) selects the next occurrence of
// the selection and Ctrl+Shift+L every occurrence — those two come from
// `@codemirror/search`'s keymap, which every editor here already wears.

import { selectNextOccurrence, selectSelectionMatches } from "@codemirror/search";
import { EditorState, Prec } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { crosshairCursor, drawSelection, keymap, rectangularSelection } from "@codemirror/view";

import { cmKeyOf } from "../lib/keymap";

export function multipleCursors(): Extension[] {
  // The two occurrence commands, on whatever the keymap says (VS Code's
  // Ctrl+D and Ctrl+Shift+L by default) — above the search keymap's own
  // bindings, so a profile that moves them wins.
  const next = cmKeyOf("editor.addNextOccurrence");
  const all = cmKeyOf("editor.selectAllOccurrences");
  return [
    EditorState.allowMultipleSelections.of(true),
    drawSelection(),
    rectangularSelection(),
    crosshairCursor(),
    Prec.high(
      keymap.of([
        ...(next ? [{ key: next, run: selectNextOccurrence }] : []),
        ...(all ? [{ key: all, run: selectSelectionMatches }] : []),
      ]),
    ),
  ];
}
