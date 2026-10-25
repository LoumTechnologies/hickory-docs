<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lineage/useLineageControls.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Captured reading surfaces share owned native actions](../../../../../contract.md)

Captured SHA-256: `3259b8ff1f884efab90eb7ca34266faf0f765fb3aff4d47539f7373c1a9beb4a`.

<book:entry path="apps/web/src/lineage/useLineageControls.ts">
import { useId, useMemo } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiValue } from "../lib/uiActions";

type Control = { key: string; label: string; enabled?: boolean; run: (args: Record<string, UiValue>) => void;
  parameters?: import("../lib/uiActions").UiCommand["parameters"] };

/** Only explicit controls and currently displayed bytes belong in this adapter. */
export function useLineageControls(label: string, reading: unknown, controls: Control[]) {
  const parent = `lineage:${useId()}`;
  const callbacks = useMemo(() => crypto.randomUUID(), [controls]);
  const revision = useUiRevision(() => [reading, callbacks, controls.map(control => [control.key, control.enabled])]);
  useUiReading(label, reading);
  useUiCommands(controls.map(control => ({ key: `${parent}:${control.key}`, command: {
    label: control.label, parent, subject: parent, parameters: control.parameters ?? {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: control.enabled !== false }),
    execute: async args => { control.run(args); return { status: "completed", value: { view_change_requested: true, source_changed: false, execution_requested: false } }; },
  } })));
}
</book:entry>

</book:doc>
