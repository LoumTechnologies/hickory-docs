<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/SurfaceInfo.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6c34dd749b33"></a>
## `apps/web/src/components/SurfaceInfo.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md).

<a id="f-6c34dd749b33-SurfaceInfo"></a>
### `SurfaceInfo`

<storycode:copy id="f-6c34dd749b33-SurfaceInfo">
import { useId, useState } from "react";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import "./SurfaceInfo.css";

/** The same three questions for every working surface: identity, editability,
 * and where a write lands. Presentation never grants a capability. */
export function SurfaceInfo({ name, editable, destination }: {
  name: string; editable: boolean; destination: string;
}) {
  const [open, setOpen] = useState(false);
  const parent = `surface-info:${useId()}`;
  const revision = useUiRevision(() => [name, editable, destination, open]);
  useUiReading("surface identity", open ? { name, editable, destination, open } : { name, editable, open });
  useUiCommands([{ key: parent, command: {
    label: `Set information visibility for ${name}`, parent, subject: parent,
    parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { open } }),
    execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: { open: args.open } }; },
  } }]);
  return <PaneHeaderContent part="info"><details className="surface-info" aria-label="About this view" open={open} onToggle={event => setOpen(event.currentTarget.open)}>
    <summary role="button" aria-label={`About ${name} · ${editable ? "Editable" : "Read-only"}`} data-tip={`${name} · ${editable ? "Editable" : "Read-only"}. ${destination}`}>
      <strong>{name}</strong><span>{editable ? "Editable" : "Read-only"}</span>
    </summary>
    <UiActionsVisibility visible={open}><p><strong>{name} · {editable ? "Editable" : "Read-only"}</strong><br />{destination}</p></UiActionsVisibility>
  </details></PaneHeaderContent>;
}
</storycode:copy>

### Assemble `apps/web/src/components/SurfaceInfo.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/SurfaceInfo.tsx"><storycode:paste select="#f-6c34dd749b33-SurfaceInfo"/></storycode:file>

</storycode:doc>
