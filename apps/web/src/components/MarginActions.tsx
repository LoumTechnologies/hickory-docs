import { useId, useState, type ReactNode } from "react";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import "./MarginActions.css";

/** Editor chrome occupies the right margin; opening it never adds text rows. */
export function MarginActions({ label, glyph, children, className = "" }: {
  label: string; glyph: string; children: ReactNode; className?: string;
}) {
  const [open, setOpen] = useState(false);
  const subject = `margin-actions:${useId()}`;
  const revision = useUiRevision(() => [label, open]);
  const invoke = useUiCommands([{ key: subject, command: { parent: subject, subject, label: `Set ${label} visibility`, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { open } }), execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: args.open }; } } }]);
  return <details open={open} className={`margin-actions ${className}`} onKeyDown={event => {
    if (event.key === "Escape") {
      void invoke(subject, { open: false });
      event.currentTarget.querySelector("summary")?.focus();
      event.stopPropagation();
    }
  }}>
    <summary aria-label={label} data-tip={label} onClick={event => { event.preventDefault(); void invoke(subject, { open: !open }); }}>{glyph}</summary>
    <div className="margin-actions__panel" aria-label={label}><UiActionsVisibility visible={open}>{children}</UiActionsVisibility></div>
  </details>;
}
