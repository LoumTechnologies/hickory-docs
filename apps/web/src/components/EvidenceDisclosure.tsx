import { useId, useState, type ReactNode } from "react";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";

/** A normal details element whose private evidence is explicitly disclosed. */
export function EvidenceDisclosure({ label, value, children, className, onOpenChange }: { label: string; value: unknown; children: ReactNode; className?: string; onOpenChange?: (open: boolean) => void }) {
  const [open, setOpen] = useState(false);
  const change = (next: boolean) => { setOpen(next); onOpenChange?.(next); };
  const subject = `evidence:${useId()}`;
  const revision = useUiRevision(() => [label, value, open]);
  const commands: { key: string; command: UiCommand }[] = [{ key: `${subject}:open`, command: {
    label: `Set evidence visibility: ${label.slice(0, 256)}`, parent: subject, subject, parameters: { open: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true, value: { open } }),
    execute: async args => { change(args.open as boolean); return { status: "completed", value: args.open }; },
  } }];
  if (open) commands.push({ key: `${subject}:read`, command: {
    label: "Read disclosed evidence bytes", parent: subject, subject,
    parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify(value));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current disclosed evidence" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    },
  } });
  useUiCommands(commands);
  return <details className={className} open={open} onToggle={event => change(event.currentTarget.open)}><summary>{label}</summary><UiActionsVisibility visible={open}>{children}</UiActionsVisibility></details>;
}
