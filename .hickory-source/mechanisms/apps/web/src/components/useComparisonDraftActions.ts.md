<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/useComparisonDraftActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9e06ed4aba7a"></a>
## `apps/web/src/components/useComparisonDraftActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9e06ed4aba7a-useComparisonDraftActions"></a>
### `useComparisonDraftActions`

<storycode:copy id="f-9e06ed4aba7a-useComparisonDraftActions">
import type { EditorView } from "@codemirror/view";
import { useEditorEditing } from "../editor/useEditorEditing";

/** Comparison edits stay drafts and invalidate evidence through the existing listener. */
export function useComparisonDraftActions(view: EditorView | null, editable: boolean, source: string) {
  useEditorEditing(view, { source, label: "comparison draft", current: () => editable, identity: () => source, localOnly: true });
}
</storycode:copy>

### Assemble `apps/web/src/components/useComparisonDraftActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/useComparisonDraftActions.ts"><storycode:paste select="#f-9e06ed4aba7a-useComparisonDraftActions"/></storycode:file>

</storycode:doc>
