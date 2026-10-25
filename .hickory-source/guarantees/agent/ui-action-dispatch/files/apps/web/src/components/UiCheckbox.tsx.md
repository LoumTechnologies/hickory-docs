<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/UiCheckbox.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `f298bfb97c7bd881a342d73685caa4f1ae1e1e73bfe7f792ae0d7a7365473d73`.

<book:entry path="apps/web/src/components/UiCheckbox.tsx">
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
</book:entry>

</book:doc>
