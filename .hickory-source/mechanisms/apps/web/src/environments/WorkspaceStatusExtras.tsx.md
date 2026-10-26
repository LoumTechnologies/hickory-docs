<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/environments/WorkspaceStatusExtras.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f3736e2b8ed5"></a>
## `apps/web/src/environments/WorkspaceStatusExtras.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f3736e2b8ed5-WorkspaceStatusExtras"></a>
### `WorkspaceStatusExtras`

<storycode:copy id="f-f3736e2b8ed5-WorkspaceStatusExtras">
import { useId, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { ComponentProps } from "react";
import { LineageSettings } from "../components/LineageSettings";
import { ProvenanceToggles } from "../shell/ProvenanceToggles";

export function WorkspaceStatusExtras({ provenance, environmentCount, onEnvironments, projectId, onGraph }: {
  provenance: ComponentProps<typeof ProvenanceToggles>;
  environmentCount: number;
  onEnvironments: () => void;
  projectId: string | null;
  onGraph: () => void;
}) {
  const parent = `workspace-status-extras:${useId()}`;
  const revision = useUiRevision(() => [environmentCount, projectId]);
  const [error, setError] = useState("");
  useUiReading("workspace environment and graph status", { environmentCount, projectId, error });
  const invoke = useUiCommands([
    { key: `${parent}:environment`, command: { label: "Show workspace environments", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async () => { await onEnvironments(); return { status: "completed", value: { environments_requested: true } }; } } },
    ...(projectId ? [{ key: `${parent}:graph`, command: { label: "Show workspace lineage graph", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!projectId }), execute: async () => { await onGraph(); return { status: "completed" as const, value: { graph_requested: true, project: projectId } }; } } }] : []),
  ]);
  const act = (id: string) => { void invoke(`${parent}:${id}`).catch(reason => setError(String(reason))); };
  return <>
    {error && <span role="alert">{error}</span>}
    <button type="button" className="status-bar__item" onClick={() => act("environment")}>
      Environments{environmentCount ? ` (${environmentCount})` : ""}
    </button>
    <ProvenanceToggles {...provenance} />
    <LineageSettings />
    {projectId && <button type="button" className="status-bar__item"
      data-tip="Zoom out: every document as a node, the edges between them"
      aria-label="Show the lineage graph" onClick={() => act("graph")}>
      <span className="status-bar__glyph" aria-hidden>⌘</span> Graph
    </button>}
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/environments/WorkspaceStatusExtras.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/WorkspaceStatusExtras.tsx"><storycode:paste select="#f-f3736e2b8ed5-WorkspaceStatusExtras"/></storycode:file>

</storycode:doc>
