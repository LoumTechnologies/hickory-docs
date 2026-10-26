<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/multiCursor.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-08df9c7d9da5"></a>
## `apps/web/src/editor/multiCursor.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Several Cursors Edit At Once](../../../../../guarantees/authoring/several-cursors-edit-at-once/contract.md), [The Workspace Tree Is An Editor Buffer](../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md), [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-08df9c7d9da5-multipleCursors"></a>
### `multipleCursors`

Several cursors, one keystroke.
CodeMirror can hold any number of selections but does not by default: the
facet is off, and the browser's own selection can draw only one. So the
three parts arrive together — allow many, draw them ourselves, and let a
dragged rectangle become a column of cursors — because any one without the
others is a feature that looks broken.
The gestures are the ones every editor shares: Alt+click adds a cursor,
Alt+drag selects a column, Ctrl+D (Cmd+D) selects the next occurrence of
the selection and Ctrl+Shift+L every occurrence — those two come from
`@codemirror/search`'s keymap, which every editor here already wears.

<storycode:copy id="f-08df9c7d9da5-multipleCursors">
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
</storycode:copy>

### Assemble `apps/web/src/editor/multiCursor.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/multiCursor.ts"><storycode:paste select="#f-08df9c7d9da5-multipleCursors"/></storycode:file>

</storycode:doc>
