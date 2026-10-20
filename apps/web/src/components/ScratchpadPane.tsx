import { useEffect, useId, useRef, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiActor, UiOutcome } from "../lib/uiActions";
import { api } from "../api/client";

/** Where the note landed, or why it did not. */
type Result =
  | { kind: "saved"; path: string }
  | { kind: "refused"; message: string }
  | null;

/**
 * The third way material gets into a notes folder, beside downloading a file
 * into the inbox and copying one there: type it here.
 *
 * This is not a file-ingest surface and deliberately does not look like one.
 * Human-only text is prose a named human wrote, in this app, right now — the
 * strongest material there is on the attributable axis of
 * `docs/specs/freeform/provenance-and-standing.md`. It is never wrapped as a
 * transcript, and nothing about it is summarized, because there is no
 * machine step between the person and the words. Agent-assisted drafts instead
 * retain an explicit authorship marker and are never presented as human-only.
 *
 * Saving is deliberately explicit rather than automatic. A scratchpad that
 * committed every keystroke to a note would turn a place to think into a place
 * that keeps a record, and people stop thinking in those.
 */
export function ScratchpadPane() {
  const [text, setText] = useState("");
  const [agentAssisted, setAgentAssisted] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const [saving, setSaving] = useState(false);
  const [result, setResult] = useState<Result>(null);

  const empty = text.trim().length === 0;

  const change = (value: string, agent: boolean) => {
    if (pending.current) return;
    if (!value) setAgentAssisted(false);
    else if (agent && value !== text) setAgentAssisted(true);
    setText(value);
  };
  async function save(actor: UiActor, signal: AbortSignal): Promise<UiOutcome> {
    if (empty || pending.current || signal.aborted || !mounted.current) return { status: "unavailable", reason: "Scratchpad is empty, closed or busy" };
    pending.current = true; setSaving(true); setResult(null);
    const assisted = agentAssisted || actor.kind === "agent";
    try {
      const saved = assisted ? await api.saveScratchpad(text, true) : await api.saveScratchpad(text);
      if (mounted.current) { setResult({ kind: "saved", path: saved.path }); setText(""); setAgentAssisted(false); }
      return { status: "completed", value: { path: saved.path, agent_assisted: assisted, saved: true, verified: false } };
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      if (mounted.current) setResult({ kind: "refused", message });
      return { status: "unavailable", reason: message };
    } finally { pending.current = false; if (mounted.current) setSaving(false); }
  }
  const parent = `scratchpad:${useId()}`;
  const revision = useUiRevision(() => [text, agentAssisted, pending.current, result]);
  const invoke = useUiCommands([
    { key: `${parent}:text`, command: { parent, subject: parent, label: "Set scratchpad draft", parameters: { text: { type: "string", maxLength: 1048576 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !pending.current }), execute: async (args, actor) => {
        if (pending.current) return { status: "unavailable", reason: "Wait for the pending save" };
        change(args.text as string, actor.kind === "agent"); return { status: "completed", value: { draft_updated: true, saved: false } };
      } } },
    { key: `${parent}:save`, command: { parent, subject: parent, label: "Save scratchpad as note", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !empty && !pending.current }), execute: async (_args, actor, signal) => save(actor, signal) } },
  ]);
  const humanSave = () => void invoke(`${parent}:save`, {}).catch(() => {});
  useUiReading("scratchpad draft and save status", { text, agent_assisted: agentAssisted, saving, result,
    scope: "Explicit save; agent-assisted drafts retain an authorship marker, without declaring verification" });

  return (
    <div className="scratchpad-pane">
      <label className="scratchpad-label" htmlFor="scratchpad-text">
        Scratchpad
      </label>
      <p className="scratchpad-hint">
        Type a note. Saving writes it into this folder as a document — the first
        line becomes its title.
      </p>
      <textarea
        id="scratchpad-text"
        className="scratchpad-text"
        value={text}
        placeholder={"# Standup\n\nWhat happened, and what it means."}
        disabled={saving}
        onChange={(e) => change(e.target.value, false)}
        onKeyDown={(e) => {
          // Save without leaving the keyboard, the way every other editor does.
          if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
            e.preventDefault();
            humanSave();
          }
        }}
      />
      {agentAssisted && <p className="scratchpad-hint">Includes agent edits; this note will retain agent-assisted authorship.</p>}
      <div className="scratchpad-actions">
        <button
          type="button"
          className="scratchpad-save"
          disabled={empty || saving}
          onClick={humanSave}
        >
          {saving ? "Saving…" : "Save as note"}
        </button>
        {result?.kind === "saved" && (
          <span className="scratchpad-saved">Saved to {result.path}</span>
        )}
      </div>
      {result?.kind === "refused" && (
        // The refusals this can hit are all about hick markup in prose, and
        // they carry a next step — so show the message rather than a generic
        // failure that leaves the text stranded.
        <p className="scratchpad-refused">{result.message}</p>
      )}
    </div>
  );
}
