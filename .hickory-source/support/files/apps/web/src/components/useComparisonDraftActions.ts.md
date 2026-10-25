<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/useComparisonDraftActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f6d6a886b8650c710704d39505f340f2a7308d07e9c5cf5d3abb659f2979a97e`.

<book:entry path="apps/web/src/components/useComparisonDraftActions.ts">
import type { EditorView } from "@codemirror/view";
import { useEditorEditing } from "../editor/useEditorEditing";

/** Comparison edits stay drafts and invalidate evidence through the existing listener. */
export function useComparisonDraftActions(view: EditorView | null, editable: boolean, source: string) {
  useEditorEditing(view, { source, label: "comparison draft", current: () => editable, identity: () => source, localOnly: true });
}
</book:entry>

</book:doc>
