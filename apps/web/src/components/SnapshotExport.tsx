import { useState } from "react";
import { request } from "../api/client";
export function SnapshotExport({ docId }: { docId?: string | null }) {
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  return <span><button className="btn-link" disabled={!docId || busy} onClick={async () => {
    setBusy(true); setMessage("");
    try {
      const snapshot = await request<unknown>("GET", `/api/docs/${encodeURIComponent(docId!)}/snapshot`);
      const url = URL.createObjectURL(new Blob([JSON.stringify(snapshot, null, 2)], { type: "application/json" }));
      const link = document.createElement("a"); link.href = url; link.download = "hickory-snapshot.json"; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
      setMessage("Exported saved source and recorded cell results. Review before publishing; outputs may contain private data. Open it in the snapshot viewer, or host the JSON beside snapshot.html and use ?src=your-file.json.");
    } catch (error) { setMessage(String(error)); } finally { setBusy(false); }
  }}>{busy ? "Exporting…" : "Export viewer snapshot"}</button>{message && <span role="status">{message}</span>}</span>;
}
