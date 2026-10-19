import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiValue } from "../lib/uiActions";

/** Selection affects future request assembly; a preview never attests delivery. */
export function useContextProjectionActions(state: {
  open: boolean; setOpen: (open: boolean) => void;
  tabs: { id: string; target: string; title?: string; kind: string }[];
  selection: Record<string, "expanded" | "pinned">;
  choose: (id: string, mode: "off" | "expanded" | "pinned") => void;
  preview: unknown; error: string;
  refresh: () => Promise<void>;
}) {
  const parent = `context-projection:${useId()}`;
  const revision = useUiRevision(() => [state.open, state.tabs.map(t => [t.id, t.target, t.title, t.kind]), state.selection, state.preview, state.error]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute } });
  add("visibility", "Set context projection visibility", { open: { type: "boolean" } }, async args => { state.setOpen(args.open as boolean); return { status: "completed", value: args.open }; });
  if (state.open) {
    add("documents", "Read context document choices", { offset: { type: "number", minimum: 0, maximum: 1000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 64, integer: true } }, async args => {
      const offset = args.offset as number;
      if (offset > state.tabs.length) return { status: "unavailable", reason: "Offset outside current document choices" };
      const documents = state.tabs.slice(offset, offset + (args.limit as number)).map(t => ({ id: t.id, path: t.target, title: t.title ?? t.target, kind: t.kind, selection: state.selection[t.id] ?? "off" }));
      return { status: "completed", value: { documents, total: state.tabs.length, offset, next_offset: offset + documents.length < state.tabs.length ? offset + documents.length : null } };
    });
    add("choose", "Select document context", { id: { type: "string", maxLength: 4096 }, mode: { type: "string", maxLength: 8, choices: ["off", "expanded", "pinned"] } }, async args => {
      if (!state.tabs.some(t => t.id === args.id)) return { status: "unavailable", reason: "Document closed; observe current choices" };
      state.choose(args.id as string, args.mode as "off" | "expanded" | "pinned");
      return { status: "completed", value: { id: args.id, selection: args.mode, effect: "Future context selection; earlier exposure stays in history" } };
    });
    add("refresh", "Capture context projection preview", {}, async () => { await state.refresh(); return { status: "completed", value: { capture_requested: true, delivery: "not established; read preview or error" } }; });
    add("read", "Read context projection preview bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ preview: state.preview, error: state.error, delivery: "Preview capture only; Send captures context again" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside captured preview" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)) as UiValue, encoding: "JSON UTF-8", total_bytes: bytes.length, offset, next_offset: end < bytes.length ? end : null } };
    });
  }
  const invoke = useUiCommands(commands);
  return { invoke: (key: string, args: Record<string, UiValue> = {}) => invoke(`${parent}:${key}`, args) };
}
