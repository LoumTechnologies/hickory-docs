<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/SnapshotExport.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A viewer snapshot keeps recorded results historical](../../../../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md)

Captured SHA-256: `3b048f4c9e005d85db7ab47fa585516742435dccd3818c1393dfc55652653b83`.

<book:entry path="apps/web/src/components/SnapshotExport.tsx">
import { useId, useRef, useState } from "react";
import { request } from "../api/client";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";

export function SnapshotExport({ docId }: { docId?: string | null }) {
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const [captured, setCaptured] = useState<{ docId: string; bytes: Uint8Array; sha256: string } | null>(null);
  const subject = `snapshot-export:${useId()}`;
  const current = captured?.docId === docId ? captured : null;
  const revision = useUiRevision(() => [docId, busy, message, current?.sha256]);
  async function exportSnapshot(signal: AbortSignal): Promise<UiOutcome> {
    if (!docId || pending.current) return { status: "unavailable", reason: "Select a document and wait for any active export" };
    pending.current = true; setBusy(true); setMessage(""); setCaptured(null);
    try {
      const snapshot = await request<unknown>("GET", `/api/docs/${encodeURIComponent(docId)}/snapshot`);
      const text = JSON.stringify(snapshot, null, 2);
      const bytes = new TextEncoder().encode(text);
      const sha256 = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))).map(b => b.toString(16).padStart(2, "0")).join("");
      setCaptured({ docId, bytes, sha256 });
      if (signal.aborted) return { status: "completed", value: { doc_id: docId, sha256, bytes: bytes.length, browser_download_requested: false, publication: "not requested" } };
      const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
      try {
        const link = document.createElement("a"); link.href = url; link.download = "hickory-snapshot.json"; link.click();
      } finally { setTimeout(() => URL.revokeObjectURL(url), 1000); }
      setMessage("Exported saved source and recorded cell results. Review before publishing; outputs may contain private data. Open it in the snapshot viewer, or host the JSON beside snapshot.html and use ?src=your-file.json.");
      return { status: "completed", value: { doc_id: docId, sha256, bytes: bytes.length, browser_download_requested: true, file_saved: "Browser-owned; not observed", publication: "not requested", scope: "Saved source and recorded cell results; not a new verification run" } };
    } catch (error) { setMessage(String(error)); return { status: "unavailable", reason: String(error) }; }
    finally { pending.current = false; setBusy(false); }
  }
  const commands: { key: string; command: UiCommand }[] = [{ key: `${subject}:export`, command: {
    label: "Export viewer snapshot", parent: subject, subject, parameters: {}, prerequisites: ["A saved document; review source and outputs before publishing"],
    read: () => ({ revision: revision(), enabled: !!docId && !busy, ...(!docId || busy ? { reason: "Select a document and wait for any active export" } : {}) }),
    execute: async (_args, _actor, signal) => exportSnapshot(signal),
  } }, { key: `${subject}:status`, command: {
    label: "Read snapshot export status", parent: subject, subject, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async () => ({ status: "completed", value: { doc_id: docId ?? null, busy, message, artifact: current ? { sha256: current.sha256, bytes: current.bytes.length } : null } }),
  } }];
  if (current) commands.push({ key: `${subject}:read`, command: {
    label: "Read exported snapshot bytes", parent: subject, subject,
    parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: ["Explicitly captured export for the current document"],
    read: () => ({ revision: revision(), enabled: !busy }),
    execute: async args => {
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), current.bytes.length);
      if (offset > current.bytes.length) return { status: "unavailable", reason: "Offset outside exported artifact" };
      return { status: "completed", value: { bytes: Array.from(current.bytes.slice(offset, end)), encoding: "JSON UTF-8", sha256: current.sha256, offset, total_bytes: current.bytes.length, next_offset: end < current.bytes.length ? end : null } };
    },
  } });
  const invoke = useUiCommands(commands);
  return <span><button className="btn-link" disabled={!docId || busy} onClick={() => void invoke(`${subject}:export`)}>{busy ? "Exporting…" : "Export viewer snapshot"}</button>{message && <span role="status">{message}</span>}</span>;
}
</book:entry>

</book:doc>
