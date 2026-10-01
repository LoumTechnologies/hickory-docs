import type { ComponentProps } from "react";
import { ProvenanceToggles } from "../shell/ProvenanceToggles";

export function WorkspaceStatusExtras({ provenance, environmentCount, onEnvironments, projectId, onGraph }: {
  provenance: ComponentProps<typeof ProvenanceToggles>;
  environmentCount: number;
  onEnvironments: () => void;
  projectId: string | null;
  onGraph: () => void;
}) {
  return <>
    <button type="button" className="status-bar__item" onClick={onEnvironments}>
      Environments{environmentCount ? ` (${environmentCount})` : ""}
    </button>
    <ProvenanceToggles {...provenance} />
    {projectId && <button type="button" className="status-bar__item"
      data-tip="Zoom out: every document as a node, the edges between them"
      aria-label="Show the lineage graph" onClick={onGraph}>
      <span className="status-bar__glyph" aria-hidden>⌘</span> Graph
    </button>}
  </>;
}
