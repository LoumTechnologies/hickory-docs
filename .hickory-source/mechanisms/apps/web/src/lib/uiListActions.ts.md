<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/uiListActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3a33412578b6"></a>
## `apps/web/src/lib/uiListActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3a33412578b6-useUiListActions"></a>
### `useUiListActions`

<storycode:copy id="f-3a33412578b6-useUiListActions">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "./uiActionBridge";
import type { UiCommand } from "./uiActions";

/** A mounted result list: explicit bounded reads and current-revision navigation. */
export function useUiListActions<T>(label: string, rows: readonly T[], metadata: unknown, onPick: (row: T) => void, onClose: () => void, selection?: { index: number; set: (index: number) => void }) {
  const parent = `list:${useId()}`;
  const revision = useUiRevision(() => [rows, metadata, selection?.index]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, name: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label: name, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { count: rows.length, ...(selection ? { selected_index: selection.index } : {}) } }), execute } });
  add("read", `Read ${label} evidence bytes`, { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ metadata, rows }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current list evidence" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("pick", `Open ${label} result`, { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async args => {
    const row = rows[args.index as number];
    if (row === undefined) return { status: "unavailable", reason: "Observe a current result index first" };
    onPick(row);
    return { status: "completed", value: { navigation_requested: true, index: args.index } };
  });
  if (selection) add("select", `Select ${label} result`, { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async args => {
    if (rows[args.index as number] === undefined) return { status: "unavailable", reason: "Observe a current result index first" };
    selection.set(args.index as number);
    return { status: "completed", value: { selected_index: args.index } };
  });
  add("close", `Close ${label}`, {}, async () => { onClose(); return { status: "completed", value: { close_requested: true } }; });
  const invoke = useUiCommands(commands);
  return { pick: (index: number) => invoke(`${parent}:pick`, { index }), close: () => invoke(`${parent}:close`), select: (index: number) => invoke(`${parent}:select`, { index }) };
}
</storycode:copy>

### Assemble `apps/web/src/lib/uiListActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiListActions.ts"><storycode:paste select="#f-3a33412578b6-useUiListActions"/></storycode:file>

</storycode:doc>
