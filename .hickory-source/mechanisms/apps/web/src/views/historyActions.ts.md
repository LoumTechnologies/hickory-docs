<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/historyActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ebd1576a1f7f"></a>
## `apps/web/src/views/historyActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ebd1576a1f7f-useHistoryActions"></a>
### `useHistoryActions`

<storycode:copy id="f-ebd1576a1f7f-useHistoryActions">
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
</storycode:copy>

### Assemble `apps/web/src/views/historyActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/historyActions.ts"><storycode:paste select="#f-ebd1576a1f7f-useHistoryActions"/></storycode:file>

</storycode:doc>
