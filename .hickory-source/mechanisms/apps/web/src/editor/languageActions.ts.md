<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/languageActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e4c9855508f2"></a>
## `apps/web/src/editor/languageActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-e4c9855508f2-EditorLanguageAction"></a>
### `EditorLanguageAction`

<storycode:copy id="f-e4c9855508f2-EditorLanguageAction">
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
</storycode:copy>
<a id="f-e4c9855508f2-editorLanguageActions"></a>
### `editorLanguageActions`

<storycode:copy id="f-e4c9855508f2-editorLanguageActions">
export const editorLanguageActions = Facet.define<EditorLanguageAction>();
</storycode:copy>

### Assemble `apps/web/src/editor/languageActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/languageActions.ts"><storycode:paste select="#f-e4c9855508f2-EditorLanguageAction"/><storycode:paste select="#f-e4c9855508f2-editorLanguageActions"/></storycode:file>

</storycode:doc>
