import { Facet } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import type { UiField, UiOutcome, UiValue } from "../lib/uiActions";
/** Installed by the actual language extension, including mapped-source views. */
export type EditorLanguageAction = {
  id: string; label: string; parameters: Record<string, UiField>;
  enabled: (view: EditorView) => boolean;
  revision?: (view: EditorView) => unknown;
  value?: (view: EditorView) => UiValue;
  execute: (view: EditorView, args: Record<string, UiValue>, signal: AbortSignal) => Promise<UiOutcome>;
};
export const editorLanguageActions = Facet.define<EditorLanguageAction>();
