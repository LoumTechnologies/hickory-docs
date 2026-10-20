import { useId, useRef, useState } from "react";
import { MoreActions } from "../components/MoreActions";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
import { representations, type Representation } from "../api/representations";
import { showRepresentation } from "../lib/openRepresentation";

type Props = {
  record: Representation; dirty: boolean; busy: boolean; historical: boolean; agent: boolean;
  live: () => unknown; save: () => Promise<UiOutcome>; refresh: () => Promise<UiOutcome>;
  discard: () => void; toggleAgent: () => void; onNotice: (notice: string) => void;
};

/** The human and model share these callbacks; menu actions exist only while disclosed. */
export function RepresentationActions(props: Props) {
  const parent = `literate-actions:${useId()}`;
  const revision = useUiRevision(() => [props.record.id, props.record.revision, props.live(), props.dirty, props.busy, props.historical]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: {
    parent, subject: parent, label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Save/discard local edits, return to current code, or wait for the pending request" }), execute,
  } });
  add("save", "Save literate view", props.dirty && !props.busy && !props.historical, props.save);
  add("refresh", "Refresh literate view from source", !props.dirty && !props.busy && !props.historical, props.refresh);
  const invoke = useUiCommands(commands);
  const human = (key: string) => void invoke(`${parent}:${key}`).catch(e => props.onNotice(String(e)));
  useUiReading("literate view status", { id: props.record.id, revision: props.record.revision, backing: props.record.backing, dirty: props.dirty, pending: props.busy, historical: props.historical, explanation_stale: props.record.explanation_stale, warning: props.record.local_warning, scope: "Loaded host revision and local buffer status; source and file contents require explicit editor reading. Saved does not establish execution correctness." });
  return <>
    <button className="btn btn-primary" disabled={!props.dirty || props.busy || props.historical} onClick={() => human("save")}>{props.busy ? "Working…" : "Save"}</button>
    <button className="btn" disabled={props.dirty || props.busy || props.historical} onClick={() => human("refresh")}>Refresh from source</button>
    <span role="status">{props.dirty ? "Unsaved changes" : "Saved"}</span>
    <MoreActions label={`More actions for literate view ${props.record.id}`}><RepresentationMenu {...props} /></MoreActions>
  </>;
}

function RepresentationMenu(props: Props) {
  const parent = `literate-menu:${useId()}`;
  const pending = useRef(false);
  const [busy, setBusy] = useState(false);
  const revision = useUiRevision(() => [props.record, props.live(), props.dirty, props.busy, props.historical, props.agent, busy]);
  const ready = !props.busy && !busy && !props.historical;
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, enabled: boolean, action: () => Promise<UiValue>) => commands.push({ key: `${parent}:${key}`, command: {
    parent, subject: parent, label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Resolve local edits or pending requests and return to current code" }),
    execute: async () => {
      if (pending.current) return { status: "unavailable", reason: "Literate view menu request already pending" };
      pending.current = true; setBusy(true);
      try { return { status: "completed", value: await action() }; }
      catch (e) { const reason = e instanceof Error ? e.message : String(e); props.onNotice(reason); return { status: "unavailable", reason }; }
      finally { pending.current = false; setBusy(false); }
    },
  } });
  add("discard", "Discard unsaved literate view edits", ready && props.dirty, async () => { props.discard(); return { discarded_local_edits: true, disk_written: false }; });
  if (props.record.backing.kind === "files") add("rebuild", "Rebuild literate reading from current files", ready && !props.dirty, async () => {
    const reply = await representations.create(props.record.backing);
    showRepresentation(reply.id);
    return { created_view: reply.id, revision: reply.revision, navigation_requested: true, verified: false };
  });
  add("keep", "Keep literate view locally", ready && !props.dirty, async () => {
    const reply = await representations.keep(props.record.id);
    props.onNotice(`View kept locally at ${reply.path}`);
    return { path: reply.path, scope: "Actual host keep reply; not a source save or execution witness" };
  });
  add("agent", "Toggle literate view agent conversation", ready, async () => { props.toggleAgent(); return { conversation_open_requested: !props.agent }; });
  const invoke = useUiCommands(commands);
  const human = (key: string) => void invoke(`${parent}:${key}`).catch(e => props.onNotice(String(e)));
  return <>
    <button className="btn" disabled={!ready || !props.dirty} onClick={() => human("discard")}>Discard unsaved edits</button>
    {props.record.backing.kind === "files" && <button className="btn" disabled={!ready || props.dirty} onClick={() => human("rebuild")}>Rebuild reading from current files</button>}
    <button className="btn" disabled={!ready || props.dirty} onClick={() => human("keep")}>Keep view locally</button>
    <button className="btn" disabled={!ready} onClick={() => human("agent")}>Explain or edit with an agent</button>
  </>;
}
