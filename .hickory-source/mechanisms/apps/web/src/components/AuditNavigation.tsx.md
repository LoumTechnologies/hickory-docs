<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AuditNavigation.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ea4be58e4cbe"></a>
## `apps/web/src/components/AuditNavigation.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ea4be58e4cbe-AuditLocationButton"></a>
### `AuditLocationButton`

<storycode:copy id="f-ea4be58e4cbe-AuditLocationButton">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { openLocation } from "../lib/revealLine";
import { EvidenceDisclosure } from "./EvidenceDisclosure";

export function AuditLocationButton({ path, line, label }: { path: string; line: number; label: string }) {
  const parent = `audit-location:${useId()}`;
  const revision = useUiRevision(() => [path, line]);
  const open = () => openLocation(path, line);
  const invoke = useUiCommands([{ key: parent, command: { parent, subject: parent, label: `${label}: ${path}:${line}`,
    parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { path, line } }),
    execute: async () => { open(); return { status: "completed", value: { navigation_requested: true, path, line } }; },
  } }]);
  return <button className="btn-link" onClick={() => void invoke(parent, {}).catch(() => {})}>{label}</button>;
}

</storycode:copy>
<a id="f-ea4be58e4cbe-AuditSourceExport"></a>
### `AuditSourceExport`

<storycode:copy id="f-ea4be58e4cbe-AuditSourceExport">
export function AuditSourceExport({ path, source }: { path: string; source: string }) {
  const parent = `audit-export:${useId()}`;
  const revision = useUiRevision(() => [path, source]);
  const download = () => {
    const url = URL.createObjectURL(new Blob([source], { type: "text/markdown" }));
    const anchor = document.createElement("a"); anchor.href = url; anchor.download = path.split(/[\\/]/).pop() || "session.md";
    anchor.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  useUiCommands([{ key: parent, command: { parent, subject: parent, label: "Download full audit session record",
    parameters: {}, prerequisites: ["Browser download request; paged session bytes are the direct read equivalent"],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async () => { download(); return { status: "completed", value: { download_requested: true, saved: false, path } }; },
  } }]);
  return <>
    <button className="btn-link" onClick={download}>Download full session record</button>
    <EvidenceDisclosure label="Full session record bytes" value={{ path, source }}><pre>{source}</pre></EvidenceDisclosure>
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/components/AuditNavigation.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AuditNavigation.tsx"><storycode:paste select="#f-ea4be58e4cbe-AuditLocationButton"/><storycode:paste select="#f-ea4be58e4cbe-AuditSourceExport"/></storycode:file>

</storycode:doc>
