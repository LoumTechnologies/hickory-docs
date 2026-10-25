<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/MoreActions.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../../agent/ui-action-dispatch/contract.md)
- [Actions belong to their subject](../../../../../contract.md)

Captured SHA-256: `e33460b8b8eae84aef875768133d618bcb370b5223a51ed1640801003f0854d5`.

<book:entry path="apps/web/src/components/MoreActions.tsx">
import { useId, useRef, useState, type ReactNode } from "react";
import { UiActionsVisibility, useUiCommands, useUiPendingHumanActions } from "../lib/uiActionBridge";

/** Occasional tab actions stay discoverable without occupying the toolbar. */
export function MoreActions({ children, label = "More actions" }: { children: ReactNode; label?: string }) {
  const id = useId();
  const details = useRef<HTMLDetailsElement>(null);
  const [open, setOpen] = useState(false);
  const [error, setError] = useState("");
  const pending = useUiPendingHumanActions();
  const beforeClick = useRef(new Set<string>());
  const epoch = useRef(0);
  const key = `menu:${id}/toggle`;
  const invoke = useUiCommands([{ key, command: {
    label, subject: `menu:${id}`, parent: "menus", parameters: {}, prerequisites: [],
    read: () => ({ revision: String(details.current?.open ?? false), enabled: !!details.current, value: details.current?.open ?? false }),
    execute: async () => {
      if (!details.current) return { status: "unavailable", reason: "Menu was closed" };
      const next = !details.current.open;
      epoch.current++;
      details.current.open = next; setOpen(next); setError("");
      return { status: "completed", value: { open: next, coverage: "Disclosed menu children use their own registered descriptors" } };
    },
  } }]);
  const close = () => { if (details.current) details.current.open = false; setOpen(false); };
  return <details ref={details} open={open} className="more-actions" onClickCapture={() => { beforeClick.current = new Set(pending().map(request => request.id)); }} onKeyDown={event => {
    if (event.key === "Escape") { close(); details.current?.querySelector("summary")?.focus(); event.stopPropagation(); }
  }}><summary aria-label={label} onClick={event => {
    event.preventDefault(); void invoke(key).catch(reason => setError(String(reason)));
  }}>⋯</summary>
    {error && <span role="alert">{error}</span>}
    <div className="more-actions__items" onClick={event => {
      if (!(event.target as HTMLElement).closest("button")) return;
      const generation = epoch.current;
      const requests = pending().filter(request => !beforeClick.current.has(request.id));
      if (!requests.length) close();
      else void Promise.allSettled(requests.map(request => request.result)).then(() => { if (epoch.current === generation) close(); });
    }}><UiActionsVisibility visible={open}>{children}</UiActionsVisibility></div>
  </details>;
}
</book:entry>

</book:doc>
