import type { EditorView } from "@codemirror/view";

/** A new editor may mount before the shell reveals its pane. Focus it once
 * it is shown, and stop observing when its owner disposes the view. */
export function focusWhenShown(view: EditorView): () => void {
  let observer: MutationObserver | null = null;
  const focus = () => {
    if (!view.dom.isConnected || view.dom.closest("[hidden]")) return;
    observer?.disconnect();
    view.focus();
  };
  const hidden = view.dom.closest("[hidden]");
  if (hidden) {
    observer = new MutationObserver(focus);
    observer.observe(hidden, { attributes: true, attributeFilter: ["hidden"] });
  } else focus();
  return () => observer?.disconnect();
}
