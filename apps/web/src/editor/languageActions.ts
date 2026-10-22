import { Facet } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import type { UiActor, UiField, UiOutcome, UiValue } from "../lib/uiActions";
/** Installed by the actual language extension, including mapped-source views. */
export type EditorLanguageAction = {
  id: string; label: string; subject?: string; parameters: Record<string, UiField>;
  visible?: (view: EditorView) => boolean;
  enabled: (view: EditorView) => boolean;
  revision?: (view: EditorView) => unknown;
  value?: (view: EditorView) => UiValue;
  execute: (view: EditorView, args: Record<string, UiValue>, signal: AbortSignal, actor?: UiActor) => Promise<UiOutcome>;
};
export const editorLanguageActions = Facet.define<EditorLanguageAction>();
