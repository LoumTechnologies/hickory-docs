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
