import { useId, useRef, useState, type ReactNode } from "react";
import { useUiCommands } from "../lib/uiActionBridge";

/** Occasional tab actions stay discoverable without occupying the toolbar. */
export function MoreActions({ children, label = "More actions" }: { children: ReactNode; label?: string }) {
  const id = useId();
  const details = useRef<HTMLDetailsElement>(null);
  const [error, setError] = useState("");
  const key = `menu:${id}/toggle`;
  const invoke = useUiCommands([{ key, command: {
    label, subject: `menu:${id}`, parent: "menus", parameters: {}, prerequisites: [],
    read: () => ({ revision: String(details.current?.open ?? false), enabled: !!details.current, value: details.current?.open ?? false }),
    execute: async () => {
      if (!details.current) return { status: "unavailable", reason: "Menu was closed" };
      details.current.open = !details.current.open;
      return { status: "completed", value: { open: details.current.open, coverage: "Menu children require their own registered descriptors" } };
    },
  } }]);
  return <details ref={details} className="more-actions"><summary aria-label={label} onClick={event => {
    event.preventDefault(); setError(""); void invoke(key).catch(reason => setError(String(reason)));
  }}>⋯</summary>
    {error && <span role="alert">{error}</span>}
    <div className="more-actions__items" onClick={event => {
      if ((event.target as HTMLElement).closest("button")) event.currentTarget.closest("details")?.removeAttribute("open");
    }}>{children}</div>
  </details>;
}
