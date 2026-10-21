import { useId, useRef } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
import type { MenuAction } from "../lib/menuBridge";
const actions = [
  ["save", "Save focused buffer"], ["save-as", "Save focused buffer as"], ["save-all", "Save all workspace buffers"],
  ["print", "Print focused buffer"], ["insert", "Open Insert menu"], ["edit-element", "Edit element at focused selection"],
] as const;
export function useWorkspaceMenuActions(snapshot: () => { visible: boolean; guard: unknown; status: UiValue }, execute: (action: MenuAction, actor: UiActor, destination?: string) => Promise<UiOutcome>) {
  const parent = `workspace-menu:${useId()}`;
  const last = useRef<UiValue>(null);
  const revision = useUiRevision(() => [snapshot().guard, last.current]);
  const commands: { key: string; command: UiCommand }[] = actions.map(([action, label]) => ({ key: `${parent}:${action}`, command: { label, parent, subject: parent, parameters: {}, prerequisites: ["Current window, focused buffer and source/UI revision", ...(action === "print" ? ["Opens the same browser/OS print callback as the native menu. Printer selection is OS-owned; printed vs cancelled and physical output are not observable."] : [])], read: () => ({ revision: revision(), enabled: snapshot().visible }), execute: async (_args, actor) => {
    const outcome = await execute(action, actor);
    last.current = JSON.parse(JSON.stringify({ action, actor, outcome })); return outcome;
  } } }));
  commands.push({ key: `${parent}:destination`, command: { label: "Save unnamed buffer to specified path", parent, subject: parent,
    parameters: { path: { type: "string", maxLength: 4096 } }, prerequisites: ["Current focused unnamed buffer; destination must be a new .md file. Existing files are refused."],
    read: () => { const status = snapshot().status as { target?: { kind?: string } } | null;
      return { revision: revision(), enabled: snapshot().visible && status?.target?.kind === "untitled" }; },
    execute: async (args, actor) => {
      const outcome = await execute("save-as", actor, args.path as string);
      last.current = JSON.parse(JSON.stringify({ action: "save-to-path", actor, destination: args.path, outcome })); return outcome;
    },
  } });
  commands.push({ key: `${parent}:read`, command: { label: "Read workspace command status bytes", parent, subject: `${parent}:read`, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: snapshot().visible }), execute: async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ current: snapshot().status, last: last.current }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside workspace command status" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  } } });
  const invoke = useUiCommands(commands);
  return async (action: MenuAction) => {
    if (actions.some(([id]) => id === action)) await invoke(`${parent}:${action}`);
    else await execute(action, { id: "local-user", kind: "human" });
  };
}
