<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/environments/WorkspaceStatusExtras.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9a774da4eb9026a2ced3ef31c96444b91ebf69e0a963715630056ca2017e0581`.

<book:entry path="apps/web/src/environments/WorkspaceStatusExtras.tsx">
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
</book:entry>

</book:doc>
