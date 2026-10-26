<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/gutterActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d8b909a5a3f8"></a>
## `apps/web/src/editor/gutterActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d8b909a5a3f8-accessibleGutterActions"></a>
### `accessibleGutterActions`

<storycode:copy id="f-d8b909a5a3f8-accessibleGutterActions">
import { ViewPlugin } from "@codemirror/view";

/** CodeMirror hides decorative gutters from accessibility; ours contain actions. */
export const accessibleGutterActions = ViewPlugin.define(view => {
  const expose = () => view.dom.querySelector(".cm-gutters")?.removeAttribute("aria-hidden");
  view.requestMeasure({ read: () => null, write: expose });
  return { update: expose };
});
</storycode:copy>

### Assemble `apps/web/src/editor/gutterActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/gutterActions.ts"><storycode:paste select="#f-d8b909a5a3f8-accessibleGutterActions"/></storycode:file>

</storycode:doc>
