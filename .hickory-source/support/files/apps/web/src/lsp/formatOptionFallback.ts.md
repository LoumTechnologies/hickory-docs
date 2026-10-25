<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lsp/formatOptionFallback.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6f0dc25a0e96bdb45d83019de3e54360424c9283b1bbc4c864d563f4f9134b19`.

<book:entry path="apps/web/src/lsp/formatOptionFallback.ts">
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
</book:entry>

</book:doc>
