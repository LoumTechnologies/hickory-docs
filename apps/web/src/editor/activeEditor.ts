// Which buffer an insert lands in.
//
// The workspace holds many editors at once — one per open document tab, plus
// the untitled buffer, plus every generated-file pane — and the Insert panel
// takes the focus away from all of them the moment it opens. So "the editor
// with the focus" is the wrong question by the time anyone asks it; the right
// one is "the editor that had it last", which is what this remembers.
//
// Only document editors register (editor/DocumentEditor.tsx). A generated
// file is written BY a document, so writing a hick element into one would put
// the tag in the woven output where it means nothing — the pane you were last
// typing hick in is the only honest target.

import type { EditorView } from "@codemirror/view";

let current: EditorView | null = null;

/** Remember this editor as the one an insert should target. */
export function markActiveEditor(view: EditorView): void {
  current = view;
}

/**
 * Forget an editor — on unmount, or when its document tab goes away.
 *
 * Only clears when it is still the active one: a tab closing behind the
 * pane you just focused must not steal the target back.
 */
export function forgetEditor(view: EditorView | null): void {
  if (view && current === view) current = null;
}

/**
 * The editor an insert targets: whichever registered last and is still
 * attached to the page.
 *
 * The liveness check matters because a destroyed CodeMirror view still
 * answers `dispatch` — it just throws the edit away — so an insert into a
 * closed tab would silently do nothing at all.
 */
export function activeEditor(): EditorView | null {
  if (current && !current.dom.isConnected) current = null;
  return current;
}

// ---------------------------------------------------------------------------
// The broader one: whichever editor was last focused, of any kind.
// ---------------------------------------------------------------------------
//
// Kept separate from `current` above rather than folded into it, because the
// two answer different questions and the distinction is load-bearing. An
// INSERT must land in a document — writing a hick element into a generated
// file would put the tag in the woven output, where it means nothing — so
// only document editors register there. PRINTING has no such rule: a plain
// file is exactly as printable as a document, and the reader means the pane
// they are looking at.

let focused: EditorView | null = null;
let untitled: EditorView | null = null;

/** Remember this editor as the last one the reader was in. Every editing
 * pane registers here, unlike `markActiveEditor`. */
export function markFocusedEditor(view: EditorView): void {
  focused = view;
}

export function forgetFocusedEditor(view: EditorView | null): void {
  if (view && focused === view) focused = null;
}

/** The editor the reader was last in, whatever kind of file it holds. */
export function focusedEditor(): EditorView | null {
  if (focused && !focused.dom.isConnected) focused = null;
  return focused;
}

/** The one unnamed note's editor. Its first Save must not depend on focus. */
export function markUntitledEditor(view: EditorView | null): void {
  untitled = view;
}

export function untitledEditor(): EditorView | null {
  if (untitled && !untitled.dom.isConnected) untitled = null;
  return untitled;
}

/** Test seam: drop whatever is remembered. */
export function resetActiveEditor(): void {
  current = null;
  focused = null;
  untitled = null;
}
