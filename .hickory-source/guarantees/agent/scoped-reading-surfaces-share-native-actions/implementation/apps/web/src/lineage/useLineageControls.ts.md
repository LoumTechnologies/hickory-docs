<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lineage/useLineageControls.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9bd624eaf8c9"></a>
## `apps/web/src/lineage/useLineageControls.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../contract.md).

<a id="f-9bd624eaf8c9-Control"></a>
### `Control`

<storycode:copy id="f-9bd624eaf8c9-Control">
import { useId, useMemo } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiValue } from "../lib/uiActions";

type Control = { key: string; label: string; enabled?: boolean; run: (args: Record<string, UiValue>) => void;
  parameters?: import("../lib/uiActions").UiCommand["parameters"] };

</storycode:copy>
<a id="f-9bd624eaf8c9-useLineageControls"></a>
### `useLineageControls`

Only explicit controls and currently displayed bytes belong in this adapter.

<storycode:copy id="f-9bd624eaf8c9-useLineageControls">
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
</storycode:copy>

### Assemble `apps/web/src/lineage/useLineageControls.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/useLineageControls.ts"><storycode:paste select="#f-9bd624eaf8c9-Control"/><storycode:paste select="#f-9bd624eaf8c9-useLineageControls"/></storycode:file>

</storycode:doc>
