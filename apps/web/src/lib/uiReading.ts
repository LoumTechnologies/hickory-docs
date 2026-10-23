import { useId } from "react";
import { useUiCommands, useUiRevision } from "./uiActionBridge";

/** Read only controller-owned rendered data; never scrape or execute arbitrary DOM. */
export function useUiReading(label: string, value: unknown, scope?: unknown, disclosureParent?: string) {
  const subject = `reading:${useId()}`;
  const revision = useUiRevision(() => [scope, value]);
  useUiCommands([{ key: subject, command: {
    label: `Read ${label} bytes`, parent: disclosureParent ?? subject, subject,
    parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify(value));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside this current reading" };
      return { status: "completed", value: { snapshot: revision(), bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null, edit_baseline: false } };
    },
  } }]);
}
