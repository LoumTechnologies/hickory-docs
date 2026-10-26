<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/formatOptionFallback.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-150129054681"></a>
## `apps/web/src/lsp/formatOptionFallback.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-150129054681-formatOptionFallback"></a>
### `formatOptionFallback`

<storycode:copy id="f-150129054681-formatOptionFallback">
import type { EditorView, KeyBinding } from "@codemirror/view";
import { isMac, parseBinding, strokeMatches } from "../lib/keymap";

/** CodeMirror preserves macOS Option characters instead of using their base
 * key. Honor an explicitly configured formatter letter when its normal
 * keybinding did not handle the event. Unbound Option text remains text. */
export function formatOptionFallback(binding: string, run: (view: EditorView) => boolean): KeyBinding[] {
  const strokes = parseBinding(binding);
  const stroke = strokes?.length === 1 ? strokes[0] : null;
  if (!isMac() || !stroke?.alt || stroke.mod || stroke.ctrl || !/^[a-z]$/.test(stroke.key)) return [];
  return [{ any: (view, event) => {
    if (event.isComposing || event.getModifierState("AltGraph") || !/^Key[A-Z]$/.test(event.code)) return false;
    const key = event.code.slice(3).toLowerCase();
    return strokeMatches({ key, ctrlKey: event.ctrlKey, metaKey: event.metaKey, altKey: event.altKey, shiftKey: event.shiftKey }, stroke, true) && run(view);
  } }];
}
</storycode:copy>

### Assemble `apps/web/src/lsp/formatOptionFallback.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/formatOptionFallback.ts"><storycode:paste select="#f-150129054681-formatOptionFallback"/></storycode:file>

</storycode:doc>
