import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";
export function useHistoryActions(subject: string, current: () => unknown, actions: {
  id: string; label: string; enabled: () => boolean; execute: UiCommand["execute"]; parameters?: UiCommand["parameters"];
}[], onError?: (reason: string) => void) {
  const parent = `history:${subject}:${useId()}`;
  const revision = useUiRevision(current);
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters ?? {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: action.enabled(), reason: action.enabled() ? undefined : "Wait for Git, resolve the working tree or return to an editable draft" }),
    execute: action.execute,
  } })));
  return (id: string, args = {}) => void invoke(`${parent}:${id}`, args).catch(error => onError?.(error instanceof Error ? error.message : String(error)));
}
