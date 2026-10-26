<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/UiCheckbox.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-01e1dcea4a76"></a>
## `apps/web/src/components/UiCheckbox.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-01e1dcea4a76-UiCheckbox"></a>
### `UiCheckbox`

<storycode:copy id="f-01e1dcea4a76-UiCheckbox">
import { useId, useMemo, useRef, useState, type InputHTMLAttributes } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";

/** Opt-in controlled settings input; both adapters call the same owned callback. */
export function UiCheckbox({ label, checked, onCheckedChange, owner, ...input }: Omit<InputHTMLAttributes<HTMLInputElement>, "type" | "checked" | "onChange" | "defaultChecked"> & {
  owner?: string; label: string; checked: boolean; onCheckedChange(value: boolean): unknown;
}) {
  const instance = useId();
  const identity = useMemo(() => crypto.randomUUID(), [owner, label, input.id, input.name, onCheckedChange]);
  const pending = useRef<string | null>(null);
  const [pendingIdentity, setPendingIdentity] = useState<string | null>(null);
  const [failure, setFailure] = useState<{ identity: string; message: string } | null>(null);
  const busy = pendingIdentity !== null;
  const revision = useUiRevision(() => [identity, checked, !!input.disabled, pending.current !== null]);
  const change = async (next: boolean): Promise<UiOutcome> => {
    if (input.disabled || pending.current !== null) return { status: "unavailable", reason: "This setting is disabled or a change callback is pending" };
    pending.current = identity; setPendingIdentity(identity); setFailure(null);
    try {
      await onCheckedChange(next);
      return { status: "completed", value: { checked_requested: next, callback_completed: true, persisted: false, scope: "Observe the owning form's actual value and error; callback completion is not persistence" } };
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setFailure({ identity, message }); return { status: "unavailable", reason: message };
    } finally {
      if (pending.current === identity) { pending.current = null; setPendingIdentity(null); }
    }
  };
  useUiCommands([{ key: `checkbox:${instance}`, command: {
    label: `Set ${label}`, parent: owner ? `${owner}/settings` : "settings-toggles", subject: `checkbox:${instance}`, parameters: { checked: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !input.disabled && pending.current === null, value: checked,
      reason: input.disabled ? "This setting is disabled in the current form" : pending.current !== null ? "A setting change callback is pending" : undefined }),
    execute: async args => change(args.checked as boolean),
  } }]);
  return <><input {...input} type="checkbox" checked={checked} disabled={input.disabled || busy} onChange={event => { void change(event.target.checked); }} />
    {failure?.identity === identity && <span role="alert">{failure.message}</span>}</>;
}
</storycode:copy>

### Assemble `apps/web/src/components/UiCheckbox.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/UiCheckbox.tsx"><storycode:paste select="#f-01e1dcea4a76-UiCheckbox"/></storycode:file>

</storycode:doc>
