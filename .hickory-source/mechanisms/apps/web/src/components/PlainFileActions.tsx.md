<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/PlainFileActions.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7123a11744a2"></a>
## `apps/web/src/components/PlainFileActions.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7123a11744a2-PlainFileActions"></a>
### `PlainFileActions`

<storycode:copy id="f-7123a11744a2-PlainFileActions">
import { useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";
import { openLiterate } from "../lib/openRepresentation";

export function PlainFileActions({ path, loaded, generatedBy, saveKind, adopting, sourceRevision, adopt, onError }: {
  path: string; loaded: boolean; generatedBy: string | null; saveKind: string; adopting: boolean;
  sourceRevision: () => unknown; adopt: () => Promise<unknown>; onError: (error: string) => void;
}) {
  const inFlight = useRef(false);
  const [busy, setBusy] = useState(false);
  const parent = `file-actions:${useId()}`;
  const revision = useUiRevision(() => [path, loaded, generatedBy, saveKind, adopting, busy, sourceRevision()]);
  const ready = loaded && !busy && !adopting && ["idle", "saved"].includes(saveKind);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, action: () => Promise<unknown>) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: ready, value: { path, loaded, generated_by: generatedBy, save_kind: saveKind, adopting, pending: busy }, reason: ready ? undefined : "Load and save/resolve the owning file before changing its representation" }), execute: async () => {
    if (inFlight.current) return { status: "unavailable", reason: "Another file representation action is pending" };
    inFlight.current = true; setBusy(true);
    try { const reply = await action(); return { status: "completed", value: { owner_reply: JSON.stringify(reply ?? null), scope: "Actual representation/adoption handler return; opening a view is not a verified correction" } }; }
    catch (e) { const reason = e instanceof Error ? e.message : String(e); onError(reason); return { status: "unavailable", reason }; }
    finally { inFlight.current = false; setBusy(false); }
  } } });
  if (!generatedBy) {
    add("literate", "Open file literate view", () => openLiterate({ kind: "files", paths: [path] }));
    add("review", "Review current file changes", () => openLiterate({ kind: "files", paths: [path] }, "HEAD"));
    add("adopt", "Make current file literate", adopt);
  }
  const invoke = useUiCommands(commands);
  const human = (key: string) => void invoke(`${parent}:${key}`).catch(error => onError(error instanceof Error ? error.message : String(error)));
  return <>
    {!generatedBy && <button className="btn" disabled={!ready} onClick={() => human("literate")} data-tip="Open an editable arrangement; Save writes code back to these source files.">Open literate view</button>}
    {!generatedBy && <button className="btn" disabled={!ready} onClick={() => human("review")}>Review changes</button>}
    {generatedBy ? <span className="muted plain-file__generated" role="status">Written by a literate document — it is already literate.</span> : <button className="btn" disabled={!ready} onClick={() => human("adopt")} data-tip="Create a .md document that owns this file’s bytes. Future edits to this generated file write back to that document.">{adopting ? "Adopting…" : "Make literate"}</button>}
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/components/PlainFileActions.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PlainFileActions.tsx"><storycode:paste select="#f-7123a11744a2-PlainFileActions"/></storycode:file>

</storycode:doc>
