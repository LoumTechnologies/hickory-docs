<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/focusWhenShown.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The App Starts As A Lightweight Editor](../../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md)
- [The Editor Does Not Wait For Workspace Preparation](../../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md)

Captured SHA-256: `659513269b4e544d06b9557e80b8241bb0686901fd8ae25ad528a8a7119a046a`.

<book:entry path="apps/web/src/lib/focusWhenShown.ts">
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
</book:entry>

</book:doc>
