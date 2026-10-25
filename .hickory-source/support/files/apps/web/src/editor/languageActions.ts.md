<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/languageActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c9f5cd696aeb3aa0d092fb010285c4b9f10c1b6c1e9b02bc71178130ed91068b`.

<book:entry path="apps/web/src/editor/languageActions.ts">
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
</book:entry>

</book:doc>
