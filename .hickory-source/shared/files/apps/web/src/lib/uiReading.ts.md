<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/uiReading.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)

Captured SHA-256: `0106a04c688187a2f26b674fae8bbb4bf98922c0b30202e3d7ead40d74d50363`.

<book:entry path="apps/web/src/lib/uiReading.ts">
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
</book:entry>

</book:doc>
