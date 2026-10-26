<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MarginActions.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c8c4e5622296"></a>
## `apps/web/src/components/MarginActions.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c8c4e5622296-MarginActions"></a>
### `MarginActions`

<storycode:copy id="f-c8c4e5622296-MarginActions">
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
</storycode:copy>

### Assemble `apps/web/src/components/MarginActions.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MarginActions.tsx"><storycode:paste select="#f-c8c4e5622296-MarginActions"/></storycode:file>

</storycode:doc>
