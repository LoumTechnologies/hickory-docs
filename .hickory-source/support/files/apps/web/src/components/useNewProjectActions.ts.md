<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/useNewProjectActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3368c0bd59a6343eaaa5fe3a008b6a6eeaeac1546fc997779967148c2e770eab`.

<book:entry path="apps/web/src/components/useNewProjectActions.ts">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";

export type ProjectField = { id: string; label: string; schema: UiField; value: UiValue;
  enabled?: boolean; set: (value: UiValue) => void };
export function useNewProjectActions(state: unknown, current: () => boolean, fields: ProjectField[], actions: {
  id: string; label: string; enabled: () => boolean; reason?: () => string; execute: UiCommand["execute"];
}[]) {
  const parent = `new-project:${useId()}`;
  const revision = useUiRevision(() => [state, fields.map(field => [field.id, field.value, field.enabled]), current()]);
  useUiReading("new project form and preview", state);
  const commands = fields.map(field => ({ key: `${parent}:${field.id}`, command: {
    parent, subject: parent, label: field.label, parameters: { value: field.schema }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: current() && field.enabled !== false, reason: !current() ? "Wait for the pending form request" : field.enabled === false ? "This field is disabled by the current project form state" : undefined, value: { current: field.value } }),
    execute: async (args: Record<string, UiValue>) => {
      if (!current() || field.enabled === false) return { status: "unavailable" as const, reason: "Project form changed or is busy" };
      field.set(args.value); return { status: "completed" as const, value: { draft_updated: true, scaffold_started: false } };
    },
  } }));
  const invoke = useUiCommands([...commands, ...actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: current() && action.enabled(), reason: !current() ? "Wait for the pending form request" : !action.enabled() ? action.reason?.() ?? "Wait for the current template options" : undefined }), execute: action.execute,
  } }))]);
  return (id: string) => void invoke(`${parent}:${id}`, {}).catch(() => {});
}
</book:entry>

</book:doc>
