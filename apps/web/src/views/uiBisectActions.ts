import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { bisects, type BisectSession } from "../api/representations";

export function useBisectActions(state: {
  open: boolean; setOpen: (open: boolean) => void; sessions: BisectSession[];
  good: string; bad: string; setGood: (good: string) => void; setBad: (bad: string) => void;
  busy: boolean; notice: string;
  run: (action: () => Promise<unknown>) => Promise<UiOutcome>; refresh: () => void;
}) {
  const parent = `bisect:${useId()}`;
  const revision = useUiRevision(() => [state.open, state.sessions, state.good, state.bad, state.busy, state.notice]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Bisect busy, modified or completed" }), execute } });
  add("visibility", "Set visual bisect visibility", { open: { type: "boolean" } }, true, async args => { state.setOpen(args.open as boolean); return { status: "completed", value: args.open }; });
  if (state.open) {
    add("draft", "Set bisect boundary commits", { good: { type: "string", maxLength: 4096 }, bad: { type: "string", maxLength: 4096 } }, !state.busy, async args => { state.setGood(args.good as string); state.setBad(args.bad as string); return { status: "completed", value: { draft_changed: true, classified: false } }; });
    add("refresh", "Refresh bisect sessions", {}, true, async () => { state.refresh(); return { status: "completed", value: { refresh_requested: true } }; });
    add("start", "Start visual bisect", {}, !state.busy && !!state.good.trim() && !!state.bad.trim(), async () => state.run(() => bisects.start(state.good, state.bad)));
    for (const session of state.sessions) {
      const prefix = session.id;
      add(`open:${prefix}`, `Open bisect candidate ${session.candidate.slice(0, 12)}`, {}, !state.busy, async () => state.run(() => bisects.open(session.id)));
      if (!session.outcome) for (const verdict of ["good", "bad", "skip"]) add(`${verdict}:${prefix}`, `Classify bisect candidate ${session.candidate.slice(0, 12)} as ${verdict}`, {}, !state.busy && !session.modified, async (_args, actor) => actor.kind === "agent" && verdict !== "skip" ? { status: "requires_input", reason: "Good/bad is a human classification; inspect and present actual execution evidence before requesting it" } : state.run(() => bisects.mark(session, verdict)));
      if (session.modified) add(`restore:${prefix}`, `Preserve experiment and restore bisect ${prefix}`, {}, !state.busy, async () => state.run(() => bisects.restore(session.id)));
      add(`finish:${prefix}`, `End bisect ${prefix}`, {}, !state.busy && !session.modified, async () => state.run(() => bisects.finish(session.id)));
    }
  }
  const invoke = useUiCommands(commands);
  return (key: string, args: Parameters<typeof invoke>[1] = {}) => invoke(`${parent}:${key}`, args).catch(() => undefined);
}
