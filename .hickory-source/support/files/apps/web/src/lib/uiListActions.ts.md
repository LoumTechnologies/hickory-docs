<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/uiListActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `71611b017135c53a3b49e2762116f64904f312227c272a3f3cae171326a98d1c`.

<book:entry path="apps/web/src/lib/uiListActions.ts">
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
</book:entry>

</book:doc>
