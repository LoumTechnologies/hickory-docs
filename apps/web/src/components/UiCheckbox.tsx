import { useId, type InputHTMLAttributes } from "react";
import { useUiCommands } from "../lib/uiActionBridge";

/** Opt-in controlled settings input; both adapters call onCheckedChange. */
export function UiCheckbox({ label, checked, onCheckedChange, owner, ...input }: Omit<InputHTMLAttributes<HTMLInputElement>, "type" | "checked" | "onChange" | "defaultChecked"> & {
  owner?: string; label: string; checked: boolean; onCheckedChange(value: boolean): void;
}) {
  const instance = useId();
  useUiCommands([{ key: `checkbox:${instance}`, command: {
    label: `Set ${label}`, parent: owner ? `${owner}/settings` : "settings-toggles", subject: `checkbox:${instance}`, parameters: { checked: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: JSON.stringify([checked, !!input.disabled]), enabled: !input.disabled, value: checked, reason: input.disabled ? "This setting is disabled in the current form" : undefined }),
    execute: async args => { onCheckedChange(Boolean(args.checked)); return { status: "completed", value: { dispatch: "change requested", checked: Boolean(args.checked), result: "Observe the form's current value and error before judging persistence" } }; },
  } }]);
  return <input {...input} type="checkbox" checked={checked} onChange={event => onCheckedChange(event.target.checked)} />;
}
