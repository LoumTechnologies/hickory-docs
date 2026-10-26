<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/useDivergenceActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8b223c469338"></a>
## `apps/web/src/components/useDivergenceActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8b223c469338-useDivergenceActions"></a>
### `useDivergenceActions`

<storycode:copy id="f-8b223c469338-useDivergenceActions">
import { useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";

export function useDivergenceActions(state: {
  what: string; reason: string; mine: string; theirs: string;
  keepLabel: string; keepEffect: string; takeLabel: string; takeEffect: string; takeTheirsHint?: string;
  error?: string | null; sourceRevision?: () => unknown;
  onKeepMine?: () => unknown; onTakeTheirs?: () => unknown; onMerge?: () => unknown;
}) {
  const parent = `divergence:${useId()}`;
  const [pending, setPending] = useState<string | null>(null), [failure, setFailure] = useState("");
  const inFlight = useRef(false);
  const metadata = { what: state.what, reason: state.reason, mine: state.mine, theirs: state.theirs, keep_label: state.keepLabel, keep_effect: state.keepEffect, take_label: state.takeLabel, take_effect: state.takeEffect, unavailable_hint: state.takeTheirsHint, error: state.error, failure, pending };
  const revision = useUiRevision(() => [metadata, state.sourceRevision?.()]);
  const commands: { key: string; command: UiCommand }[] = [];
  commands.push({ key: `${parent}:read`, command: { parent, subject: `${parent}:reading`, label: `Read divergence ${state.what} bytes`, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ ...metadata, scope: "Current divergence controller; handler return is not independent verification. Inspect owning save state or host replies." }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    return offset > bytes.length ? { status: "unavailable", reason: "Offset outside divergence state" } : { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  } } });
  for (const [key, label, effect, action] of [["keep", state.keepLabel, state.keepEffect, state.onKeepMine], ["take", state.takeLabel, state.takeEffect, state.onTakeTheirs], ["merge", "Open divergence merge", "Open a preview without accepting it", state.onMerge]] as const) {
    if (!action) continue;
    commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !pending, value: { owner: state.what, effect } }), execute: async () => {
      if (inFlight.current) return { status: "unavailable", reason: "Another divergence decision is pending" };
      inFlight.current = true; setPending(key); setFailure("");
      try {
        const result = action();
        const reply = result && typeof (result as Promise<unknown>).then === "function" ? await result : result;
        return { status: "completed", value: { handler_returned: true, owner_reply: JSON.stringify(reply ?? null), saved: "Inspect actual owning save state or reply; a void callback only dispatches its request", verified: false } };
      } catch (e) { const reason = e instanceof Error ? e.message : String(e); setFailure(reason); return { status: "unavailable", reason }; }
      finally { inFlight.current = false; setPending(null); }
    } } });
  }
  const invoke = useUiCommands(commands);
  return { pending, failure, invoke: (key: string) => void invoke(`${parent}:${key}`).catch(() => undefined) };
}
</storycode:copy>

### Assemble `apps/web/src/components/useDivergenceActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/useDivergenceActions.ts"><storycode:paste select="#f-8b223c469338-useDivergenceActions"/></storycode:file>

</storycode:doc>
