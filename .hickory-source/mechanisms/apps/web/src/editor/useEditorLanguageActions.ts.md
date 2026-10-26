<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/useEditorLanguageActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d7896e614a46"></a>
## `apps/web/src/editor/useEditorLanguageActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d7896e614a46-states"></a>
### `states`

<storycode:copy id="f-d7896e614a46-states">
import { useEffect, useId, useRef, useState } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { useUiCommands } from "../lib/uiActionBridge";
import { editorLanguageActions, type EditorLanguageAction } from "./languageActions";
const states = new WeakMap<object, string>();
</storycode:copy>
<a id="f-d7896e614a46-identity"></a>
### `identity`

<storycode:copy id="f-d7896e614a46-identity">
function identity(state: object) { let id = states.get(state); if (!id) { id = crypto.randomUUID(); states.set(state, id); } return id; }

</storycode:copy>
<a id="f-d7896e614a46-useEditorLanguageActions"></a>
### `useEditorLanguageActions`

Reconfiguration can install/remove providers without recreating the editor.

<storycode:copy id="f-d7896e614a46-useEditorLanguageActions">
/** Reconfiguration can install/remove providers without recreating the editor. */
export function useEditorLanguageActions(view: EditorView | null, subject: string) {
  const parent = `editor-language:${subject}:${useId()}`;
  const jobs = useRef(new Set<{ action: EditorLanguageAction; controller: AbortController }>());
  const revisions = useRef(new Map<EditorLanguageAction, { signature: string; token: string }>());
  const [actions, setActions] = useState<readonly EditorLanguageAction[]>([]);
  useEffect(() => {
    if (!view) { setActions([]); return; }
    const compartment = new Compartment();
    const refresh = () => {
      const installed = view.state.facet(editorLanguageActions).filter(action => action.visible?.(view) !== false);
      for (const job of jobs.current) if (!installed.includes(job.action)) job.controller.abort();
      setActions(previous => previous.length === installed.length && previous.every((action, index) => action === installed[index]) ? previous : installed);
      for (const action of revisions.current.keys()) if (!installed.includes(action)) revisions.current.delete(action);
    };
    view.dispatch({ effects: StateEffect.appendConfig.of(compartment.of(EditorView.updateListener.of(refresh))) });
    refresh();
    return () => { revisions.current.clear(); for (const job of jobs.current) job.controller.abort(); if (view.dom.isConnected) view.dispatch({ effects: compartment.reconfigure([]) }); };
  }, [view]);
  const current = (action: EditorLanguageAction) => !!view?.dom.isConnected && view.state.facet(editorLanguageActions).includes(action) && action.visible?.(view) !== false;
  const revision = (action: EditorLanguageAction) => {
    if (!view || !current(action)) return `retired:${view ? identity(view.state) : "closed"}`;
    const signature = JSON.stringify([identity(view.state), action.revision?.(view)]);
    let retained = revisions.current.get(action);
    if (!retained || retained.signature !== signature) { retained = { signature, token: crypto.randomUUID() }; revisions.current.set(action, retained); }
    return retained.token;
  };
  useUiCommands(view ? actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: action.subject ? `${parent}:${action.subject}` : parent, label: action.label, parameters: action.parameters,
    prerequisites: ["Current editor source, selection and installed language provider; mapped-source positions only"],
    read: () => ({ revision: revision(action), value: current(action) ? action.value?.(view) : undefined, enabled: current(action) && action.enabled(view), reason: "Language provider must be installed and the selected position mapped to current source" }),
    execute: async (args, actor, signal) => {
      if (!current(action) || !action.enabled(view)) return { status: "unavailable", reason: "Language action or source position is no longer available" };
      const controller = new AbortController();
      const abort = () => controller.abort();
      signal.addEventListener("abort", abort, { once: true });
      if (signal.aborted) controller.abort();
      const job = { action, controller }; jobs.current.add(job);
      try { return await action.execute(view, args, controller.signal, actor); }
      finally { jobs.current.delete(job); signal.removeEventListener("abort", abort); }
    },
  } })) : []);
}
</storycode:copy>

### Assemble `apps/web/src/editor/useEditorLanguageActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/useEditorLanguageActions.ts"><storycode:paste select="#f-d7896e614a46-states"/><storycode:paste select="#f-d7896e614a46-identity"/><storycode:paste select="#f-d7896e614a46-useEditorLanguageActions"/></storycode:file>

</storycode:doc>
