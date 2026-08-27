// Ctrl+← and Ctrl+→, under whichever definition of "word" the person chose.
//
// These are the two bindings `standardKeymap` already owns — same keys, same
// mac spelling (Alt+arrow there, because Cmd+arrow is line start), same
// `preventDefault`. They are copied rather than referred to so that the pair
// stays readable as a pair; if CodeMirror ever changes the chord, this file is
// where the divergence shows up.
//
// The choice is read AT KEYPRESS, not at mount. Every other display
// preference in this app is read when the workspace mounts and takes effect
// when it next does — which is fine for tab placement, and would be a
// surprise here: a person changes what an arrow key does precisely because
// the arrow key just did the wrong thing, and would press it again to check.
// One localStorage read per Ctrl+arrow is not a cost worth plumbing a
// compartment to avoid.
//
// There is no `cursorSubwordLeft`, only `cursorSubwordBackward`: CodeMirror's
// subword motion is logical rather than visual, so in a right-to-left run
// these keys move by document order where the whole-word motion moves by
// direction. Nothing in this product writes RTL code, and the honest note is
// cheaper than a wrapper that pretends otherwise.

import { keymap } from "@codemirror/view";
import type { Command, KeyBinding } from "@codemirror/view";
import {
  cursorGroupLeft,
  cursorGroupRight,
  cursorSubwordBackward,
  cursorSubwordForward,
  selectGroupLeft,
  selectGroupRight,
  selectSubwordBackward,
  selectSubwordForward,
} from "@codemirror/commands";
import type { Extension } from "@codemirror/state";

import { loadWordMotion } from "../lib/wordMotion";

/** The same gesture under both definitions, decided when it happens. */
const byMotion =
  (word: Command, subword: Command): Command =>
  (view) =>
    (loadWordMotion() === "subword" ? subword : word)(view);

/**
 * The two bindings, for anyone assembling a keymap by hand (the tests, and
 * the editors below).
 */
export const wordMotionBindings: readonly KeyBinding[] = [
  {
    key: "Mod-ArrowLeft",
    mac: "Alt-ArrowLeft",
    run: byMotion(cursorGroupLeft, cursorSubwordBackward),
    shift: byMotion(selectGroupLeft, selectSubwordBackward),
    preventDefault: true,
  },
  {
    key: "Mod-ArrowRight",
    mac: "Alt-ArrowRight",
    run: byMotion(cursorGroupRight, cursorSubwordForward),
    shift: byMotion(selectGroupRight, selectSubwordForward),
    preventDefault: true,
  },
];

/**
 * The extension every editor loads.
 *
 * Must come BEFORE `defaultKeymap` in the array: bindings for one key run in
 * registration order and the first to return true wins, so an override that
 * arrives second never runs at all.
 */
export function wordMotionKeymap(): Extension {
  return keymap.of([...wordMotionBindings]);
}
