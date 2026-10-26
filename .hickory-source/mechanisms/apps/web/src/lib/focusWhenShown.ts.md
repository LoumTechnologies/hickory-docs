<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/focusWhenShown.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e9524c90facf"></a>
## `apps/web/src/lib/focusWhenShown.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The App Starts As A Lightweight Editor](../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [The Editor Does Not Wait For Workspace Preparation](../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md).

<a id="f-e9524c90facf-focusWhenShown"></a>
### `focusWhenShown`

<storycode:copy id="f-e9524c90facf-focusWhenShown">
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
</storycode:copy>

### Assemble `apps/web/src/lib/focusWhenShown.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/focusWhenShown.ts"><storycode:paste select="#f-e9524c90facf-focusWhenShown"/></storycode:file>

</storycode:doc>
