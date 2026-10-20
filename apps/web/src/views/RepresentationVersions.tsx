import { useId } from "react";
import { EditorView } from "@codemirror/view";
import { foldEffect, foldedRanges, unfoldAll } from "@codemirror/language";
import { EvidenceDisclosure } from "../components/EvidenceDisclosure";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import type { Comparison } from "../api/representations";
import { comparisonChanges } from "../editor/comparison";

type Props = { id: string; base: string; target: string; comparison: Comparison | null; filesBacking: boolean;
  source: string; editor: EditorView | null; dirty: boolean; busy: boolean; live: () => unknown;
  setBase: (value: string) => void; setTarget: (value: string) => void;
  compare: () => Promise<UiOutcome>; reset: () => UiOutcome; onError: (reason: string) => void;
};
export function RepresentationVersions(props: Props) {
  const c = props.comparison;
  return <EvidenceDisclosure className="representation-versions" label={c ? `${c.base.slice(0, 8)} → ${c.target?.slice(0, 8) ?? "current"}` : "Compare revisions…"}
    value={{ id: props.id, base: props.base, target: props.target, comparison: c && { base: c.base, target: c.target, editable: c.editable, revision: c.revision }, scope: "Comparison identity; read displayed source and diff separately. Explanations are not evidence from the compared commit." }}>
    <VersionControls {...props} />
  </EvidenceDisclosure>;
}
function VersionControls(props: Props) {
  const parent = `literate-versions:${useId()}`;
  const revision = useUiRevision(() => [props.id, props.base, props.target, props.comparison, props.source, props.dirty, props.busy, props.live()]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Save or discard edits and wait for pending requests" }), execute } });
  add("base", "Set literate comparison base", { value: { type: "string", maxLength: 4096 } }, !props.busy, async args => { props.setBase(args.value as string); return { status: "completed", value: { base_requested: args.value } }; });
  add("target", "Set literate comparison target", { value: { type: "string", maxLength: 4096 } }, !props.busy, async args => { props.setTarget(args.value as string); return { status: "completed", value: { target_requested: args.value, scope: "Empty means current code" } }; });
  add("compare", "Show literate revision diff", {}, !props.dirty && !props.busy, props.compare);
  if (props.comparison) {
    add("reset", "Return literate view to current code", {}, !props.dirty && !props.busy, async () => props.reset());
    add("fold", "Fold unchanged literate comparison regions", {}, !!props.editor && !props.busy, async () => {
      const editor = props.editor;
      if (!editor || !editor.dom.isConnected || editor.state.doc.toString() !== props.source) return { status: "unavailable", reason: "Comparison editor changed or closed" };
      unfoldAll(editor);
      const changes = comparisonChanges(props.comparison!.source, props.source);
      let previous = 0;
      for (const change of [...changes, { from: props.source.length, to: props.source.length, removed: "" }]) {
        const start = editor.state.doc.lineAt(previous).number + 3, end = editor.state.doc.lineAt(change.from).number - 3;
        if (end > start) editor.dispatch({ effects: foldEffect.of({ from: editor.state.doc.line(start).to, to: editor.state.doc.line(end).to }) });
        previous = change.to;
      }
      return { status: "completed", value: { folded_regions: foldedRanges(editor.state).size, source_written: false } };
    });
  }
  const invoke = useUiCommands(commands);
  const human = (key: string, args = {}) => void invoke(`${parent}:${key}`, args).catch(e => props.onError(String(e)));
  const c = props.comparison;
  return <div className="representation-versions__menu"><form className="representation-comparison" onSubmit={e => { e.preventDefault(); human("compare"); }}>
    <label>Earlier revision <input aria-label="Comparison base" value={props.base} disabled={props.busy} placeholder="HEAD, branch, or commit" onChange={e => props.setBase(e.target.value)} /></label>
    <label>Compare with <input aria-label="Comparison target" value={props.target} disabled={props.busy} placeholder="Current code (editable)" onChange={e => props.setTarget(e.target.value)} /></label>
    <button className="btn" disabled={props.dirty || props.busy}>Show diff</button>
    {c && <button type="button" className="btn" disabled={props.dirty || props.busy} onClick={() => human("reset")}>Return to current code</button>}
    {c && <button type="button" className="btn" disabled={!props.editor || props.busy} onClick={() => human("fold")}>Fold unchanged regions</button>}
    <p className="muted">Leave the second revision empty to compare with editable current code. A historical target is read-only; use a worktree to run it.</p>
    </form>
    {c && props.filesBacking && <p className="muted">Explanations belong to this reading arrangement; they are not evidence from the compared commit.</p>}
    {c && <p className="muted">{c.base.slice(0, 12)} → {c.target?.slice(0, 12) ?? "working tree"}{!c.editable ? " · historical, read-only; open a worktree to run this revision" : " · current side is editable"}</p>}
  </div>;
}
