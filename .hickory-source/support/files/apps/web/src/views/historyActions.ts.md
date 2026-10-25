<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/historyActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `af5158a9a5d28613bd4500e47194ecb21ba168639561ccf1a90e8c4978ed6d04`.

<book:entry path="apps/web/src/views/historyActions.ts">
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
</book:entry>

</book:doc>
