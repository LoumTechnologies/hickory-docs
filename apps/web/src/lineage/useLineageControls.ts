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
