<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiBisectActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-46be7510f3e7"></a>
## `apps/web/src/views/uiBisectActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-46be7510f3e7-useBisectActions"></a>
### `useBisectActions`

<storycode:copy id="f-46be7510f3e7-useBisectActions">
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
</storycode:copy>

### Assemble `apps/web/src/views/uiBisectActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiBisectActions.ts"><storycode:paste select="#f-46be7510f3e7-useBisectActions"/></storycode:file>

</storycode:doc>
