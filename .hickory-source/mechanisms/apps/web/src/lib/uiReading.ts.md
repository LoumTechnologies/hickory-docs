<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/uiReading.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fb2fe531fbd6"></a>
## `apps/web/src/lib/uiReading.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-fb2fe531fbd6-useUiReading"></a>
### `useUiReading`

<storycode:copy id="f-fb2fe531fbd6-useUiReading">
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
</storycode:copy>

### Assemble `apps/web/src/lib/uiReading.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiReading.ts"><storycode:paste select="#f-fb2fe531fbd6-useUiReading"/></storycode:file>

</storycode:doc>
