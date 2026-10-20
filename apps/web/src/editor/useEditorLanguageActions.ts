import { useEffect, useId, useRef, useState } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { useUiCommands } from "../lib/uiActionBridge";
import { editorLanguageActions, type EditorLanguageAction } from "./languageActions";
const states = new WeakMap<object, string>();
function identity(state: object) { let id = states.get(state); if (!id) { id = crypto.randomUUID(); states.set(state, id); } return id; }

/** Reconfiguration can install/remove providers without recreating the editor. */
export function useEditorLanguageActions(view: EditorView | null, subject: string) {
  const parent = `editor-language:${subject}:${useId()}`;
  const jobs = useRef(new Set<{ action: EditorLanguageAction; controller: AbortController }>());
  const [actions, setActions] = useState<readonly EditorLanguageAction[]>([]);
  useEffect(() => {
    if (!view) { setActions([]); return; }
    const compartment = new Compartment();
    const refresh = () => {
      const installed = view.state.facet(editorLanguageActions);
      for (const job of jobs.current) if (!installed.includes(job.action)) job.controller.abort();
      setActions(installed);
    };
    view.dispatch({ effects: StateEffect.appendConfig.of(compartment.of(EditorView.updateListener.of(refresh))) });
    refresh();
    return () => { for (const job of jobs.current) job.controller.abort(); if (view.dom.isConnected) view.dispatch({ effects: compartment.reconfigure([]) }); };
  }, [view]);
  useUiCommands(view ? actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters,
    prerequisites: ["Current editor source, selection and installed language provider; mapped-source positions only"],
    read: () => ({ revision: `${identity(view.state)}:${JSON.stringify(action.revision?.(view))}`, enabled: view.dom.isConnected && view.state.facet(editorLanguageActions).includes(action) && action.enabled(view), reason: "Language provider must be installed and the selected position mapped to current source" }),
    execute: async (args, _actor, signal) => {
      if (!view.dom.isConnected || !view.state.facet(editorLanguageActions).includes(action) || !action.enabled(view)) return { status: "unavailable", reason: "Language action or source position is no longer available" };
      const controller = new AbortController();
      const abort = () => controller.abort();
      signal.addEventListener("abort", abort, { once: true });
      if (signal.aborted) controller.abort();
      const job = { action, controller }; jobs.current.add(job);
      try { return await action.execute(view, args, controller.signal); }
      finally { jobs.current.delete(job); signal.removeEventListener("abort", abort); }
    },
  } })) : []);
}
