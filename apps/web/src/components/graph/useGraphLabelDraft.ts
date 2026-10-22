import type { UiActor } from "../../lib/uiActions";
import { useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../../lib/uiActionBridge";
import { useUiReading } from "../../lib/uiReading";
import type { UiOutcome } from "../../lib/uiActions";

/** A visible label draft is separate from the committed scene. */
export function useGraphLabelDraft(input: {
  identity: string; value: string; disabled?: boolean; initiallyOpen?: boolean; nonempty?: boolean;
  commit: (value: string, actor?: UiActor) => boolean | void; close?: () => void;
}) {
  const parent = `graph-label:${useId()}`;
  type Draft = { identity: string; baseline: string; value: string };
  const [draft, setDraft] = useState<Draft | null>(input.initiallyOpen
    ? { identity: input.identity, baseline: input.value, value: input.value } : null);
  const state = useRef(draft);
  const [error, setError] = useState<string | null>(null);
  const update = (next: Draft | null) => { state.current = next; setDraft(next); };
  const revision = useUiRevision(() => [input.identity, input.value, input.disabled, state.current, error]);
  const open = () => { if (input.disabled || state.current) return; setError(null); update({ identity: input.identity, baseline: input.value, value: input.value }); };
  const change = (value: string) => { if (state.current) update({ ...state.current, value }); };
  const finish = (keep: boolean, actor?: UiActor): UiOutcome => {
    const current = state.current;
    if (!current) return { status: "unavailable", reason: "Open the label editor first" };
    if (keep && (input.disabled || current.identity !== input.identity || current.baseline !== input.value)) {
      const reason = "The label changed while this draft was open. Read the current label, then cancel and reopen before applying.";
      setError(reason);
      return { status: "requires_input", reason, request: `${parent}:changed-label` };
    }
    if (keep && input.nonempty && !current.value.trim()) {
      setError("Enter a nonempty label or cancel the draft.");
      return { status: "requires_input", reason: "Enter a nonempty label or cancel the draft", request: `${parent}:empty-label` };
    }
    if (keep && current.value.includes("</hick:")) {
      setError("A diagram label cannot contain </hick:.");
      return { status: "requires_input", reason: "Remove the diagram closing delimiter or cancel", request: `${parent}:delimiter` };
    }
    if (keep) {
      const applied = actor?.kind === "agent" ? input.commit(current.value.trim(), actor) : input.commit(current.value.trim());
      if (applied === false) { setError("The current source refused this label; the draft is retained."); return { status: "unavailable", reason: "The current source refused this label; the draft is retained" }; }
    } else input.close?.();
    update(null); setError(null);
    return { status: "completed", value: { draft_closed: true, commit_callback_requested: keep, saved: false } };
  };
  useUiReading(`graph label draft ${input.identity}`, { committed: input.value, draft: draft?.value ?? null,
    editing: !!draft, error, disabled: !!input.disabled }, input.identity);
  const invoke = useUiCommands([
    { key: `${parent}:open`, command: { parent, subject: parent, label: `Open graph label ${input.identity}`,
      parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !input.disabled && !state.current }),
      execute: async () => { open(); return { status: "completed", value: { editing: true } }; } } },
    { key: `${parent}:draft`, command: { parent, subject: parent, label: `Set graph label draft ${input.identity}`,
      parameters: { value: { type: "string", maxLength: 8192 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !!state.current }),
      execute: async args => { change(args.value as string); return { status: "completed", value: { draft_updated: true } }; } } },
    ...([true, false] as const).map(keep => ({ key: `${parent}:${keep ? "commit" : "cancel"}`, command: {
      parent, subject: parent, label: `${keep ? "Commit" : "Cancel"} graph label draft ${input.identity}`,
      parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!state.current }),
      execute: async (_args: unknown, actor: UiActor) => finish(keep, actor),
    } })),
  ]);
  return { editing: !!draft, value: draft?.value ?? input.value, error,
    open: () => void invoke(`${parent}:open`, {}).catch(() => {}), change,
    finish: (keep: boolean) => finish(keep),
  };
}
