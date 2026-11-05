<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/environments/WorkspaceStatusExtras.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f3736e2b8ed5"></a>
## `apps/web/src/environments/WorkspaceStatusExtras.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f3736e2b8ed5-WorkspaceStatusExtras"></a>
### `WorkspaceStatusExtras`

A healthy environment contributes no status item. The count names dependency problems rather than the implementation concept “Environments”. Connection preferences and the workspace graph belong in explicit tools; this component only receives the overlay's painted-family set.

<storycode:copy id="f-f3736e2b8ed5-WorkspaceStatusExtras">
import { useUiCommands } from "../lib/uiActionBridge";
import { ProvenanceToggles } from "../shell/ProvenanceToggles";
import type { ComponentProps } from "react";

export function WorkspaceStatusExtras({ provenance, environmentCount, onEnvironments }: {
  provenance: ComponentProps<typeof ProvenanceToggles>;
  environmentCount: number;
  onEnvironments: () => void;
}) {
  const invoke = useUiCommands(environmentCount > 0 ? [{ key: "workspace:dependencies", command: {
    label: "Show workspace environments", parent: "workspace-status", subject: "workspace-status", parameters: {}, prerequisites: [],
    read: () => ({ revision: String(environmentCount), enabled: true }),
    execute: async () => { onEnvironments(); return { status: "completed", value: { environments_requested: true } }; },
  } }] : []);
  return <>
    {environmentCount > 0 && <button type="button" className="status-bar__item" onClick={() => { void invoke("workspace:dependencies"); }}
      title="Project dependencies need attention" aria-label={`${environmentCount} dependency problems`}>
      Dependencies ({environmentCount})
    </button>}
    <ProvenanceToggles {...provenance} />
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/environments/WorkspaceStatusExtras.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/WorkspaceStatusExtras.tsx"><storycode:paste select="#f-f3736e2b8ed5-WorkspaceStatusExtras"/></storycode:file>

</storycode:doc>
