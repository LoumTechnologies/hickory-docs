import { useEffect, useId, useRef, useState } from "react";
import { request } from "../api/client";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiActor, UiOutcome, UiValue } from "../lib/uiActions";

/** This endpoint records a human declaration; model audit tools retain model attribution. */
export function AuditNoteForm({ path }: { path: string }) {
  const parent = `audit-note:${useId()}`;
  const [statement, setStatement] = useState("");
  const [reason, setReason] = useState("");
  const [supersedes, setSupersedes] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [receipt, setReceipt] = useState<UiValue>(null);
  const [saving, setSaving] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const submit = async (actor: UiActor, signal: AbortSignal): Promise<UiOutcome> => {
    if (pending.current || signal.aborted || !mounted.current) return { status: "unavailable", reason: "Audit form is busy or closed" };
    if (!statement.trim() || !reason.trim()) return { status: "requires_input", request: `${parent}:fields`, reason: "State the declaration and explain its reason" };
    if (actor.kind !== "human") {
      setMessage(`Human declaration request ${parent}:human-declaration is ready for review. Model-authored audit entries use the agent's record_assumption/record_decision tools.`);
      return { status: "requires_input", request: `${parent}:human-declaration`, reason: "This form records a user declaration. A human must inspect and submit it; model declarations use the existing agent audit tools." };
    }
    pending.current = true; setSaving(true); setMessage(null);
    try {
      const result = await request<UiValue>("POST", "/api/sessions/audit", { path, entry: { statement, reason,
        ...(supersedes.trim() ? { supersedes: supersedes.trim() } : {}) } });
      if (mounted.current) {
        setStatement(""); setReason(""); setSupersedes(""); setReceipt(result);
        setMessage("Recorded as a user declaration. Earlier records remain; reopen or wait for the session to refresh.");
      }
      return { status: "completed", value: { response: result, actor: "user-declared", verified: false } };
    } catch (error) {
      const text = error instanceof Error ? error.message : String(error);
      if (mounted.current) { setMessage(text); setReceipt({ error: text, effects: "Inspect the session before retrying" }); }
      return { status: "unavailable", reason: text };
    } finally { pending.current = false; if (mounted.current) setSaving(false); }
  };
  const revision = useUiRevision(() => [path, statement, reason, supersedes, pending.current, message, receipt]);
  const fields = [
    { id: "statement", label: "Set audit note statement", value: statement, set: setStatement },
    { id: "reason", label: "Set audit note reason", value: reason, set: setReason },
    { id: "supersedes", label: "Set audit note superseded entry", value: supersedes, set: setSupersedes },
  ];
  const invoke = useUiCommands([
    ...fields.map(field => ({ key: `${parent}:${field.id}`, command: { parent, subject: parent, label: field.label,
      parameters: { text: { type: "string" as const, maxLength: 65536 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !pending.current, value: { current: field.value } }),
      execute: async (args: Record<string, UiValue>): Promise<UiOutcome> => {
        if (pending.current) return { status: "unavailable", reason: "Wait for the pending declaration" };
        field.set(args.text as string); return { status: "completed", value: { draft_updated: true, recorded: false } };
      },
    } })),
    { key: `${parent}:submit`, command: { parent, subject: parent, label: "Record user audit declaration", parameters: {}, prerequisites: ["Actual human declaration; cannot be impersonated by an agent"],
      read: () => ({ revision: revision(), enabled: !pending.current }), execute: async (_args, actor, signal) => submit(actor, signal) } },
  ]);
  useUiReading("audit note draft and result", { path, statement, reason, supersedes, saving, message, receipt,
    declaration_request: `${parent}:human-declaration`, authority: "Human declaration; model audit tools retain model attribution" });
  return <form onSubmit={event => { event.preventDefault(); void invoke(`${parent}:submit`, {}).catch(error => setMessage(String(error))); }}>
    <label>Statement<textarea required disabled={saving} value={statement} onChange={event => { if (!pending.current) setStatement(event.target.value); }} /></label>
    <label>Reason<textarea required disabled={saving} value={reason} onChange={event => { if (!pending.current) setReason(event.target.value); }} /></label>
    <label>Supersedes entry id (optional)<input disabled={saving} value={supersedes} onChange={event => { if (!pending.current) setSupersedes(event.target.value); }} /></label>
    <button disabled={saving} className="btn" type="submit">Record audit note</button>
    {message && <p role="status">{message}</p>}
  </form>;
}
