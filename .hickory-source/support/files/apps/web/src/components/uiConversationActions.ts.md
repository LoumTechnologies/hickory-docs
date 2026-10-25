<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/uiConversationActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f3113bd53087221c49110abe2aa596362bf2170b3516d0a4c7825ca3e1271de5`.

<book:entry path="apps/web/src/components/uiConversationActions.ts">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";

export function useConversationActions(state: {
  doc: string; draft: string; tip: string | null; turns: { id: string }[];
  running: string | null; sending: boolean; stopping: boolean; ready: boolean;
  tree: boolean; collapsed: boolean; evidence: unknown;
  setDraft: (text: string) => void; discard: () => void;
  setTip: (id: string | null) => void; setTree: (open: boolean) => void;
  send: () => Promise<string | undefined>; stop: () => Promise<boolean>;
  toggleCollapsed?: () => void;
}) {
  const parent = `conversation:${useId()}`;
  const revision = useUiRevision(() => [state.doc, state.draft, state.tip, state.turns, state.running, state.sending, state.stopping, state.ready, state.tree, state.collapsed, state.evidence]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label, subject: parent, parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, ...(!enabled ? { reason: "Conversation busy, unavailable or control not applicable" } : {}) }), execute } });
  add("read", "Read conversation state bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ doc: state.doc, draft: state.draft, tip: state.tip, turns: state.turns, running: state.running, sending: state.sending, stopping: state.stopping, ready: state.ready, tree: state.tree, collapsed: state.collapsed, evidence: state.evidence, scope: "Captured UI state; a queued turn is not a completed response or verification" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current conversation reading" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("draft", "Set conversation response draft", { text: { type: "string", maxLength: 1048576 } }, !state.collapsed && !state.tree, async args => { state.setDraft(args.text as string); return { status: "completed", value: { draft_changed: true, provider_request_sent: false } }; });
  add("discard", "Discard conversation response", {}, !!state.draft, async () => { state.discard(); return { status: "completed", value: { discarded: true, prior_exposure_removed: false } }; });
  add("send", "Send conversation response", {}, !!state.draft.trim() && !state.running && !state.sending && state.ready, async (): Promise<UiOutcome> => {
    const turn = await state.send();
    return turn ? { status: "queued", job: turn, value: { turn, completion: "Await actual turn state; this is not verification" } } : { status: "completed", value: { turn_started: false, interpretation: "A local slash command or unavailable submission; read current note/error" } };
  });
  if (state.running) add("stop", "Stop conversation agent", {}, !state.stopping, async () => {
    const requested = await state.stop();
    return { status: "completed", value: { turn: state.running, stop_requested: requested, stopped: "Await actual terminal turn state" } };
  });
  add("tip", "Select conversation turn", { id: { type: "string", maxLength: 4096 } }, !state.running && !state.sending, async args => {
    if (!state.turns.some(turn => turn.id === args.id)) return { status: "unavailable", reason: "Observe an existing turn identity first" };
    state.setTip(args.id as string); state.setTree(false);
    return { status: "completed", value: { selected_turn: args.id, provider_request_sent: false } };
  });
  add("new", "Start new conversation thread", {}, !state.running && !state.sending, async () => { state.setTip(null); return { status: "completed", value: { tip: null, provider_request_sent: false, previous_history_retained: true } }; });
  add("tree", "Set conversation tree visibility", { open: { type: "boolean" } }, true, async args => { state.setTree(args.open as boolean); return { status: "completed", value: args.open }; });
  if (state.toggleCollapsed) add("collapse", "Toggle conversation visibility", {}, true, async () => { state.toggleCollapsed?.(); return { status: "completed", value: { visibility_change_requested: true } }; });
  const invoke = useUiCommands(commands);
  return { invoke: (key: string, args: Parameters<typeof invoke>[1] = {}) => invoke(`${parent}:${key}`, args) };
}
</book:entry>

</book:doc>
