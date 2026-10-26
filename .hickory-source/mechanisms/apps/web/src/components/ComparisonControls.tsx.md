<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ComparisonControls.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-77a2cb6a5532"></a>
## `apps/web/src/components/ComparisonControls.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-77a2cb6a5532-ComparisonControls"></a>
### `ComparisonControls`

<storycode:copy id="f-77a2cb6a5532-ComparisonControls">
import { useId, type ReactNode } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";
import { forkComparison, type ComparisonModel } from "../lib/projectComparison";
import { useUiReading } from "../lib/uiReading";

export function ComparisonControls({ path, model, layout, setLayout, onChange, allowFork, compact, options }: {
  path: string; model: ComparisonModel; layout: string; setLayout: (layout: string) => void;
  onChange?: (model: ComparisonModel) => void; allowFork: boolean; compact: boolean; options?: ReactNode;
}) {
  const parent = `comparison:${useId()}`;
  const old = model.sources.find(s => s.id === model.old)!, next = model.sources.find(s => s.id === model.new)!;
  const sources = model.sources.map(({ id, label, kind, text, path, target, codeUri }) => ({ id, label, kind, text, path, target, codeUri }));
  const revision = useUiRevision(() => [path, sources, model.old, model.new, model.policy, model.evidence, layout]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "No editable comparison callback" }), execute } });
  add("layout", "Set comparison layout", { layout: { type: "string", maxLength: 32, choices: ["inline", "side-by-side", "patch"] } }, true, async args => { setLayout(args.layout as string); return { status: "completed", value: { layout_requested: args.layout } }; });
  if (allowFork && model.policy === "neither") add("fork", "Fork editable comparison", {}, !!onChange, async () => { onChange!(forkComparison(model)); return { status: "completed", value: { draft_fork_requested: true, saved: false, verified: false } }; });
  if (model.sources.length > 2) add("source", "Select comparison new source", { id: { type: "string", maxLength: 4096, choices: model.sources.filter(s => s.id !== model.old).map(s => s.id) } }, !!onChange, async args => { onChange!({ ...model, new: args.id as string, policy: "neither", evidence: "unverified" }); return { status: "completed", value: { source_requested: args.id, verified: false } }; });
  const invoke = useUiCommands(commands);
  useUiReading("comparison choices", { path, owner: parent, layout, old: model.old, new: model.new, policy: model.policy, evidence: model.evidence, sources: sources.map(({ text: _text, ...source }) => source), scope: "Source labels and controller-reported equality evidence, not independent verification; read visible editor/diff content separately." });
  return <>
    {!compact && <span>{old.label} → {next.label} · {model.evidence === "verified" ? "output equality verified" : "unverified"}</span>}
    <label>Layout <select aria-label="Comparison layout" value={layout} onChange={e => void invoke(`${parent}:layout`, { layout: e.target.value })}><option value="inline">Inline</option><option value="side-by-side">Side by side</option><option value="patch">Patch</option></select></label>
    {allowFork && model.policy === "neither" && <button className="btn" disabled={!onChange} onClick={() => void invoke(`${parent}:fork`)}>Fork editable comparison</button>}
    {model.sources.length > 2 && <label>New source <select value={model.new} disabled={!onChange} onChange={e => void invoke(`${parent}:source`, { id: e.target.value })}>{model.sources.filter(s => s.id !== model.old).map(s => <option key={s.id} value={s.id}>{s.label}</option>)}</select></label>}
    {options}
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/components/ComparisonControls.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ComparisonControls.tsx"><storycode:paste select="#f-77a2cb6a5532-ComparisonControls"/></storycode:file>

</storycode:doc>
