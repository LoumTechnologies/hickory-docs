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
