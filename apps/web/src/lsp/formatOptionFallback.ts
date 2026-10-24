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
