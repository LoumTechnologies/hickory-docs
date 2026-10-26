<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/useEditorEditing.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0c8056b9e95d"></a>
## `apps/web/src/editor/useEditorEditing.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0c8056b9e95d-states"></a>
### `states`

<storycode:copy id="f-0c8056b9e95d-states">
import { prepareEditorInput } from "./textInput";
import { useId } from "react";
import type { EditorView } from "@codemirror/view";
import { undo, redo, undoDepth, redoDepth } from "@codemirror/commands";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";

const states = new WeakMap<object, string>();
</storycode:copy>
<a id="f-0c8056b9e95d-stateId"></a>
### `stateId`

<storycode:copy id="f-0c8056b9e95d-stateId">
function stateId(state: object) { let id = states.get(state); if (!id) { id = crypto.randomUUID(); states.set(state, id); } return id; }

</storycode:copy>
<a id="f-0c8056b9e95d-useEditorEditing"></a>
### `useEditorEditing`

Explicit owner opt-in; actual CodeMirror filters/listeners remain authoritative.

<storycode:copy id="f-0c8056b9e95d-useEditorEditing">
/** Explicit owner opt-in; actual CodeMirror filters/listeners remain authoritative. */
export function useEditorEditing(view: EditorView | null, owner: {
  source: string; label: string; identity: () => unknown; current: () => boolean; localOnly?: boolean;
}) {
  const parent = `editor-edit:${owner.source}:${useId()}`;
  const writable = () => !!view?.dom.isConnected && !view.state.readOnly && owner.current();
  const revision = useUiRevision(() => [owner.identity(), writable(), view ? stateId(view.state) : null]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: () => boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: ["Current owned editor state; ranges use UTF-16 code units"], read: () => ({ revision: revision(), enabled: writable() && enabled(), reason: !writable() ? "Editor owner changed, closed or became read-only" : !enabled() ? "No matching history operation" : undefined, value: { source: owner.source, length: view?.state.doc.length ?? 0, undo_depth: view ? undoDepth(view.state) : 0, redo_depth: view ? redoDepth(view.state) : 0, edit_baseline: false } }), execute } });
  const changed = (before: string, applied: boolean): UiOutcome => ({ status: "completed", value: { requested_applied: applied, source_changed: view!.state.doc.toString() !== before, draft_changed: view!.state.doc.toString() !== before, saved: owner.localOnly ? false : "Inspect owning save state; an editor transaction does not establish persistence", approved: false, verified: false } });
  const range = { type: "number" as const, minimum: 0, maximum: 100000000, integer: true };
  add("replace", `Replace ${owner.label} range`, { from: range, to: range, text: { type: "string", maxLength: 1048576 } }, () => true, async (args, actor) => {
    if (!writable() || !view) return { status: "unavailable", reason: "Editor is not writable" };
    const from = args.from as number, to = args.to as number, text = args.text as string;
    if (from > to || to > view.state.doc.length) return { status: "unavailable", reason: "Range outside current editor" };
    const before = view.state.doc.toString(), prepared = prepareEditorInput(before, [{ from, to }], text);
    if ("reason" in prepared) return { status: "unavailable", reason: prepared.reason };
    view.dispatch({ changes: { from, to, insert: text }, userEvent: actor.kind === "agent" ? "input.uiAgentSource" : "input" });
    return changed(before, view.state.doc.toString() === prepared.expected);
  });
  add("replace-selections", `Replace ${owner.label} selections`, { text: { type: "string", maxLength: 1048576 } }, () => true, async (args, actor) => {
    if (!writable() || !view) return { status: "unavailable", reason: "Editor is not writable" };
    const before = view.state.doc.toString(), text = args.text as string;
    const prepared = prepareEditorInput(before, view.state.selection.ranges, text);
    if ("reason" in prepared) return { status: "unavailable", reason: prepared.reason };
    view.dispatch({ ...view.state.replaceSelection(text), userEvent: actor.kind === "agent" ? "input.uiAgentSource" : "input" });
    return changed(before, view.state.doc.toString() === prepared.expected);
  });
  for (const [key, command, depth] of [["undo", undo, undoDepth], ["redo", redo, redoDepth]] as const) add(key, `${key === "undo" ? "Undo" : "Redo"} ${owner.label} edit`, {}, () => !!view && depth(view.state) > 0, async () => {
    if (!writable() || !view) return { status: "unavailable", reason: "Editor is not writable" };
    const before = view.state.doc.toString(); return changed(before, command(view));
  });
  useUiCommands(view ? commands : []);
}
</storycode:copy>

### Assemble `apps/web/src/editor/useEditorEditing.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/useEditorEditing.ts"><storycode:paste select="#f-0c8056b9e95d-states"/><storycode:paste select="#f-0c8056b9e95d-stateId"/><storycode:paste select="#f-0c8056b9e95d-useEditorEditing"/></storycode:file>

</storycode:doc>
