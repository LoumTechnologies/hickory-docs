import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
// The one question a dired verb asks, in the pane: a name, a destination,
// or "really?". Not a browser dialog — a `prompt()` in a webview is a modal
// nobody styled, and a `confirm()` in the desktop shell blocks the event
// loop the terminals run on.
import { useEffect, useId, useRef, useState } from "react";

export interface TreePromptProps {
  owner?: string;
  value?: string;
  onValueChange?: (value: string) => void;
  subject?: string;
  /** What is being asked, e.g. "Rename src/a.txt". */
  label: string;
  /** For a text question: the starting value. Absent means a confirmation. */
  initial?: string;
  /** The word on the button: "Rename", "Move", "Delete". */
  verb: string;
  /** What the person typed, or `null` for a confirmation. */
  onSubmit: (value: string | null) => unknown | Promise<unknown>;
  onCancel: () => void;
  /** Said under the input while a previous try is refused. */
  error?: string | null;
  /** True while the verb is running. */
  busy?: boolean;
}

export function TreePrompt(props: TreePromptProps) {
  return <OwnedTreePrompt key={JSON.stringify([props.owner, props.subject, props.label])} {...props} />;
}
function OwnedTreePrompt({ label, initial, verb, onSubmit, onCancel, error, busy, owner, subject, value: controlledValue, onValueChange }: TreePromptProps) {
  const [localValue, setValue] = useState(initial ?? "");
  const value = controlledValue ?? localValue;
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const el = input.current;
    if (!el) return;
    el.focus();
    // A rename starts with the stem selected, the way every file manager
    // does it: the extension is the part you almost never mean to change.
    const dot = (initial ?? "").lastIndexOf(".");
    el.setSelectionRange(0, dot > 0 ? dot : (initial ?? "").length);
  }, [initial]);
  const confirm = initial === undefined;
  const parent = `tree-prompt:${useId()}`;
  const [failure, setFailure] = useState("");
  const [pending, setPending] = useState(false);
  const inFlight = useRef(false), alive = useRef(true);
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
  const revision = useUiRevision(() => [owner, label, initial, verb, value, error, busy, pending, inFlight.current, failure]);
  const change = (next: string) => { if (onValueChange) onValueChange(next); else setValue(next); };
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (id: string, label: string, parameters: UiCommand["parameters"], enabled: () => boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${id}`, command: { label, parent, subject: subject ?? parent, parameters, prerequisites: ["Current visible prompt, draft and operation owner"], read: () => ({ revision: revision(), enabled: !busy && !inFlight.current && enabled(), reason: "Wait for the current operation and enter a nonempty value" }), execute } });
  if (!confirm) add("draft", `Edit visible file prompt: ${verb}`, { value: { type: "string", maxLength: 8192 } }, () => true, async args => { change(String(args.value)); return { status: "completed", value: { draft_changed: true, submitted: false } }; });
  add("submit", `Submit visible file prompt: ${verb}`, confirm ? { confirm: { type: "boolean" } } : {}, () => confirm || !!value.trim(), async (args): Promise<UiOutcome> => {
    if (busy || inFlight.current || !confirm && !value.trim()) return { status: "unavailable", reason: "The prompt is empty or already submitting" };
    if (confirm && args.confirm !== true) return { status: "unavailable", reason: "Explicit confirmation is required" };
    inFlight.current = true; setPending(true); setFailure("");
    try { const result = await onSubmit(confirm ? null : value); return result === false ? { status: "unavailable", reason: "The prompt owner refused the operation; inspect its retained error" } : { status: "completed", value: { submitted: true, host_completion: "Inspect the actual owning operation status" } }; }
    finally { inFlight.current = false; if (alive.current) setPending(false); }
  });
  add("cancel", `Cancel visible file prompt: ${verb}`, {}, () => true, async () => { if (busy || inFlight.current) return { status: "unavailable", reason: "An operation is pending; cancellation cannot undo its side effects" }; onCancel(); return { status: "completed", value: { cancellation_requested: true } }; });
  const invoke = useUiCommands(commands);
  const act = (id: string, args = {}) => { void invoke(`${parent}:${id}`, args).catch(cause => { if (alive.current) setFailure(cause instanceof Error ? cause.message : String(cause)); }); };
  useUiReading("visible file prompt", { owner: owner ?? null, label, verb, draft: confirm ? null : value, confirmation: confirm, error: error ?? null, busy: !!busy || pending, failure });
  return (
    <form
      className="tree-prompt"
      role="dialog"
      aria-label={label}
      onSubmit={(event) => {
        event.preventDefault();
        act("submit", confirm ? { confirm: true } : {});
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          if (!busy && !inFlight.current) act("cancel");
        }
      }}
    >
      <span className="tree-prompt__label">{label}</span>
      {!confirm && (
        <input
          ref={input}
          className="tree-prompt__input mono"
          value={value}
          onChange={(event) => change(event.target.value)}
          disabled={busy || pending}
          aria-label={label}
        />
      )}
      {error && <span className="tree-prompt__error">{error}</span>}
      {failure && <span role="alert">{failure}</span>}
      <span className="tree-prompt__buttons">
        <button type="submit" className="tree-prompt__verb" disabled={busy || pending || (!confirm && !value.trim())} autoFocus={confirm}>
          {busy ? `${verb}…` : verb}
        </button>
        <button type="button" onClick={() => act("cancel")} disabled={busy || pending}>
          Cancel
        </button>
      </span>
    </form>
  );
}
