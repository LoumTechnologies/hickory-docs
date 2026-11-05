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
