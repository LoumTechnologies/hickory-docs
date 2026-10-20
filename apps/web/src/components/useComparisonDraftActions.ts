import type { EditorView } from "@codemirror/view";
import { useEditorEditing } from "../editor/useEditorEditing";

/** Comparison edits stay drafts and invalidate evidence through the existing listener. */
export function useComparisonDraftActions(view: EditorView | null, editable: boolean, source: string) {
  useEditorEditing(view, { source, label: "comparison draft", current: () => editable, identity: () => source, localOnly: true });
}
